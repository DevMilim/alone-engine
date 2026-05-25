use alone_engine::*;

#[derive(GameObject)]
pub struct Wall {
    #[game(base)]
    base: Base,
    #[game(component)]
    colision: BoxCollider,
    id: Id,
}

impl GameObject for Wall {
    type Message = ();
    fn start(&mut self, _ctx: &mut impl EngineApi) {
        self.top_level();
        _ctx.send(self.id, ());
    }
}

#[derive(GameObject)]
#[game(connect(on_collide: TriggerEvent))]
pub struct Player {
    #[game(base)]
    base: Base,
    #[game(component)]
    camera: Camera2D,
    #[game(component)]
    colision: BoxCollider,
    texture: Option<Handler<ImageAsset>>,
    last_position: Vector2,
    #[game(object)]
    wall: Wall,
}

impl Player {
    pub fn on_collide(&mut self, ctx: &mut impl EngineApi, event: &TriggerEvent) {
        println!("Colidiu");
    }
}

impl GameObject for Player {
    type Message = ();
    fn start(&mut self, _ctx: &mut impl EngineApi) {
        let texture = _ctx.load_texture("./assets/player.png");
        _ctx.load_texture_and_resize("./assets/player.png", 64, 64);
        self.texture = Some(texture);
        self.last_position = self.position();
        println!("Start")
    }
    fn draw(&mut self, render: &mut impl RenderApi, blending: f32) {
        render.draw_rect(Rect::new(10.0, 30.0, 20, 40), Color::BLACK, 0);
        render.draw_rect(Rect::new(90.0, 400.0, 20, 70), Color::BLACK, 0);
        render.draw_rect(Rect::new(100.0, 100.0, 10, 40), Color::BLACK, 0);
        render.draw_rect(Rect::new(300.0, 450.0, 20, 40), Color::BLACK, 0);
        render.draw_sprite(Vector2::new(0.0, 44.0), Handler::new(2), Anchor::Center, 0);
        if let Some(texture) = self.texture {
            let current_position = self.last_position.lerp(self.position(), blending);
            render.draw_sprite(current_position, texture, Anchor::Center, self.z_index());
        }
    }

    fn on_message(&mut self, _ctx: &mut impl EngineApi, _msg: &Self::Message) {
        println!("Message")
    }

    fn update(&mut self, _ctx: &mut impl EngineApi, delta: f32) {
        let velocity = 200.0 * delta;
        self.last_position = self.position();
        if _ctx.is_key_pressed(KeyCode::KeyW) {
            self.position_mut().y -= velocity
        }
        if _ctx.is_key_pressed(KeyCode::KeyS) {
            self.position_mut().y += velocity
        }
        if _ctx.is_key_pressed(KeyCode::KeyA) {
            self.position_mut().x -= velocity
        }
        if _ctx.is_key_pressed(KeyCode::KeyD) {
            self.position_mut().x += velocity
        }
    }
}

#[derive(Scene)]
enum GameScenes {
    MainScene(Player),
}

fn main() {
    let player_base = Base::empty();
    let id = player_base.id.clone();
    let player = Player {
        base: player_base,
        colision: BoxCollider {
            is_sensor: true,
            debug: true,
            width: 30.0,
            height: 30.0,
            ..Default::default()
        },
        texture: None,
        camera: Camera2D {
            active: true,
            lerp_speed: 10.0,
            deadzone: Vector2::new(50.0, 50.0),
            half: Vector2::new(32.0, 32.0),
            ..Default::default()
        },
        last_position: Vector2::new(0.0, 0.0),
        wall: Wall {
            base: Base::empty(),
            colision: BoxCollider {
                debug: true,
                width: 100.0,
                height: 40.0,
                ..Default::default()
            },
            id,
        },
    };

    let main_scene = GameScenes::MainScene(player);
    run_application(main_scene);
}
