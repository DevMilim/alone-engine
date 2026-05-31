use std::time::Duration;

use alone_engine::{
    Base, Body2D, Camera2D, Collider, Color, Component, EngineApi, GameObject, GameObjectBase,
    KeyCode, MouseButton, Rect, Scene, SpawnEvent, Sprite, TileCollision, Tilemap, Timer, Vector2,
    run_application,
};
#[derive(GameObject)]
struct Bullet {
    #[base]
    base: Base,
    #[component]
    timer: Timer,
}

impl Bullet {
    pub fn new() -> Self {
        Bullet {
            base: Base::empty(),
            timer: Timer::new(),
        }
    }
}

#[derive(Clone)]
pub enum BulletEvent {
    Free,
}

impl GameObject for Bullet {
    type Message = BulletEvent;
    fn start(&mut self, _ctx: &mut impl EngineApi) {
        println!("Bullet");
        self.timer.start_timer(Duration::from_secs(2), false);
        self.timer.set_event(BulletEvent::Free);
    }
    fn on_message(&mut self, _ctx: &mut impl EngineApi, msg: &Self::Message) {
        match msg {
            BulletEvent::Free => {
                println!("Exit");
                self.base.queue_free();
            }
        }
    }
    fn destroy(&mut self, _ctx: &mut impl EngineApi) {
        println!("destroy chamado")
    }
}

#[derive(GameObject)]
pub struct Player {
    #[base]
    base: Base,
    #[component]
    collider: Collider,
    #[component]
    camera: Camera2D,
    #[component]
    body: Body2D,
    #[component]
    sprite: Option<Sprite>,
}

impl Player {
    pub fn new(position: Vector2) -> Self {
        Self {
            base: Base::new(position),
            collider: Collider {
                ..Default::default()
            },
            camera: Camera2D::new(Vector2::new(0.0, 0.0)),
            body: Body2D::default(),
            sprite: None,
        }
    }
}

impl GameObject for Player {
    type Message = ();
    fn start(&mut self, ctx: &mut impl EngineApi) {
        self.sprite = Some(Sprite {
            texture: ctx.load_texture("assets/player.png"),
            ..Default::default()
        })
    }
    fn update(&mut self, ctx: &mut impl EngineApi, _delta: f32) {
        if ctx.is_mouse_just_pressed(MouseButton::Left) {
            self.set_position(ctx.mouse_position());
        }
    }
    fn fixed_update(&mut self, ctx: &mut impl EngineApi, delta: f32) {
        let direction = ctx
            .get_key_vector(KeyCode::KeyW, KeyCode::KeyS, KeyCode::KeyA, KeyCode::KeyD)
            .normalize();

        self.body.set_velocity(direction * 100.0 * delta);

        self.body.move_and_slide(ctx, &mut self.base);
    }
}

#[derive(GameObject)]
#[game(subscribe(spawn_bullet: SpawnEvent<Bullet>))]
pub struct MainScene {
    #[base]
    base: Base,
    #[object]
    player: Player,
    #[component]
    tilemap: Option<Tilemap>,
    #[object]
    bullets: Vec<Bullet>,
}
impl MainScene {
    pub fn new() -> Self {
        Self {
            base: Base::default(),
            player: Player::new(Vector2::ZERO),
            tilemap: None,
            bullets: Vec::new(),
        }
    }
    fn spawn_bullet(&mut self, _ctx: &mut impl EngineApi, event: &SpawnEvent<Bullet>) {
        self.bullets
            .push(event.take().expect("Erro ao spawnar bullet"));
    }
}

impl GameObject for MainScene {
    type Message = ();
    fn start(&mut self, ctx: &mut impl alone_engine::EngineApi) {
        //self.tilemap = Some(
        //    Tilemap::from_ldtk_file(
        //        ctx,
        //        "assets/tilemap.ldtk",
        //        "Level_0",
        //        &vec![(1, TileCollision::Full)],
        //    )
        //    .unwrap(),
        //);
    }
    fn draw(&mut self, renderer: &mut impl alone_engine::RenderApi, _blending: f32) {
        renderer.draw_rect(Rect::new(10.0, 10.0, 30, 60), Color::BLACK, 0);
    }
}

#[derive(Scene)]
pub enum GameScenes {
    MainScene(MainScene),
}

impl GameScenes {
    fn new() -> Self {
        GameScenes::MainScene(MainScene::new())
    }
}

fn main() {
    run_application(GameScenes::new());
}
