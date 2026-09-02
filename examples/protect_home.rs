use std::time::Duration;

use alone_engine::prelude::*;

const PLAYER_SIZE: i32 = 16;

#[derive(GameObject)]
pub struct Player {
    #[base]
    base: Base,
    #[component(interface = IBody)]
    body: Body,
    #[component]
    collider: Collider,
    #[component]
    enemy_hitbox: Collider,
    state: PlayerState,
}

pub enum PlayerState {
    Move,
    Slide,
}

impl Player {
    pub fn new() -> Self {
        Self {
            base: Base::default(),
            body: Body {
                body_type: BodyType::Character,
                ..Default::default()
            },
            collider: Collider {
                width: PLAYER_SIZE,
                height: PLAYER_SIZE,
                offset_x: PLAYER_SIZE / 2,
                offset_y: PLAYER_SIZE / 2,
                debug: true,
                ..Default::default()
            },
            enemy_hitbox: Collider {
                width: PLAYER_SIZE,
                height: PLAYER_SIZE,
                offset_x: PLAYER_SIZE / 2,
                offset_y: PLAYER_SIZE / 2,
                layer: Layer::LAYER_1,
                mask: Layer::LAYER_1,
                debug: true,
                ..Default::default()
            },
            state: PlayerState::Move,
        }
    }
}

impl GameObject for Player {
    type Message = ();

    fn fixed_update(&mut self, ctx: &mut impl EngineApi, delta: f32) {
        match self.state {
            PlayerState::Move => {
                self.enemy_hitbox.disabled = false;
                if ctx.is_key_just_pressed(KeyCode::Space) {
                    ctx.spawn(Pilar::new(self.position().into()));
                }
                let speed = 100.0;
                let direction =
                    ctx.get_key_vector(KeyCode::KeyW, KeyCode::KeyS, KeyCode::KeyA, KeyCode::KeyD);
                self.velocity_mut().x = direction.x * speed;
                self.velocity_mut().y = direction.y * speed;
                self.move_and_slide(ctx, delta);
            }

            PlayerState::Slide => {
                self.enemy_hitbox.disabled = true;
            }
        }
    }

    fn draw(&mut self, renderer: &mut impl RenderApi, _blending: f32) {
        renderer.draw_rect(
            Rect::new(
                self.position().x as i32,
                self.position().y as i32,
                PLAYER_SIZE,
                PLAYER_SIZE,
            ),
            Color::BLACK,
            2,
        );
    }
}

#[derive(GameObject)]
#[connect(collision: TriggerEvent)]
pub struct Enemy {
    #[base]
    base: Base,
    #[component]
    collision: Collider,
    #[component]
    hitbox: Collider,
    #[component(interface = IBody)]
    body: Body,
    home_pos: Option<Vector2>,
}

impl Enemy {
    pub fn new(position: Vector2) -> Self {
        Self {
            base: Base::new(position),
            collision: Collider {
                width: 16,
                height: 16,
                mask: Layer::LAYER_2,
                layer: Layer::LAYER_2,
                ..Default::default()
            },
            hitbox: Collider {
                is_sensor: true,
                width: 16,
                height: 16,
                mask: Layer::LAYER_1,
                layer: Layer::LAYER_1,
                debug: true,

                ..Default::default()
            },
            body: Body {
                body_type: BodyType::Character,
                ..Default::default()
            },
            home_pos: None,
        }
    }

    pub fn collision(&mut self, _ctx: &mut impl EngineApi, _event: &TriggerEvent) {
        self.queue_free();
    }
}

impl GameObject for Enemy {
    type Message = ();
    fn start(&mut self, ctx: &mut impl EngineApi) {
        if let Some(home_pos) = ctx.get_state::<HomePosition>() {
            self.home_pos = Some(home_pos.position);
        }
    }
    fn fixed_update(&mut self, ctx: &mut impl EngineApi, delta: f32) {
        let speed = 100.0;
        if let Some(home_pos) = self.home_pos {
            let to_home = home_pos - self.position();
            if to_home.length() > 1.0 {
                *self.velocity_mut() = to_home.normalize() * speed;
            } else {
                *self.velocity_mut() = Vector2::ZERO;
            }
            self.move_and_slide(ctx, delta);
        }
    }
    fn draw(&mut self, renderer: &mut impl RenderApi, _blending: f32) {
        renderer.draw_rect(
            Rect::new(
                self.position().x as i32 - 8,
                self.position().y as i32 - 8,
                16,
                16,
            ),
            Color::rgb(255, 50, 50),
            0,
        );
    }
}

