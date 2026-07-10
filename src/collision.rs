use indexmap::{IndexMap, IndexSet};
use std::collections::{HashMap, HashSet};

use crate::{core::Id, math::Vector2};

#[derive(Clone, Copy, Debug)]
pub struct AABB {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl AABB {
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

        Some(if overlap_x.abs() < overlap_y.abs() {
            Vector2::new(overlap_x, 0.0)
        } else {
            Vector2::new(0.0, overlap_y)
        })
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
    pub on_way_collision: bool,
}

impl ColliderData {
    pub fn can_collide(&self, other: &Self) -> bool {
        (self.mask & other.layer) != 0 && (other.mask & self.layer) != 0
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, Debug)]
pub struct ColliderKey {
    pub key: Id,
    pub id: Id,
}

type Cell = (i32, i32);

pub fn cell_of(aabb: &AABB, cell_size: f32) -> impl Iterator<Item = Cell> {
    let min_x = (aabb.x / cell_size).floor() as i32;
    let min_y = (aabb.y / cell_size).floor() as i32;

    let max_x = (((aabb.x + aabb.width) - 0.001) / cell_size).floor() as i32;
    let max_y = (((aabb.y + aabb.height) - 0.001) / cell_size).floor() as i32;

    (min_x..=max_x).flat_map(move |x| (min_y..=max_y).map(move |y| (x, y)))
}

#[derive(Default)]
pub struct CollisionWorld {
    pub colliders: IndexMap<ColliderKey, ColliderData>,
    grid: HashMap<Cell, Vec<ColliderKey>>,
    last_overlaps: IndexSet<(ColliderKey, ColliderKey)>,
    current_overlaps: IndexSet<(ColliderKey, ColliderKey)>,
    platform_deltas: HashMap<Id, Vector2>,
}

impl CollisionWorld {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn step(&mut self) {
        self.current_overlaps.clear();
        self.grid.clear();

        for (&key, data) in &self.colliders {
            for cell in cell_of(&data.aabb, 64.0) {
                self.grid.entry(cell).or_default().push(key);
            }
        }

        let mut tested = HashSet::new();

        for bucket in self.grid.values() {
            for i in 0..bucket.len() {
                for j in (i + 1)..bucket.len() {
                    let (a, b) = (bucket[i], bucket[j]);

                    let pair = if a < b { (a, b) } else { (b, a) };

                    if !tested.insert(pair) {
                        continue;
                    }

                    let da = self.colliders.get(&a).unwrap();
                    let db = self.colliders.get(&b).unwrap();

                    if da.can_collide(db) && da.aabb.intersects(&db.aabb) {
                        self.current_overlaps.insert(pair);
                    }
                }
            }
        }
    }

    pub fn commit(&mut self) {
        self.last_overlaps = self.current_overlaps.clone();
    }

    pub fn get_entered_pairs(&self) -> Vec<(ColliderKey, ColliderKey)> {
        self.current_overlaps
            .difference(&self.last_overlaps)
            .copied()
            .collect()
    }

    pub fn get_exited_pairs(&self) -> Vec<(ColliderKey, ColliderKey)> {
        self.last_overlaps
            .difference(&self.current_overlaps)
            .copied()
            .collect()
    }

    fn filter_pairs(
        &self,
        pairs: impl Iterator<Item = (ColliderKey, ColliderKey)>,
        my_id: Id,
    ) -> Vec<ColliderKey> {
        pairs
            .filter_map(|(a, b)| {
                (a.id == my_id)
                    .then_some(b)
                    .or_else(|| (b.id == my_id).then_some(a))
            })
            .collect()
    }

    pub fn get_entered_for(&self, my_id: Id) -> Vec<ColliderKey> {
        self.filter_pairs(self.get_entered_pairs().into_iter(), my_id)
    }

    pub fn get_exited_for(&self, my_id: Id) -> Vec<ColliderKey> {
        self.filter_pairs(self.get_exited_pairs().into_iter(), my_id)
    }

    pub fn update_collider(&mut self, key: ColliderKey, data: ColliderData) {
        self.colliders.insert(key, data);
    }

