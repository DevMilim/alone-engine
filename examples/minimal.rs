use alone_engine::prelude::*;

#[derive(GameObject)]
#[connect(msg => i32)]
#[subscribe(msg => i32)]
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
    pub fn msg(&mut self, ctx: &mut impl EngineApi, ev: &i32) {}
}

impl GameObject for MainScene {}

#[derive(Scene)]
pub enum GameScenes {
    MainScene(MainScene),
}

fn main() {
    App::<GameScenes>::new(MainScene::new().into()).run();
}
