use alone_engine::prelude::*;
use gilrs::Button;

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
    fn start(&mut self, ctx: &mut impl EngineApi) {
        ctx.bind_action("a", InputType::Gamepad(Button::LeftTrigger2));
    }
    fn fixed_update(&mut self, ctx: &mut impl EngineApi, _delta: f32) {
        let a = ctx.action_strength("a");
        println!("{a}")
    }
}
#[derive(Scene)]
pub enum GameScenes {
    MainScene(MainScene),
}

fn main() {
    App::<GameScenes>::new(MainScene::new().into()).run();
}
