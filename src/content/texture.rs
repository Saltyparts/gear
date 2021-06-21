use std::path::Path;

use crate::content::Loadable;

pub struct Texture;

impl Loadable for Texture {
    fn load<P: AsRef<Path>>(_path: P) -> Result<Self, ()> where Self: Sized {
        Ok(Texture)
    }
}
