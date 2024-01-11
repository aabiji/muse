use std::fs::File;
use std::io::BufReader;

use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink};
use serde::Deserialize;

#[derive(Deserialize)]
enum PlaybackOrder {
    Random,
    Alphabetical,
}

#[derive(Deserialize)]
struct Config {
    audio_folder_path: String,
    resume_playback: bool,
    playback_order: PlaybackOrder,
}

impl Config {
    fn new() -> Self {
        let path = "config.toml";
        let file = std::fs::read_to_string(path).unwrap();
        let config: Config = toml::from_str(&file).unwrap();
        config
    }
}

pub struct Playback {
    _stream: OutputStream,
    _handle: OutputStreamHandle,
    config: Config,
    sink: Sink,
}

impl Playback {
    pub fn new() -> Self {
        let (_stream, _handle) = OutputStream::try_default().unwrap();
        let sink = Sink::try_new(&_handle).unwrap();
        Playback {
            config: Config::new(),
            _stream, _handle, sink
        }
    }

    // TODO: sort entries
    // TODO: continue playback from timestamp
    // TODO: expand relative path into absolute path
    fn load_audio_directory(&mut self) {
        let directory = std::fs::read_dir(&self.config.audio_folder_path).unwrap();
        for entry in directory {
            let entry = entry.unwrap();
            if entry.metadata().unwrap().is_dir() {
                continue;
            }

            let path = entry.path();
            let file = File::open(&path).unwrap();
            let reader = BufReader::new(file);

            match Decoder::new(reader) {
                Ok(source) => self.sink.append(source),
                Err(_) => { // See rodio::decoder::DecoderError
                    println!("Unable to load {}", path.display());
                    continue;
                },
            };
        }
    }

    pub fn start(&mut self) -> Result<String, String> {
        if !self.sink.empty() && !self.sink.is_paused() {
            return Err(String::from("Audio is already playing."));
        }

        self.load_audio_directory();
        Ok(String::from("starting ..."))
    }

    pub fn stop(&mut self) -> Result<String, String> {
        self.sink.pause();
        Ok(String::from("stopping ..."))
    }
}