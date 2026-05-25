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

use crate::{Anchor, DrawCommand, Engine, Rect, RenderApi, RenderCommands, Scene};
pub const LOGICAL_WIDTH: u32 = 854;
pub const LOGICAL_HEIGHT: u32 = 480;

struct Render<'a, S: Scene> {
    state: Option<Pixels<'a>>,
    window: Option<Arc<Window>>,
    queue: [Vec<DrawCommand>; 6],
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
            queue: [const { Vec::new() }; 6],
            runtime: Engine::new(main_scene),
            window_size: (LOGICAL_WIDTH, LOGICAL_HEIGHT),
            last_frame: Instant::now(),
            frame_count: 0,
            fps_timer: Instant::now(),
        }
    }
    pub fn sort(&mut self) {
        for queue in &mut self.queue {
            queue.sort_unstable_by_key(|cmd| match cmd {
                DrawCommand::Sprite { image, .. } => (1, image.id),
                DrawCommand::Rect { .. } => (0, 0),
            });
        }
    }
    pub fn clear(&mut self) {
        for queue in &mut self.queue {
            queue.clear();
        }
    }
    pub fn render(&mut self) {
        self.sort();
        let frame = self.state.as_mut().unwrap().frame_mut();

        let frame_width = self.window_size.0 as usize;
        let frame_height = self.window_size.1 as usize;

        let cam_x = self.runtime.camera_pos.x.round();
        let cam_y = self.runtime.camera_pos.y.round();

        let frame_pixels: &mut [[u8; 4]] = unsafe {
            std::slice::from_raw_parts_mut(frame.as_mut_ptr() as *mut [u8; 4], frame.len() / 4)
        };

        for layer in self.queue.iter() {
            let mut last_texture_id = None;
            let mut current_texture = None;
            for cmd in layer {
                match cmd {
                    DrawCommand::Sprite {
                        position,
                        image,
                        anchor,
                    } => {
                        let texture_id = image.id;
                        if Some(texture_id) != last_texture_id {
                            last_texture_id = Some(texture_id);
                            current_texture = self.runtime.get_texture(*image);
                        }
                        if let Some(texture) = current_texture {
                            let tex_width = texture.width as usize;
                            let tex_height = texture.height as usize;

                            let (start_x, start_y) = match anchor {
                                Anchor::Center => {
                                    let center_x = position.x - (tex_width as f32 / 2.0);
                                    let center_y = position.y - (tex_height as f32 / 2.0);
                                    (
                                        (center_x - cam_x).round() as isize,
                                        (center_y - cam_y).round() as isize,
                                    )
                                }
                                Anchor::TopLeft => (
                                    (position.x - cam_x).round() as isize,
                                    (position.y - cam_y).round() as isize,
                                ),
                            };
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
                    DrawCommand::Rect { color, rect } => {
                        let color_bytes = color.bytes();

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
        self.clear();
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
                if let Ok(mouse) = state.window_pos_to_pixel((position.x as f32, position.y as f32))
                {
                    self.runtime.events(RenderCommands::MousePosition(
                        mouse.0 as f32,
                        mouse.1 as f32,
                    ));
                }
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
                let (is_running, blending) = self.runtime.update_step();

                self.runtime.render(&mut self.queue, blending);
                self.render();

                let _ = self.state.as_mut().unwrap().render();
                if !is_running {
                    event_loop.exit();
                }
                self.window.as_mut().unwrap().request_redraw();
            }
            WindowEvent::Resized(size) => {
                if size.width > 0 && size.height > 0 {
                    let _ = state.resize_surface(size.width, size.height);

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