#[derive(GameObject)]
pub struct Pilar {
    #[base]
    base: Base,
}

impl Pilar {
    pub fn new(position: Vector2i) -> Self {
        Self {
            base: Base::new(position.into()),
        }
    }
}

impl GameObject for Pilar {
    type Message = ();

    fn draw(&mut self, renderer: &mut impl RenderApi, _blending: f32) {
        renderer.draw_rect(
            Rect::new(
                self.position().x as i32 - 2,
                self.position().y as i32,
                4,
                16,
            ),
            Color::BLUE,
            1,
        );
    }
}

#[derive(GameObject)]
#[subscribe(spawn_pilar: SpawnEvent<Pilar>)]
#[connect(enemy_hit: TriggerEvent)]
pub struct MainScene {
    #[base]
    base: Base,
    #[object]
    player: Slot<Player>,
    #[object]
    pilars: Pool<Pilar>,
    #[object]
    enemies: Pool<Enemy>,
    #[component]
    collision: Collider,
    #[component]
    timer: Timer,
    game_over: bool,
}

impl MainScene {
    pub fn new() -> Self {
        Self {
            base: Base::default(),
            player: Slot::new(Player::new()),
            enemies: Pool::default(),
            pilars: Pool::default(),
            collision: Collider {
                width: 64,
                height: 48,
                offset_x: (480.0 / 2.0) as i32,
                offset_y: (270.0 / 2.0) as i32 - 40,
                mask: Layer::LAYER_2,
                layer: Layer::LAYER_2,
                debug: true,
                is_sensor: true,
                ..Default::default()
            },
            timer: Timer::new(),
            game_over: false,
        }
    }
    pub fn enemy_hit(&mut self, _ctx: &mut impl EngineApi, _event: &TriggerEvent) {
        self.enemies.queue_free_all();

        self.player.queue_free();

        self.pilars.queue_free_all();
        self.game_over = true;
    }
    pub fn spawn_pilar(&mut self, _ctx: &mut impl EngineApi, spawn: &SpawnEvent<Pilar>) {
        self.pilars.spawn(spawn.take().unwrap());
    }
}

pub struct HomePosition {
    pub position: Vector2,
}

#[derive(Clone)]
pub enum MainEvent {
    SpawnEnemy,
}

impl GameObject for MainScene {
    type Message = MainEvent;

    fn start(&mut self, ctx: &mut impl EngineApi) {
        self.pilars
            .spawn(Pilar::new(Vector2::new(480.0 / 2.0, 270.0 / 2.0).into()));

        self.timer.start_timer(Duration::from_secs_f32(2.0), true);
        ctx.set_state(HomePosition {
            position: Vector2::new((480.0 / 2.0) - 32.0, (270.0 / 2.0) - 64.0),
        });
        self.timer.set_event(MainEvent::SpawnEnemy);
    }
    fn on_message(&mut self, ctx: &mut impl EngineApi, msg: &Self::Message) {
        match msg {
            MainEvent::SpawnEnemy => {
                if !self.game_over {
                    let bottom_y = 260.0;
                    let x = ctx.range(10..450);

                    self.enemies
                        .spawn(Enemy::new(Vector2::new(x as f32, bottom_y)));
                }
            }
        }
    }

    fn draw(&mut self, renderer: &mut impl RenderApi, _blending: f32) {
        renderer.draw_rect(
            Rect::new((480.0 / 2.0) as i32 - 32, (270.0 / 2.0) as i32 - 64, 64, 48),
            Color::rgb(124, 124, 124),
            0,
        );

        let mut last = None;

        for pilar in self.pilars.iter() {
            if let Some(last) = last {
                renderer.draw_line(last, pilar.position(), Color::rgb(255, 0, 0), 2.0, 2);
            }

            last = Some(pilar.position());
        }
    }
}

#[derive(Scene)]

pub enum GameScenes {
    MainScene(MainScene),
}

fn main() {
    App::<GameScenes>::new(MainScene::new().into()).run();
}
