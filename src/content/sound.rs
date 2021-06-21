use std::path::Path;

use crate::content::Loadable;

pub struct Sound;

impl Loadable for Sound {
    fn load<P: AsRef<Path>>(_path: P) -> Result<Self, ()> where Self: Sized {
        Ok(Sound)
    }
}
