use std::sync::Arc;

use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::PhysicalKey,
    window::{Window, WindowId},
};

use crate::{Engine, Scene, State};

struct App<S: Scene> {
    state: Option<State>,
    engine: Engine<S>,
}

impl<S: Scene> App<S> {
    pub fn new(main_scene: S) -> Self {
        let engine = Engine::new(main_scene);
        Self {
            state: None,
            engine,
        }
    }
}

impl<S: Scene> ApplicationHandler for App<S> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // Create window object
        let window = Arc::new(
            event_loop
                .create_window(Window::default_attributes())
                .unwrap(),
        );

        let state = pollster::block_on(State::new(
            event_loop.owned_display_handle(),
            window.clone(),
        ));

        self.state = Some(state);

        window.request_redraw();
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let state = self.state.as_mut().unwrap();
        match event {
            WindowEvent::CloseRequested => {
                self.engine.quit();
                event_loop.exit();
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if let PhysicalKey::Code(keycode) = event.physical_key {
                    if event.state.is_pressed() {
                        self.engine
                            .event_queue
                            .push_back(crate::EngineCommands::KeyDown(keycode));
                    } else {
                        self.engine
                            .event_queue
                            .push_back(crate::EngineCommands::KeyUp(keycode));
                    }
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.engine
                    .event_queue
                    .push_back(crate::EngineCommands::MousePosition(
                        position.x as f32,
                        position.y as f32,
                    ));
            }
            WindowEvent::RedrawRequested => {
                let keep_running = self.engine.update_step();

                if !keep_running {
                    event_loop.exit();
                    return;
                }

                let render_queue = self.engine.draw();

                state.render(&render_queue, &self.engine.resources);
                // Emits a new redraw requested event.
                state.get_window().request_redraw();
            }
            WindowEvent::Resized(size) => {
                // Reconfigures the size of the surface. We do not re-render
                // here as this event is always followed up by redraw request.
                state.resize(size);
            }
            _ => (),
        }
    }
}

pub fn run<S: Scene>(scene: S) {
    let event_loop = EventLoop::new().unwrap();

    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::new(scene);
    event_loop.run_app(&mut app).unwrap();
}
