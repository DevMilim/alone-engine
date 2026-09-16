mod font;
mod image;
mod rasterizer;
mod render_queue;

pub use font::*;
pub use image::*;
pub use rasterizer::*;
pub use render_queue::*;

use pixels::{Pixels, PixelsBuilder, SurfaceTexture};
use rayon::prelude::*;
use std::{sync::Arc, time::Instant};
use winit::window::Window;

use crate::{math::Vector2, resources::Resources};

pub const LOGICAL_WIDTH: u32 = 480;
pub const LOGICAL_HEIGHT: u32 = 270;

const PARALLEL_COMMAND_THRESHOLD: usize = 1500;
const SWITCH_UP_MS: f32 = 3.0;
const SWITCH_DOWN_MS: f32 = 1.5;
const RASTER_TIME_EMA_ALPHA: f32 = 0.1;

pub struct Render<'a> {
    pub pixels: Pixels<'a>,
    pub queue: [Vec<DrawCommand>; 6],
    pub window_size: (u32, u32),
    pub last_frame: Instant,
    pub frame_count: u32,
    pub fps_timer: Instant,
    bands: Vec<[Vec<u32>; 6]>,
    avg_raster_time: f32,
}

impl<'a> Render<'a> {
    pub fn new(window: &Arc<Window>) -> Self {
        let mut pixels = {
            let window_size = window.inner_size();
            let surface_texture =
                SurfaceTexture::new(window_size.width, window_size.height, window.clone());
            PixelsBuilder::new(LOGICAL_WIDTH, LOGICAL_HEIGHT, surface_texture).build()
        }
        .unwrap();

        pixels.set_scaling_mode(pixels::ScalingMode::Fill);
        Self {
            pixels,
            queue: [const { Vec::new() }; 6],
            window_size: (LOGICAL_WIDTH, LOGICAL_HEIGHT),
            last_frame: Instant::now(),
            frame_count: 0,
            fps_timer: Instant::now(),
            bands: Vec::new(),
            avg_raster_time: 0.001,
        }
    }

