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
                layer: 3,
                mask: 3,
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
}

impl Enemy {
    pub fn new(position: Vector2) -> Self {
        Self {
            base: Base::new(position),
            collision: Collider {
                width: 32,
                height: 32,
                mask: 2,
                layer: 2,
                ..Default::default()
            },
            hitbox: Collider {
                is_sensor: true,
                width: 32,
                height: 32,
                mask: 3,
                layer: 3,

                ..Default::default()
            },
        }
    }

    pub fn collision(&mut self, ctx: &mut impl EngineApi, _event: &TriggerEvent) {
        self.queue_free();
    }
}

impl GameObject for Enemy {
    type Message = ();
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
#[connect(enemy_hit: TriggerEvent, timer_event: TimerEvent)]
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
                mask: 2,
                layer: 2,
                debug: true,
                is_sensor: true,
                ..Default::default()
            },
            timer: Timer::new(),
        }
    }

    pub fn spawn_pilar(&mut self, ctx: &mut impl EngineApi, spawn: &SpawnEvent<Pilar>) {
        self.pilars.spawn(spawn.take().unwrap());
    }

    pub fn enemy_hit(&mut self, ctx: &mut impl EngineApi, _event: &TriggerEvent) {
        self.enemies.queue_free_all();

        self.player.queue_free();

        self.pilars.queue_free_all();
    }

    pub fn timer_event(&mut self, ctx: &mut impl EngineApi, _event: &TimerEvent) {
        let bottom_y = 260.0;

        self.enemies
            .spawn(Enemy::new(Vector2::new(200.0, bottom_y)));
    }
}

impl GameObject for MainScene {
    type Message = ();

    fn start(&mut self, _ctx: &mut impl EngineApi) {
        self.pilars
            .spawn(Pilar::new(Vector2::new(480.0 / 2.0, 270.0 / 2.0).into()));

        self.timer.start_timer(Duration::from_secs_f32(2.0), true);
    }

    fn fixed_update(&mut self, _ctx: &mut impl EngineApi, _delta: f32) {}

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
