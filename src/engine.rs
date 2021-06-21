use std::time::{Duration, SystemTime};

use log::info;
use winit::{event::{Event as WinitEvent, WindowEvent as WinitWindowEvent}, event_loop::{ControlFlow, EventLoop}};

use crate::audio::Audio;
use crate::input::{Input, InputEvent};
use crate::network::{Network, NetworkEvent};
use crate::renderer::Renderer;
use crate::window::{Window, WindowEvent};

pub enum Event {
    Initialize,
    Update(Duration), // delta_time in nanoseconds
    Terminate,
    WindowEvent(WindowEvent),
    InputEvent(InputEvent),
    NetworkEvent(NetworkEvent),
}

pub struct Engine {
    event_loop: Option<EventLoop<()>>,
    pub window: Window,
    pub input: Input,
    pub renderer: Renderer,
    pub audio: Audio,
    pub network: Network,
}

impl Engine {
    pub async fn new() -> Engine {
        let event_loop = EventLoop::new();
        let window = Window::new(&event_loop);
        let input = Input::new();
        let renderer = Renderer::new(&window).await.unwrap();
        let audio = Audio::new();
        let network = Network::new();

        Engine {
            event_loop: Some(event_loop),
            window,
            input,
            renderer,
            audio,
            network,
        }
    }

    pub fn run<F: 'static + FnMut(&mut Engine, Event)>(mut self, mut event_handler: F) {
        info!("Initializing game");
        event_handler(&mut self, Event::Initialize);

        let time = SystemTime::now();

        let event_loop = self.event_loop.take().unwrap();
        event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Poll;

            match event {
                WinitEvent::WindowEvent { event, window_id: _ } => {
                    match event {
                        WinitWindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                        WinitWindowEvent::Resized(size) => {
                            let size = [size.width.max(1), size.height.max(1)];
                            self.renderer.resize(size);
                            event_handler(&mut self, Event::WindowEvent(WindowEvent::Resized(size)));
                        },
                        WinitWindowEvent::KeyboardInput { input, .. } => event_handler(&mut self, Event::InputEvent(InputEvent::KeyboardEvent(input))),
                        _ => (),
                    };
                },
                WinitEvent::MainEventsCleared => {
                    event_handler(&mut self, Event::Update(time.elapsed().unwrap_or_default()));
                },
                WinitEvent::LoopDestroyed => {
                    info!("Terminating game");
                    event_handler(&mut self, Event::Terminate); // terminate event
                },
                _ => (),
            }
        })
    }

    pub fn terminate(&mut self) {
        drop(&self.window);
    }
}