    pub fn remove_collider(&mut self, key: ColliderKey) {
        self.colliders.swap_remove(&key);
    }

    pub fn get_correction(
        &self,
        my_id: Id,
        my_data: &ColliderData,
        velocity: Vector2,
    ) -> Option<Vector2> {
        let mut correction = Vector2::ZERO;

        for (key, other) in &self.colliders {
            if key.id == my_id
                || my_data.is_sensor
                || other.is_sensor
                || !my_data.can_collide(other)
            {
                continue;
            }

            if other.on_way_collision {
                if velocity.y <= 0.0 {
                    continue;
                }
                let old_bottom = my_data.aabb.y + my_data.aabb.height - velocity.y;
                let new_bottom = my_data.aabb.y + my_data.aabb.height;

                let platform_top = other.aabb.y;

                if !self.should_collide_oneway(old_bottom, new_bottom, platform_top, velocity.y) {
                    continue;
                }
            }

            if let Some(overlap) = my_data.aabb.get_overlap(&other.aabb) {
                let update_axis = |corr: &mut f32, over: f32| {
                    if over.abs() > corr.abs() {
                        *corr = over + (0.001 * over.signum());
                    }
                };
                update_axis(&mut correction.x, overlap.x);
                update_axis(&mut correction.y, overlap.y);
            }
        }
        (correction != Vector2::ZERO).then_some(correction)
    }

    pub fn move_and_slide(
        &mut self,
        my_id: Id,
        position: &mut Vector2,
        velocity: &mut Vector2,
    ) -> CollisionFlag {
        let mut flags = CollisionFlag::default();

        position.x += velocity.x;
        self.translate_my_colliders(my_id, Vector2::new(velocity.x, 0.0));
        flags.on_wall = self.resolve_axis(my_id, position, velocity, true).x != 0.0;

        position.y += velocity.y;
        self.translate_my_colliders(my_id, Vector2::new(0.0, velocity.y));
        let corr_y = self.resolve_axis(my_id, position, velocity, false).y;

        flags.on_floor = corr_y < 0.0;
        flags.on_ceiling = corr_y > 0.0;
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
        let my_colliders: Vec<_> = self
            .colliders
            .iter()
            .filter(|(k, _)| k.id == my_id)
            .map(|(_, &d)| d)
            .collect();

        let mut final_correction = Vector2::ZERO;

        for my_data in my_colliders {
            if let Some(c) = self.get_correction(my_id, &my_data, *velocity) {
                if is_x_axis && c.x.abs() > final_correction.x.abs() {
                    final_correction.x = c.x;
                }
                if !is_x_axis && c.y.abs() > final_correction.y.abs() {
                    final_correction.y = c.y;
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
        let my_colliders: Vec<_> = self
            .colliders
            .iter()
            .filter(|(k, _)| k.id == my_id)
            .map(|(_, &d)| d)
            .collect();

        let mut best_snap = None;

        for my_data in my_colliders.iter().filter(|d| !d.is_sensor) {
            let my_left = my_data.aabb.x;
            let my_right = my_data.aabb.x + my_data.aabb.width;
            let my_bottom = my_data.aabb.y + my_data.aabb.height;

            for (key, other) in &self.colliders {
                if key.id == my_id || other.is_sensor || !my_data.can_collide(other) {
                    continue;
                }

                let other_left = other.aabb.x;
                let other_right = other.aabb.x + other.aabb.width;
                let other_top = other.aabb.y;

                if my_left < other_right && my_right > other_left && my_bottom <= other_top {
                    let gap = other_top - my_bottom;
                    if gap <= snap_length {
                        best_snap = Some(best_snap.map_or(gap, |curr: f32| gap.min(curr)));
                    }
                }
            }
        }
        best_snap
    }
    fn should_collide_oneway(
        &self,
        my_old_bottom: f32,
        my_new_bottom: f32,
        platform_top: f32,
        velocity_y: f32,
    ) -> bool {
        velocity_y >= 0.0 && my_old_bottom <= platform_top && my_new_bottom >= platform_top
    }
}