    pub fn sort(&mut self) {
        for queue in &mut self.queue {
            queue.sort_unstable_by_key(|cmd| match cmd {
                DrawCommand::Sprite { image, .. } => (1, image.id),
                DrawCommand::Rect { .. } => (0, 0),
                DrawCommand::Line { .. } => (0, 0),
                DrawCommand::Text { .. } => (0, 0),
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

    fn command_y_bounds(
        cmd: &DrawCommand,
        cam_y: f32,
        resources: &Resources,
    ) -> Option<(isize, isize)> {
        match cmd {
            DrawCommand::Rect { rect, .. } => {
                let top = rect.y as f32 - cam_y;
                let bottom = top + rect.height as f32;
                Some((top.floor() as isize, bottom.ceil() as isize))
            }
            DrawCommand::Sprite {
                position,
                image,
                anchor,
                source,
                rotation,
                ..
            } => {
                let texture = resources.textures.get(*image)?;

                let (sprite_w, sprite_h) = match source {
                    Some(rect) => (rect.width as f32, rect.height as f32),
                    None => (texture.width as f32, texture.height as f32),
                };

                let hw = sprite_w / 2.0;
                let hh = sprite_h / 2.0;

                let cy = match anchor {
                    Anchor::Center => position.y - cam_y,
                    Anchor::TopLeft => position.y - cam_y + hh,
                };

                let bbox_hh = if rotation.abs() < 0.001 {
                    hh
                } else {
                    let (_, cos) = rotation.sin_cos();
                    hw * rotation.sin().abs() + hh * cos.abs()
                };

                Some((
                    (cy - bbox_hh).floor() as isize,
                    (cy + bbox_hh).ceil() as isize,
                ))
            }
            DrawCommand::Line {
                start,
                end,
                thickness,
                ..
            } => {
                let y0 = start.y - cam_y;
                let y1 = end.y - cam_y;
                let (top, bottom) = if y0 <= y1 { (y0, y1) } else { (y1, y0) };
                let pad = (thickness / 2.0).max(1.0);
                Some(((top - pad).floor() as isize, (bottom + pad).ceil() as isize))
            }
            DrawCommand::Text {
                text,
                font,
                position,
                size_px,
                ..
            } => {
                let font_obj = resources.fonts.get(*font)?;

                let width: f32 = text
                    .chars()
                    .map(|ch| font_obj.font.metrics(ch, *size_px as f32).advance_width)
                    .sum();

                let top = position.y - cam_y - *size_px as f32;
                let bottom = position.y - cam_y;

                let _ = width;
                Some((top.floor() as isize, bottom.ceil() as isize))
            }
        }
    }

    pub fn render_auto(&mut self, camera_position: Vector2, resources: &Resources) {
        let total_commands: usize = self.queue.iter().map(|layer| layer.len()).sum();
        let avg_ms = self.avg_raster_time * 1000.0;

        let use_parallel = if avg_ms > SWITCH_UP_MS {
            true
        } else if avg_ms < SWITCH_DOWN_MS {
            false
        } else {
            total_commands > PARALLEL_COMMAND_THRESHOLD
        };

        if use_parallel {
            self.render_paralel(camera_position, resources);
        } else {
            self.render_sequential(camera_position, resources);
        }
    }

    pub fn render_sequential(&mut self, camera_position: Vector2, resources: &Resources) {
        let t0 = Instant::now();

        self.sort();
        self.pixels.frame_mut().fill(255);

        let frame = self.pixels.frame_mut();

        let frame_width = self.window_size.0 as usize;
        let frame_height = self.window_size.1 as usize;

        let cam_x = camera_position.x;
        let cam_y = camera_position.y;

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
                        rotation,
                    } => {
                        let texture_id = image.id;
                        if Some(texture_id) != last_texture_id {
                            last_texture_id = Some(texture_id);
                            current_texture = resources.textures.get(*image);
                        }
                        let Some(texture) = current_texture else {
                            continue;
                        };

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

                        let tex_pixels: &[[u8; 4]] = unsafe {
                            std::slice::from_raw_parts(
                                texture.pixels.as_ptr() as *const [u8; 4],
                                texture.pixels.len() / 4,
                            )
                        };

                        Rasterizer::blit_sprite(
                            frame_pixels,
                            frame_width,
                            0,
                            0,
                            frame_height,
                            tex_pixels,
                            tex_width,
                            src_x,
                            src_y,
                            sprite_w,
                            sprite_h,
                            anchor,
                            *position,
                            cam_x,
                            cam_y,
                            *rotation,
                            *flip_h,
                            *flip_v,
                        );
                    }
                    DrawCommand::Text {
                        text,
                        font,
                        position,
                        color,
                        size_px,
                    } => {
                        Rasterizer::blit_text(
                            frame_pixels,
                            frame_width,
                            0,
                            0,
                            frame_height,
                            resources,
                            *font,
                            text,
                            *position,
                            *color,
                            *size_px,
                            cam_x,
                            cam_y,
                        );
                    }
                    DrawCommand::Line {
                        start,
                        end,
                        color,
                        thickness,
                    } => {
                        Rasterizer::blit_line(
                            frame_pixels,
                            frame_width,
                            0,
                            0,
                            frame_height,
                            color.bytes(),
                            *start,
                            *end,
                            *thickness,
                            cam_x,
                            cam_y,
                        );
                    }
                    DrawCommand::Rect { color, rect } => {
                        Rasterizer::blit_rect(
                            frame_pixels,
                            frame_width,
                            0,
                            0,
                            frame_height,
                            color.bytes(),
                            rect.x as f32,
                            rect.y as f32,
                            rect.width as f32,
                            rect.height as f32,
                            cam_x,
                            cam_y,
                        );
                    }
                }
            }
        }

        let raster_time = t0.elapsed().as_secs_f32();
        self.avg_raster_time = self.avg_raster_time * (1.0 - RASTER_TIME_EMA_ALPHA)
            + raster_time * RASTER_TIME_EMA_ALPHA;

        let _ = self.pixels.render();
        self.clear();
    }

