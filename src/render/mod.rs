mod image;
mod render_queue;

pub use image::*;
pub use render_queue::*;

use winit::window::Window;

use std::{sync::Arc, time::Instant};

use pixels::{Pixels, SurfaceTexture};

use crate::{math::Vector2, resources::Resources};

pub const LOGICAL_WIDTH: u32 = 340;
pub const LOGICAL_HEIGHT: u32 = 180;

pub struct Render<'a> {
    pub pixels: Pixels<'a>,
    pub queue: [Vec<DrawCommand>; 6],
    pub window_size: (u32, u32),
    pub last_frame: Instant,
    pub frame_count: u32,
    pub fps_timer: Instant,
}

impl<'a> Render<'a> {
    pub fn new(window: &Arc<Window>) -> Self {
        let mut pixels = {
            let window_size = window.inner_size();
            let surface_texture =
                SurfaceTexture::new(window_size.width, window_size.height, window.clone());
            Pixels::new(LOGICAL_WIDTH, LOGICAL_HEIGHT, surface_texture).unwrap()
        };

        pixels.set_scaling_mode(pixels::ScalingMode::Fill);
        Self {
            pixels,
            queue: [const { Vec::new() }; 6],
            window_size: (LOGICAL_WIDTH, LOGICAL_HEIGHT),
            last_frame: Instant::now(),
            frame_count: 0,
            fps_timer: Instant::now(),
        }
    }
    pub fn sort(&mut self) {
        for queue in &mut self.queue {
            queue.sort_by_key(|cmd| match cmd {
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
    pub fn set_window_size(&mut self, size: (u32, u32)) {
        self.window_size = size;
    }
    pub fn render(&mut self, camera_position: Vector2, resources: &Resources) {
        self.sort();

        self.pixels.frame_mut().fill(255);
        let frame = self.pixels.frame_mut();

        let frame_width = self.window_size.0 as usize;
        let frame_height = self.window_size.1 as usize;

        let cam_x = camera_position.x.round();
        let cam_y = camera_position.y.round();

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
                        source,
                        flip_v,
                        flip_h,
                    } => {
                        let texture_id = image.id;
                        if Some(texture_id) != last_texture_id {
                            last_texture_id = Some(texture_id);
                            current_texture = resources.textures.get(*image);
                        }
                        if let Some(texture) = current_texture {
                            let tex_width = texture.width as usize;
                            let tex_height = texture.height as usize;

                            let (src_x, src_y, sprite_w, sprite_h) = match source {
                                Some(rect) => (
                                    rect.x as usize,
                                    rect.y as usize,
                                    rect.width as usize,
                                    rect.height as usize,
                                ),
                                None => (0, 0, tex_width, tex_height),
                            };

                            if src_x + sprite_w > tex_width || src_y + sprite_h > tex_height {
                                continue;
                            }

                            let (start_x, start_y) = match anchor {
                                Anchor::Center => {
                                    let center_x = position.x - (sprite_w as f32 / 2.0);
                                    let center_y = position.y - (sprite_h as f32 / 2.0);
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

                            let screen_max_x = (start_x + sprite_w as isize)
                                .min(frame_width as isize)
                                .max(0) as usize;
                            let screen_max_y = (start_y + sprite_h as isize)
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
                                let base_tex_y = (dst_y as isize - start_y) as usize;
                                let tex_y = if *flip_v {
                                    sprite_h - 1 - base_tex_y
                                } else {
                                    base_tex_y
                                };

                                let dst_row_start = dst_y * frame_width;
                                let tex_row_start = (src_y + tex_y) * tex_width;

                                let tex_min_x = (screen_min_x as isize - start_x) as usize;

                                let lenght = screen_max_x - screen_min_x;

                                let (actual_tex_x_start, actual_tex_x_end) = if *flip_h {
                                    (
                                        src_x + sprite_w - tex_min_x - lenght,
                                        src_x + sprite_w - tex_min_x,
                                    )
                                } else {
                                    (src_x + tex_min_x, src_x + tex_min_x + lenght)
                                };

                                let dst_row = &mut frame_pixels
                                    [dst_row_start + screen_min_x..dst_row_start + screen_max_x];

                                let tex_row = &tex_pixels[tex_row_start + actual_tex_x_start
                                    ..tex_row_start + actual_tex_x_end];

                                if *flip_h {
                                    for (dst_px, src_px) in
                                        dst_row.iter_mut().zip(tex_row.iter().rev())
                                    {
                                        Self::blending_pixel(dst_px, src_px);
                                    }
                                } else {
                                    for (dst_px, src_px) in dst_row.iter_mut().zip(tex_row.iter()) {
                                        Self::blending_pixel(dst_px, src_px);
                                    }
                                }
                            }
                        }
                    }
                    DrawCommand::Rect { color, rect } => {
                        let color_bytes = color.bytes();
                        let sa = color_bytes[3] as u32;

                        if sa == 0 {
                            continue;
                        }

                        let screen_x = (rect.x - cam_x).round();
                        let screen_y = (rect.y - cam_y).round();

                        let start_x = (screen_x as i32).max(0).min(frame_width as i32) as usize;
                        let start_y = (screen_y as i32).max(0).min(frame_height as i32) as usize;
                        let end_x = ((screen_x + rect.width as f32) as i32)
                            .max(0)
                            .min(frame_width as i32) as usize;
                        let end_y = ((screen_y + rect.height as f32) as i32)
                            .max(0)
                            .min(frame_height as i32) as usize;

                        if start_x >= end_x || start_y >= end_y {
                            continue;
                        }

                        if sa == 255 {
                            for y in start_y..end_y {
                                let row_start = y * frame_width + start_x;
                                let row_end = y * frame_width + end_x;

                                frame_pixels[row_start..row_end].fill(color_bytes);
                            }
                        } else {
                            let inv = 255u32 - sa;
                            let sr = color_bytes[0] as u32;
                            let sg = color_bytes[1] as u32;
                            let sb = color_bytes[2] as u32;

                            for y in start_y..end_y {
                                let row_start = y * frame_width + start_x;
                                let row_end = y * frame_width + end_x;

                                for dst_px in &mut frame_pixels[row_start..row_end] {
                                    let dr = dst_px[0] as u32;
                                    let dg = dst_px[1] as u32;
                                    let db = dst_px[2] as u32;

                                    *dst_px = [
                                        (((sr * sa + dr * inv + 128) * 257) >> 16) as u8,
                                        (((sg * sa + dg * inv + 128) * 257) >> 16) as u8,
                                        (((sb * sa + db * inv + 128) * 257) >> 16) as u8,
                                        255,
                                    ];
                                }
                            }
                        }
                    }
                }
            }
        }
        let _ = self.pixels.render();
        self.clear();
    }
    pub fn blending_pixel(dst_px: &mut [u8; 4], src_px: &[u8; 4]) {
        let sa = src_px[3] as u32;

        if sa == 0 {
            return;
        }

        if sa == 255 {
            *dst_px = *src_px;
        } else {
            let inv = 255u32 - sa;

            let sr = src_px[0] as u32;
            let sg = src_px[1] as u32;
            let sb = src_px[2] as u32;

            let dr = dst_px[0] as u32;
            let dg = dst_px[1] as u32;
            let db = dst_px[2] as u32;

            *dst_px = [
                (((sr * sa + dr * inv + 128) * 257) >> 16) as u8,
                (((sg * sa + dg * inv + 128) * 257) >> 16) as u8,
                (((sb * sa + db * inv + 128) * 257) >> 16) as u8,
                255,
            ]
        }
    }
}
