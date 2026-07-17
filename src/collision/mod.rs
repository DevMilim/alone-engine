mod aabb;

pub use aabb::*;

use rustc_hash::FxHashMap;

use crate::{core::Id, math::Vector2i};

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
    #[inline]
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

const CELL_SIZE: i32 = 64;

#[inline]
pub fn cell_of(aabb: &AABB, cell_size: i32) -> impl Iterator<Item = Cell> {
    let min_x = aabb.x / cell_size;
    let min_y = aabb.y / cell_size;

    let max_x = (aabb.x + aabb.width) / cell_size;
    let max_y = (aabb.y + aabb.height) / cell_size;

    (min_x..=max_x).flat_map(move |x| (min_y..=max_y).map(move |y| (x, y)))
}

type DenseIndex = u32;

#[derive(Default)]
pub struct CollisionWorld {
    keys: Vec<ColliderKey>,
    data: Vec<ColliderData>,

    key_to_index: FxHashMap<ColliderKey, DenseIndex>,

    owners: FxHashMap<Id, Vec<ColliderKey>>,

    cell_offsets: FxHashMap<Cell, (u32, u32)>,
    cell_entries: Vec<DenseIndex>,
    scratch_cells: Vec<(Cell, DenseIndex)>,

    last_overlaps: Vec<(DenseIndex, DenseIndex)>,
    current_overlaps: Vec<(DenseIndex, DenseIndex)>,

    seen_stamp: Vec<u32>,
    current_stamp: u32,

    query_result: Vec<DenseIndex>,
}

impl CollisionWorld {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn update_collider_geometry(
        &mut self,
        key: ColliderKey,
        layer: u32,
        mask: u32,
        is_sensor: bool,
        on_way_collision: bool,
        size: (i32, i32),
        fallback_pos: (i32, i32),
    ) {
        if let Some(&idx) = self.key_to_index.get(&key) {
            let d = &mut self.data[idx as usize];
            d.layer = layer;
            d.mask = mask;
            d.is_sensor = is_sensor;
            d.on_way_collision = on_way_collision;
            d.aabb.width = size.0;
            d.aabb.height = size.1;
            return;
        }
        let data = ColliderData {
            aabb: AABB {
                x: fallback_pos.0,
                y: fallback_pos.1,
                width: size.0,
                height: size.1,
            },
            layer,
            mask,
            is_sensor,
            on_way_collision,
        };
        self.update_collider(key, data);
    }

    pub fn update_collider(&mut self, key: ColliderKey, data: ColliderData) {
        if let Some(&idx) = self.key_to_index.get(&key) {
            self.data[idx as usize] = data;
            return;
        }

        let idx = self.data.len() as DenseIndex;
        self.keys.push(key);
        self.data.push(data);
        self.seen_stamp.push(0);
        self.key_to_index.insert(key, idx);
        self.owners.entry(key.id).or_default().push(key);
    }

    pub fn remove_collider(&mut self, key: ColliderKey) {
        let Some(idx) = self.key_to_index.remove(&key) else {
            return;
        };
        let idx = idx as usize;
        let last = self.data.len() - 1;

        self.data.swap_remove(idx);
        self.keys.swap_remove(idx);
        self.seen_stamp.swap_remove(idx);

        if idx != last {
            let moved_key = self.keys[idx];
            self.key_to_index.insert(moved_key, idx as DenseIndex);
        }

        if let Some(list) = self.owners.get_mut(&key.id) {
            list.retain(|k| *k != key);
            if list.is_empty() {
                self.owners.remove(&key.id);
            }
        }
    }

    #[inline]
    pub fn get(&self, key: &ColliderKey) -> Option<&ColliderData> {
        self.key_to_index
            .get(key)
            .map(|&idx| &self.data[idx as usize])
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.data.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&ColliderKey, &ColliderData)> {
        self.keys.iter().zip(self.data.iter())
    }

    pub fn rebuild_grid(&mut self) {
        self.scratch_cells.clear();
        self.cell_offsets.clear();

        for (idx, data) in self.data.iter().enumerate() {
            for cell in cell_of(&data.aabb, CELL_SIZE) {
                self.scratch_cells.push((cell, idx as DenseIndex));
                self.cell_offsets.entry(cell).or_insert((0, 0)).1 += 1;
            }
        }

        self.cell_entries.clear();
        self.cell_entries.resize(self.scratch_cells.len(), 0);

        let mut cursor = 0u32;
        for slot in self.cell_offsets.values_mut() {
            let count = slot.1;
            slot.0 = cursor;
            slot.1 = 0;
            cursor += count;
        }

        for &(cell, idx) in &self.scratch_cells {
            let slot = self.cell_offsets.get_mut(&cell).unwrap();
            let write_pos = slot.0 + slot.1;
            self.cell_entries[write_pos as usize] = idx;
            slot.1 += 1;
        }
    }

