use crate::world::cell::{Cell, ChunkCoord, RuntimeCellState};
use crate::world::coordinate::WorldCoord;
use crate::scripting::binding::ScriptBinding;
use glam::Vec3;
use std::collections::{HashMap, HashSet};

pub mod dirty;
pub mod mutations;
pub mod queries;
pub mod runtime;
pub mod settings;
pub mod spatial_index;

pub use dirty::DirtyReason;
pub use settings::{LightingSettings, SkySettings};
pub use spatial_index::{ChunkCellIndex, WorldSpatialIndex};

#[derive(Clone)]
pub struct World {
    // Authored grid data. We use a HashMap because the world is unbounded
    // and most coordinates are empty.
    pub(crate) cells: HashMap<WorldCoord, Cell>,

    // Temporary runtime-only overrides for cell state, keyed by Cell ID.
    pub(crate) runtime_state: HashMap<u64, RuntimeCellState>,

    // Tracks which cells have had physics-relevant properties changed at runtime.
    pub(crate) physics_dirty_cells: HashSet<u64>,

    // Optimized lookup for cell coordinates by ID.
    pub(crate) id_to_coord: HashMap<u64, WorldCoord>,
    /// Next globally unique authored cell ID.
    ///
    /// This is persistent world state. It must not depend on which chunks
    /// happen to be resident in memory.
    pub(crate) next_cell_id: u64,
    // Storage for cells created at runtime via cell.new()
    pub(crate) runtime_cells: HashMap<u64, Cell>,
    // ID -> Coord index for runtime cells
    pub(crate) runtime_id_to_coord: HashMap<u64, WorldCoord>,
    // Coord -> ID index for runtime cells (spatial index)
    pub(crate) coord_to_runtime_id: HashMap<WorldCoord, u64>,

    /// Resident authored/runtime audio emitters. Audio uses this index rather
    /// than scanning every resident cell each frame.
    pub(crate) audio_emitter_ids: HashSet<u64>,

    // The world wide gravity vector used by the physics simulation.
    pub gravity: Vec3,

    // Authoritative scene lighting settings.
    pub lighting: LightingSettings,

    // Authoritative scene sky settings.
    pub sky: SkySettings,

    /// Authored script bindings for entities in this world.
    pub script_bindings: Vec<ScriptBinding>,

    /// Paths to script files that are globally disabled.
    pub disabled_scripts: Vec<String>,

    /// Selected character package name for spawning.
    pub selected_character: String,

    /// Selected controller profile name.
    pub selected_controller: String,

    /// Selected camera profile name.
    pub selected_camera: String,

    /// Default Play-mode mouse cursor visibility setting.
    pub cursor_visible: bool,

    /// Default Play-mode mouse screen/cursor locking setting.
    pub screen_locked: bool,

    /// Incremented whenever World geometry or render-relevant properties change.
    pub(crate) render_revision: u64,

    /// Lightweight runtime-only record of coordinates that need render invalidation.
    pub(crate) render_dirty_cells: std::cell::RefCell<HashMap<WorldCoord, DirtyReason>>,

    /// Persistent spatial index partitioning cells into 16x16x16 chunks.
    pub spatial_index: WorldSpatialIndex,

    /// Authored chunks changed while resident.
    pub(crate) dirty_authored_chunks: HashSet<ChunkCoord>,
}

impl World {
    pub fn new() -> Self {
        Self {
            cells: HashMap::new(),
            runtime_state: HashMap::new(),
            physics_dirty_cells: HashSet::new(),
            id_to_coord: HashMap::new(),
            next_cell_id: 10_000_000,
            runtime_cells: HashMap::new(),
            runtime_id_to_coord: HashMap::new(),
            coord_to_runtime_id: HashMap::new(),
            audio_emitter_ids: HashSet::new(),
            gravity: Vec3::new(0.0, -9.81, 0.0),
            lighting: LightingSettings::default(),
            sky: SkySettings::default(),
            script_bindings: Vec::new(),
            disabled_scripts: Vec::new(),
            selected_character: "custom".to_string(),
            selected_controller: "thirdPerson_Controller".to_string(),
            selected_camera: "thirdPerson".to_string(),
            cursor_visible: false,
            screen_locked: true,
            render_revision: 1,
            render_dirty_cells: std::cell::RefCell::new(HashMap::new()),
            spatial_index: WorldSpatialIndex::default(),
            dirty_authored_chunks: HashSet::new(),
        }
    }
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::editor::ClipboardCell;
    use crate::editor::editor::History;
    use crate::scripting::binding::ScriptBinding;
    use crate::world::cell::CellType;
    use glam::Vec3;

