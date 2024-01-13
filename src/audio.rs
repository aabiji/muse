use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::time::Duration;

use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink, Source};
use serde::Deserialize;

#[derive(Deserialize)]
enum PlaybackOrder {
    Random,
    Alphabetical,
}

#[derive(Deserialize)]
struct Config {
    audio_folder_path: String,
    playback_order: PlaybackOrder,
    resume_playback: bool,
    stopped_timestamp: u32,
}

impl Config {
    fn new() -> Self {
        let path = "config.toml";
        let file = std::fs::read_to_string(path).unwrap();
        let config: Config = toml::from_str(&file).unwrap();
        config
    }
}

#[derive(Debug)]
struct Track {
    file: String,
    length: Duration,
}

pub struct Playback {
    _stream: OutputStream,
    _handle: OutputStreamHandle,
    config: Config,
    sink: Sink,
    tracks: Vec<Track>,
}

impl Playback {
    pub fn new() -> Self {
        let (_stream, _handle) = OutputStream::try_default().unwrap();
        let sink = Sink::try_new(&_handle).unwrap();
        Playback {
            config: Config::new(),
            _stream,
            _handle,
            sink,
            tracks: Vec::new(),
        }
    }

    fn sort_audio_files(&self, paths: &mut Vec<PathBuf>) {
        let alpha_sort = |p1: &PathBuf, p2: &PathBuf| {
            let a = p1.file_name().unwrap().to_ascii_lowercase();
            let b = p2.file_name().unwrap().to_ascii_lowercase();
            a.cmp(&b)
        };

        match self.config.playback_order {
            PlaybackOrder::Alphabetical => paths.sort_by(alpha_sort),
            PlaybackOrder::Random => {}
        };
    }

    // TODO: continue playback from timestamp
    fn load_audio_directory(&mut self) {
        let mut paths: Vec<PathBuf> = Vec::new();

        let directory = std::fs::read_dir(&self.config.audio_folder_path).unwrap();
        for entry in directory {
            let entry = entry.unwrap();
            if entry.metadata().unwrap().is_file() {
                paths.push(entry.path());
            }
        }

        self.sort_audio_files(&mut paths);

        for path in paths {
            let file = File::open(&path).unwrap();
            let reader = BufReader::new(file);

            match Decoder::new(reader) {
                Ok(source) => {
                    let file = path.display().to_string();
                    let length = match source.total_duration() {
                        Some(len) => len,
                        None => Duration::from_secs(0),
                    };
                    self.tracks.push(Track { file, length });
                    self.sink.append(source);
                }
                Err(_) => {
                    // See rodio::decoder::DecoderError
                    println!("Unable to load {}.", path.display());
                    continue;
                }
            };
        }
    }

    pub fn play(&mut self) -> Result<String, String> {
        if !self.sink.empty() && !self.sink.is_paused() {
            return Err(String::from("Audio is already playing."));
        }

        self.load_audio_directory();
        self.sink.play();

        Ok(String::from("starting ..."))
    }

    pub fn pause(&mut self) -> Result<String, String> {
        if self.sink.empty() {
            return Err(String::from("No audio is playing."));
        }

        self.sink.pause();
        Ok(String::from("stopping ..."))
    }
}