    fn query_nearby(&mut self, aabb: &AABB) {
        self.current_stamp += 1;
        self.query_result.clear();

        for cell in cell_of(aabb, CELL_SIZE) {
            let Some(&(start, count)) = self.cell_offsets.get(&cell) else {
                continue;
            };
            let slice = &self.cell_entries[start as usize..(start + count) as usize];
            for &idx in slice {
                let stamp = &mut self.seen_stamp[idx as usize];
                if *stamp != self.current_stamp {
                    *stamp = self.current_stamp;
                    self.query_result.push(idx);
                }
            }
        }
    }
    pub fn step(&mut self) {
        self.rebuild_grid();
        self.current_overlaps.clear();

        for slot in self.cell_offsets.values().copied().collect::<Vec<_>>() {
            let (start, count) = slot;
            let bucket = &self.cell_entries[start as usize..(start + count) as usize];

            for i in 0..bucket.len() {
                for j in (i + 1)..bucket.len() {
                    let (a, b) = (bucket[i], bucket[j]);
                    let pair = if a < b { (a, b) } else { (b, a) };

                    let da = &self.data[pair.0 as usize];
                    let db = &self.data[pair.1 as usize];

                    if da.can_collide(db) && da.aabb.intersects(&db.aabb) {
                        self.current_overlaps.push(pair);
                    }
                }
            }
        }

        self.current_overlaps.sort_unstable();
        self.current_overlaps.dedup();
    }

    pub fn commit(&mut self) {
        self.last_overlaps.clear();
        self.last_overlaps.extend_from_slice(&self.current_overlaps);
    }

    fn diff_pairs(
        &self,
        a: &[(DenseIndex, DenseIndex)],
        b: &[(DenseIndex, DenseIndex)],
    ) -> Vec<(ColliderKey, ColliderKey)> {
        let mut result = Vec::new();
        let mut j = 0usize;

        for &pair in a {
            while j < b.len() && b[j] < pair {
                j += 1;
            }
            let present_in_b = j < b.len() && b[j] == pair;
            if !present_in_b {
                result.push((self.keys[pair.0 as usize], self.keys[pair.1 as usize]));
            }
        }

        result
    }

    pub fn get_entered_pairs(&self) -> Vec<(ColliderKey, ColliderKey)> {
        self.diff_pairs(&self.current_overlaps, &self.last_overlaps)
    }

