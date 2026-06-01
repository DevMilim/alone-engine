use std::time::Duration;

use alone_engine::{
    App, Base, Body2D, Collider, Component, EngineApi, GameObject, GameObjectBase, KeyCode, Scene,
    Timer, TriggerEvent, Vector2,
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
    direction: Vector2,
    speed: f32,
}

impl Ball {
    pub fn new(position: Vector2) -> Self {
        Ball {
            base: Base::new(position),
            sensor: Collider {
                debug: true,
                width: 32.0,
                height: 32.0,
                ..Default::default()
            },
            body: Body2D::default(),
            timer: Timer::default(),
            direction: Vector2::new(2.0, 1.0),
            speed: 50.0,
        }
    }
}

#[derive(Clone)]
pub enum BallMsg {
    SpeedUp,
}

impl GameObject for Ball {
    type Message = BallMsg;
    fn start(&mut self, _ctx: &mut impl EngineApi) {
        self.timer.start_timer(Duration::from_secs(1), true);
        self.timer.set_event(BallMsg::SpeedUp);
    }
    fn on_message(&mut self, _ctx: &mut impl EngineApi, msg: &Self::Message) {
        match msg {
            BallMsg::SpeedUp => self.speed += 0.5,
        }
    }
    fn fixed_update(&mut self, ctx: &mut impl EngineApi, delta: f32) {
        let window_size = ctx.window_size();
        if self.body.is_on_wall() {
            self.direction.x *= -1.0;
        }
        if self.position().y + self.sensor.height as f32 / 2.0 > window_size.1 as f32
            || self.position().y - self.sensor.height as f32 / 2.0 < 0.0
        {
            self.direction.y *= -1.0;
        }
        if self.position().x + self.sensor.width as f32 / 2.0 > window_size.0 as f32 {
            self.direction.x *= -1.0;
        }
        if self.position().x - self.sensor.width as f32 / 2.0 < 0.0 {
            ctx.emit(GameEvent::GameOver);
        }
        let velocity = self.direction * self.speed * delta;
        self.body.set_velocity(velocity);
    }

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
        let direction = if ctx.is_key_pressed(KeyCode::KeyW) {
            Vector2::new(0.0, -1.0)
        } else if ctx.is_key_pressed(KeyCode::KeyS) {
            Vector2::new(0.0, 1.0)
        } else {
            Vector2::ZERO
        };
        if self.body.is_on_wall() {
            ctx.emit(GameEvent::AddPoint);
        }

        self.body.set_velocity(direction * 100.0 * delta);
    }
}

enum GameEvent {
    GameOver,
    AddPoint,
}

#[derive(GameObject)]
#[game(subscribe(events: GameEvent))]
pub struct MainScene {
    #[base]
    base: Base,
    #[object]
    player: Option<Player>,
    #[object]
    ball: Option<Ball>,
    count: i32,
}
impl MainScene {
    pub fn new() -> Self {
        Self {
            base: Base::default(),
            player: None,
            ball: None,
            count: 0,
        }
    }
    fn events(&mut self, ctx: &mut impl EngineApi, event: &GameEvent) {
        match event {
            GameEvent::GameOver => {
                self.ball.queue_free();
                self.player.queue_free();
            }
            GameEvent::AddPoint => {
                self.count += 1;
                println!("> Points: {}", self.count);
            }
        }
    }
}

impl GameObject for MainScene {
    type Message = ();
    fn start(&mut self, ctx: &mut impl alone_engine::EngineApi) {
        let window_size = ctx.window_size();
        self.top_level();
        self.player = Some(Player::new(Vector2::new(10.0, window_size.1 as f32 / 2.0)));
        self.ball = Some(Ball::new(Vector2::new(
            window_size.0 as f32 / 2.0,
            window_size.1 as f32 / 2.0,
        )))
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
