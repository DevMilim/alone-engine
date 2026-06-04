use alone_engine::{
    AnimationData, Base, Body, Collider, Component, GameObject, GameObjectBase, KeyCode,
    SpriteAnimation, SpriteSrc, Vector2,
};

#[derive(GameObject)]
pub struct Player {
    #[base]
    base: Base,
    #[component]
    sprite_animation: Option<SpriteAnimation>,
    #[component]
    body: Body,
    #[component]
    collision: Collider,
}

impl Player {
    pub fn new() -> Self {
        Self {
            base: Base::new(Vector2::new(10.0, 10.0)),
            sprite_animation: None,
            collision: Collider {
                offset_y: 6.0,
                height: 12.0,
                width: 12.0,
                ..Default::default()
            },
            body: Body::default(),
        }
    }
}

impl GameObject for Player {
    type Message = ();
    fn start(&mut self, ctx: &mut impl alone_engine::EngineApi) {
        let mut animation = SpriteAnimation::new();
        let mut iddle_frames = AnimationData::default();

        let mut texture = SpriteSrc::new(
            ctx.load_texture("assets/sprites/knight.png"),
            Some(Vector2::new(32.0, 32.0)),
        );
        texture.set_src(0, 0);
        iddle_frames.insert_frame(texture.clone());
        texture.set_src(1, 0);
        iddle_frames.insert_frame(texture.clone());
        texture.set_src(2, 0);
        iddle_frames.insert_frame(texture.clone());
        texture.set_src(3, 0);
        iddle_frames.insert_frame(texture.clone());

        iddle_frames.set_fps(10.0);

        animation.new_animation(iddle_frames, "iddle");
        animation.play("iddle");
        self.sprite_animation = Some(animation)
    }
    fn fixed_update(&mut self, ctx: &mut impl alone_engine::EngineApi, delta: f32) {
        let gravity = 9.7;
        let speed = 13.0;
        let jump_speed = -3.0;
        let direction =
            ctx.get_key_vector(KeyCode::KeyW, KeyCode::KeyS, KeyCode::KeyA, KeyCode::KeyD);
        if !self.body.is_on_floor() {
            self.body.velocity.y += gravity * delta;
        }
        if ctx.is_key_just_pressed(KeyCode::Space) && self.body.is_on_floor() {
            self.body.velocity.y = jump_speed;
        }
    }
}
