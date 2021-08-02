use std::path::Path;

pub struct Texture;

impl Texture {
    fn load<P: AsRef<Path>>(_path: P) -> Result<Self, ()> where Self: Sized {
        Ok(Texture)
    }
}
