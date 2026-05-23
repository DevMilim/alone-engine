use alone_engine::prelude::*;

#[derive(GameObject)]
pub struct Player {
    #[game(base)]
    base: Base,
    position: Vector2,
    texture: Option<TextureHandler>,
}

impl GameObject for Player {
    type Message = ();
    fn start(&mut self, _ctx: &mut impl EngineApi) {
        let texture = _ctx.load_texture("./assets/player.png");
        self.texture = Some(texture);
        println!("Start")
    }
    fn draw(&mut self, render: &mut impl RenderApi) {
        if let Some(texture) = self.texture {
            render.draw(DrawCommand {
                cmd_type: DrawCommandType::Sprite,
                material: DrawData {
                    pos: self.position.clone(),
                    image: texture,
                    ..Default::default()
                },
                z_index: self.z_index(),
            });
        }
    }
    fn fixed_update(&mut self, _ctx: &mut impl EngineApi, _delta: f32) {
        if _ctx.is_key_pressed(KeyCode::Space) {
            self.position.x += 1.0
        }
    }
}

#[derive(Scene)]
enum GameScenes {
    MainScene(Player),
}

fn main() {
    let player = Player {
        base: Base::new(Transform2D::EMPTY),
        position: Vector2::new(10.0, 10.0),
        texture: None,
    };

    let main_scene = GameScenes::MainScene(player);
    let engine = Engine::new(main_scene);
    engine.run();
}
