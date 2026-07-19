use std::time::Duration;

use crate::{platform::Platform, player::Player};
use alone_engine::{
    GameObject, Scene,
    components::{Collider, PlaybackMode, Sound, TileCollision, Tilemap},
    core::{Base, Component, EngineApi, GameObject, GameObjectBase},
    event::{TriggerEvent, TriggerKind},
    math::Vector2,
    objects::network::NetworkClient,
    runtime::App,
};

mod platform;
mod player;

#[derive(GameObject)]
#[game(connect(collision: TriggerEvent))]
pub struct MainScene {
    #[base]
    base: Base,
    #[object]
    player: Player,
    #[object]
    platform: Option<Platform>,
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
            sensor: Collider {
                is_sensor: true,
                offset_x: 40,
                offset_y: 100,
                ..Default::default()
            },
            tilemap: None,
            music: None,
            coin_sound: None,
            player: Player::new(),
            platform: Some(Platform::new(
                Vector2::new(10.0, 117.0),
                Vector2::new(50.0, 117.0),
                Duration::from_secs_f32(1.5),
            )),
        }
    }
    fn collision(&mut self, ctx: &mut impl EngineApi, event: &TriggerEvent) {
        match event.kind {
            TriggerKind::Enter => self.coin_sound.as_mut().unwrap().play(ctx),
            TriggerKind::Exit => {}
        }
    }
}

impl GameObject for MainScene {
    type Message = ();
    fn start(&mut self, ctx: &mut impl EngineApi) {
        self.coin_sound = Some(Sound::new(
            ctx.load_audio(self.base.id, "assets/sounds/coin.wav"),
            PlaybackMode::OneShot,
        ));
        self.music = Some(Sound::new(
            ctx.load_audio(self.base.id, "assets/music/time_for_adventure.mp3"),
            PlaybackMode::Loop,
        ));
        self.tilemap = Some(
            Tilemap::from_ldtk_file(
                self.base.id,
                ctx,
                "assets/tilemap/ldtk_tilemap.ldtk",
                "Level_0",
                &vec![(1, TileCollision::Full), (2, TileCollision::OnWay)],
            )
            .unwrap(),
        );
        self.music.as_mut().unwrap().play(ctx);
        println!("{:?}", self.base.id)
    }
}
#[derive(Scene)]
pub enum GameScenes {
    MainScene(MainScene),
    PlayerScene(Player),
}

fn main() {
    App::<GameScenes, Globals>::new(MainScene::new().into())
        .with_globals(Globals::new())
        .run();
}

#[derive(GameObject)]
pub struct Globals {
    #[base]
    base: Base,
    #[object]
    client: Option<NetworkClient>,
}
impl Globals {
    pub fn new() -> Self {
        Self {
            base: Base::default(),
            client: None,
        }
    }
}

impl GameObject for Globals {
    type Message = ();
    fn start(&mut self, ctx: &mut impl EngineApi) {
        self.client = Some(NetworkClient::new("localhost:3000", ctx.async_handle()).unwrap());
    }
}
