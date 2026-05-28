use std::time::Duration;

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
        _ctx.send(self.id, 0i32);
    }
}

#[derive(GameObject)]
#[game(connect(on_collide: TriggerEvent, timer: TimerEvent))]
pub struct Player {
    #[game(base)]
    base: Base,
    #[game(component)]
    camera: Camera2D,
    #[game(component)]
    body: Body2D,
    #[game(component)]
    timer: Timer,
    #[game(component)]
    animation: SpriteAnimation,
    #[game(component)]
    colision: BoxCollider,
    #[game(object)]
    wall: Wall,
}

impl Player {
    pub fn on_collide(&mut self, ctx: &mut impl EngineApi, event: &TriggerEvent) {
        println!("Colidiu");
    }
    fn timer(&mut self, ctx: &mut impl EngineApi, event: &TimerEvent) {
        println!("Connect event");
        self.timer.set_event(0i32);
    }
}

impl GameObject for Player {
    type Message = i32;
    fn start(&mut self, ctx: &mut impl EngineApi) {
        let animation_data = AnimationData::new(
            vec![
                ctx.load_texture("./assets/player.png"),
                ctx.load_texture("./assets/triangle.png"),
            ],
            2.0,
        );
        self.animation.new_animation(animation_data, "idle");
        self.timer.start_timer(Duration::from_secs(1), true);
        self.animation.play("idle");
        println!("Start")
    }
    fn draw(&mut self, render: &mut impl RenderApi, blending: f32) {
        render.draw_rect(Rect::new(10.0, 30.0, 20, 40), Color::BLACK, 0);
        render.draw_rect(Rect::new(90.0, 400.0, 20, 70), Color::BLACK, 0);
        render.draw_rect(Rect::new(100.0, 100.0, 10, 40), Color::BLACK, 0);
        render.draw_rect(Rect::new(300.0, 450.0, 20, 40), Color::BLACK, 0);
        render.draw_sprite(Vector2::new(0.0, 44.0), Handler::new(2), Anchor::Center, 0);
    }

    fn on_message(&mut self, _ctx: &mut impl EngineApi, _msg: &Self::Message) {
        println!("Message")
    }

    fn fixed_update(&mut self, ctx: &mut impl EngineApi, delta: f32) {
        let velocity = 200.0;
        let direction =
            ctx.get_key_vector(KeyCode::KeyW, KeyCode::KeyS, KeyCode::KeyA, KeyCode::KeyD);

        self.body.velocity = direction * velocity;

        self.body.move_and_slide(ctx, &mut self.base, delta);
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
            debug: true,
            width: 30.0,
            height: 30.0,
            ..Default::default()
        },
        camera: Camera2D {
            active: true,
            lerp_speed: 10.0,
            deadzone: Vector2::new(50.0, 50.0),
            half: Vector2::new(32.0, 32.0),
            ..Default::default()
        },
        wall: Wall {
            base: Base::new(Transform2D::new(100.0, 100.0)),
            colision: BoxCollider {
                debug: true,
                width: 100.0,
                height: 40.0,
                ..Default::default()
            },
            id,
        },
        body: Body2D {
            velocity: Vector2::ZERO,
        },
        timer: Timer::new(),
        animation: SpriteAnimation::new(),
    };

    let main_scene = GameScenes::MainScene(player);
    run_application(main_scene);
}
