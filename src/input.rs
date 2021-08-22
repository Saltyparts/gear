pub use winit::event::ElementState as KeyState;
pub use winit::event::KeyboardInput as KeyboardEvent;
pub use winit::event::VirtualKeyCode as KeyCode;

#[derive(Clone, Copy, Debug)]
pub enum JoystickEvent {}

#[derive(Clone, Copy, Debug)]
pub enum MouseEvent {
    CursorMoved([f64; 2]),
}

#[derive(Clone, Copy, Debug)]
pub enum InputEvent {
    JoystickEvent(JoystickEvent),
    KeyboardEvent(KeyboardEvent),
    MouseEvent(MouseEvent),
}

#[derive(Debug)]
pub struct Input;

impl Input {
    pub(crate) fn new() -> Self {
        Self
    }
}
