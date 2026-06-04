use std::collections::{HashMap, HashSet};

pub use winit::event::MouseButton;
pub use winit::keyboard::KeyCode;

use crate::Vector2;

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub enum InputType {
    Key(KeyCode),
    Mouse(MouseButton),
}

pub struct InputState {
    pub pressed_input: HashSet<InputType>,
    pub just_pressed_input: HashMap<InputType, (u64, u64)>,
    pub mouse_position: Vector2,
    pub map: InputMap,
    pub current_update_frame: u64,
    pub current_fixed_frame: u64,
}

impl InputState {
    pub fn new() -> Self {
        Self {
            pressed_input: HashSet::new(),
            just_pressed_input: HashMap::new(),
            mouse_position: Vector2::ZERO,
            map: InputMap::new(),
            current_update_frame: 0,
            current_fixed_frame: 0,
        }
    }

    pub fn set_mouse_position(&mut self, x: f32, y: f32) {
        self.mouse_position = Vector2::new(x, y);
    }
    pub fn is_action_pressed(&self, action: &str) -> bool {
        if let Some(input) = self.map.bindings.get(action) {
            return input.iter().any(|input| self.pressed_input.contains(input));
        }
        false
    }
    pub fn is_action_just_pressed(&self, action: &str, is_fixed_update: bool) -> bool {
        if let Some(inputs) = self.map.bindings.get(action) {
            return inputs.iter().any(|input| {
                if let Some(&(target_u, target_f)) = self.just_pressed_input.get(input) {
                    if is_fixed_update {
                        target_f == self.current_fixed_frame
                    } else {
                        target_u == self.current_update_frame
                    }
                } else {
                    false
                }
            });
        }
        false
    }
    pub fn mouse_position(&self) -> Vector2 {
        self.mouse_position
    }
    pub fn is_key_pressed(&self, key: KeyCode) -> bool {
        self.pressed_input.contains(&InputType::Key(key))
    }
    pub fn is_key_just_pressed(&self, key: KeyCode, is_fixed_update: bool) -> bool {
        if let Some(&(target_u, target_f)) = self.just_pressed_input.get(&InputType::Key(key)) {
            if is_fixed_update {
                return target_f == self.current_fixed_frame;
            } else {
                return target_u == self.current_update_frame;
            }
        }
        false
    }
    pub fn is_mouse_pressed(&self, key: MouseButton) -> bool {
        self.pressed_input.contains(&InputType::Mouse(key))
    }
    pub fn is_mouse_just_pressed(&self, key: MouseButton, is_fixed_update: bool) -> bool {
        if let Some(&(target_u, target_f)) = self.just_pressed_input.get(&InputType::Mouse(key)) {
            if is_fixed_update {
                return target_f == self.current_fixed_frame;
            } else {
                return target_u == self.current_update_frame;
            }
        }
        false
    }
    pub fn clear_frame_data(&mut self) {
        let current_u = self.current_update_frame;
        let current_f = self.current_fixed_frame;
        self.just_pressed_input
            .retain(|_, (target_u, target_f)| *target_u >= current_u || *target_f >= current_f)
    }
    pub fn update_input_state(&mut self, key: InputType, pressed: bool) {
        if pressed {
            if !self.pressed_input.contains(&key) {
                let target_u = self.current_update_frame + 1;
                let target_f = self.current_fixed_frame + 1;
                self.just_pressed_input.insert(key, (target_u, target_f));
            }
            self.pressed_input.insert(key);
        } else {
            self.pressed_input.remove(&key);
        }
    }
    pub fn get_vector(
        &self,
        action_up: &str,
        action_down: &str,
        action_left: &str,
        action_right: &str,
    ) -> Vector2 {
        let x = (if self.is_action_pressed(action_right) {
            1.0
        } else {
            0.0
        }) - (if self.is_action_pressed(action_left) {
            1.0
        } else {
            0.0
        });
        let y = (if self.is_action_pressed(action_down) {
            1.0
        } else {
            0.0
        }) - (if self.is_action_pressed(action_up) {
            1.0
        } else {
            0.0
        });

        let vec = Vector2::new(x, y);
        if vec.is_zero() {
            Vector2::ZERO
        } else {
            vec.normalize()
        }
    }
    pub fn get_key_vector(
        &self,
        key_up: KeyCode,
        key_down: KeyCode,
        key_left: KeyCode,
        key_right: KeyCode,
    ) -> Vector2 {
        let x = (if self.is_key_pressed(key_right) {
            1.0
        } else {
            0.0
        }) - (if self.is_key_pressed(key_left) {
            1.0
        } else {
            0.0
        });
        let y = (if self.is_key_pressed(key_down) {
            1.0
        } else {
            0.0
        }) - (if self.is_key_pressed(key_up) {
            1.0
        } else {
            0.0
        });

        let vec = Vector2::new(x, y);
        if vec.is_zero() {
            Vector2::ZERO
        } else {
            vec.normalize()
        }
    }
    pub fn get_key_axis(&self, negative_key: KeyCode, positive_key: KeyCode) -> f32 {
        let neg = self.is_key_pressed(negative_key) as i32 as f32;
        let pos = self.is_key_pressed(positive_key) as i32 as f32;

        pos - neg
    }
    pub fn get_axis(&self, negative_action: &str, positive_action: &str) -> f32 {
        let neg = self.is_action_pressed(negative_action) as i32 as f32;
        let pos = self.is_action_pressed(positive_action) as i32 as f32;

        pos - neg
    }
}

impl Default for InputState {
    fn default() -> Self {
        Self::new()
    }
}

pub struct InputActions {
    pub action: String,
    pub keys: Vec<InputType>,
}

impl InputActions {}

pub struct InputMap {
    pub bindings: HashMap<String, Vec<InputType>>,
}

impl Default for InputMap {
    fn default() -> Self {
        Self::new()
    }
}

impl InputMap {
    pub fn new() -> Self {
        Self {
            bindings: HashMap::new(),
        }
    }
    pub fn insert_actions() {}
    pub fn bind_action(&mut self, action: &str, key: InputType) {
        self.bindings
            .entry(action.to_string())
            .or_default()
            .push(key);
    }
}
