// Copyright 2021 Chay Nabors.

use std::path::Path;

use crate::result::Result;

pub trait Loadable {
    fn load<P: AsRef<Path>>(path: P) -> Result<Self>
    where
        Self: Sized;
}
