use alone_engine::prelude::*;

// ───────────── Constantes: uma única fonte de verdade ─────────────
const SCREEN: Vector2 = Vector2::new(480.0, 270.0);
const ACTOR_SIZE: Vector2 = Vector2::new(16.0, 16.0);
const PLAYER_SPEED: f32 = 100.0;
const ENEMY_SPEED: f32 = 100.0;
const BASE_RECT: Rect = Rect::from_center(Vector2::new(240.0, 95.0), Vector2::new(64.0, 48.0));

mod layers {
    use alone_engine::prelude::Layer;
    pub const COMBAT: Layer = Layer::named(1); // hurtbox do player <-> hitbox do inimigo
    pub const BASE: Layer = Layer::named(2); // corpo do inimigo  <-> zona da base
}

mod z {
    pub const GROUND: i32 = 0;
    pub const PILLAR: i32 = 1;
    pub const ACTOR: i32 = 2;
}

// ───────────── Player ─────────────
#[derive(GameObject)]
pub struct Player {
    #[base]
    base: Base,

    #[component(Body::character())]
    body: Body,

    #[component(Collider::rect(ACTOR_SIZE).centered().debug())]
    collider: Collider,

    #[component(
        Collider::rect(ACTOR_SIZE).centered()
            .layer(layers::COMBAT).mask(layers::COMBAT).debug()
    )]
    hurtbox: Collider,

    state: PlayerState,
}

#[derive(Default)]
pub enum PlayerState {
    #[default]
    Walk,
    Slide,
}

impl GameObject for Player {
    fn fixed_update(&mut self, ctx: &mut Ctx, dt: f32) {
        match self.state {
            PlayerState::Walk => {
                self.hurtbox.enable();

                if ctx.input.just_pressed(Key::Space) {
                    ctx.spawn(Pilar::at(self.position()));
                }

                let dir = ctx.input.vector(Keys::WASD);
                self.slide(ctx, dir * PLAYER_SPEED, dt);
            }
            PlayerState::Slide => self.hurtbox.disable(),
        }
    }

    fn draw(&self, g: &mut Draw) {
        g.rect(Rect::from_center(self.position(), ACTOR_SIZE))
            .stroke(Color::BLACK)
            .z(z::ACTOR);
    }
}

// ───────────── Enemy ─────────────
#[derive(GameObject)]
#[connect(hitbox.entered => on_hit)]
pub struct Enemy {
    #[base]
    base: Base,

    #[component(Body::character())]
    body: Body,

    #[component(Collider::rect(ACTOR_SIZE).layer(layers::BASE).mask(layers::BASE))]
    collider: Collider,

    #[component(
        Collider::rect(ACTOR_SIZE).sensor()
            .layer(layers::COMBAT).mask(layers::COMBAT).debug()
    )]
    hitbox: Collider,

    target: Vector2, // o destino vem no spawn, sem estado global
}

impl Enemy {
    pub fn new(position: Vector2, target: Vector2) -> Self {
        Self {
            target,
            ..Self::at(position)
        } // `at` é gerado pela macro
    }

    fn on_hit(&mut self, _ctx: &mut Ctx, _hit: &Collision) {
        self.queue_free();
    }
}

impl GameObject for Enemy {
    fn fixed_update(&mut self, ctx: &mut Ctx, dt: f32) {
        self.seek(ctx, self.target, ENEMY_SPEED, dt); // sem overshoot
    }

    fn draw(&self, g: &mut Draw) {
        g.rect(Rect::from_center(self.position(), ACTOR_SIZE))
            .fill(Color::rgb(255, 50, 50))
            .z(z::GROUND);
    }
}

// ───────────── Pilar ─────────────
#[derive(GameObject)]
pub struct Pilar {
    #[base]
    base: Base,
}

impl GameObject for Pilar {
    fn draw(&self, g: &mut Draw) {
        g.rect(Rect::anchored(
            self.position(),
            vec2(4.0, 16.0),
            Anchor::TopCenter,
        ))
        .fill(Color::BLUE)
        .z(z::PILLAR);
    }
}

// ───────────── Cena ─────────────
#[derive(GameObject)]
#[connect(spawn_timer.timeout => spawn_enemy)]
#[connect(base_zone.entered   => on_base_reached)]
pub struct MainScene {
    #[base]
    base: Base,

    #[object(Player::default())]
    player: Slot<Player>,

    #[object]
    pillars: Pool<Pilar>, // ctx.spawn(Pilar) cai aqui automaticamente

    #[object]
    enemies: Pool<Enemy>,

    #[component(
        Collider::from_rect(BASE_RECT).sensor()
            .layer(layers::BASE).mask(layers::BASE).debug()
    )]
    base_zone: Collider,

    #[component(Timer::repeating(2.0))]
    spawn_timer: Timer,

    #[component(
        Text::new("Hello World!")
            .font("assets/fonts/PixelOperator8.ttf")
            .size(50)
            .at(vec2(10.0, 50.0))
    )]
    label: Text,
}

impl MainScene {
    fn spawn_enemy(&mut self, ctx: &mut Ctx) {
        let x = ctx.random_range(10.0..450.0);
        self.enemies
            .spawn(Enemy::new(vec2(x, SCREEN.y - 10.0), BASE_RECT.top_left()));
    }

    fn on_base_reached(&mut self, _ctx: &mut Ctx, _hit: &Collision) {
        self.spawn_timer.stop(); // no lugar do `if !self.game_over`
        self.enemies.queue_free_all();
        self.pillars.queue_free_all();
        self.player.queue_free();
        self.label.set_text("Game Over");
    }
}

impl GameObject for MainScene {
    fn start(&mut self, _ctx: &mut Ctx) {
        self.pillars.spawn(Pilar::at(SCREEN / 2.0));
    }

    fn draw(&self, g: &mut Draw) {
        g.rect(BASE_RECT).fill(Color::GRAY).z(z::GROUND);
        g.polyline(self.pillars.iter().map(|p| p.position()))
            .color(Color::RED)
            .width(2.0)
            .z(z::ACTOR);
    }
}

#[derive(Scene)]
pub enum GameScenes {
    Main(MainScene),
}

fn main() {
    App::new("Alone Demo")
        .resolution(480, 270)
        .debug_colliders(cfg!(debug_assertions)) // no lugar de `debug: true` em cada collider
        .run(GameScenes::Main(MainScene::default()));
}
