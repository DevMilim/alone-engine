use std::time::Duration;

use alone_engine::{
    GameObject,
    components::{Collider, Sprite},
    core::{Base, Component, EngineApi, GameObject, GameObjectBase},
    math::Vector2,
    render::SpriteSrc,
};

#[derive(GameObject)]
pub struct Platform {
    #[base]
    base: Base,
    #[component]
    sprite: Option<Sprite>,
    #[component]
    collision: Collider,
    start_point: Vector2,
    end_point: Vector2,
    duration: f32,
    current_time: f32,
    direction: f32,
}

impl Platform {
    pub fn new(start_point: Vector2, end_point: Vector2, time: Duration) -> Self {
        Self {
            base: Base::default(),
            sprite: None,
            collision: Collider {
                one_way_collision: true,
                width: 32.0,
                height: 8.0,
                ..Default::default()
            },
            start_point,
            end_point,
            duration: time.as_secs_f32(),
            current_time: 0.0,
            direction: 1.0,
        }
    }
}

impl GameObject for Platform {
    type Message = ();
    fn start(&mut self, ctx: &mut impl EngineApi) {
        let mut sprite = SpriteSrc::new(
            ctx.load_texture("assets/sprites/platforms.png"),
            Some(Vector2::new(16.0, 10.0)),
        );
        sprite.set_src(1, 0);
        sprite.add_tile(2, 0);
        self.sprite = Some(Sprite {
            texture: sprite,
            ..Default::default()
        });
        self.base.set_position(self.start_point);
    }
    fn fixed_update(&mut self, _ctx: &mut impl EngineApi, delta: f32) {
        self.current_time += delta * self.direction;

        if self.current_time >= self.duration {
            self.current_time = self.duration;
            self.direction = -1.0;
        } else if self.current_time <= 0.0 {
            self.current_time = 0.0;
            self.direction = 1.0;
        }
        let t = self.current_time / self.duration;
        let new_position = self.start_point.lerp(self.end_point, t);
        self.base.set_position(new_position);
    }
}
