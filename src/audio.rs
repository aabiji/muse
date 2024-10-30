use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};

use crate::util;
use lofty::AudioFile;
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink, Source};

use crate::config;

pub fn format_time(d: Duration) -> String {
    let mut index = 0;
    let mut n = d.as_secs_f64();
    while n > 60.0 {
        n /= 60.0;
        index += 1;
    }

    let units = ["seconds", "minutes", "hours"];
    let mut unit = String::from(units[index]);
    if n <= 1.0 {
        unit.pop();
    }
    format!("{:.1} {}", n, unit)
}

fn is_supported_codec(file: &Path) -> bool {
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
    config: config::Config,
    loaded_config: bool,
    start_time: SystemTime,
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
            config: config::Config::new(),
            loaded_config: false,
            start_time: SystemTime::now(),
            tracks: Vec::new(),
            current_track: Arc::new(Mutex::new(0)),
            tracks_duration: 0,
        }
    }

    fn init(&mut self) -> Result<String, String> {
        if self.loaded_config {
            return Ok(String::new());
        }

        let result = config::load();
        if let Err(err) = result {
            return Err(err.to_string());
        }
        self.config = result.unwrap();
        self.loaded_config = true;

        self.load_tracks();
        if self.tracks.len() == 0 {
            return Err(String::from("Couldn't read any audio files"));
        }

        Ok(String::new())
    }

    fn load_tracks(&mut self) {
        let directories = self.config.audio_directories.clone();
        for path in directories {
            self.read_tracks(&Path::new(&path));
        }

        // Sort tracks if necessary
        match self.config.playback_order {
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

            if !is_supported_codec(&path) {
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

    fn format_output(&self, mode: &str) -> String {
        let pathbuf = &self.tracks[*self.current_track.lock().unwrap()].path;
        let path = pathbuf.file_name().unwrap().to_str().unwrap();
        format!("{} {}", mode, path)
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
            return Ok(self.format_output("Playing"));
        }

        let sink = self.sink.clone();
        let tracks = self.tracks.clone();
        let current = self.current_track.clone();
        let start = Duration::from_secs(self.config.start_point);
        std::thread::spawn(move || {
            Playback::play_tracks(sink, tracks, start, current);
        });

        Ok(self.format_output("Playing"))
    }

    pub fn pause(&mut self) -> Result<String, String> {
        if self.sink.lock().unwrap().empty() {
            return Err(String::from("No audio is playing."));
        }

        self.sink.lock().unwrap().pause();
        Ok(self.format_output("Pausing"))
    }

    pub fn stop(&mut self, save_config: bool) -> Result<String, String> {
        self.sink.lock().unwrap().stop();
        let duration = self.start_time.elapsed().unwrap();
        if save_config {
            self.config.clamp_seek_start(duration.as_secs(), self.tracks_duration);
            config::save(&self.config);
        }

        let msg = format!("Uptime: {}\nPlayback stopped", format_time(duration));
        Ok(msg)
    }
}
