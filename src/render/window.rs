use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use pixels::{Pixels, SurfaceTexture};
use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::PhysicalKey,
    window::{Window, WindowId},
};

use crate::{
    Color, DrawCommand, DrawCommandType, DrawData, Engine, Rect, RenderApi, RenderCommands, Scene,
};
pub const LOGICAL_WIDTH: u32 = 620;
pub const LOGICAL_HEIGHT: u32 = 480;

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

    fn draw_rect(&mut self, rect: Rect, color: Color, z_index: u8) {
        self.queue[z_index as usize].push(DrawCommand {
            cmd_type: DrawCommandType::Rect,
            material: DrawData {
                color,
                rect,
                ..Default::default()
            },
        });
    }
}

struct Render<'a, S: Scene> {
    state: Option<Pixels<'a>>,
    window: Option<Arc<Window>>,
    queue: RenderQueue,
    runtime: Engine<S>,
    window_size: (u32, u32),
    last_frame: Instant,

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
            window_size: (LOGICAL_WIDTH, LOGICAL_HEIGHT),
            last_frame: Instant::now(),
            frame_count: 0,
            fps_timer: Instant::now(),
        }
    }
    pub fn render(&mut self) {
        let frame = self.state.as_mut().unwrap().frame_mut();
        self.queue.sort();

        let frame_width = self.window_size.0 as usize;
        let frame_height = self.window_size.1 as usize;

        let cam_x = self.runtime.camera_pos.x.round();
        let cam_y = self.runtime.camera_pos.y.round();

        let frame_pixels: &mut [[u8; 4]] = unsafe {
            std::slice::from_raw_parts_mut(frame.as_mut_ptr() as *mut [u8; 4], frame.len() / 4)
        };

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
                            let tex_height = texture.height as usize;
                            let position = &cmd.material.rect;

                            let start_x = (position.x - cam_x).round() as isize;
                            let start_y = (position.y - cam_y).round() as isize;

                            let screen_min_x = start_x.max(0) as usize;
                            let screen_min_y = start_y.max(0) as usize;

                            let screen_max_x = (start_x + tex_width as isize)
                                .min(frame_width as isize)
                                .max(0) as usize;
                            let screen_max_y = (start_y + tex_height as isize)
                                .min(frame_height as isize)
                                .max(0) as usize;

                            if screen_min_x >= screen_max_x || screen_min_y >= screen_max_y {
                                continue;
                            }

                            let tex_pixels: &[[u8; 4]] = unsafe {
                                std::slice::from_raw_parts(
                                    texture.pixels.as_ptr() as *const [u8; 4],
                                    texture.pixels.len() / 4,
                                )
                            };

                            for dst_y in screen_min_y..screen_max_y {
                                let tex_y = (dst_y as isize - start_y) as usize;

                                let dst_row_start = dst_y * frame_width;
                                let tex_row_start = tex_y * tex_width;

                                for dst_x in screen_min_x..screen_max_x {
                                    let tex_x = (dst_x as isize - start_x) as usize;

                                    let src_px = tex_pixels[tex_row_start + tex_x];

                                    let sa = src_px[3];

                                    if sa == 0 {
                                        continue;
                                    }
                                    let dst_idx = dst_row_start + dst_x;
                                    if sa == 255 {
                                        frame_pixels[dst_idx] = src_px;
                                    } else {
                                        let dst_px = frame_pixels[dst_idx];
                                        let inv = 255u32 - sa as u32;

                                        let sr = src_px[0] as u32;
                                        let sg = src_px[1] as u32;
                                        let sb = src_px[2] as u32;

                                        let dr = dst_px[0] as u32;
                                        let dg = dst_px[1] as u32;
                                        let db = dst_px[2] as u32;

                                        frame_pixels[dst_idx] = [
                                            (((sr * sa as u32 + dr * inv + 128) * 257) >> 16) as u8,
                                            (((sg * sa as u32 + dg * inv + 128) * 257) >> 16) as u8,
                                            (((sb * sa as u32 + db * inv + 128) * 257) >> 16) as u8,
                                            255,
                                        ]
                                    }
                                }
                            }
                        }
                    }
                    DrawCommandType::Rect => {
                        let rect = &cmd.material.rect;
                        let color_bytes = cmd.material.color.bytes();

                        let screen_x = rect.x - cam_x;
                        let screen_y = rect.y - cam_y;

                        let start_x = (screen_x as i32).max(0).min(frame_width as i32) as usize;
                        let start_y = (screen_y as i32).max(0).min(frame_height as i32) as usize;
                        let end_x = ((screen_x + rect.width as f32) as i32)
                            .max(0)
                            .min(frame_width as i32) as usize;
                        let end_y = ((screen_y + rect.height as f32) as i32)
                            .max(0)
                            .min(frame_height as i32) as usize;
                        for y in start_y..end_y {
                            let row_start = y * frame_width + start_x;
                            let row_end = y * frame_width + end_x;

                            frame_pixels[row_start..row_end].fill(color_bytes);
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
            .with_inner_size(LogicalSize::new(LOGICAL_WIDTH, LOGICAL_HEIGHT));

        let window = Arc::new(event_loop.create_window(attrs).unwrap());

        let mut pixels = {
            let window_size = window.inner_size();
            let surface_texture =
                SurfaceTexture::new(window_size.width, window_size.height, window.clone());
            Pixels::new(LOGICAL_WIDTH, LOGICAL_HEIGHT, surface_texture).unwrap()
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

                    let base_w = LOGICAL_WIDTH as f32;
                    let base_h = LOGICAL_HEIGHT as f32;

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
