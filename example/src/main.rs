use alone_engine::{
    App, Base, Body2D, Collider, Color, Component, EngineApi, GameObject, GameObjectBase, KeyCode,
    MouseButton, Rect, Scene, Timer, Vector2,
};

#[derive(GameObject)]
struct Ball {
    #[base]
    base: Base,
    #[component]
    sensor: Collider,
    #[component]
    body: Body2D,
    #[component]
    timer: Timer,
}

impl Ball {
    pub fn new() -> Self {
        Ball {
            base: Base::default(),
            sensor: Collider {
                debug: true,
                is_sensor: true,
                ..Default::default()
            },
            body: Body2D::default(),
            timer: Timer::default(),
        }
    }
}

impl GameObject for Ball {
    type Message = ();
    fn start(&mut self, _ctx: &mut impl EngineApi) {}

    fn destroy(&mut self, _ctx: &mut impl EngineApi) {
        println!("destroy chamado")
    }
}

#[derive(GameObject)]
pub struct Player {
    #[base]
    base: Base,
    #[component]
    collider: Collider,
    #[component]
    body: Body2D,
}

impl Player {
    pub fn new(position: Vector2) -> Self {
        Self {
            base: Base::new(position),
            collider: Collider {
                debug: true,
                width: 10.0,
                height: 50.0,
                ..Default::default()
            },
            body: Body2D::default(),
        }
    }
}

impl GameObject for Player {
    type Message = ();
    fn start(&mut self, ctx: &mut impl EngineApi) {}
    fn fixed_update(&mut self, ctx: &mut impl EngineApi, delta: f32) {
        if ctx.is_mouse_pressed(MouseButton::Left) {
            self.set_position(ctx.mouse_position());
        }
        let direction = ctx
            .get_key_vector(KeyCode::KeyW, KeyCode::KeyS, KeyCode::KeyA, KeyCode::KeyD)
            .normalize();

        self.body.set_velocity(direction * 100.0 * delta);
        println!(
            "X: {}; Y: {}",
            self.base.position().x,
            self.base.position().y
        )
    }
}

#[derive(GameObject)]
pub struct MainScene {
    #[base]
    base: Base,
    #[object]
    player: Option<Player>,
    #[object]
    bullets: Option<Ball>,
}
impl MainScene {
    pub fn new() -> Self {
        Self {
            base: Base::default(),
            player: None,
            bullets: None,
        }
    }
}

impl GameObject for MainScene {
    type Message = ();
    fn start(&mut self, ctx: &mut impl alone_engine::EngineApi) {
        self.top_level();
        self.player = Some(Player::new(Vector2::new(10.0, 120.0)))
    }
    fn draw(&mut self, renderer: &mut impl alone_engine::RenderApi, _blending: f32) {
        renderer.draw_rect(Rect::new(10.0, 10.0, 30, 60), Color::BLACK, 0);
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
    App::new(GameScenes::new()).run();
}
