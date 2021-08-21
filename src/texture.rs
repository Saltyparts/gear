use std::path::Path;

#[derive(Debug)]
pub struct Texture;

impl Texture {
    fn _load<P: AsRef<Path>>(_path: P) -> Result<Self, ()> where Self: Sized {
        Ok(Texture)
    }
}
