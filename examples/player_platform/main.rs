use std::time::Duration;

use crate::{platform::Platform, player::Player};
use alone_engine::{
    GameObject, Scene,
    components::{Collider, PlaybackMode, Sound, TileCollision, Tilemap},
    core::{Base, Component, EngineApi, GameObject, GameObjectBase, Slot},
    event::{TriggerEvent, TriggerKind},
    math::Vector2,
    objects::network::NetworkClient,
    runtime::App,
};

mod platform;
mod player;

pub enum MainEvent {}

#[derive(GameObject)]
#[game(connect(collision: TriggerEvent))]
#[game(subscribe(main_event: MainEvent))]
pub struct MainScene {
    #[base]
    base: Base,
    #[object]
    player: Player,
    #[object]
    platform: Platform,
    #[component]
    tilemap: Slot<Tilemap>,
    #[component]
    sensor: Collider,
    #[component]
    coin_sound: Slot<Sound>,
    #[component]
    music: Slot<Sound>,
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
            tilemap: Slot::default(),
            music: Slot::default(),
            coin_sound: Slot::default(),
            player: Player::new(),
            platform: Platform::new(
                Vector2::new(10.0, 117.0),
                Vector2::new(50.0, 117.0),
                Duration::from_secs_f32(1.5),
            ),
        }
    }
    fn collision(&mut self, ctx: &mut impl EngineApi, event: &TriggerEvent) {
        match event.kind {
            TriggerKind::Enter => self.coin_sound.unwrap().play(ctx),
            TriggerKind::Exit => {}
        }
    }
    fn main_event(&mut self, _ctx: &mut impl EngineApi, _event: &MainEvent) {}
}

impl GameObject for MainScene {
    type Message = ();
    fn start(&mut self, ctx: &mut impl EngineApi) {
        self.coin_sound = Slot::new(Sound::new(
            ctx.load_audio(self.base.id, "assets/sounds/coin.wav"),
            PlaybackMode::OneShot,
        ));
        self.music = Slot::new(Sound::new(
            ctx.load_audio(self.base.id, "assets/music/time_for_adventure.mp3"),
            PlaybackMode::Loop,
        ));
        self.tilemap = Slot::new(
            Tilemap::from_ldtk_file(
                self.base.id,
                ctx,
                "assets/tilemap/ldtk_tilemap.ldtk",
                "Level_0",
                &vec![(1, TileCollision::Full), (2, TileCollision::OnWay)],
            )
            .unwrap(),
        );
        self.music.unwrap().play(ctx);
        println!("{:?}", self.base.id)
    }
}
#[derive(Scene)]
pub enum GameScenes {
    MainScene(MainScene),
    PayerScene(Player),
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
    client: Slot<NetworkClient>,
}
impl Globals {
    pub fn new() -> Self {
        Self {
            base: Base::default(),
            client: Slot::default(),
        }
    }
}

impl GameObject for Globals {
    type Message = ();
    fn start(&mut self, ctx: &mut impl EngineApi) {
        ctx.register_service::<Globals>(self.base.id);

        self.client = Slot::new(NetworkClient::new("localhost:3000", ctx.async_handle()).unwrap());
    }
}
