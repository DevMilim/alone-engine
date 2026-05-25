use std::collections::{HashMap, HashSet};

use winit::keyboard::KeyCode;

use crate::Vector2;

pub struct InputState {
    key_pressed: HashSet<KeyCode>,
    key_just_pressed: HashSet<KeyCode>,
    mouse_position: Vector2,
    pub map: InputMap,
}

impl InputState {
    pub fn new() -> Self {
        Self {
            key_pressed: HashSet::new(),
            key_just_pressed: HashSet::new(),
            mouse_position: Vector2::ZERO,
            map: InputMap::new(),
        }
    }
    pub fn set_mouse_position(&mut self, x: f32, y: f32) {
        self.mouse_position = Vector2::new(x, y);
    }
    pub fn is_action_pressed(&self, action: &str) -> bool {
        if let Some(keys) = self.map.bindings.get(action) {
            return keys.iter().any(|key| self.is_key_pressed(*key));
        }
        false
    }
    pub fn is_action_just_pressed(&self, action: &str) -> bool {
        if let Some(keys) = self.map.bindings.get(action) {
            return keys.iter().any(|key| self.is_key_just_pressed(*key));
        }
        false
    }
    pub fn mouse_position(&self) -> Vector2 {
        self.mouse_position
    }
    pub fn is_key_pressed(&self, key: KeyCode) -> bool {
        self.key_pressed.contains(&key)
    }
    pub fn is_key_just_pressed(&self, key: KeyCode) -> bool {
        self.key_just_pressed.contains(&key)
    }
    pub fn clear_frame_data(&mut self) {
        self.key_just_pressed.clear();
    }
    pub fn update_key(&mut self, key: KeyCode, pressed: bool) {
        if pressed {
            if !self.key_pressed.contains(&key) {
                self.key_just_pressed.insert(key);
            }
            self.key_pressed.insert(key);
        } else {
            self.key_pressed.remove(&key);
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
}

impl Default for InputState {
    fn default() -> Self {
        Self::new()
    }
}

pub struct InputMap {
    pub bindings: HashMap<String, Vec<KeyCode>>,
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
    pub fn bind_action(&mut self, action: &str, key: KeyCode) {
        self.bindings
            .entry(action.to_string())
            .or_default()
            .push(key);
    }
}
