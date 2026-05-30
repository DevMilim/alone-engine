use alone_engine::{Base, GameObject, GameObjectBase, Scene, Vector2, run_application};

use crate::player::Player;

mod player;

#[derive(GameObject)]
pub struct MainScene {
    #[game(base)]
    base: Base,
    #[game(object)]
    player: Player,
}
impl MainScene {
    pub fn new() -> Self {
        Self {
            base: Base::default(),
            player: Player::new(Vector2::ZERO),
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
    run_application(GameScenes::new());
}
