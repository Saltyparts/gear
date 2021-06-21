use log::info;
use winit::dpi::PhysicalSize;

#[cfg(target_os = "windows")]
use winit::platform::windows::WindowBuilderExtWindows;

pub enum WindowEvent {
    Resized([u32; 2]),
}

pub struct Window {
    window: winit::window::Window,
}

impl Window {
    pub(crate) fn new(event_loop: &winit::event_loop::EventLoop<()>) -> Self {
        info!("Initializing windowing backend");

        Self {
            window: create_window(event_loop),
        }
    }

    pub fn rename(&self, name: &str) -> &Self {
        self.window.set_title(name);
        self
    }

    pub fn resize(&self, size: [u32; 2]) -> &Self {
        self.window.set_inner_size(PhysicalSize::new(size[0], size[1]));
        self
    }

    pub fn request_redraw(&self) {
        self.window.request_redraw();
    }

    pub fn size(&self) -> [u32; 2] {
        let size = self.window.inner_size();
        [size.width, size.height]
    }
}

unsafe impl raw_window_handle::HasRawWindowHandle for Window {
    fn raw_window_handle(&self) -> raw_window_handle::RawWindowHandle {
        self.window.raw_window_handle()
    }
}

// This is a workaround since rodio and winit can't run in parallel when drag and drop is enabled on windows
#[cfg(target_os = "windows")]
fn create_window(event_loop: &winit::event_loop::EventLoop<()>) -> winit::window::Window {
    winit::window::WindowBuilder::new()
        .with_drag_and_drop(false)
        .build(event_loop).unwrap()
}

#[cfg(not(target_os = "windows"))]
fn create_window(event_loop: &winit::event_loop::EventLoop<()>) -> winit::window::Window {
    winit::window::WindowBuilder::new().build(event_loop).unwrap()
}
