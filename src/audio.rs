use std::fmt::Debug;

use log::info;
use rodio::OutputStream;
use rodio::OutputStreamHandle;

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
    _stream: OutputStream,
    stream_handle: OutputStreamHandle,
}

impl Audio {
    pub(crate) fn new() -> Audio {
        info!("Initializing audio backend");
        let (stream, stream_handle) = OutputStream::try_default().unwrap();

        Audio { _stream: stream, stream_handle }
    }

    pub fn create_audio_source(&self) -> AudioSource {
        AudioSource { sink: rodio::Sink::try_new(&self.stream_handle).unwrap() }
    }
}

impl Debug for Audio {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Audio").field("_stream", &"stream").field("stream_handle", &"stream_handle").finish()
    }
}
