use std::net::SocketAddr;

use alone_engine::{
    App, Base, EngineApi, GameObject, GameObjectBase, KeyCode, NetworkEvent, Scene, serialize_bytes,
};
use bincode::{Decode, Encode};

#[derive(Debug, Clone, Encode, Decode)]
pub struct PingEvent {
    pub mensagem: String,
}

#[derive(GameObject)]
#[game(server_subscribe(event: String))]
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

    fn event(&mut self, ctx: &mut impl EngineApi, event: String, socket_addr: Option<SocketAddr>) {
        println!("Evento recebido");
        println!("{}", event)
    }
}

impl GameObject for MainScene {
    type Message = ();

    fn update(&mut self, ctx: &mut impl EngineApi, _delta: f32) {
        if ctx.is_key_pressed(KeyCode::Space) {
            ctx.emit("Hello".to_string());
        }
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
    app.start_client("localhost:3000");
    app.run();
}
