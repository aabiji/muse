use std::fs::File;
use std::io::BufReader;

use serde::Deserialize;
use rodio::{OutputStream, OutputStreamHandle, Decoder, Sink};

#[derive(Deserialize)]
pub enum PlaybackOrder {
    Random,
    Alphabetical,
}

#[derive(Deserialize)]
pub struct Config {
    audio_folder_path: String,
    continue_playback: bool,
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
    handle: OutputStreamHandle,
    config: Config,
    current_sink: Sink,
}

impl Playback {
    pub fn new() -> Self {
        let (stream, handle) = OutputStream::try_default().unwrap();
        let mut pb = Playback {
            config: Config::new(),
            _stream: stream,
            handle,
            current_sink: Sink::new_idle().0
        };

        pb.current_sink = Sink::try_new(&pb.handle).unwrap();
        pb
    }

    pub fn start(&mut self) -> Result<String, String> {
        if !self.current_sink.empty() && !self.current_sink.is_paused() {
            return Err(String::from("Audio is already playing."));
        }

        let file = File::open(&self.config.audio_folder_path).unwrap();
        let reader = BufReader::new(file);
        let source = Decoder::new(reader).unwrap();

        self.current_sink.append(source);
        self.current_sink.play();

        Ok(String::from("starting ..."))
    }

    pub fn stop(&mut self) -> Result<String, String> {
        self.current_sink.pause();
        Ok(String::from("stopping ..."))
    }
}
