// Copyright 2021 Chay Nabors.

use log::info;
use winit::dpi::PhysicalPosition;
use winit::dpi::PhysicalSize;
#[cfg(target_os = "windows")]
use winit::platform::windows::WindowBuilderExtWindows;

#[derive(Clone, Copy, Debug)]
pub enum WindowEvent {
    Resized([u32; 2]),
    Moved([i32; 2]),
}

#[derive(Debug)]
pub struct Window {
    window: Option<winit::window::Window>,
}

impl Window {
    pub(crate) fn new(event_loop: &winit::event_loop::EventLoop<()>) -> Self {
        info!("Initializing windowing backend");

        Self { window: Some(create_window(event_loop)) }
    }

    pub fn rename(&self, name: &str) -> &Self {
        if let Some(window) = &self.window {
            window.set_title(name);
        }
        self
    }

    pub fn resize(&self, size: [u32; 2]) -> &Self {
        if let Some(window) = &self.window {
            window.set_inner_size(PhysicalSize::new(size[0], size[1]));
        }
        self
    }

    pub fn set_cursor_grab(&self, grab: bool) -> &Self {
        if let Some(window) = &self.window {
            window.set_cursor_grab(grab).unwrap();
        }
        self
    }

    pub fn set_cursor_position(&self, position: [u32; 2]) -> &Self {
        if let Some(window) = &self.window {
            window.set_cursor_position(PhysicalPosition::new(position[0] as f64, position[1] as f64)).unwrap();
        }
        self
    }

    pub fn request_redraw(&self) {
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }

    pub fn size(&self) -> [u32; 2] {
        if let Some(window) = &self.window {
            let size = window.inner_size();
            return [size.width, size.height];
        }

        [0, 0]
    }

    pub fn close(&mut self) {
        self.window.take();
    }
}

unsafe impl raw_window_handle::HasRawWindowHandle for Window {
    fn raw_window_handle(&self) -> raw_window_handle::RawWindowHandle {
        self.window.as_ref().unwrap().raw_window_handle()
    }
}

// This is a workaround since rodio and winit can't run in parallel when drag
// and drop is enabled on windows
#[cfg(target_os = "windows")]
fn create_window(event_loop: &winit::event_loop::EventLoop<()>) -> winit::window::Window {
    winit::window::WindowBuilder::new().with_drag_and_drop(false).build(event_loop).unwrap()
}

#[cfg(not(target_os = "windows"))]
fn create_window(event_loop: &winit::event_loop::EventLoop<()>) -> winit::window::Window {
    winit::window::WindowBuilder::new().build(event_loop).unwrap()
}
