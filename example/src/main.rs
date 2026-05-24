use alone_engine::*;

#[derive(GameObject)]
pub struct Player {
    #[game(base)]
    base: Base,
    #[game(component)]
    camera: Camera2D,
    #[game(component)]
    colision: BoxCollider,
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
                        rect: Rect::new(self.position().x, self.position().y, 0, 0),
                        image: texture,
                        ..Default::default()
                    },
                },
            );
        }
    }

    fn fixed_update(&mut self, _ctx: &mut impl EngineApi, _delta: f32) {
        if _ctx.is_key_pressed(KeyCode::KeyW) {
            self.position_mut().y -= 1.0
        }
        if _ctx.is_key_pressed(KeyCode::KeyS) {
            self.position_mut().y += 1.0
        }
        if _ctx.is_key_pressed(KeyCode::KeyA) {
            self.position_mut().x -= 1.0
        }
        if _ctx.is_key_pressed(KeyCode::KeyD) {
            self.position_mut().x += 1.0
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
        colision: BoxCollider {
            width: 30.0,
            height: 30.0,
            ..Default::default()
        },
        texture: None,
        camera: Camera2D {
            active: true,
            lerp_speed: 10.0,
            deadzone: Vector2::new(50.0, 50.0),
            half: Vector2::new(64.0, 64.0),
            ..Default::default()
        },
    };

    let main_scene = GameScenes::MainScene(player);
    run_application(main_scene);
}
