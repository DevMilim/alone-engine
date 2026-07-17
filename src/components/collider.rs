use std::any::Any;

use crate::{
    collision::{AABB, ColliderData, ColliderKey},
    core::{Base, Component, EngineApi, Id, RenderApi},
    math::{Color, Rect, Vector2i},
};

pub enum ColliderType {
    World,
    Box,
}

pub struct Collider {
    pub key: Id,
    pub width: i32,
    pub height: i32,
    pub offset_x: i32,
    pub offset_y: i32,
    pub layer: u32,
    pub mask: u32,
    pub debug: bool,
    pub is_sensor: bool,
    pub collider_type: ColliderType,
    pub one_way_collision: bool,
    pub event: Option<Box<dyn Fn() -> Box<dyn Any + 'static>>>,

    pub follow_transform: bool,
}

impl Collider {
    pub fn set_event<T: Clone + 'static>(&mut self, event: T) {
        self.event = Some(Box::new(move || Box::new(event.clone())));
    }
}

impl Default for Collider {
    fn default() -> Self {
        Self {
            key: Id::new(),
            width: 16,
            height: 16,
            offset_x: 0,
            offset_y: 0,
            layer: 1,
            mask: 1,
            debug: false,
            is_sensor: false,
            collider_type: ColliderType::Box,
            event: None,
            one_way_collision: false,
            follow_transform: true,
        }
    }
}

impl Component for Collider {
    fn fixed_update(&mut self, ctx: &mut impl EngineApi, base: &mut Base, _delta: f32) {
        let key = ColliderKey {
            key: self.key,
            id: base.id,
        };
        let iglobal_position: Vector2i = base.transform.global_position.into();
        let pos_x = iglobal_position.x - (self.width / 2) + self.offset_x;
        let pos_y = iglobal_position.y - (self.height / 2) + self.offset_y;
        if self.follow_transform {
            let data = ColliderData {
                aabb: AABB {
                    x: pos_x,
                    y: pos_y,
                    width: self.width,
                    height: self.height,
                },
                layer: self.layer,
                mask: self.mask,
                is_sensor: self.is_sensor,
                on_way_collision: self.one_way_collision,
            };

            ctx.update_collider(key, data);
        } else {
            ctx.update_collider_geometry(
                key,
                self.layer,
                self.mask,
                self.is_sensor,
                self.one_way_collision,
                (self.width, self.height),
                (pos_x, pos_y),
            );
        }
    }
    fn draw(&mut self, ctx: &mut impl RenderApi, base: &Base, _blending: f32) {
        if self.debug {
            let color = if self.is_sensor {
                Color::rgba(0, 0, 255, 255 / 2)
            } else {
                Color::rgba(255, 0, 0, 255 / 2)
            };
            let iglobal_position: Vector2i = base.transform.global_position.into();
            let draw_pos = Vector2i::new(
                iglobal_position.x - (self.width / 2) + self.offset_x,
                iglobal_position.y - (self.height / 2) + self.offset_y,
            );
            ctx.draw_rect(
                Rect::new(draw_pos.x, draw_pos.y, self.width, self.height),
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
