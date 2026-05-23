use std::{
    sync::Arc,
    thread::sleep,
    time::{Duration, Instant},
};

use pixels::{Pixels, SurfaceTexture};
use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::{Fullscreen, Window, WindowId},
};

use crate::{Color, DrawCommand, DrawCommandType, Engine, RenderApi, RenderCommands, Scene};
const WIDTH: u32 = 620;
const HEIGHT: u32 = 480;

pub struct RenderQueue {
    queue: [Vec<DrawCommand>; 6],
}
impl RenderQueue {
    pub fn new() -> Self {
        Self {
            queue: [const { Vec::new() }; 6],
        }
    }
    pub fn sort(&mut self) {
        for queue in &mut self.queue {
            queue.sort_unstable_by_key(|cmd| (cmd.cmd_type, cmd.material.image.id));
        }
    }
    pub fn clear(&mut self) {
        for queue in &mut self.queue {
            queue.clear();
        }
    }
}

impl RenderApi for RenderQueue {
    fn draw(&mut self, z_index: u8, command: DrawCommand) {
        self.queue[z_index as usize].push(command);
    }
}

struct Render<'a, S: Scene> {
    state: Option<Pixels<'a>>,
    window: Option<Arc<Window>>,
    queue: RenderQueue,
    runtime: Engine<S>,
    window_size: (u32, u32),
    last_frame: Instant,
    fps: f64,

    frame_count: u32,
    fps_timer: Instant,
}

impl<'a, S: Scene> Render<'a, S> {
    pub fn new(main_scene: S) -> Self {
        Self {
            state: None,
            window: None,
            queue: RenderQueue::new(),
            runtime: Engine::new(main_scene),
            window_size: (WIDTH, HEIGHT),
            last_frame: Instant::now(),
            fps: 60.0,
            frame_count: 0,
            fps_timer: Instant::now(),
        }
    }
    pub fn render(&mut self) {
        let frame = self.state.as_mut().unwrap().frame_mut();
        self.queue.sort();

        let frame_width = self.window_size.0 as usize;
        let frame_height = self.window_size.1 as usize;

        for layer in self.queue.queue.iter() {
            let mut last_texture_id = None;
            let mut current_texture = None;
            for cmd in layer {
                match cmd.cmd_type {
                    DrawCommandType::Sprite => {
                        let texture_id = cmd.material.image.id;
                        if Some(texture_id) != last_texture_id {
                            last_texture_id = Some(texture_id);
                            current_texture = self.runtime.get_texture(cmd.material.image);
                        }
                        if let Some(texture) = current_texture {
                            let tex_width = texture.width as usize;

                            let position = &cmd.material.rect;
                            for y in 0..texture.height as usize {
                                let dst_y = position.y as isize + y as isize;
                                if dst_y < 0 || dst_y >= frame_height as isize {
                                    continue;
                                }
                                for x in 0..tex_width {
                                    let dst_x = position.x as isize + x as isize;
                                    if dst_x >= frame_width as isize
                                        || dst_y >= frame_height as isize
                                    {
                                        continue;
                                    }
                                    let src = (y * tex_width + x) * 4;

                                    let alpha = texture.pixels[src + 3];
                                    if alpha == 0 {
                                        continue;
                                    }
                                    let dst = (dst_y as usize * frame_width + dst_x as usize) * 4;

                                    let mut frame_pixel_buffer = [0u8; 4];
                                    frame_pixel_buffer.copy_from_slice(&frame[dst..dst + 4]);

                                    let mut texture_pixel_buffer = [0u8; 4];
                                    texture_pixel_buffer
                                        .copy_from_slice(&texture.pixels[src..src + 4]);

                                    let blended =
                                        Color::blend(&frame_pixel_buffer, &texture_pixel_buffer);

                                    frame[dst..dst + 4].copy_from_slice(&blended);
                                }
                            }
                        }
                    }
                    DrawCommandType::Rect => {
                        let rect = &cmd.material.rect;
                        let color_bytes = cmd.material.color.bytes();

                        let start_x = (rect.x as i32).max(0).min(frame_width as i32) as usize;
                        let start_y = (rect.y as i32).max(0).min(frame_height as i32) as usize;
                        let end_x = ((rect.x + rect.width as f32) as i32)
                            .max(0)
                            .min(frame_width as i32) as usize;
                        let end_y = ((rect.y + rect.height as f32) as i32)
                            .max(0)
                            .min(frame_height as i32) as usize;
                        for y in start_y..end_y {
                            for x in start_x..end_x {
                                let dst = (y * frame_width + x) * 4;
                                frame[dst..dst + 4].copy_from_slice(&color_bytes);
                            }
                        }
                    }
                }
            }
        }
        self.queue.clear();
    }
}

impl<'a, S: Scene> ApplicationHandler for Render<'a, S> {
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
                let target_ft = Duration::from_secs_f64(1.0 / self.fps);
                let elapsed = self.last_frame.elapsed();

                if elapsed < target_ft {
                    sleep(target_ft - elapsed);
                }

                self.frame_count += 1;

                if self.fps_timer.elapsed() >= Duration::from_secs(1) {
                    println!("> FPS: {}", self.frame_count);
                    self.frame_count = 0;
                    self.fps_timer = Instant::now();
                }

                self.last_frame = Instant::now();
                state.frame_mut().fill(255);
                self.runtime.update_step();

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

pub fn run_application<S: Scene>(runtime: S) {
    let event_loop = EventLoop::new().unwrap();

    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = Render::new(runtime);
    event_loop.run_app(&mut app).unwrap();
}
