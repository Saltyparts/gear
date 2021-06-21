pub use winit::event::KeyboardInput as KeyboardEvent;
pub use winit::event::ElementState as KeyState;

pub enum JoystickEvent {

}

pub enum MouseEvent {

}

pub enum InputEvent {
    JoystickEvent(JoystickEvent),
    KeyboardEvent(KeyboardEvent),
    MouseEvent(MouseEvent),
}

pub struct Input;

impl Input {
    pub(crate) fn new() -> Self {
        Self
    }
}
