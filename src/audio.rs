use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};

use colored::Colorize;
use lofty::AudioFile;
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink, Source};

use crate::config::{Config, PlaybackOrder};

fn is_supported_codec(file: &Path) -> bool {
    // Taken from the rodio readme
    let supported = ["mp3", "mp4", "wav", "ogg", "flac"];
    let extension = file.extension().unwrap().to_str().unwrap();
    if !supported.contains(&extension) {
        return false;
    }
    true
}

#[derive(Clone)]
struct Track {
    path: PathBuf,
    duration: Duration,
}

pub struct Playback {
    _stream: OutputStream,
    _handle: OutputStreamHandle,
    sink: Arc<Mutex<Sink>>,

    config: Config,
    loaded_config: bool,
    start_time: SystemTime, // The time at which playback is started

    tracks: Vec<Track>,  // The tracks to be played
    total_duration: u64, // Total duration of all the tracks
}

impl Playback {
    pub fn new() -> Self {
        let (_stream, _handle) = OutputStream::try_default().unwrap();
        let sink = Sink::try_new(&_handle).unwrap();
        Playback {
            _stream,
            _handle,
            sink: Arc::new(Mutex::new(sink)),

            loaded_config: false,
            config: Config::default(),
            start_time: SystemTime::now(),

            tracks: Vec::new(),
            total_duration: 0,
        }
    }

    fn init(&mut self) -> Result<String, String> {
        if self.loaded_config {
            return Ok(String::new());
        }

        if let Err(err) = self.config.load() {
            return Err(err.to_string());
        }

        self.load_tracks();
        if self.tracks.len() == 0 {
            return Err(String::from("No audio directories specified."));
        }

        // Assure that the resumption point is smaller than the
        // total length of all audio tracks
        if self.config.start_point > self.total_duration {
            self.config.start_point %= self.total_duration;
        }

        Ok(String::new())
    }

    fn sort_tracks(&mut self) {
        let alpha_sort = |t1: &Track, t2: &Track| {
            let a = t1.path.file_name().unwrap().to_ascii_lowercase();
            let b = t2.path.file_name().unwrap().to_ascii_lowercase();
            a.cmp(&b)
        };

        match self.config.playback_order {
            PlaybackOrder::Alphabetical => self.tracks.sort_by(alpha_sort),
            PlaybackOrder::Random => {}
        };
    }

    fn load_tracks(&mut self) {
        let dir = self.config.audio_directories.clone();
        for path in dir {
            self.read_tracks(&Path::new(&path));
        }

        self.sort_tracks();
    }

    fn read_tracks(&mut self, directory: &Path) {
        let directory = std::fs::read_dir(directory).unwrap();
        for entry in directory {
            let entry = entry.unwrap();
            let path = entry.path();
            let warning = format!(
                "Couldn't load {}. {} files are not supported.",
                path.to_str().unwrap(),
                path.extension().unwrap().to_str().unwrap()
            );

            if entry.metadata().unwrap().is_dir() {
                self.read_tracks(&entry.path());
                continue;
            }

            if !is_supported_codec(&path) {
                println!("{}", warning.yellow());
                continue;
            }

            let result = lofty::read_from_path(&path);
            match result {
                Err(_) => {
                    println!("{}", warning.yellow());
                    continue;
                }
                Ok(tags) => {
                    let duration = tags.properties().duration();
                    self.tracks.push(Track { path, duration });
                    self.total_duration += duration.as_secs();
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

    fn play_tracks(mut sink: Arc<Mutex<Sink>>, tracks: Vec<Track>, mut start: Duration) {
        let mut current = 0;
        loop {
            if !sink.lock().unwrap().empty() {
                std::thread::sleep(Duration::from_secs(5));
                continue;
            }

            let track = &tracks[current];
            if start < track.duration {
                Playback::play_track(&mut sink, &track.path, start);
            }

            start = start.saturating_sub(track.duration);

            current += 1;
            if current == tracks.len() {
                current = 0;
            }
        }
    }

    pub fn play(&mut self) -> Result<String, String> {
        self.init()?;

        if !self.sink.lock().unwrap().empty() && !self.sink.lock().unwrap().is_paused() {
            return Err(String::from("Audio is already playing."));
        }

        // Continue playback if we don't have to load a new track
        // For example: unpausing
        if !self.sink.lock().unwrap().empty() {
            self.sink.lock().unwrap().play();
            return Ok(String::from("Starting audio playback ..."));
        }

        let cloned = self.sink.clone();
        let tracks = self.tracks.clone();
        let start = Duration::from_secs(self.config.start_point);
        std::thread::spawn(move || {
            Playback::play_tracks(cloned, tracks, start);
        });

        Ok(String::from("Starting audio playback ..."))
    }

    pub fn pause(&mut self) -> Result<String, String> {
        if self.sink.lock().unwrap().empty() {
            return Err(String::from("No audio is playing."));
        }

        self.sink.lock().unwrap().pause();
        Ok(String::from("Stopped audio playback."))
    }

    pub fn stop(&mut self, save_config: bool) -> Result<String, String> {
        if save_config {
            let duration = self.start_time.elapsed().unwrap();
            self.config.start_point = duration.as_secs();
            self.config.save();
        }
        Ok(String::from("Stopped the playback server."))
    }
}
