use crate::{Id, Vector2};
use std::{
    collections::{HashMap, HashSet},
    hash::{DefaultHasher, Hash, Hasher},
};

use indexmap::IndexMap;

#[derive(Clone, Copy, Debug)]
pub struct AABB {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl AABB {
    /// Verifica se existe algoma intersecçao entre o proprio colisor e outro
    pub fn intersects(&self, other: &AABB) -> bool {
        self.x < other.x + other.width
            && self.x + self.width > other.x
            && self.y < other.y + other.height
            && self.y + self.height > other.y
    }
    pub fn get_overlap(&self, other: &AABB) -> Option<Vector2> {
        let left = other.x - (self.x + self.width);

        let right = (other.x + other.width) - self.x;

        let top = other.y - (self.y + self.height);
        let bottom = (other.y + other.height) - self.y;

        if left >= 0.0 || right <= 0.0 || top >= 0.0 || bottom <= 0.0 {
            return None;
        }
        let overlap_x = if left.abs() < right.abs() {
            left
        } else {
            right
        };

        let overlap_y = if top.abs() < bottom.abs() {
            top
        } else {
            bottom
        };

        if overlap_x.abs() < overlap_y.abs() {
            Some(Vector2::new(overlap_x, 0.0))
        } else {
            Some(Vector2::new(0.0, overlap_y))
        }
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct CollisionFlag {
    pub on_floor: bool,
    pub on_wall: bool,
    pub on_ceiling: bool,
}

#[derive(Clone, Copy)]
pub struct ColliderData {
    pub aabb: AABB,
    pub layer: u32,
    pub mask: u32,
    pub is_sensor: bool,
}

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub struct ColliderKey {
    pub key: u32,
    pub id: Id,
}

type Cell = (i32, i32);
pub fn cell_of(aabb: &AABB, cell_size: f32) -> Vec<Cell> {
    let min_x = (aabb.x / cell_size).floor() as i32;
    let min_y = (aabb.y / cell_size).floor() as i32;

    let max_x = ((aabb.x + aabb.width) / cell_size).floor() as i32;
    let max_y = ((aabb.y + aabb.height) / cell_size).floor() as i32;

    let mut cells = Vec::new();

    for x in min_x..=max_x {
        for y in min_y..=max_y {
            cells.push((x, y));
        }
    }

    cells
}
pub struct CollisionWorld {
    pub colliders: IndexMap<ColliderKey, ColliderData>,

    grid: HashMap<Cell, Vec<ColliderKey>>,
    last_overlaps: IndexMap<(ColliderKey, ColliderKey), ()>,
    current_overlaps: IndexMap<(ColliderKey, ColliderKey), ()>,
}

impl CollisionWorld {
    pub fn new() -> Self {
        Self {
            colliders: IndexMap::new(),
            last_overlaps: IndexMap::new(),
            current_overlaps: IndexMap::new(),
            grid: HashMap::new(),
        }
    }
    pub fn step(&mut self) {
        self.current_overlaps.clear();
        self.grid.clear();

        let cell_size = 64.0;

        for (key, data) in &self.colliders {
            for cell in cell_of(&data.aabb, cell_size) {
                self.grid.entry(cell).or_default().push(key.clone());
            }
        }

        let mut tested = HashSet::new();

        for bucket in self.grid.values() {
            for i in 0..bucket.len() {
                for j in (i + 1)..bucket.len() {
                    let a = bucket[i].clone();
                    let b = bucket[j].clone();

                    let pair = Self::ordered_pair(a.clone(), b.clone());

                    if tested.contains(&pair) {
                        continue;
                    }

                    tested.insert(pair.clone());

                    let da = self.colliders.get(&a).unwrap();
                    let db = self.colliders.get(&b).unwrap();

                    let can = (da.mask & db.layer) != 0 && (db.mask & da.layer) != 0;

                    if !can {
                        continue;
                    }
                    if da.aabb.intersects(&db.aabb) {
                        self.current_overlaps.insert(pair, ());
                    }
                }
            }
        }
    }

    fn ordered_pair(a: ColliderKey, b: ColliderKey) -> (ColliderKey, ColliderKey) {
        if Self::hash_key(&a) <= Self::hash_key(&b) {
            (a, b)
        } else {
            (b, a)
        }
    }
    fn hash_key(k: &ColliderKey) -> u64 {
        let mut hasher = DefaultHasher::new();

        k.key.hash(&mut hasher);
        k.id.hash(&mut hasher);
        hasher.finish()
    }
    pub fn commit(&mut self) {
        self.last_overlaps.clear();
        for (k, ()) in &self.current_overlaps {
            self.last_overlaps.insert(k.clone(), ());
        }
    }

    pub fn get_entered_pairs(&self) -> Vec<(ColliderKey, ColliderKey)> {
        self.current_overlaps
            .keys()
            .filter(|k| !self.last_overlaps.contains_key(*k))
            .cloned()
            .collect()
    }

    pub fn get_exited_pairs(&self) -> Vec<(ColliderKey, ColliderKey)> {
        self.last_overlaps
            .keys()
            .filter(|k| !self.current_overlaps.contains_key(*k))
            .cloned()
            .collect()
    }
    pub fn get_entered_for(&self, my_id: Id) -> Vec<ColliderKey> {
        self.get_entered_pairs()
            .into_iter()
            .filter_map(|(a, b)| {
                if a.id == my_id {
                    Some(b)
                } else if b.id == my_id {
                    Some(a)
                } else {
                    None
                }
            })
            .collect()
    }
    pub fn get_exited_for(&self, my_id: Id) -> Vec<ColliderKey> {
        self.get_exited_pairs()
            .into_iter()
            .filter_map(|(a, b)| {
                if a.id == my_id {
                    Some(b)
                } else if b.id == my_id {
                    Some(a)
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn update_collider(&mut self, key: ColliderKey, data: ColliderData) {
        self.colliders.insert(key, data);
    }
    pub fn remove_collider(&mut self, key: ColliderKey) {
        self.colliders.swap_remove(&key);
    }
    pub fn get_currection(&self, my_id: Id, my_data: &ColliderData) -> Option<Vector2> {
        let mut correction = Vector2::ZERO;

        for (key, other) in &self.colliders {
            if key.id == my_id {
                continue;
            }

            if my_data.is_sensor || other.is_sensor {
                continue;
            }

            let can_colide = (my_data.mask & other.layer) != 0 && (other.mask & my_data.layer) != 0;

            if !can_colide {
                continue;
            }

            if let Some(overlap) = my_data.aabb.get_overlap(&other.aabb) {
                const MARGIN: f32 = 0.001;

                if overlap.x.abs() > correction.x.abs() {
                    let sign = if overlap.x > 0.0 { 1.0 } else { -1.0 };
                    correction.x = overlap.x + (MARGIN * sign);
                }
                if overlap.y.abs() > correction.y.abs() {
                    let sign = if overlap.y > 0.0 { 1.0 } else { -1.0 };
                    correction.y = overlap.y + (MARGIN * sign);
                }
            }
        }
        if correction != Vector2::ZERO {
            Some(correction)
        } else {
            None
        }
    }
    pub fn move_and_slide(
        &mut self,
        my_id: Id,
        position: &mut Vector2,
        velocity: &mut Vector2,
    ) -> CollisionFlag {
        let mut flags = CollisionFlag::default();
        let movement = *velocity;

        position.x += movement.x;
        self.translate_my_colliders(my_id, Vector2::new(movement.x, 0.0));
        let corr_x = self.resolve_axis(my_id, position, velocity, true).x;
        if corr_x != 0.0 {
            flags.on_wall = true;
        }
        position.y += movement.y;
        self.translate_my_colliders(my_id, Vector2::new(0.0, movement.y));
        let corr_y = self.resolve_axis(my_id, position, velocity, false).y;

        if corr_y < 0.0 {
            flags.on_floor = true;
        } else if corr_y > 0.0 {
            flags.on_ceiling = true;
        }

        flags
    }
    pub fn translate_my_colliders(&mut self, my_id: Id, offset: Vector2) {
        for (key, data) in &mut self.colliders {
            if key.id == my_id {
                data.aabb.x += offset.x;
                data.aabb.y += offset.y;
            }
        }
    }
    pub fn resolve_axis(
        &mut self,
        my_id: Id,
        position: &mut Vector2,
        velocity: &mut Vector2,
        is_x_axis: bool,
    ) -> Vector2 {
        let my_colliders: Vec<ColliderData> = self
            .colliders
            .iter()
            .filter(|(key, _)| key.id == my_id)
            .map(|(_, data)| *data)
            .collect();
        let mut final_correction = Vector2::ZERO;

        for data in my_colliders {
            if let Some(c) = self.get_currection(my_id, &data) {
                if is_x_axis {
                    if c.x.abs() > final_correction.x.abs() {
                        final_correction.x = c.x;
                    }
                } else {
                    if c.y.abs() > final_correction.y.abs() {
                        final_correction.y = c.y;
                    }
                }
            }
        }
        if is_x_axis {
            position.x += final_correction.x;
            if final_correction.x != 0.0 {
                velocity.x = 0.0;
                self.translate_my_colliders(my_id, Vector2::new(final_correction.x, 0.0));
            }
        } else {
            position.y += final_correction.y;
            if final_correction.y != 0.0 {
                velocity.y = 0.0;
                self.translate_my_colliders(my_id, Vector2::new(0.0, final_correction.y));
            }
        }
        final_correction
    }
    pub fn snap_to_floor(&mut self, my_id: Id, snap_length: f32) -> Option<f32> {
        let my_colliders: Vec<ColliderData> = self
            .colliders
            .iter()
            .filter(|(key, _)| key.id == my_id)
            .map(|(_, data)| *data)
            .collect();
        let mut best_snap = None;

        for my_data in my_colliders {
            if my_data.is_sensor {
                continue;
            }

            let my_left = my_data.aabb.x;
            let my_right = my_data.aabb.x + my_data.aabb.width;
            let my_bottom = my_data.aabb.y + my_data.aabb.height;

            for (key, other) in &self.colliders {
                if key.id == my_id || other.is_sensor {
                    continue;
                }

                let can_collide =
                    (my_data.mask & other.layer) != 0 && (other.mask & my_data.layer) != 0;

                if !can_collide {
                    continue;
                }

                let other_left = other.aabb.x;
                let other_right = other.aabb.x + other.aabb.width;
                let other_top = other.aabb.y;

                let horizontal_overlap = my_left < other_right && my_right > other_left;

                if !horizontal_overlap {
                    continue;
                }

                if my_bottom <= other_top {
                    let gap = other_top - my_bottom;

                    if gap <= snap_length {
                        match best_snap {
                            Some(current) if gap < current => best_snap = Some(gap),
                            None => best_snap = Some(gap),
                            _ => {}
                        }
                    }
                }
            }
        }

        best_snap
    }
}

impl Default for CollisionWorld {
    fn default() -> Self {
        Self::new()
    }
}
