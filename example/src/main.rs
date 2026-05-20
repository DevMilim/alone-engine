use alone_engine::*;

#[derive(GameObject)]
struct Player {
    #[game(base)]
    base: Base,
    pos: Vector2,
    texture_handle: Option<Handler<ImageAsset>>,
}

enum PlayerMessage {
    Morreu,
}

impl GameObject for Player {
    type Message = PlayerMessage;
    fn start(&mut self, ctx: &mut EngineContext) {
        ctx.load_texture("assets/player.png");
    }
    fn on_message(&mut self, _ctx: &mut EngineContext, _msg: &Self::Message) {
        match _msg {
            PlayerMessage::Morreu => self.queue_free(),
        }
    }
    fn draw(&mut self, ctx: &mut EngineContext, _base: &Base) {
        if let Some(texture) = self.texture_handle {
            ctx.draw(DrawCommand {
                cmd_type: DrawCommandType::Sprite,
                material: DrawData {
                    pos: Vector2::ZERO,
                    size: Vector2::new(64.0, 64.0),
                    image: texture,
                    ..Default::default()
                },
            });
            println!("Draw")
        }
    }
}

#[derive(Scene)]
enum GameScenes {
    Main(Player),
}

fn main() {
    let main_scene = GameScenes::Main(Player {
        base: Base::new(Transform2D::EMPTY),
        pos: Vector2::ZERO,
        texture_handle: None,
    });
    run(main_scene);
}
