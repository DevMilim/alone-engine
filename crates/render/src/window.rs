use std::sync::Arc;

use pixels::{Pixels, SurfaceTexture};
use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowId},
};
const WIDTH: u32 = 320;
const HEIGHT: u32 = 240;
const BOX_SIZE: i16 = 64;

struct World {
    box_x: i16,
    box_y: i16,
    velocity_x: i16,
    velocity_y: i16,
}
struct App<'a> {
    state: Option<Pixels<'a>>,
    window: Option<Arc<Window>>,
    word: World,
}

impl<'a> App<'a> {
    pub fn new() -> Self {
        Self {
            state: None,
            window: None,
            word: World::new(),
        }
    }
}

impl<'a> ApplicationHandler for App<'a> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let attrs = Window::default_attributes()
            .with_title("winit + pixels")
            .with_inner_size(LogicalSize::new(800.0, 600.0));

        let window = Arc::new(event_loop.create_window(attrs).unwrap());

        let pixels = {
            let window_size = window.inner_size();
            let surface_texture =
                SurfaceTexture::new(window_size.width, window_size.height, window.clone());
            Pixels::new(WIDTH, HEIGHT, surface_texture).unwrap()
        };

        self.state = Some(pixels);
        window.request_redraw();
        self.window = Some(window);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let state = self.state.as_mut().unwrap();
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }

            WindowEvent::KeyboardInput { event, .. } => {
                /*
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
                 */
            }
            WindowEvent::CursorMoved { position, .. } => {
                /*
                self.engine
                    .event_queue
                    .push_back(crate::EngineCommands::MousePosition(
                        position.x as f32,
                        position.y as f32,
                    ));
                     */
            }
            WindowEvent::RedrawRequested => {
                self.word.update();

                self.word.draw(state.frame_mut());

                let _ = self.state.as_mut().unwrap().render();
                self.window.as_mut().unwrap().request_redraw();
            }
            WindowEvent::Resized(size) => {
                // Reconfigures the size of the surface. We do not re-render
                // here as this event is always followed up by redraw request.
            }
            _ => (),
        }
    }
}

impl World {
    /// Create a new `World` instance that can draw a moving box.
    fn new() -> Self {
        Self {
            box_x: 24,
            box_y: 16,
            velocity_x: 1,
            velocity_y: 1,
        }
    }

    /// Update the `World` internal state; bounce the box around the screen.
    fn update(&mut self) {
        if self.box_x <= 0 || self.box_x + BOX_SIZE > WIDTH as i16 {
            self.velocity_x *= -1;
        }
        if self.box_y <= 0 || self.box_y + BOX_SIZE > HEIGHT as i16 {
            self.velocity_y *= -1;
        }

        self.box_x += self.velocity_x;
        self.box_y += self.velocity_y;
    }

    /// Draw the `World` state to the frame buffer.
    ///
    /// Assumes the default texture format: `wgpu::TextureFormat::Rgba8UnormSrgb`
    fn draw(&self, frame: &mut [u8]) {
        for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {
            let x = (i % WIDTH as usize) as i16;
            let y = (i / WIDTH as usize) as i16;

            let inside_the_box = x >= self.box_x
                && x < self.box_x + BOX_SIZE
                && y >= self.box_y
                && y < self.box_y + BOX_SIZE;

            let rgba = if inside_the_box {
                [0x5e, 0x48, 0xe8, 0xff]
            } else {
                [0x48, 0xb2, 0xe8, 0xff]
            };

            pixel.copy_from_slice(&rgba);
        }
    }
}
pub fn run_render() {
    let event_loop = EventLoop::new().unwrap();

    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::new();
    event_loop.run_app(&mut app).unwrap();
}
