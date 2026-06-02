use alone_engine::{App, Base, GameObject, GameObjectBase, Scene};

#[derive(GameObject)]
pub struct MainScene {
    #[base]
    base: Base,
}
impl MainScene {
    pub fn new() -> Self {
        Self {
            base: Base::default(),
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