    pub fn render_paralel(&mut self, camera_position: Vector2, resources: &Resources) {
        let t0 = Instant::now();

        self.sort();

        self.pixels.frame_mut().fill(255);

        let frame_width = self.window_size.0 as usize;
        let frame_height = self.window_size.1 as usize;

        let cam_x = camera_position.x;
        let cam_y = camera_position.y;

        let num_bands = rayon::current_num_threads().min(frame_height).max(1);
        let rows_per_band = frame_height.div_ceil(num_bands);

        if self.bands.len() != num_bands {
            self.bands = (0..num_bands).map(|_| [const { Vec::new() }; 6]).collect();
        } else {
            for band in &mut self.bands {
                for layer in band {
                    layer.clear();
                }
            }
        }

        for (layer_idx, layer) in self.queue.iter().enumerate() {
            for (cmd_idx, cmd) in layer.iter().enumerate() {
                let Some((min_y, max_y)) = Self::command_y_bounds(cmd, cam_y, resources) else {
                    continue;
                };

                let clipped_min = min_y.max(0) as usize;
                let clipped_max = (max_y.max(0) as usize).min(frame_height);
                if clipped_min >= clipped_max {
                    continue;
                }

                let band_start = (clipped_min / rows_per_band).min(num_bands - 1);
                let band_end = ((clipped_max - 1) / rows_per_band).min(num_bands - 1);

                for band in &mut self.bands[band_start..=band_end] {
                    band[layer_idx].push(cmd_idx as u32);
                }
            }
        }

        let queue = &self.queue;
        let bands = &self.bands;

        let frame = self.pixels.frame_mut();
        let frame_pixels: &mut [[u8; 4]] = unsafe {
            std::slice::from_raw_parts_mut(frame.as_mut_ptr() as *mut [u8; 4], frame.len() / 4)
        };

        frame_pixels
            .par_chunks_mut(frame_width * rows_per_band)
            .zip(bands.par_iter())
            .enumerate()
            .for_each(|(band_idx, (band_pixels, band))| {
                let y0 = band_idx * rows_per_band;
                let y1 = (y0 + rows_per_band).min(frame_height);

                for (layer_idx, indices) in band.iter().enumerate() {
                    let layer = &queue[layer_idx];
                    let mut last_texture_id = None;
                    let mut current_texture = None;

                    for &cmd_idx in indices {
                        let cmd = &layer[cmd_idx as usize];
                        match cmd {
                            DrawCommand::Sprite {
                                position,
                                image,
                                anchor,
                                source,
                                flip_v,
                                flip_h,
                                rotation,
                            } => {
                                let texture_id = image.id;
                                if Some(texture_id) != last_texture_id {
                                    last_texture_id = Some(texture_id);
                                    current_texture = resources.textures.get(*image);
                                }
                                let Some(texture) = current_texture else {
                                    continue;
                                };

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

                                let tex_pixels: &[[u8; 4]] = unsafe {
                                    std::slice::from_raw_parts(
                                        texture.pixels.as_ptr() as *const [u8; 4],
                                        texture.pixels.len() / 4,
                                    )
                                };

                                Rasterizer::blit_sprite(
                                    band_pixels,
                                    frame_width,
                                    y0,
                                    y0,
                                    y1,
                                    tex_pixels,
                                    tex_width,
                                    src_x,
                                    src_y,
                                    sprite_w,
                                    sprite_h,
                                    anchor,
                                    *position,
                                    cam_x,
                                    cam_y,
                                    *rotation,
                                    *flip_h,
                                    *flip_v,
                                );
                            }
                            DrawCommand::Text {
                                text,
                                font,
                                position,
                                color,
                                size_px,
                            } => {
                                Rasterizer::blit_text(
                                    band_pixels,
                                    frame_width,
                                    y0,
                                    y0,
                                    y1,
                                    resources,
                                    *font,
                                    text,
                                    *position,
                                    *color,
                                    *size_px,
                                    cam_x,
                                    cam_y,
                                );
                            }
                            DrawCommand::Line {
                                start,
                                end,
                                color,
                                thickness,
                            } => {
                                Rasterizer::blit_line(
                                    band_pixels,
                                    frame_width,
                                    y0,
                                    y0,
                                    y1,
                                    color.bytes(),
                                    *start,
                                    *end,
                                    *thickness,
                                    cam_x,
                                    cam_y,
                                );
                            }
                            DrawCommand::Rect { color, rect } => {
                                Rasterizer::blit_rect(
                                    band_pixels,
                                    frame_width,
                                    y0,
                                    y0,
                                    y1,
                                    color.bytes(),
                                    rect.x as f32,
                                    rect.y as f32,
                                    rect.width as f32,
                                    rect.height as f32,
                                    cam_x,
                                    cam_y,
                                );
                            }
                        }
                    }
                }
            });

        let raster_time = t0.elapsed().as_secs_f32();
        self.avg_raster_time = self.avg_raster_time * (1.0 - RASTER_TIME_EMA_ALPHA)
            + raster_time * RASTER_TIME_EMA_ALPHA;

        let _ = self.pixels.render();
        self.clear();
    }
}