    #[test]
    fn test_copy_preserves_full_cell_data_and_relative_offsets() {
        let mut world = World::new();
        let c1 = WorldCoord::new(10, 2, 10);
        let c2 = WorldCoord::new(11, 2, 10);

        let id1 = world.set_cell(c1, CellType::Block);
        if let Some(cell) = world.get_mut(c1) {
            cell.color_rgb = Vec3::new(1.0, 0.0, 0.0);
            cell.texture = "CustomTexture".to_string();
            cell.entity_identity = Some("MyEntity".to_string());
            cell.attributes.insert(
                "hp".to_string(),
                crate::world::cell::AttributeValue::Number(100.0),
            );
        }

        let id2 = world.set_cell(c2, CellType::Light);
        world
            .script_bindings
            .push(ScriptBinding::new(id1, "scripts/player.aeo"));

        let pivot = c1;
        let cell1 = world.get(c1).unwrap().clone();
        let cell2 = world.get(c2).unwrap().clone();

        let clip_cell1 = ClipboardCell {
            offset: WorldCoord::new(c1.x - pivot.x, c1.y - pivot.y, c1.z - pivot.z),
            cell: cell1,
            script_binding: world
                .script_bindings
                .iter()
                .find(|b| b.target_identity == id1)
                .cloned(),
        };

        let clip_cell2 = ClipboardCell {
            offset: WorldCoord::new(c2.x - pivot.x, c2.y - pivot.y, c2.z - pivot.z),
            cell: cell2,
            script_binding: world
                .script_bindings
                .iter()
                .find(|b| b.target_identity == id2)
                .cloned(),
        };

        // 1. Copy preserves full cell data
        assert_eq!(clip_cell1.cell.color_rgb, Vec3::new(1.0, 0.0, 0.0));
        assert_eq!(clip_cell1.cell.texture, "CustomTexture");
        assert_eq!(
            clip_cell1.cell.entity_identity,
            Some("MyEntity".to_string())
        );
        assert_eq!(
            clip_cell1.script_binding.unwrap().script_path,
            "scripts/player.aeo"
        );

        // 2. Relative coordinates stored correctly
        assert_eq!(clip_cell1.offset, WorldCoord::new(0, 0, 0));
        assert_eq!(clip_cell2.offset, WorldCoord::new(1, 0, 0));
    }

    #[test]
    fn test_authored_ids_are_monotonic() {
        let mut world = World::new();

        let id1 = world.set_cell(
            WorldCoord::new(0, 0, 0),
            CellType::Block,
        );

        let id2 = world.set_cell(
            WorldCoord::new(1, 0, 0),
            CellType::Block,
        );

        let id3 = world.set_cell(
            WorldCoord::new(2, 0, 0),
            CellType::Block,
        );

        assert_eq!(id1, 10_000_000);
        assert_eq!(id2, 10_000_001);
        assert_eq!(id3, 10_000_002);

        assert_eq!(world.next_cell_id, 10_000_003);
    }

