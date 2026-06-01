use std::sync::Arc;

use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::PhysicalKey,
    window::{Window, WindowId},
};

use crate::{
    Base, CoreSystems, EngineContext, EventManager, GameObjectDispatch, InputType, LOGICAL_WIDTH,
    Render, RuntimeEvent, Scene, Vector2, WorldState,
};

pub struct App<S: Scene> {
    pub systems: CoreSystems,
    pub events: EventManager,
    pub world: WorldState<S>,
    pub render: Option<Render<'static>>,

    pub window: Option<Arc<Window>>,
    pub base: Base,
    pub camera_position: Vector2,
}

impl<S: Scene> App<S> {
    pub fn new(root_scene: S) -> Self {
        Self {
            systems: CoreSystems::default(),
            events: EventManager::default(),
            world: WorldState::new(root_scene),
            render: None,
            window: None,
            base: Base::empty(),
            camera_position: Vector2::new(LOGICAL_WIDTH as f32 / 2.0, LOGICAL_WIDTH as f32 / 2.0),
        }
    }
    pub fn events(&mut self, cmd: RuntimeEvent) {
        match cmd {
            RuntimeEvent::Quit => self.world.is_running = false,
            RuntimeEvent::KeyDown(keycode) => self
                .systems
                .input
                .update_input_state(InputType::Key(keycode), true),
            RuntimeEvent::KeyUp(keycode) => self
                .systems
                .input
                .update_input_state(InputType::Key(keycode), false),
            RuntimeEvent::MousePosition(x, y) => self.systems.input.set_mouse_position(x, y),
            RuntimeEvent::MouseInput(mouse_button, is_pressed) => self
                .systems
                .input
                .update_input_state(InputType::Mouse(mouse_button), is_pressed),
        }
    }
    pub fn flush_messages_and_events(ctx: &mut EngineContext, world: &mut WorldState<S>) {
        for _ in 0..10 {
            let mut something_processed = false;
            while let Some(event) = &ctx.events.global_events.pop_front() {
                something_processed = true;
                world.last_scene().dispatch_event(ctx, event);
            }
            if !ctx.events.mailbox.is_empty() {
                something_processed = true;

                world.last_scene().dispatch_message(ctx);
            }
            if !something_processed {
                break;
            }
        }
    }
    pub fn run(&mut self) {
        let event_loop = EventLoop::new().unwrap();

        event_loop.set_control_flow(ControlFlow::Poll);

        event_loop.run_app(self).unwrap();
    }
}

impl<S: Scene> ApplicationHandler for App<S> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let attrs = Window::default_attributes()
            .with_title("winit + pixels")
            .with_inner_size(LogicalSize::new(800, 600));

        let window = Arc::new(event_loop.create_window(attrs).unwrap());

        self.render = Some(Render::new(&window));
        window.request_redraw();
        self.window = Some(window);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let state = self.render.as_mut().unwrap();
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }

            WindowEvent::KeyboardInput { event, .. } => {
                if let PhysicalKey::Code(keycode) = event.physical_key {
                    self.systems
                        .input
                        .update_input_state(InputType::Key(keycode), event.state.is_pressed());
                }
            }
            WindowEvent::MouseInput { state, button, .. } => {
                self.systems
                    .input
                    .update_input_state(InputType::Mouse(button), state.is_pressed());
            }
            WindowEvent::CursorMoved { position, .. } => {
                if let Ok(mouse) = self
                    .render
                    .as_mut()
                    .unwrap()
                    .pixels
                    .window_pos_to_pixel((position.x as f32, position.y as f32))
                {
                    let camera = self.camera_position;
                    self.systems
                        .input
                        .set_mouse_position(mouse.0 as f32 + camera.x, mouse.1 as f32 + camera.y);
                }
            }
            WindowEvent::Resized(size)
                if size.width > 0 && size.height > 0 => {
                    let _ = state.pixels.resize_surface(size.width, size.height);

                    if let Some(window) = &self.window {
                        window.request_redraw();
                    }
                }
            _ => (),
        }
    }
    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        let Some(render) = &mut self.render else {
            panic!("Erro ao obter render")
        };

        let mut ctx = EngineContext {
            systems: &mut self.systems,
            events: &mut self.events,
            camera_position: &mut self.camera_position,
        };

        let (is_running, blending) = self.world.update(&mut ctx, &self.base);

        Self::flush_messages_and_events(&mut ctx, &mut self.world);

        self.world
            .render(&mut render.queue, &mut ctx, &self.base, blending);

        render.render(self.camera_position, &self.systems.resources);

        if !is_running {
            event_loop.exit();
        }
        self.window.as_mut().unwrap().request_redraw();
        self.systems.input.clear_frame_data();
    }
}
