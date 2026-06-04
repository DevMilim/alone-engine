use alone_engine::{App, Base, Collider, Component, GameObject, GameObjectBase, Scene};

use crate::player::Player;

mod player;

#[derive(GameObject)]
pub struct MainScene {
    #[base]
    base: Base,
    #[object]
    player: Player,
    #[component]
    collider: Collider,
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
        }
    }
}

impl GameObject for MainScene {
    type Message = ();
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
