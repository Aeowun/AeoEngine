use super::CharacterSystem;
use crate::engine::entity::EntityId;
use crate::world::{World, WorldCoord};
use glam::Vec3;
use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap, HashSet};

impl CharacterSystem {
    /// Returns a navigation path from the character to the target position.
    ///
    /// The result contains world-space voxel-center waypoints.
    /// AeoScript is responsible for deciding how to follow the path.
    pub fn find_path_for_entity(
        &self,
        entity_id: EntityId,
        world: &World,
        target: Vec3,
    ) -> Option<Vec<Vec3>> {
        let character = self.get_character_for_entity(entity_id)?;

        let start = (
            character.transform.position.x.round() as i32,
            character.transform.position.y.round() as i32,
            character.transform.position.z.round() as i32,
        );

        let goal = (
            target.x.round() as i32,
            target.y.round() as i32,
            target.z.round() as i32,
        );

        if start == goal {
            return Some(Vec::new());
        }

        let active = world.active_blocks();

        if active.is_empty() {
            return None;
        }

        let mut min_x = start.0.min(goal.0);
        let mut max_x = start.0.max(goal.0);
        let mut min_y = start.1.min(goal.1);
        let mut max_y = start.1.max(goal.1);
        let mut min_z = start.2.min(goal.2);
        let mut max_z = start.2.max(goal.2);

        for coord in active {
            min_x = min_x.min(coord.x);
            max_x = max_x.max(coord.x);
            min_y = min_y.min(coord.y);
            max_y = max_y.max(coord.y);
            min_z = min_z.min(coord.z);
            max_z = max_z.max(coord.z);
        }

        const PADDING: i32 = 2;
        const MAX_NODES: usize = 8192;

        min_x -= PADDING;
        max_x += PADDING;
        min_y -= PADDING;
        max_y += PADDING;
        min_z -= PADDING;
        max_z += PADDING;

        let mut open: BinaryHeap<(Reverse<i32>, Reverse<i32>, (i32, i32, i32))> = BinaryHeap::new();

        let mut came_from: HashMap<(i32, i32, i32), (i32, i32, i32)> = HashMap::new();

        let mut cost_so_far: HashMap<(i32, i32, i32), i32> = HashMap::new();

        let mut closed: HashSet<(i32, i32, i32)> = HashSet::new();

        let start_cost = 0;
        let start_heuristic = Self::path_heuristic(start, goal);

        cost_so_far.insert(start, start_cost);

        open.push((
            Reverse(start_cost + start_heuristic),
            Reverse(start_cost),
            start,
        ));

        while let Some((Reverse(_f), Reverse(g), current)) = open.pop() {
            if current == goal {
                let mut path = Vec::new();
                let mut cursor = current;

                while cursor != start {
                    path.push(Vec3::new(cursor.0 as f32, cursor.1 as f32, cursor.2 as f32));

                    cursor = *came_from.get(&cursor)?;
                }

                path.reverse();

                return Some(path);
            }

            if !closed.insert(current) {
                continue;
            }

            let (x, y, z) = current;

            const DIRECTIONS: [(i32, i32, i32); 6] = [
                (1, 0, 0),
                (-1, 0, 0),
                (0, 1, 0),
                (0, -1, 0),
                (0, 0, 1),
                (0, 0, -1),
            ];

            for (dx, dy, dz) in DIRECTIONS {
                let next = (x + dx, y + dy, z + dz);

                if next.0 < min_x
                    || next.0 > max_x
                    || next.1 < min_y
                    || next.1 > max_y
                    || next.2 < min_z
                    || next.2 > max_z
                {
                    continue;
                }

                if !Self::path_cell_walkable(world, next) {
                    continue;
                }

                let next_cost = g + 1;

                let Some(existing_cost) = cost_so_far.get(&next) else {
                    cost_so_far.insert(next, next_cost);

                    came_from.insert(next, current);

                    open.push((
                        Reverse(next_cost + Self::path_heuristic(next, goal)),
                        Reverse(next_cost),
                        next,
                    ));

                    continue;
                };

                if next_cost >= *existing_cost {
                    continue;
                }

                cost_so_far.insert(next, next_cost);

                came_from.insert(next, current);

                open.push((
                    Reverse(next_cost + Self::path_heuristic(next, goal)),
                    Reverse(next_cost),
                    next,
                ));
            }

            if cost_so_far.len() >= MAX_NODES {
                return None;
            }
        }

        None
    }

    fn path_heuristic(a: (i32, i32, i32), b: (i32, i32, i32)) -> i32 {
        (a.0 - b.0).abs() + (a.1 - b.1).abs() + (a.2 - b.2).abs()
    }

    fn path_cell_walkable(world: &World, position: (i32, i32, i32)) -> bool {
        let (x, y, z) = position;

        let floor = WorldCoord::new(x, y - 1, z);

        let feet = WorldCoord::new(x, y, z);

        let head = WorldCoord::new(x, y + 1, z);

        world.is_cell_solid(floor) && !world.is_cell_solid(feet) && !world.is_cell_solid(head)
    }
}
