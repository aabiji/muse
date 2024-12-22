use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};

use lofty::AudioFile;
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink, Source};

use crate::util;
use crate::config;

#[derive(Clone)]
struct Track {
    path: PathBuf,
    duration: Duration,
}

pub struct Playback {
    _stream: OutputStream,
    _handle: OutputStreamHandle,
    sink: Arc<Mutex<Sink>>,

    config: Option<config::Config>,

    start_time: SystemTime,
    uptime: Duration,

    tracks: Vec<Track>,
    tracks_duration: u64,
    current_track: Arc<Mutex<usize>>,
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
            tracks_duration: 0,
            current_track: Arc::new(Mutex::new(0)),
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

        // Sort tracks if necessary
        // TODO: randomize tracks
        let order = &self.config.as_ref().unwrap().playback_order;
        match order {
            config::PlaybackOrder::Alphabetical => self.tracks.sort_by_key(|t| t.path.clone()),
            config::PlaybackOrder::Random => {}
        };
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

            let warning = format!(
                "Couldn't load {}. {} files are not supported.",
                path.to_str().unwrap(),
                path.extension().unwrap().to_str().unwrap()
            );

            if !util::is_supported_codec(&path) {
                util::log(warning, util::LogType::Warning);
                continue;
            }

            let result = lofty::read_from_path(&path);
            match result {
                Err(_) => {
                    util::log(warning, util::LogType::Warning);
                    continue;
                }
                Ok(tags) => {
                    let duration = tags.properties().duration();
                    self.tracks.push(Track { path, duration });
                    self.tracks_duration += duration.as_secs();
                }
            };
        }
    }

    fn play_track(sink: &mut Arc<Mutex<Sink>>, path: &PathBuf, starting_point: Duration) {
        let file = File::open(path).unwrap();
        let reader = BufReader::new(file);

        let source = Decoder::new(reader).unwrap();
        let source = source.skip_duration(starting_point);

        sink.lock().unwrap().append(source);
        sink.lock().unwrap().play();
    }

    fn play_tracks(
        mut sink: Arc<Mutex<Sink>>,
        tracks: Vec<Track>,
        mut start: Duration,
        current_track: Arc<Mutex<usize>>,
    ) {
        loop {
            if !sink.lock().unwrap().empty() {
                std::thread::sleep(Duration::from_secs(5));
                continue;
            }

            let mut index = current_track.lock().unwrap();
            let track = &tracks[*index];
            if start < track.duration {
                Playback::play_track(&mut sink, &track.path, start);
            }

            start = start.saturating_sub(track.duration);

            *index += 1;
            if *index == tracks.len() {
                *index = 0;
            }
        }
    }

    fn get_current_track(&self) -> String {
        let pathbuf = &self.tracks[*self.current_track.lock().unwrap()].path;
        let path = pathbuf.file_name().unwrap().to_str().unwrap();
        path.to_string()
    }

    pub fn play(&mut self) -> Result<String, String> {
        self.init()?;

        if !self.sink.lock().unwrap().empty() && !self.sink.lock().unwrap().is_paused() {
            // TODO: we shouldn't shut down here
            return Err(String::from("Audio is already playing."));
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

        let elapsed = self.start_time.elapsed().unwrap();
        self.uptime += elapsed;

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

        let elapsed = self.start_time.elapsed().unwrap();
        self.uptime += elapsed;

        if let Some(config) = &mut self.config {
            if save_config {
                config.clamp_seek_start(self.uptime.as_secs(), self.tracks_duration);
                config::save(&config);
            }
        }

        let msg = format!("Playback stopped after {}", util::format_time(self.uptime));
        Ok(msg)
    }
}