    pub fn get_exited_pairs(&self) -> Vec<(ColliderKey, ColliderKey)> {
        self.diff_pairs(&self.last_overlaps, &self.current_overlaps)
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

    pub fn check_collisions(&mut self, my_id: Id) -> bool {
        let my_keys: Vec<ColliderKey> = self.owners.get(&my_id).cloned().unwrap_or_default();

        for key in &my_keys {
            let Some(&idx) = self.key_to_index.get(key) else {
                continue;
            };
            let my_data = self.data[idx as usize];
            if my_data.is_sensor {
                continue;
            }

            self.query_nearby(&my_data.aabb);

            for i in 0..self.query_result.len() {
                let other_idx = self.query_result[i];

                if self.keys[other_idx as usize].id == my_id {
                    continue;
                }

                let other = self.data[other_idx as usize];

                if other.is_sensor || !my_data.can_collide(&other) {
                    continue;
                }
                if my_data.aabb.intersects(&other.aabb) {
                    return true;
                }
            }
        }
        false
    }

    pub fn get_correction(
        &mut self,
        my_id: Id,
        my_data: &ColliderData,
        velocity: Vector2i,
    ) -> Option<Vector2i> {
        self.query_nearby(&my_data.aabb);
        for i in 0..self.query_result.len() {
            let idx = self.query_result[i];

            if self.keys[idx as usize].id == my_id {
                continue;
            }
            let other = self.data[idx as usize];

            if my_data.is_sensor || other.is_sensor || !my_data.can_collide(&other) {
                continue;
            }

            if other.on_way_collision {
                if velocity.y <= 0 {
                    continue;
                }
                let old_bottom = my_data.aabb.y + my_data.aabb.height - velocity.y;
                let new_bottom = my_data.aabb.y + my_data.aabb.height;

                let platform_top = other.aabb.y;

                if !self.should_collide_oneway(old_bottom, new_bottom, platform_top, velocity.y) {
                    continue;
                }
            }
            if my_data.aabb.intersects(&other.aabb) {
                let mut overlap = Vector2i::ZERO;

                if velocity.x != 0 {
                    overlap.x = if velocity.x > 0 {
                        other.aabb.min_x() - my_data.aabb.max_x()
                    } else {
                        other.aabb.max_x() - my_data.aabb.min_x()
                    };
                } else if velocity.y != 0 {
                    overlap.y = if velocity.y > 0 {
                        other.aabb.min_y() - my_data.aabb.max_y()
                    } else {
                        other.aabb.max_y() - my_data.aabb.min_y()
                    };
                } else {
                    if let Some(o) = my_data.aabb.get_overlap(&other.aabb) {
                        overlap = o;
                    }
                }

                return Some(overlap);
            }
        }
        None
    }

    pub fn move_and_slide(
        &mut self,
        my_id: Id,
        position: &mut Vector2i,
        velocity: &mut Vector2i,
    ) -> CollisionFlag {
        let mut flags = CollisionFlag::default();
        position.x += velocity.x;
        self.translate_my_colliders(my_id, Vector2i::new(velocity.x, 0));
        flags.on_wall = self.resolve_axis(my_id, position, velocity, true).x != 0;
        position.y += velocity.y;
        self.translate_my_colliders(my_id, Vector2i::new(0, velocity.y));
        let corr_y = self.resolve_axis(my_id, position, velocity, false).y;
        flags.on_floor = corr_y < 0;
        flags.on_ceiling = corr_y > 0;

        flags
    }

    pub fn translate_my_colliders(&mut self, my_id: Id, offset: Vector2i) {
        let Some(owner_keys) = self.owners.get(&my_id) else {
            return;
        };
        for key in owner_keys.clone() {
            if let Some(&idx) = self.key_to_index.get(&key) {
                let d = &mut self.data[idx as usize];
                d.aabb.x += offset.x;
                d.aabb.y += offset.y;
            }
        }
    }

    pub fn resolve_axis(
        &mut self,
        my_id: Id,
        position: &mut Vector2i,
        velocity: &mut Vector2i,
        is_x_axis: bool,
    ) -> Vector2i {
        let my_keys: Vec<ColliderKey> = self
            .owners
            .get(&my_id)
            .map(|keys| keys.clone())
            .unwrap_or_default();

        let mut final_correction = Vector2i::ZERO;
        let mut iterations = 0;
        const MAX_ITERATIONS: u32 = 4;

        while iterations < MAX_ITERATIONS {
            let mut moved = false;

            for key in &my_keys {
                let Some(&idx) = self.key_to_index.get(key) else {
                    continue;
                };
                let my_data = self.data[idx as usize];
                let vel = if is_x_axis {
                    Vector2i::new(velocity.x, 0)
                } else {
                    Vector2i::new(0, velocity.y)
                };

                if let Some(c) = self.get_correction(my_id, &my_data, vel) {
                    let axis_correction = if is_x_axis {
                        Vector2i::new(c.x, 0)
                    } else {
                        Vector2i::new(0, c.y)
                    };

                    if axis_correction != Vector2i::ZERO {
                        if is_x_axis {
                            final_correction.x += axis_correction.x;
                            position.x += axis_correction.x;
                            velocity.x = 0;
                        } else {
                            final_correction.y += axis_correction.y;
                            position.y += axis_correction.y;
                            velocity.y = 0;
                        }

                        self.translate_my_colliders(my_id, axis_correction);
                        moved = true;

                        break;
                    }
                }
            }

            if !moved {
                break;
            }

            iterations += 1;
        }

        final_correction
    }

    pub fn snap_to_floor(&mut self, my_id: Id, snap_length: i32) -> Option<i32> {
        let my_keys: Vec<ColliderKey> = self
            .owners
            .get(&my_id)
            .map(|keys| keys.clone())
            .unwrap_or_default();

        let mut best_snap = None;

        for key in &my_keys {
            let Some(&idx) = self.key_to_index.get(key) else {
                continue;
            };
            let my_data = self.data[idx as usize];
            if my_data.is_sensor {
                continue;
            }

            let my_left = my_data.aabb.x;
            let my_right = my_data.aabb.x + my_data.aabb.width;
            let my_bottom = my_data.aabb.y + my_data.aabb.height;

            let search_aabb = AABB {
                x: my_data.aabb.x,
                y: my_data.aabb.y,
                width: my_data.aabb.width,
                height: my_data.aabb.height + snap_length,
            };
            self.query_nearby(&search_aabb);

            for i in 0..self.query_result.len() {
                let other_idx = self.query_result[i];
                if self.keys[other_idx as usize].id == my_id {
                    continue;
                }
                let other = self.data[other_idx as usize];
                if other.is_sensor || !my_data.can_collide(&other) {
                    continue;
                }

                let other_left = other.aabb.x;
                let other_right = other.aabb.x + other.aabb.width;
                let other_top = other.aabb.y;

                if my_left < other_right && my_right > other_left && my_bottom <= other_top {
                    let gap = other_top - my_bottom;

                    if gap > 0 && gap <= snap_length {
                        best_snap = Some(best_snap.map_or(gap, |curr: i32| gap.min(curr)));
                    }
                }
            }
        }

        best_snap
    }

    fn should_collide_oneway(
        &self,
        my_old_bottom: i32,
        my_new_bottom: i32,
        platform_top: i32,
        velocity_y: i32,
    ) -> bool {
        velocity_y >= 0 && my_old_bottom <= platform_top && my_new_bottom >= platform_top
    }
}
