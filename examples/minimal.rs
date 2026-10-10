use alone_engine::prelude::*;

#[derive(GameObject)]
pub struct Player {
    #[base]
    base: Base,
    #[component]
    rect: RectRenderer,
}
impl Player {
    pub fn new() -> Self {
        Self {
            base: Base::default(),
            rect: RectRenderer::new(32, 32, Color::BLUE),
        }
    }
}
impl GameObject for Player {
    type Message = ();
    fn fixed_update(&mut self, ctx: &mut impl EngineApi, _delta: f32) {
        let direction =
            ctx.get_key_vector(KeyCode::KeyW, KeyCode::KeyS, KeyCode::KeyA, KeyCode::KeyD);
        let transform = &mut self.base.transform;
        transform.position += direction * 5.0;
    }
}

#[derive(GameObject)]
pub struct MainScene {
    #[base]
    base: Base,
    #[object]
    player: Player,
}
impl MainScene {
    pub fn new() -> Self {
        Self {
            base: Base::default(),
            player: Player::new(),
        }
    }
}

impl GameObject for MainScene {
    type Message = ();
}
#[derive(Scene)]
pub enum GameScenes {
    MainScene(MainScene),
}

fn main() {
    App::<GameScenes>::new(MainScene::new().into()).run();
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct RectRenderer {
    pub width: i32,
    pub height: i32,
    pub color: Color,
    pub z_index: u8,
}

impl RectRenderer {
    pub fn new(width: i32, height: i32, color: Color) -> Self {
        Self {
            width,
            height,
            color,
            z_index: 0,
        }
    }
    pub fn set_color(&mut self, color: Color) {
        self.color = color
    }
}

impl Component for RectRenderer {
    fn draw(&mut self, renderer: &mut impl RenderApi, base: &Base, _blending: f32) {
        let position = base.position();
        let rect = Rect::new(
            position.x as i32,
            position.y as i32,
            self.width,
            self.height,
        );

        renderer.draw_rect(rect, self.color, self.z_index);
    }
}
