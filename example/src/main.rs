use alone_engine::*;

#[derive(GameObject)]
pub struct Player {
    #[game(base)]
    base: Base,
    position: Vector2,
    texture: Option<Handler<ImageAsset>>,
}

impl GameObject for Player {
    type Message = ();
    fn start(&mut self, _ctx: &mut impl EngineApi) {
        let texture = _ctx.load_texture("./assets/triangle.png");
        self.texture = Some(texture);
        println!("Start")
    }
    fn draw(&mut self, render: &mut impl RenderApi) {
        if let Some(texture) = self.texture {
            render.draw(
                self.z_index(),
                DrawCommand {
                    cmd_type: DrawCommandType::Sprite,
                    material: DrawData {
                        rect: Rect::new(self.position.x, self.position.y, 0, 0),
                        image: texture,
                        ..Default::default()
                    },
                },
            );
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
    run_application(main_scene);
}
