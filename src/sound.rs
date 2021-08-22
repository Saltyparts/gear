// Copyright 2021 Chay Nabors.

use std::convert::AsRef;
use std::fs::File;
use std::io::Read;
use std::io::{self,};
use std::path::Path;
use std::sync::Arc;

use crate::Loadable;

pub struct Sound(Arc<Vec<u8>>);

impl Sound {
    pub(crate) fn cursor(self: &Self) -> io::Cursor<Sound> {
        io::Cursor::new(Sound(self.0.clone()))
    }

    pub(crate) fn decoder(self: &Self) -> rodio::Decoder<io::Cursor<Sound>> {
        rodio::Decoder::new(self.cursor()).unwrap()
    }
}

impl Loadable for Sound {
    fn load<P: AsRef<Path>>(path: P) -> crate::Result<Self>
    where
        Self: Sized,
    {
        let mut buf = vec![];
        let mut file = File::open(path)?;
        file.read_to_end(&mut buf)?;
        Ok(Sound(Arc::new(buf)))
    }
}

impl AsRef<[u8]> for Sound {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}
