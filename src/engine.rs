use std::time::{Duration, Instant};

use log::info;
use winit::{event::{Event as WinitEvent, WindowEvent as WinitWindowEvent}, event_loop::{ControlFlow, EventLoop}};

use crate::{audio::Audio, input::MouseEvent};
use crate::input::{Input, InputEvent};
use crate::network::{Network, NetworkEvent};
use crate::renderer::Renderer;
use crate::window::{Window, WindowEvent};

pub enum Event {
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
        let mut size = [0, 0];

        let mut prev_now = Instant::now();

        let event_loop = self.event_loop.take().unwrap();
        event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Poll;

            match event {
                WinitEvent::WindowEvent { event, window_id: _ } => {
                    match event {
                        WinitWindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                        WinitWindowEvent::Resized(new_size) => {
                            size = [new_size.width, new_size.height];
                            self.renderer.resize(size);
                            event_handler(&mut self, Event::WindowEvent(WindowEvent::Resized(size)));
                        },
                        WinitWindowEvent::KeyboardInput { input, .. } => event_handler(&mut self, Event::InputEvent(InputEvent::KeyboardEvent(input))),
                        WinitWindowEvent::CursorMoved { position, .. } => {
                            event_handler(&mut self, Event::InputEvent(InputEvent::MouseEvent(MouseEvent::CursorMoved([position.x - size[0] as f64 / 2., position.y - size[1] as f64 / 2.]))));
                        },
                        WinitWindowEvent::Destroyed => *control_flow = ControlFlow::Exit,
                        _ => (),
                    };
                },
                WinitEvent::MainEventsCleared => self.window.request_redraw(),
                WinitEvent::RedrawRequested(_) => {
                    while let Some(event) = self.network.get_event() {
                        event_handler(&mut self, Event::NetworkEvent(event));
                    }

                    let now = Instant::now();
                    let delta_time = now - prev_now;
                    prev_now = now;
                    event_handler(&mut self, Event::Update(delta_time));
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
        self.window.close();
    }
}
