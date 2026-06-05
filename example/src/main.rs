use alone_engine::{
    App, Base, Collider, Component, EngineApi, GameObject, GameObjectBase, KeyCode, PlaybackMode,
    Scene, Sound, TileCollision, Tilemap, TriggerEvent,
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
    tilemap: Option<Tilemap>,
    #[component]
    sensor: Collider,
    #[component]
    coin_sound: Option<Sound>,
    #[component]
    music: Option<Sound>,
}
impl MainScene {
    pub fn new() -> Self {
        Self {
            base: Base::default(),
            player: Player::new(),
            tilemap: None,
            music: None,
            sensor: Collider {
                is_sensor: true,
                debug: true,
                height: 16.0,
                width: 16.0,
                offset_y: 100.0 - 20.0,
                offset_x: 150.0,
                ..Default::default()
            },
            coin_sound: None,
        }
    }
    fn collision(&mut self, ctx: &mut impl EngineApi, event: &TriggerEvent) {
        match event.kind {
            alone_engine::TriggerKind::Enter => self.coin_sound.as_mut().unwrap().play(ctx),
            alone_engine::TriggerKind::Exit => {}
        }
    }
}

impl GameObject for MainScene {
    type Message = ();
    fn start(&mut self, ctx: &mut impl alone_engine::EngineApi) {
        self.coin_sound = Some(Sound::new(
            ctx.load_audio("assets/sounds/coin.wav"),
            PlaybackMode::OneShot,
        ));
        self.music = Some(Sound::new(
            ctx.load_audio("assets/music/time_for_adventure.mp3"),
            PlaybackMode::Loop,
        ));
        self.tilemap = Some(
            Tilemap::from_ldtk_file(
                ctx,
                "assets/tilemap/ldtk_tilemap.ldtk",
                "Level_0",
                &vec![(1, TileCollision::Full)],
            )
            .unwrap(),
        );
        self.music.as_mut().unwrap().play(ctx);
    }
    fn fixed_update(&mut self, ctx: &mut impl alone_engine::EngineApi, _delta: f32) {}
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
