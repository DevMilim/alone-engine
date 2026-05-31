use winit::{event::MouseButton, keyboard::KeyCode};

#[derive(Debug, Clone)]
pub enum RuntimeCommands {
    KeyDown(KeyCode),
    KeyUp(KeyCode),
    MouseInput(MouseButton, bool),
    MousePosition(f32, f32),
    Quit,
}
