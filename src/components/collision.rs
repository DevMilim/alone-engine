use crate::{
    AABB, Base, ColliderData, ColliderKey, Color, Component, EngineApi, Rect, RenderApi, Vector2,
};

pub struct Collider {
    pub key: u32,
    pub width: f32,
    pub height: f32,
    pub offset_x: f32,
    pub offset_y: f32,
    pub layer: u32,
    pub mask: u32,
    pub debug: bool,
    pub is_sensor: bool,
}

impl Default for Collider {
    fn default() -> Self {
        Self {
            key: 0,
            width: 16.0,
            height: 16.0,
            offset_x: 0.0,
            offset_y: 0.0,
            layer: 1,
            mask: 1,
            debug: false,
            is_sensor: false,
        }
    }
}

impl Component for Collider {
    fn update(&mut self, ctx: &mut impl EngineApi, base: &mut Base, _delta: f32) {
        let aabb = AABB {
            x: base.transform.global_position.x - (self.width / 2.0) + self.offset_x,
            y: base.transform.global_position.y - (self.height / 2.0) + self.offset_y,
            width: self.width,
            height: self.height,
        };

        let data = ColliderData {
            aabb,
            layer: self.layer,
            mask: self.mask,
            is_sensor: self.is_sensor,
        };

        ctx.update_collider(
            ColliderKey {
                key: self.key,
                id: base.id,
            },
            data,
        );
    }
    fn draw(&mut self, ctx: &mut impl RenderApi, base: &Base, _blending: f32) {
        if self.debug {
            let color = if self.is_sensor {
                Color::rgb(0, 0, 255)
            } else {
                Color::rgb(255, 0, 0)
            };
            let draw_pos = Vector2::new(
                base.transform.global_position.x - (self.width / 2.0) + self.offset_x,
                base.transform.global_position.y - (self.height / 2.0) + self.offset_y,
            );
            ctx.draw_rect(
                Rect::new(
                    draw_pos.x,
                    draw_pos.y,
                    self.width as u32,
                    self.height as u32,
                ),
                color,
                1,
            );
        }
    }
    fn destroy(&mut self, ctx: &mut impl EngineApi, base: &Base) {
        ctx.remove_collider(ColliderKey {
            key: self.key,
            id: base.id,
        });
    }
}
