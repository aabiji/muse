use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};

use lofty::AudioFile;
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink, Source};

use rand::seq::SliceRandom;
use rand::thread_rng;

use crate::config;
use crate::util;

pub struct Playback {
    _stream: OutputStream,
    _handle: OutputStreamHandle,
    sink: Arc<Mutex<Sink>>,

    config: Option<config::Config>,

    start_time: SystemTime,
    uptime: Duration,

    tracks: Vec<PathBuf>,
    current_track: Arc<Mutex<usize>>,
    total_duration: u64,
}

impl Playback {
    pub fn new() -> Self {
        let (_stream, _handle) = OutputStream::try_default().unwrap();
        let sink = Sink::try_new(&_handle).unwrap();
        Playback {
            _stream,
            _handle,
            sink: Arc::new(Mutex::new(sink)),

            config: None,

            start_time: SystemTime::now(),
            uptime: Duration::new(0, 0),

            tracks: Vec::new(),
            current_track: Arc::new(Mutex::new(0)),
            total_duration: 0,
        }
    }

    fn init(&mut self) -> Result<String, String> {
        if let Some(_) = self.config {
            return Ok(String::new());
        }

        let result = config::load();
        if let Err(err) = result {
            return Err(err.to_string());
        }
        self.config = Some(result.unwrap());

        self.load_tracks();
        if self.tracks.len() == 0 {
            return Err(String::from("Couldn't read any audio files"));
        }

        Ok(String::new())
    }

    fn load_tracks(&mut self) {
        let directories = self.config.as_ref().unwrap().audio_directories.clone();
        for path in directories {
            self.read_tracks(&Path::new(&path));
        }

        let randomize = self.config.as_ref().unwrap().randomize_tracks.unwrap();
        if randomize {
            self.tracks.shuffle(&mut thread_rng());
        } else {
            self.tracks.sort_by_key(|path| path.clone());
        }

        self.determine_starting_track();
    }

    fn read_tracks(&mut self, directory: &Path) {
        let directory = std::fs::read_dir(directory).unwrap();
        for entry in directory {
            let entry = entry.unwrap();
            let path = entry.path();
            if let None = path.extension() {
                continue;
            }

            if entry.metadata().unwrap().is_dir() {
                self.read_tracks(&entry.path());
                continue;
            }

            let warning = format!("Couldn't load {}", path.to_str().unwrap());

            if !util::is_supported_codec(&path) {
                util::log(warning, util::LogType::Warning);
                continue;
            }

            if let Err(_) = lofty::read_from_path(&path) {
                util::log(warning, util::LogType::Warning);
                continue;
            }

            self.tracks.push(path);
        }
    }

    fn determine_starting_track(&mut self) {
        // Get the duration of all the tracks
        let mut durations = Vec::new();
        for path in self.tracks.clone() {
            let tags = lofty::read_from_path(&path).unwrap();
            let duration = tags.properties().duration().as_secs();
            self.total_duration += duration;
            durations.push(duration);
        }

        // Find the track that the start point is situated in
        let mut index = self.current_track.lock().unwrap();
        let mut start = self.config.as_ref().unwrap().start_point;
        while start > durations[*index] {
            start -= durations[*index];
            *index += 1;
            if *index > durations.len() {
                *index = 0;
            }
        }
        self.config.as_mut().unwrap().start_point = start;
    }

    fn play_tracks(
        sink: Arc<Mutex<Sink>>,
        tracks: Vec<PathBuf>,
        mut initial_start: Duration,
        current_track: Arc<Mutex<usize>>,
    ) {
        loop {
            // Continue playing
            if !sink.lock().unwrap().empty() {
                std::thread::sleep(Duration::from_secs(5));
                continue;
            }

            // Start playing the next track
            let mut index = current_track.lock().unwrap();
            let path = &tracks[*index];

            let file = File::open(path).unwrap();
            let reader = BufReader::new(file);

            let source = Decoder::new(reader).unwrap();
            let source = source.skip_duration(initial_start);

            sink.lock().unwrap().append(source);
            sink.lock().unwrap().play();

            // Move on to the next track
            *index += 1;
            if *index == tracks.len() {
                *index = 0;
            }
            initial_start = Duration::new(0, 0);
        }
    }

    fn get_current_track(&self) -> String {
        let pathbuf = &self.tracks[*self.current_track.lock().unwrap()];
        let path = pathbuf.file_name().unwrap().to_str().unwrap();
        path.to_string()
    }

    fn cache_elapsed_time(&mut self) {
        let elapsed = self.start_time.elapsed().unwrap();
        self.uptime += elapsed;

        if let Some(config) = &mut self.config {
            config.clamp_seek_start(self.uptime.as_secs(), self.total_duration);
            config::save(&config);
        }
    }

    pub fn play(&mut self) -> Result<String, String> {
        self.init()?;

        if !self.sink.lock().unwrap().empty() && !self.sink.lock().unwrap().is_paused() {
            return Ok(String::from("Audio is already playing."));
        }

        self.start_time = SystemTime::now();

        // Continue playback if we don't have to load a new track
        // For example: unpausing
        if !self.sink.lock().unwrap().empty() {
            self.sink.lock().unwrap().play();
            return Ok(format!("Unpausing {}", self.get_current_track()));
        }

        let sink = self.sink.clone();
        let tracks = self.tracks.clone();
        let current = self.current_track.clone();

        let start_point = self.config.as_ref().unwrap().start_point;
        let start = Duration::from_secs(start_point);

        std::thread::spawn(move || {
            Playback::play_tracks(sink, tracks, start, current);
        });
        Ok(format!("Playing {}", self.get_current_track()))
    }

    pub fn pause(&mut self) -> Result<String, String> {
        if self.sink.lock().unwrap().empty() {
            return Err(String::from("No audio is playing."));
        }

        self.cache_elapsed_time();
        self.sink.lock().unwrap().pause();

        let msg = format!(
            "Pausing {} after {}",
            self.get_current_track(),
            util::format_time(self.uptime)
        );
        Ok(msg)
    }

    pub fn stop(&mut self, save_config: bool) -> Result<String, String> {
        self.sink.lock().unwrap().stop();
        if save_config {
            self.cache_elapsed_time();
        }

        let msg = format!("Playback stopped after {}", util::format_time(self.uptime));
        Ok(msg)
    }
}
