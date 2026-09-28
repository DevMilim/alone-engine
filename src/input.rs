pub use gilrs::{Axis, Button, GamepadId};
use rustc_hash::{FxHashMap, FxHashSet};
pub use winit::event::MouseButton;
pub use winit::keyboard::KeyCode;

use crate::math::Vector2;
#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub enum AxisDir {
    Positive,
    Negative,
}

#[derive(Clone, Copy, Debug)]
pub struct Rumble {
    pub strong: f32, // motor grave (0.0..=1.0)
    pub weak: f32,   // motor agudo (0.0..=1.0)
    pub duration_ms: u32,
}

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub enum InputType {
    Key(KeyCode),
    Mouse(MouseButton),
    Gamepad(Button),
    GamepadAxis(Axis, AxisDir),
}
pub struct InputState {
    pub pressed_input: FxHashSet<InputType>,
    pub just_pressed_input: FxHashMap<InputType, (u64, u64)>,
    pub mouse_position: Vector2,
    pub map: InputMap,
    pub current_update_frame: u64,
    pub current_fixed_frame: u64,
    pub axes: FxHashMap<Axis, f32>,
    pub active_gamepad: Option<GamepadId>,
    pub axis_press_threshold: f32,
    pub deadzone: f32,
    pub button_values: FxHashMap<Button, f32>,
    pub rumble_requests: Vec<Rumble>,
    pub continuous_rumble: f32,
}

impl InputState {
    pub fn new() -> Self {
        Self {
            pressed_input: FxHashSet::default(),
            just_pressed_input: FxHashMap::default(),
            mouse_position: Vector2::ZERO,
            map: InputMap::new(),
            current_update_frame: 0,
            current_fixed_frame: 0,
            axes: FxHashMap::default(),
            active_gamepad: None,
            axis_press_threshold: 0.0,
            deadzone: 0.0,
            button_values: FxHashMap::default(),
            rumble_requests: Vec::new(),
            continuous_rumble: 0.0,
        }
    }
    pub fn update_axis(&mut self, axis: Axis, value: f32) {
        self.axes.insert(axis, value);
        let t = self.axis_press_threshold;
        self.update_input_state(InputType::GamepadAxis(axis, AxisDir::Positive), value > t);
        self.update_input_state(InputType::GamepadAxis(axis, AxisDir::Negative), value < -t);
    }

    pub fn release_gamepad(&mut self) {
        self.pressed_input
            .retain(|i| !matches!(i, InputType::Gamepad(_) | InputType::GamepadAxis(..)));
        self.axes.clear();
        self.button_values.clear();
    }
    pub fn update_button_value(&mut self, button: Button, value: f32) {
        self.button_values.insert(button, value);
    }

    pub fn gamepad_button_value(&self, button: Button) -> f32 {
        self.input_strength(&InputType::Gamepad(button))
    }

    pub fn get_gamepad_axis(&self, negative_action: &str, positive_action: &str) -> f32 {
        self.action_strength(positive_action) - self.action_strength(negative_action)
    }

    pub fn get_gamepad_vector(&self, up: &str, down: &str, left: &str, right: &str) -> Vector2 {
        let x = self.action_strength(right) - self.action_strength(left);
        let y = self.action_strength(down) - self.action_strength(up);
        let vec = Vector2::new(x, y);
        if vec.length() > 1.0 {
            vec.normalize()
        } else {
            vec
        }
    }
    pub fn action_strength(&self, action: &str) -> f32 {
        self.map
            .bindings
            .get(action)
            .map(|b| b.iter().map(|i| self.input_strength(i)).fold(0.0, f32::max))
            .unwrap_or(0.0)
    }
    pub fn input_strength(&self, input: &InputType) -> f32 {
        match input {
            InputType::GamepadAxis(axis, dir) => {
                let v = self.axes.get(axis).copied().unwrap_or(0.0);
                let v = match dir {
                    AxisDir::Positive => v.max(0.0),
                    AxisDir::Negative => (-v).max(0.0),
                };
                if v < self.deadzone { 0.0 } else { v }
            }
            InputType::Gamepad(b) => match self.button_values.get(b) {
                Some(&v) if v < self.deadzone => 0.0,
                Some(&v) => v,
                None => self.pressed_input.contains(input) as i32 as f32,
            },
            other => self.pressed_input.contains(other) as i32 as f32,
        }
    }
    pub fn rumble(&mut self, strong: f32, weak: f32, duration_ms: u32) {
        self.rumble_requests.push(Rumble {
            strong,
            weak,
            duration_ms,
        });
    }
    pub fn set_continuous_rumble(&mut self, intensity: f32) {
        self.continuous_rumble = self.continuous_rumble.max(intensity.clamp(0.0, 1.0));
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
            .retain(|_, (target_u, target_f)| *target_u >= current_u || *target_f >= current_f);
        self.continuous_rumble = 0.0;
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
    pub bindings: FxHashMap<String, Vec<InputType>>,
}

impl Default for InputMap {
    fn default() -> Self {
        Self::new()
    }
}

impl InputMap {
    pub fn new() -> Self {
        Self {
            bindings: FxHashMap::default(),
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
