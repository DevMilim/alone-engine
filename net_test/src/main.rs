use alone_engine::{App, Base, EngineApi, GameObject, GameObjectBase, Scene};
use bincode::{Decode, Encode};

#[derive(Debug, Clone, Encode, Decode)]
pub struct PingEvent {
    pub mensagem: String,
}

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
    fn update(&mut self, ctx: &mut impl EngineApi, _delta: f32) {
        ctx.emit("Hello".to_string());
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
    let mut app = App::<GameScenes, ()>::new(GameScenes::new());
    app.start_server("localhost:3000");
    app.run();
}
