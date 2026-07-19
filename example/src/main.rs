use alone_engine::{
    GameObject, Scene,
    core::{Base, EngineApi, GameObject, GameObjectBase},
    runtime::App,
};

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
    fn start(&mut self, _ctx: &mut impl EngineApi) {}
}
#[derive(Scene)]
pub enum GameScenes {
    MainScene(MainScene),
}

fn main() {
    App::<GameScenes>::new(MainScene::new().into()).run();
}
