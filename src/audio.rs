use log::info;
use rodio::{OutputStream, OutputStreamHandle};

use crate::Sound;

pub struct AudioSource {
    pub(crate) sink: rodio::Sink,
}

impl AudioSource {
    pub fn queue_sound(&self, sound: &Sound) -> &Self {
        self.sink.append(sound.decoder());
        self
    }

    pub fn set_volume(&self, volume: f32) -> &Self {
        self.sink.set_volume(volume);
        self
    }
}

pub struct Audio {
    stream: OutputStream,
    stream_handle: OutputStreamHandle,
}

impl Audio {
    pub(crate) fn new() -> Audio {
        info!("Initializing audio backend");
        let (stream, stream_handle) = OutputStream::try_default().unwrap();

        Audio {
            stream,
            stream_handle,
        }
    }

    pub fn create_audio_source(&self) -> AudioSource {
        AudioSource {
            sink: rodio::Sink::try_new(&self.stream_handle).unwrap()
        }
    }
}
