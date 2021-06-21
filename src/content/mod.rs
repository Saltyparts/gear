pub(crate) mod model;
pub(crate) mod sound;
pub(crate) mod texture;

use std::{path::Path, rc::Rc};

use log::error;

pub trait Loadable {
    fn load<P: AsRef<Path>>(path: P) -> Result<Self, ()> where Self: Sized;
}

#[derive(Clone, Debug)]
pub struct Content<T: Loadable>(pub(crate) Rc<T>);

impl<T: Loadable> Content<T> {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, ()> {
        match T::load(path.as_ref()) {
            Ok(content) => Ok(Content(Rc::<T>::new(content))),
            Err(e) => {
                error!("Failed to load content at path: {:?}", path.as_ref());
                Err(e)
            }
        }
    }
}

pub type Model = model::Model;
pub type Sound = sound::Sound;
pub type Texture = Content<texture::Texture>;
