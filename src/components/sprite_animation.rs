use std::collections::HashMap;

use crate::{
    core::{Base, Component, EngineApi, GameObjectBase, RenderApi},
    math::Vector2,
    render::{Anchor, SpriteSrc},
};

#[derive(Debug)]
pub struct AnimationData {
    sprites: Vec<SpriteSrc>,
    fps: f32,
    looped: bool,
}

impl Default for AnimationData {
    fn default() -> Self {
        Self {
            sprites: Vec::new(),
            fps: 1.0,
            looped: true,
        }
    }
}

impl AnimationData {
    pub fn new(sprites: Vec<SpriteSrc>, fps: f32) -> Self {
        Self {
            sprites,
            fps,
            looped: true,
        }
    }
    pub fn empty() -> Self {
        Self {
            sprites: Vec::new(),
            fps: 1.0,
            looped: true,
        }
    }
    pub fn set_fps(&mut self, fps: f32) {
        self.fps = fps
    }
    pub fn looped(&mut self, looped: bool) {
        self.looped = looped
    }
    pub fn insert_frame(&mut self, texture: SpriteSrc) {
        self.sprites.push(texture);
    }
    pub fn insert_src(&mut self, texture: SpriteSrc) {
        self.sprites.push(texture);
    }
    pub fn frame_duration(&self) -> f32 {
        if self.fps <= 0.0 { 1.0 } else { 1.0 / self.fps }
    }
    fn frame_count(&self) -> usize {
        self.sprites.len()
    }
    fn get_sprite(&self, frame: usize) -> Option<SpriteSrc> {
        self.sprites.get(frame).copied()
    }
}

pub struct SpriteAnimation {
    animations: HashMap<String, AnimationData>,
    current_animation: Option<String>,

    visible: bool,
    anchor: Anchor,
    offset: Vector2,
    previous_position: Vector2,

    timer: f32,
    current_frame: usize,

    pub flip_v: bool,
    pub flip_h: bool,
}

impl Default for SpriteAnimation {
    fn default() -> Self {
        Self::new()
    }
}

impl SpriteAnimation {
    pub fn new() -> Self {
        Self {
            animations: HashMap::new(),
            current_animation: None,
            visible: true,
            anchor: Anchor::Center,
            offset: Vector2::ZERO,
            previous_position: Vector2::ZERO,
            timer: 0.0,
            current_frame: 0,
            flip_v: false,
            flip_h: false,
        }
    }
}

impl SpriteAnimation {
    pub fn play(&mut self, animation: &str) {
        if self.current_animation.as_deref() != Some(animation) {
            self.current_animation = Some(animation.to_owned());
            self.timer = 0.0;
            self.current_frame = 0;
        }
    }
    pub fn new_animation(&mut self, animation: AnimationData, animation_name: &str) {
        self.animations.insert(animation_name.to_owned(), animation);
    }
    pub fn anchor_center(&mut self) {
        self.anchor = Anchor::Center;
    }
}

impl Component for SpriteAnimation {
    fn start(&mut self, _ctx: &mut impl EngineApi, base: &mut Base) {
        self.previous_position = base.transform.global_position
    }
    fn update(&mut self, _ctx: &mut impl EngineApi, base: &mut Base, delta: f32) {
        self.previous_position = base.transform.global_position;

        let Some(animation_name) = self.current_animation.as_ref() else {
            return;
        };
        let Some(animation) = self.animations.get(animation_name) else {
            return;
        };

        let frame_time = animation.frame_duration();
        self.timer += delta;
        while self.timer >= frame_time {
            self.timer -= frame_time;
            self.current_frame += 1;
            if self.current_frame >= animation.frame_count() {
                if animation.looped {
                    self.current_frame = 0;
                } else {
                    self.current_frame = animation.frame_count().saturating_sub(1);
                    self.timer = 0.0;
                    break;
                }
            }
        }
    }
    fn draw(&mut self, renderer: &mut impl RenderApi, base: &Base, blending: f32) {
        if !self.visible {
            return;
        }
        let Some(animation_name) = self.current_animation.as_ref() else {
            return;
        };
        let Some(animation) = self.animations.get(animation_name) else {
            return;
        };

        let Some(texture) = animation.get_sprite(self.current_frame) else {
            return;
        };

        let current_position = self
            .previous_position
            .lerp(base.global_position(), blending);

        renderer.draw_sprite(
            current_position + self.offset,
            texture.texture,
            self.anchor,
            texture.src,
            self.flip_v,
            self.flip_h,
            base.z_index,
            base.transform.rotation,
        );
    }
}
