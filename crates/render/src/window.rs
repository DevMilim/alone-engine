use core::{DrawCommand, DrawCommandType, RenderApi};
use std::{collections::BTreeMap, sync::Arc};

use pixels::{Pixels, SurfaceTexture};
use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::{Fullscreen, Window, WindowId},
};

use crate::{RenderCommands, Runtime};
const WIDTH: u32 = 320;
const HEIGHT: u32 = 240;

pub struct RenderQueue {
    queue: BTreeMap<usize, DrawCommand>,
}
impl RenderQueue {
    pub fn new() -> Self {
        Self {
            queue: BTreeMap::new(),
        }
    }
}

impl RenderApi for RenderQueue {
    fn draw(&mut self, command: DrawCommand) {
        self.queue.insert(command.z_index, command);
    }
}

struct Render<'a, R: Runtime> {
    state: Option<Pixels<'a>>,
    window: Option<Arc<Window>>,
    queue: RenderQueue,
    runtime: R,
    window_size: (u32, u32),
}

impl<'a, R: Runtime> Render<'a, R> {
    pub fn new(runtime: R) -> Self {
        Self {
            state: None,
            window: None,
            queue: RenderQueue::new(),
            runtime,
            window_size: (WIDTH, HEIGHT),
        }
    }
    pub fn render(&mut self) {
        let frame = self.state.as_mut().unwrap().frame_mut();
        for cmd in self.queue.queue.values() {
            match cmd.cmd_type {
                DrawCommandType::Sprite => {
                    let texture = self.runtime.get_texture(cmd.material.image);
                    match texture {
                        Some(texture) => {
                            let tex_width = texture.width as usize;
                            let tex_height = texture.height as usize;
                            let frame_width = self.window_size.0 as usize;
                            let frame_height = self.window_size.1 as usize;
                            let position = &cmd.material.pos;
                            for y in 0..texture.height as usize {
                                for x in 0..tex_width {
                                    let src = (y * tex_width + x) * 4;
                                    let dst_x = position.x as usize + x;
                                    let dst_y = position.y as usize + y;

                                    if dst_x >= frame_width || dst_y >= frame_height {
                                        continue;
                                    }

                                    let alpha = texture.pixels[src + 3];
                                    if alpha == 0 {
                                        continue;
                                    }
                                    let dst = (dst_y * frame_width + dst_x) * 4;
                                    frame[dst..dst + 4]
                                        .copy_from_slice(&texture.pixels[src..src + 4]);
                                }
                            }
                            /*
                            for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {
                                let x = (i % self.window_size.0 as usize) as f32;
                                let y = (i / self.window_size.0 as usize) as f32;

                                let position = &cmd.material.pos;
                                let size = &cmd.material.size;

                                let inside_the_box = x >= position.x
                                    && x < position.x + texture.width as f32
                                    && y >= position.y
                                    && y < position.y + texture.height as f32;

                                if inside_the_box {
                                    pixel.copy_from_slice(&color.bytes());
                                }
                            }
                            */
                        }
                        None => println!("Textura não encontrada"),
                    }
                }
                DrawCommandType::Rect => {
                    for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {
                        let x = (i % self.window_size.0 as usize) as f32;
                        let y = (i / self.window_size.0 as usize) as f32;

                        let position = &cmd.material.pos;
                        let size = &cmd.material.size;
                        let color = &cmd.material.color;

                        let inside_the_box = x >= position.x
                            && x < position.x + size.x
                            && y >= position.y
                            && y < position.y + size.y;

                        if inside_the_box {
                            pixel.copy_from_slice(&color.bytes());
                        }
                    }
                }
            }
        }
        self.queue.queue.clear();
    }
}

impl<'a, R: Runtime> ApplicationHandler for Render<'a, R> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let attrs = Window::default_attributes()
            .with_title("winit + pixels")
            .with_inner_size(LogicalSize::new(800.0, 600.0));

        let window = Arc::new(event_loop.create_window(attrs).unwrap());

        let mut pixels = {
            let window_size = window.inner_size();
            let surface_texture =
                SurfaceTexture::new(window_size.width, window_size.height, window.clone());
            Pixels::new(WIDTH, HEIGHT, surface_texture).unwrap()
        };

        pixels.set_scaling_mode(pixels::ScalingMode::Fill);

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
                if let PhysicalKey::Code(keycode) = event.physical_key {
                    if event.state.is_pressed() {
                        self.runtime.events(RenderCommands::KeyDown(keycode));
                        if keycode == KeyCode::KeyF {
                            let window = self.window.as_mut().unwrap();
                            if window.fullscreen().is_some() {
                                window.set_fullscreen(None);
                                println!("Modo Janela ativado");
                            } else {
                                window.set_fullscreen(Some(Fullscreen::Borderless(None)));
                                println!("Fullscreen ativado");
                            }
                        }
                    } else {
                        self.runtime.events(RenderCommands::KeyUp(keycode));
                    }
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.runtime.events(RenderCommands::MousePosition(
                    position.x as f32,
                    position.y as f32,
                ));
            }
            WindowEvent::RedrawRequested => {
                state.frame_mut().fill(255);
                self.runtime.update();

                self.runtime.render(&mut self.queue);
                self.render();

                let _ = self.state.as_mut().unwrap().render();
                self.window.as_mut().unwrap().request_redraw();
            }
            WindowEvent::Resized(size) => {
                if size.width > 0 && size.height > 0 {
                    let _ = state.resize_surface(size.width, size.height);

                    let win_w = size.width as f32;
                    let win_h = size.height as f32;
                    let ar_window = win_w / win_h;

                    let base_w = WIDTH as f32;
                    let base_h = HEIGHT as f32;

                    let ar_base = base_w / base_h;

                    let (target_w, target_h) = if ar_window < ar_base {
                        let calc_w = (base_h * ar_window).round() as u32;
                        (calc_w as u32, base_h as u32)
                    } else {
                        let calc_h = (base_w / ar_window).round() as u32;
                        (base_w as u32, calc_h.max(1))
                    };
                    self.window_size = (target_w, target_h);
                    let _ = state.resize_buffer(target_w, target_h);
                    if let Some(window) = &self.window {
                        window.request_redraw();
                    }
                }
            }
            _ => (),
        }
    }
}

pub fn run_application<R: Runtime>(runtime: R) {
    let event_loop = EventLoop::new().unwrap();

    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = Render::new(runtime);
    event_loop.run_app(&mut app).unwrap();
}
