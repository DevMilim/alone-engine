mod aabb;

use crate::{FIXED_DT, Id, Vector2};
use std::{
    collections::{HashMap, HashSet},
    hash::{DefaultHasher, Hash, Hasher},
};

pub use aabb::*;
use indexmap::IndexMap;

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

        let _keys: Vec<_> = self.colliders.keys().cloned().collect();

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
        let mut total_correction = Vector2::ZERO;
        let mut collided = false;

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
                total_correction.x += overlap.x;
                total_correction.y += overlap.y;
                collided = true;
            }
        }
        if collided {
            Some(total_correction)
        } else {
            None
        }
    }
    pub fn move_and_slide(
        &mut self,
        my_id: Id,
        position: &mut Vector2,
        velocity: &mut Vector2,
        delta: f32,
    ) {
        let delta = FIXED_DT;
        let movement = *velocity * delta;

        position.x += movement.x;
        self.translate_my_colliders(my_id, Vector2::new(movement.x, 0.0));
        self.resolve_axis(my_id, position, velocity, true);

        position.y += movement.y;
        self.translate_my_colliders(my_id, Vector2::new(0.0, movement.y));
        self.resolve_axis(my_id, position, velocity, false);
    }
    fn translate_my_colliders(&mut self, my_id: Id, offset: Vector2) {
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
    ) {
        let my_colliders: Vec<ColliderData> = self
            .colliders
            .iter()
            .filter(|(key, _)| key.id == my_id)
            .map(|(_, data)| *data)
            .collect();

        for my_data in my_colliders {
            if let Some(corrections) = self.get_currection(my_id, &my_data) {
                if is_x_axis {
                    position.x += corrections.x;
                    velocity.x = 0.0;
                    self.translate_my_colliders(my_id, Vector2::new(corrections.x, 0.0));
                } else {
                    position.y += corrections.y;
                    velocity.y = 0.0;
                    self.translate_my_colliders(my_id, Vector2::new(0.0, corrections.y));
                }
            }
        }
    }
}

impl Default for CollisionWorld {
    fn default() -> Self {
        Self::new()
    }
}