    #[test]
    fn test_deleted_authored_id_is_not_reused() {
        let mut world = World::new();

        let first = WorldCoord::new(0, 0, 0);
        let second = WorldCoord::new(1, 0, 0);

        let id1 = world.set_cell(first, CellType::Block);

        world.set_cell(first, CellType::Empty);

        let id2 = world.set_cell(second, CellType::Block);

        assert_eq!(id2, id1 + 1);
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_observing_existing_id_advances_allocator() {
        let mut world = World::new();

        world.observe_authored_id(50_000_000);

        let id = world.set_cell(
            WorldCoord::new(0, 0, 0),
            CellType::Block,
        );

        assert_eq!(id, 50_000_001);
        assert_eq!(world.next_cell_id, 50_000_002);
    }

    #[test]
    fn test_paste_creates_new_ids_and_preserves_properties_bindings_and_layout() {
        let mut world = World::new();
        let c1 = WorldCoord::new(10, 2, 10);
        let c2 = WorldCoord::new(11, 2, 10);

        let id1 = world.set_cell(c1, CellType::Block);
        if let Some(cell) = world.get_mut(c1) {
            cell.color_rgb = Vec3::new(0.8, 0.1, 0.2);
            cell.entity_identity = Some("Hero".to_string());
        }

        let id2 = world.set_cell(c2, CellType::Block);
        world
            .script_bindings
            .push(ScriptBinding::new(id1, "scripts/hero.aeo"));

        let clipboard = vec![
            ClipboardCell {
                offset: WorldCoord::new(0, 0, 0),
                cell: world.get(c1).unwrap().clone(),
                script_binding: world
                    .script_bindings
                    .iter()
                    .find(|b| b.target_identity == id1)
                    .cloned(),
            },
            ClipboardCell {
                offset: WorldCoord::new(1, 0, 0),
                cell: world.get(c2).unwrap().clone(),
                script_binding: None,
            },
        ];

        let target_pivot = WorldCoord::new(20, 5, 20);
        let pasted_coords = world
            .paste_cells(&clipboard, target_pivot)
            .expect("Paste should succeed");

        // 3. New IDs
        let pasted_cell1 = world.get(WorldCoord::new(20, 5, 20)).unwrap();
        let pasted_cell2 = world.get(WorldCoord::new(21, 5, 20)).unwrap();
        assert_ne!(pasted_cell1.id, id1);
        assert_ne!(pasted_cell2.id, id2);

        // 4. Pasted properties preserved
        assert_eq!(pasted_cell1.color_rgb, Vec3::new(0.8, 0.1, 0.2));
        assert_eq!(pasted_cell1.entity_identity, Some("Hero".to_string()));

        // 5. Pasted script binding remapped
        let new_binding = world
            .script_bindings
            .iter()
            .find(|b| b.target_identity == pasted_cell1.id);
        assert!(new_binding.is_some());
        assert_eq!(new_binding.unwrap().script_path, "scripts/hero.aeo");

        // 6. Multi-cell paste relative layout preserved
        assert_eq!(
            pasted_coords,
            vec![WorldCoord::new(20, 5, 20), WorldCoord::new(21, 5, 20)]
        );
    }

    #[test]
    fn test_move_preserves_ids_bindings_layout_and_allows_self_overlap() {
        let mut world = World::new();
        let c1 = WorldCoord::new(10, 2, 10);
        let c2 = WorldCoord::new(11, 2, 10);

        let id1 = world.set_cell(c1, CellType::Block);
        let id2 = world.set_cell(c2, CellType::Block);
        world
            .script_bindings
            .push(ScriptBinding::new(id1, "scripts/block1.aeo"));

        let source_coords = vec![c1, c2];
        let delta = WorldCoord::new(1, 0, 0);

        // 11. Moving onto own positions allowed!
        world
            .move_cells(&source_coords, delta)
            .expect("Move onto own positions should succeed");

        // 7. Multi-cell move preserves IDs
        let moved1 = world.get(WorldCoord::new(11, 2, 10)).unwrap();
        let moved2 = world.get(WorldCoord::new(12, 2, 10)).unwrap();
        assert_eq!(moved1.id, id1);
        assert_eq!(moved2.id, id2);

        // 8. Script bindings preserved
        assert_eq!(world.resolve_cell_id(id1), Some(WorldCoord::new(11, 2, 10)));

        // 9. Relative layout preserved
        assert!(world.get(c1).is_none());
    }

    #[test]
    fn test_destination_collision_rejects_atomically() {
        let mut world = World::new();
        let c1 = WorldCoord::new(10, 2, 10);
        let c_obstacle = WorldCoord::new(15, 2, 10);

        let id1 = world.set_cell(c1, CellType::Block);
        let _id_obs = world.set_cell(c_obstacle, CellType::Block);

        // Try moving c1 to c_obstacle
        let res = world.move_cells(&[c1], WorldCoord::new(5, 0, 0));

        // 10. Destination collision rejects operation atomically
        assert!(res.is_err());
        assert_eq!(world.get(c1).unwrap().id, id1);
    }

    #[test]
    fn test_noop_move_does_nothing() {
        let mut world = World::new();
        let c1 = WorldCoord::new(10, 2, 10);
        let id1 = world.set_cell(c1, CellType::Block);

        // 12. No-op move does nothing
        assert!(world.move_cells(&[c1], WorldCoord::new(0, 0, 0)).is_ok());
        assert_eq!(world.get(c1).unwrap().id, id1);
    }

    #[test]
    fn test_undo_and_redo_restore_exact_previous_world() {
        let mut world = World::new();
        let c1 = WorldCoord::new(10, 2, 10);
        let id1 = world.set_cell(c1, CellType::Block);
        world
            .script_bindings
            .push(ScriptBinding::new(id1, "scripts/test.aeo"));

        let mut history = History::new();
        history.push(world.cells.clone(), world.script_bindings.clone());

        // Perform move
        world.move_cells(&[c1], WorldCoord::new(5, 5, 5)).unwrap();
        assert!(world.get(c1).is_none());
        assert!(world.get(WorldCoord::new(15, 7, 15)).is_some());

        // 13. Undo restores exact previous world
        let (prev_cells, prev_bindings) = history.undo_stack.pop().unwrap();
        history
            .redo_stack
            .push((world.cells.clone(), world.script_bindings.clone()));
        world.cells = prev_cells;
        world.script_bindings = prev_bindings;
        world.rebuild_id_mapping();

        assert_eq!(world.get(c1).unwrap().id, id1);
        assert!(world.get(WorldCoord::new(15, 7, 15)).is_none());
        assert_eq!(world.script_bindings.len(), 1);

        // 14. Redo reapplies move
        let (next_cells, next_bindings) = history.redo_stack.pop().unwrap();
        world.cells = next_cells;
        world.script_bindings = next_bindings;
        world.rebuild_id_mapping();

        assert!(world.get(c1).is_none());
        assert_eq!(world.get(WorldCoord::new(15, 7, 15)).unwrap().id, id1);
    }
}
