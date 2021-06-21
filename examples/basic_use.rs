use engine::{Engine, event::{Event, WindowEvent}};

#[tokio::main(flavor = "current_thread")]
async fn main() {
    env_logger::init();

    Engine::new().await.run(move |engine, event| {
        match event {
            Event::Initialize => (),
            Event::Update(delta_time) => {
                engine.renderer
                    .set_clear_color([0.03, 0.03, 0.03, 1.0])
                    .submit();
            },
            Event::Terminate => (),
            Event::WindowEvent(event) => match event {
                WindowEvent::Resized(size) => {},
            },
            _ => (),
        }
    });
}
