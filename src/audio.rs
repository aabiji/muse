use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

use lofty::AudioFile;
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink, Source};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
enum PlaybackOrder {
    Random,
    Alphabetical,
}

#[derive(Serialize, Deserialize)]
struct Config {
    audio_folder_path: String,
    playback_order: PlaybackOrder,
    resume_playback: bool,
    elapsed_secs: u64,
}

impl Config {
    fn new() -> Self {
        let path = "config.toml"; // TODO: find path
        let file = std::fs::read_to_string(path).unwrap();
        let mut config: Config = toml::from_str(&file).unwrap();
        if !config.resume_playback {
            config.elapsed_secs = 0;
        }
        config
    }

    fn save(&self) {
        let serialized = toml::to_string(&self).unwrap();
        let path = "config.toml";
        std::fs::write(path, serialized).unwrap();
    }
}

#[derive(Clone)]
struct Track {
    path: PathBuf,
    duration: Duration,
}

pub struct Playback {
    _stream: OutputStream,
    _handle: OutputStreamHandle,
    sink: Sink,
    start_time: SystemTime,
    config: Config,
    tracks: Vec<Track>,
    current_track: usize,
}

impl Playback {
    pub fn new() -> Self {
        let (_stream, _handle) = OutputStream::try_default().unwrap();
        let sink = Sink::try_new(&_handle).unwrap();
        let mut playback = Playback {
            config: Config::new(),
            _stream,
            _handle,
            sink,
            current_track: 0,
            start_time: SystemTime::now(),
            tracks: Vec::new(),
        };

        playback.load_tracks();
        playback.sort_tracks();
        playback
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
        let directory = std::fs::read_dir(&self.config.audio_folder_path).unwrap();
        for entry in directory {
            let entry = entry.unwrap();
            if entry.metadata().unwrap().is_dir() {
                continue;
            }

            let path = entry.path();
            let tags = lofty::read_from_path(&path).unwrap();
            let duration = tags.properties().duration();

            self.tracks.push(Track { path, duration });
        }
    }

    pub fn check_track(&mut self) {
        if !self.sink.empty() {
            return;
        }

        self.current_track += 1;
        self.play_tracks();
    }

    fn play_track(&mut self, path: &PathBuf, starting_point: Duration) {
        let file = File::open(path).unwrap();
        let reader = BufReader::new(file);

        let source = Decoder::new(reader).unwrap();
        let source = source.skip_duration(starting_point);

        self.sink.append(source);
        self.sink.play();
    }

    fn play_tracks(&mut self) {
        let mut start = Duration::from_secs(self.config.elapsed_secs);

        while start > self.tracks[self.current_track].duration {
            start = start.saturating_sub(self.tracks[self.current_track].duration);
            self.current_track += 1;
        }

        let track = &self.tracks[self.current_track].clone();
        self.play_track(&track.path, start);
}

    pub fn play(&mut self) -> Result<String, String> {
        if !self.sink.empty() && !self.sink.is_paused() {
            return Err(String::from("Audio is already playing."));
        }

        self.play_tracks();

        Ok(String::from("Started audio playback."))
    }

    pub fn pause(&mut self) -> Result<String, String> {
        if self.sink.empty() {
            return Err(String::from("No audio is playing."));
        }

        self.sink.pause();
        Ok(String::from("Stopped audio playback."))
    }

    pub fn stop(&mut self) -> Result<String, String> {
        let duration = self.start_time.elapsed().unwrap();
        self.config.elapsed_secs = duration.as_secs();
        self.config.save();

        Ok(String::from("Stopped the playback server."))
    }
}
