use alone_engine::{
    GameObject,
    components::{AnimationData, Body, BodyType, Camera, Collider, IBody, SpriteAnimation},
    core::{Base, Component, EngineApi, GameObject, GameObjectBase, Slot},
    input::KeyCode,
    math::{Vector2, Vector2i},
    render::SpriteSrc,
};

#[derive(GameObject)]
pub struct Player {
    #[base]
    base: Base,
    #[component]
    sprite_animation: Slot<SpriteAnimation>,
    #[component(interface = IBody)]
    body: Body,
    #[component]
    collision: Collider,
    #[component]
    camera: Camera,
}

impl Player {
    pub fn new() -> Self {
        Self {
            base: Base::new(Vector2::new(20.0 - 14.0, 10.0)),
            sprite_animation: Slot::default(),
            collision: Collider {
                offset_y: 6,
                height: 12,
                width: 12,
                follow_transform: false,
                ..Default::default()
            },
            body: Body {
                body_type: BodyType::Character,
                ..Default::default()
            },
            camera: Camera {
                active: true,
                ..Default::default()
            },
        }
    }
    pub fn create_idle_animation(&self, texture: &mut SpriteSrc) -> AnimationData {
        let mut idle_frames = AnimationData::default();
        for sprite in 0..3 {
            texture.set_src(sprite, 0);
            idle_frames.insert_frame(texture.clone());
        }

        idle_frames.set_fps(10.0);
        idle_frames
    }
}

impl GameObject for Player {
    fn start(&mut self, ctx: &mut impl EngineApi) {
        println!("Player inicializado");
        let mut animation = SpriteAnimation::new();

        let mut texture = SpriteSrc::new(
            ctx.load_texture(self.base.id, "assets/sprites/knight.png"),
            Some(Vector2i::new(32, 32)),
        );
        let iddle_frames = self.create_idle_animation(&mut texture);

        animation.new_animation(iddle_frames, "idle");
        animation.play("idle");
        self.sprite_animation = Slot::new(animation);

        let id = self.base.id.clone();
        println!("Player id{:?}", id);
    }
    fn fixed_update(&mut self, ctx: &mut impl EngineApi, delta: f32) {
        let gravity = 9.7;
        let speed = 150.0;
        let jump_speed = -200.0;

        self.velocity_mut().y += gravity;

        if ctx.is_key_just_pressed(KeyCode::Space) && self.is_on_floor() {
            self.velocity_mut().y = jump_speed;
        }

        let direction = ctx.get_key_axis(KeyCode::KeyA, KeyCode::KeyD);

        if direction < 0.0 {
            self.sprite_animation.get_mut().unwrap().flip_h = true;
        } else if direction > 0.0 {
            self.sprite_animation.get_mut().unwrap().flip_h = false
        }

        self.velocity_mut().x = speed * direction;
        self.move_and_slide(ctx, delta);
    }
}
