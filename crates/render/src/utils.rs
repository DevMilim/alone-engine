use core::{RenderApi, TextureHandler};

use assets::ImageAsset;
use winit::keyboard::KeyCode;

#[derive(Debug, Clone)]
pub enum RenderCommands {
    KeyDown(KeyCode),
    KeyUp(KeyCode),
    MousePosition(f32, f32),
    Quit,
}

pub trait Runtime {
    fn update(&mut self) -> bool;
    fn render(&mut self, renderer: &mut impl RenderApi);
    fn get_texture(&self, handler: TextureHandler) -> Option<&ImageAsset>;
    fn events(&mut self, cmd: RenderCommands);
}
