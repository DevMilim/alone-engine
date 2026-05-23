use winit::keyboard::KeyCode;

#[derive(Debug, Clone)]
pub enum RenderCommands {
    KeyDown(KeyCode),
    KeyUp(KeyCode),
    MousePosition(f32, f32),
    Quit,
}
