use alone_engine::{
    GameObject,
    components::{AnimationData, Body, Camera, Collider, SpriteAnimation},
    core::{Base, Component, EngineApi, GameObject, GameObjectBase},
    input::KeyCode,
    math::Vector2,
    render::SpriteSrc,
    sleep_tokio,
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
    #[component]
    camera: Camera,
}

impl Player {
    pub fn new() -> Self {
        Self {
            base: Base::new(Vector2::new(20.0, 10.0)),
            sprite_animation: None,
            collision: Collider {
                offset_y: 6.0,
                height: 12.0,
                width: 12.0,
                ..Default::default()
            },
            body: Body::default(),
            camera: Camera {
                active: true,
                ..Default::default()
            },
        }
    }
    pub fn create_iddle_animation(&self, texture: &mut SpriteSrc) -> AnimationData {
        let mut iddle_frames = AnimationData::default();
        for sprite in 0..3 {
            texture.set_src(sprite, 0);
            iddle_frames.insert_frame(texture.clone());
        }

        iddle_frames.set_fps(10.0);
        iddle_frames
    }
}

pub enum PlayerMessage {
    AsyncTask,
}

impl GameObject for Player {
    type Message = PlayerMessage;
    fn start(&mut self, ctx: &mut impl EngineApi) {
        println!("Player inicializado");
        let mut animation = SpriteAnimation::new();

        let mut texture = SpriteSrc::new(
            ctx.load_texture("assets/sprites/knight.png"),
            Some(Vector2::new(32.0, 32.0)),
        );
        let iddle_frames = self.create_iddle_animation(&mut texture);

        animation.new_animation(iddle_frames, "iddle");
        animation.play("iddle");
        self.sprite_animation = Some(animation);
        let async_ctx = ctx.async_ctx();

        let id = self.base.id.clone();
        println!("Player id{:?}", id);

        ctx.async_task(async move {
            sleep_tokio(5.0).await;
            async_ctx.send(id, PlayerMessage::AsyncTask);
        });
    }
    fn fixed_update(&mut self, ctx: &mut impl EngineApi, delta: f32) {
        let gravity = 9.7;
        let speed = 90.0;
        let jump_speed = -200.0 * delta;

        if !self.body.is_on_floor() {
            self.body.velocity.y += gravity * delta;
        }
        if ctx.is_key_just_pressed(KeyCode::Space) && self.body.is_on_floor() {
            self.body.velocity.y = jump_speed;
        }
        let direction = ctx.get_key_axis(KeyCode::KeyA, KeyCode::KeyD);
        if direction < 0.0 {
            self.sprite_animation.as_mut().unwrap().flip_h = true;
        } else if direction > 0.0 {
            self.sprite_animation.as_mut().unwrap().flip_h = false
        }
        self.body.velocity.x = speed * direction * delta;
    }
    fn on_message(&mut self, _ctx: &mut impl EngineApi, _msg: &Self::Message) {
        println!("Mensagem recebida")
    }
}
