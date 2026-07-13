use std::time::Duration;

use alone_engine::{
    GameObject, ObjectEnum, Scene,
    components::{Collider, PlaybackMode, Sound, TileCollision, Tilemap},
    core::{Base, Component, EngineApi, GameObject, GameObjectBase},
    event::{TriggerEvent, TriggerKind},
    math::Vector2,
    runtime::App,
};

use crate::{platform::Platform, player::Player};

mod platform;
mod player;

#[derive(ObjectEnum)]
pub enum GameObjects {
    Platform(Platform),
    Player(Player),
}

impl GameObject for GameObjects {
    type Message = ();
}

#[derive(GameObject)]
#[game(connect(collision: TriggerEvent))]
pub struct MainScene {
    #[base]
    base: Base,
    #[object]
    objects: GameObjects,
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
                offset_x: 40.0,
                offset_y: 100.0,
                ..Default::default()
            },
            tilemap: None,
            music: None,
            coin_sound: None,
            objects: GameObjects::Player(Player::new()),
        }
    }
    fn collision(&mut self, ctx: &mut impl EngineApi, event: &TriggerEvent) {
        println!("{:?}", event);
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
    PayerScene(Player),
}

fn main() {
    App::<GameScenes>::new(MainScene::new().into()).run();
}
