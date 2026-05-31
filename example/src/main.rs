use std::time::Duration;

use alone_engine::{
    Base, Component, EngineApi, GameObject, GameObjectBase, KeyCode, Scene, SpawnEvent, Tilemap,
    Timer, Vector2, run_application,
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
}

impl Player {
    pub fn new(position: Vector2) -> Self {
        Self {
            base: Base::new(position),
        }
    }
}

impl GameObject for Player {
    type Message = ();
    fn update(&mut self, ctx: &mut impl EngineApi, _delta: f32) {
        if ctx.is_key_just_pressed(KeyCode::Space) {
            ctx.spawn(Bullet::new());
        }
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
    fn start(&mut self, _ctx: &mut impl alone_engine::EngineApi) {
        //self.tilemap =
        //    Some(Tilemap::from_ldtk_file(ctx, "assets/tilemap.ldtk", "Level_0").unwrap());
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
