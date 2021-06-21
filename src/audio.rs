use log::info;
pub use rodio::Sink as AudioSource;

use rodio::{OutputStream, OutputStreamHandle};

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
        AudioSource::try_new(&self.stream_handle).unwrap()
    }
}
