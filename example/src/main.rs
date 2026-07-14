use std::time::Duration;

use alone_engine::{
    GameObject, Scene,
    components::{
        self, Collider, PlaybackMode, Sound, Sprite, TileCollision, Tilemap, Timer, TimerEvent,
    },
    core::{Base, Component, EngineApi, GameObject, GameObjectBase, Handler},
    event::{SpawnEvent, TriggerEvent, TriggerKind},
    math::Vector2,
    render::{Anchor::TopLeft, ImageAsset, LOGICAL_HEIGHT, LOGICAL_WIDTH, SpriteSrc},
    runtime::App,
};
use rand::RngExt;

use crate::{platform::Platform, player::Player};

mod platform;
mod player;

#[derive(GameObject)]
#[game(connect(collision: TriggerEvent))]
pub struct MainScene {
    #[base]
    base: Base,
    #[object]
    player: Player,
    #[object]
    platform: Platform,
    #[component]
    tilemap: Option<Tilemap>,
    #[component]
    sensor: Collider,
    #[component]
    coin_sound: Option<Sound>,
    #[component]
    music: Option<Sound>,
}
impl MainScene {
    pub fn new() -> Self {
        Self {
            base: Base::default(),
            sensor: Collider {
                is_sensor: true,
                offset_x: 40.0,
                offset_y: 100.0,
                ..Default::default()
            },
            tilemap: None,
            music: None,
            coin_sound: None,
            player: Player::new(),
            platform: Platform::new(
                Vector2::new(10.0, 117.0),
                Vector2::new(50.0, 117.0),
                Duration::from_secs_f32(1.5),
            ),
        }
    }
    fn collision(&mut self, ctx: &mut impl EngineApi, event: &TriggerEvent) {
        println!("{:?}", event);
        match event.kind {
            TriggerKind::Enter => self.coin_sound.as_mut().unwrap().play(ctx),
            TriggerKind::Exit => {}
        }
    }
}

impl GameObject for MainScene {
    type Message = ();
    fn start(&mut self, ctx: &mut impl EngineApi) {
        self.coin_sound = Some(Sound::new(
            ctx.load_audio("assets/sounds/coin.wav"),
            PlaybackMode::OneShot,
        ));
        self.music = Some(Sound::new(
            ctx.load_audio("assets/music/time_for_adventure.mp3"),
            PlaybackMode::Loop,
        ));
        self.tilemap = Some(
            Tilemap::from_ldtk_file(
                ctx,
                "assets/tilemap/ldtk_tilemap.ldtk",
                "Level_0",
                &vec![(1, TileCollision::Full), (2, TileCollision::OnWay)],
            )
            .unwrap(),
        );
        self.music.as_mut().unwrap().play(ctx);
        println!("{:?}", self.base.id)
    }
}
#[derive(Scene)]
pub enum GameScenes {
    MainScene(MainScene),
    PayerScene(Player),
    SpriteTest(SpriteTest),
}

fn main() {
    App::<GameScenes>::new(SpriteTest::new().into()).run();
}

#[derive(GameObject)]
pub struct SpriteObj {
    #[base]
    base: Base,
    #[component]
    sprite: Sprite,
}

impl SpriteObj {
    pub fn new(texture: Handler<ImageAsset>, position: Vector2, ctx: &mut impl EngineApi) -> Self {
        let mut image = SpriteSrc::new(texture, Some(Vector2::new(32.0, 32.0)));
        image.set_src(0, 0);
        Self {
            base: Base::new(position),
            sprite: Sprite {
                texture: image,
                anchor: TopLeft,
                ..Default::default()
            },
        }
    }
}

impl GameObject for SpriteObj {
    type Message = ();
}

#[derive(GameObject)]
#[game(connect(add_batch: TimerEvent))]
pub struct SpriteTest {
    #[base]
    base: Base,

    #[component]
    timer: Timer,
    #[object]
    sprites: Vec<SpriteObj>,

    sprite_count: usize,
    texture: Handler<ImageAsset>,

    fps: f32,
    // NOVO CAMPO: Acumulador de tempo para FPS baixo
    time_below_60: f32,
}

impl SpriteTest {
    pub fn new() -> Self {
        Self {
            base: Base::default(),
            timer: Timer::new(),
            sprites: Vec::new(),
            sprite_count: 0,
            texture: Handler::new(0),
            fps: 70.0,
            time_below_60: 0.0, // Inicializa em zero
        }
    }

    fn add_batch(&mut self, ctx: &mut impl EngineApi, _: &TimerEvent) {
        let cols = LOGICAL_WIDTH / 32; // 340 / 32 = 10 colunas
        let rows = LOGICAL_HEIGHT / 32; // 180 / 32 = 5 linhas
        let max_sprites = cols * rows; // No máximo 50 sprites na tela por vez

        for _ in 0..1000 {
            // Usamos % max_sprites para que o índice volte a 0 quando passar de 50
            let index = self.sprite_count % max_sprites as usize;

            let x = (index % cols as usize) as f32 * 32.0;
            let y = (index / cols as usize) as f32 * 32.0;

            let position = Vector2::new(x, y);

            self.sprites
                .push(SpriteObj::new(self.texture, position, ctx));

            // Incrementamos o contador aqui dentro para que cada um dos 10
            // sprites do lote ganhe uma posição única na grade
            self.sprite_count += 1;
        }

        println!("Sprites: {}", self.sprite_count);
    }
}

impl GameObject for SpriteTest {
    type Message = ();

    fn start(&mut self, ctx: &mut impl EngineApi) {
        self.texture = ctx.load_texture("assets/sprites/knight.png");
        self.timer.start_timer(Duration::from_secs(1), true);
    }

    fn update(&mut self, ctx: &mut impl EngineApi, delta: f32) {
        if delta > 0.0 {
            let inst_fps = 1.0 / delta;
            self.fps = (self.fps * 0.9) + (inst_fps * 0.1);
        }

        if self.fps < 60.0 && self.sprite_count > 0 {
            self.time_below_60 += delta;

            if self.time_below_60 >= 1.0 {
                self.timer.stop();

                println!(
                    "Limite encontrado: {} sprites rodando a {:.1} FPS",
                    self.sprite_count, self.fps
                );
                panic!(
                    "Limite encontrado: {:.1} FPS, sprites: {}",
                    self.fps,
                    self.sprites.len()
                );
            }
        } else {
            self.time_below_60 = 0.0;
        }
    }
}
