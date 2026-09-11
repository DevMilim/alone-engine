use std::sync::Arc;

use crate::{
    core::{Component, GameObjectBase, Handler},
    math::{Color, Vector2},
    render::FontAsset,
};

pub struct Text {
    pub content: Arc<str>,
    pub font: Option<Handler<FontAsset>>,
    pub color: Color,
    pub size_px: u32,
    pub offset: Vector2,
    pub z_index: u8,
    dirty: bool,
}

impl Text {
    pub fn new(content: impl Into<Arc<str>>, color: Color, size_px: u32) -> Self {
        Self {
            content: content.into(),
            font: None,
            color,
            size_px,
            offset: Vector2::ZERO,
            z_index: 0,
            dirty: true,
        }
    }
    pub fn set_content(&mut self, content: impl Into<Arc<str>>) {
        self.content = content.into();
        self.dirty = true;
    }
    pub fn set_font(&mut self, font: Handler<FontAsset>) {
        self.font = Some(font)
    }
}

impl Component for Text {
    fn update(
        &mut self,
        ctx: &mut impl crate::prelude::EngineApi,
        _base: &mut crate::prelude::Base,
        _delta: f32,
    ) {
        if self.dirty {
            if let Some(font) = self.font {
                ctx.ensure_text_glyphs(font, &self.content, self.size_px);
                self.dirty = false
            }
        }
    }
    fn draw(
        &mut self,
        renderer: &mut impl crate::prelude::RenderApi,
        base: &crate::prelude::Base,
        _blending: f32,
    ) {
        if let Some(font) = self.font {
            renderer.draw_text(
                self.content.clone(),
                font,
                self.offset + base.position(),
                self.color,
                self.size_px,
                self.z_index,
            );
        }
    }
}
