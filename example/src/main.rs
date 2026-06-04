use alone_engine::{
    App, Base, Collider, Component, EngineApi, GameObject, GameObjectBase, KeyCode, PlaybackMode,
    Scene, Sound, TriggerEvent,
};

use crate::player::Player;

mod player;

#[derive(GameObject)]
#[game(connect(collision: TriggerEvent))]
pub struct MainScene {
    #[base]
    base: Base,
    #[object]
    player: Player,
    #[component]
    collider: Collider,
    #[component]
    sensor: Collider,
    #[component]
    music: Option<Sound>,
}
impl MainScene {
    pub fn new() -> Self {
        Self {
            base: Base::default(),
            player: Player::new(),
            collider: Collider {
                debug: true,
                offset_x: 150.0,
                offset_y: 100.0,
                width: 800.0,
                ..Default::default()
            },
            music: None,
            sensor: Collider {
                is_sensor: true,
                key: 1,
                debug: true,
                height: 16.0,
                width: 16.0,
                offset_y: 100.0 - 20.0,
                offset_x: 150.0,
                ..Default::default()
            },
        }
    }
    fn collision(&mut self, ctx: &mut impl EngineApi, event: &TriggerEvent) {
        match event.kind {
            alone_engine::TriggerKind::Enter => self.music.as_mut().unwrap().play(ctx),
            alone_engine::TriggerKind::Exit => {}
        }
    }
}

impl GameObject for MainScene {
    type Message = ();
    fn start(&mut self, ctx: &mut impl alone_engine::EngineApi) {
        self.music = Some(Sound::new(
            ctx.load_audio("assets/sounds/coin.wav"),
            PlaybackMode::OneShot,
        ))
    }
    fn fixed_update(&mut self, ctx: &mut impl alone_engine::EngineApi, _delta: f32) {
        if ctx.is_key_just_pressed(KeyCode::KeyQ) {
            self.music.as_mut().unwrap().play(ctx);
        }
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
    App::new(GameScenes::new()).run();
}
