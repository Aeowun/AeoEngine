use super::cell::{Cell, CellType, ChunkCoord, RuntimeCellState};
use super::coordinate::WorldCoord;
use crate::scripting::binding::ScriptBinding;
use glam::Vec3;
use std::collections::{HashMap, HashSet};

#[derive(Clone)]
pub struct LightingSettings {
    pub shadows_enabled: bool,
    pub global_light_enabled: bool,
    /// The direction light travels through the scene.
    pub global_light_direction: Vec3,
    pub global_light_color: Vec3,
    pub global_light_intensity: f32,
    pub ambient_intensity: f32,
}

impl Default for LightingSettings {
    fn default() -> Self {
        Self {
            shadows_enabled: true,
            global_light_enabled: true,
            // Default downward diagonal.
            global_light_direction: Vec3::new(0.5, -1.0, 0.5).normalize(),
            global_light_color: Vec3::ONE,
            global_light_intensity: 1.0,
            ambient_intensity: 0.20,
        }
    }
}

#[derive(Clone, Debug)]
pub struct SkySettings {
    pub enabled: bool,
    pub preset: String,
    pub texture: String,
}

impl Default for SkySettings {
    fn default() -> Self {
        let mut s = Self {
            enabled: true,
            preset: "Temperate".to_string(),
            texture: String::new(),
        };
        s.update_preset_textures();
        s
    }
}

impl SkySettings {
    pub fn update_preset_textures(&mut self) {
        self.texture = match self.preset.as_str() {
            "Tropical" => "Cubemap_Tropical_01-512x512.png",
            "Desert" => "Cubemap_Desert_01-512x512.png",
            "Snowy" => "Cubemap_Snowy_01-512x512.png",
            "Mars" => "Cubemap_Mars_01-512x512.png",
            _ => "Cubemap_Temperate_01-512x512.png",
        }
        .to_string();
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DirtyReason {
    /// Geometry, occupancy, visibility, solidity, anchored, or cell_type changed.
    /// Invalidation neighborhood: own chunk + 6 neighboring chunks.
    Geometry,
    /// Color, texture, or visual offset changed.
    /// Invalidation neighborhood: own chunk only.
    MaterialOrOffset,
}

#[derive(Clone)]
pub struct World {
    // Authored grid data. We use a HashMap because the world is unbounded
    // and most coordinates are empty.
    pub(crate) cells: HashMap<WorldCoord, Cell>,

    // Temporary runtime-only overrides for cell state, keyed by Cell ID.
    pub(crate) runtime_state: HashMap<u64, RuntimeCellState>,

    // Tracks which cells have had physics-relevant properties changed at runtime.
    pub(crate) physics_dirty_cells: std::collections::HashSet<u64>,

    /// Streamed chunk transitions are reconciled as a single physics rebuild,
    /// rather than one dirty-cell update per cell in a chunk.
    pub(crate) streaming_physics_rebuild_requested: bool,

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

    /// Authored chunks changed while resident.  The streamer uses this to
    /// flush only chunks that need to be written before eviction.
    pub(crate) dirty_authored_chunks: HashSet<ChunkCoord>,
}

#[derive(Clone, Debug, Default)]
pub struct ChunkCellIndex {
    pub active_coords: HashSet<WorldCoord>,
    pub solid_coords: HashSet<WorldCoord>,
}

#[derive(Clone, Debug, Default)]
pub struct WorldSpatialIndex {
    pub chunks: HashMap<ChunkCoord, ChunkCellIndex>,
}

impl WorldSpatialIndex {
    pub fn clear(&mut self) {
        self.chunks.clear();
    }

    pub fn update_cell(&mut self, coord: WorldCoord, active: bool, solid: bool) {
        let chunk_coord = ChunkCoord::from_world_coord(coord);
        if active {
            let chunk = self.chunks.entry(chunk_coord).or_default();
            chunk.active_coords.insert(coord);
            if solid {
                chunk.solid_coords.insert(coord);
            } else {
                chunk.solid_coords.remove(&coord);
            }
        } else if let Some(chunk) = self.chunks.get_mut(&chunk_coord) {
            chunk.active_coords.remove(&coord);
            chunk.solid_coords.remove(&coord);
            if chunk.active_coords.is_empty() {
                self.chunks.remove(&chunk_coord);
            }
        }
    }
}

impl World {
    pub fn new() -> Self {
        Self {
            cells: HashMap::new(),
            runtime_state: HashMap::new(),
            physics_dirty_cells: std::collections::HashSet::new(),
            streaming_physics_rebuild_requested: false,
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

    pub fn render_revision(&self) -> u64 {
        self.render_revision
    }

    pub(crate) fn bump_render_revision(&mut self) {
        self.render_revision = self.render_revision.wrapping_add(1);
    }

    pub fn mark_render_dirty(&self, coord: WorldCoord, reason: DirtyReason) {
        let mut map = self.render_dirty_cells.borrow_mut();
        match map.get(&coord) {
            Some(&DirtyReason::Geometry) => {
                // Geometry is already the broader invalidation
            }
            _ => {
                map.insert(coord, reason);
            }
        }
    }

    pub fn drain_render_dirty_cells(&self) -> HashMap<WorldCoord, DirtyReason> {
        std::mem::take(&mut *self.render_dirty_cells.borrow_mut())
    }

    pub fn update_spatial_index_at(&mut self, coord: WorldCoord) {
        if let Some(cell) = self.get_effective_cell(coord) {
            let active = cell.visible && cell.cell_type != CellType::Empty;
            let solid = active && self.is_cell_solid(coord);
            self.spatial_index.update_cell(coord, active, solid);
        } else {
            self.spatial_index.update_cell(coord, false, false);
        }
    }

    pub fn rebuild_spatial_index(&mut self) {
        self.spatial_index.clear();

        let coords: Vec<WorldCoord> = self.cells.keys().copied().collect();
        for coord in coords {
            self.update_spatial_index_at(coord);
        }

        let runtime_coords: Vec<WorldCoord> = self.runtime_id_to_coord.values().copied().collect();
        for coord in runtime_coords {
            self.update_spatial_index_at(coord);
        }
    }

    pub fn iter_active_chunks(&self) -> impl Iterator<Item = ChunkCoord> + '_ {
        self.spatial_index.chunks.keys().copied()
    }

    pub fn get_active_coords_in_chunk(
        &self,
        chunk_coord: ChunkCoord,
    ) -> Option<&HashSet<WorldCoord>> {
        self.spatial_index
            .chunks
            .get(&chunk_coord)
            .map(|c| &c.active_coords)
    }

    pub fn get_solid_coords_in_chunk(
        &self,
        chunk_coord: ChunkCoord,
    ) -> Option<&HashSet<WorldCoord>> {
        self.spatial_index
            .chunks
            .get(&chunk_coord)
            .map(|c| &c.solid_coords)
    }

    pub fn query_solid_coords_in_aabb(
        &self,
        min_coord: WorldCoord,
        max_coord: WorldCoord,
    ) -> Vec<(WorldCoord, Vec3)> {
        let min_chunk = ChunkCoord::from_world_coord(min_coord);
        let max_chunk = ChunkCoord::from_world_coord(max_coord);

        let mut results = Vec::new();

        for cx in min_chunk.x..=max_chunk.x {
            for cy in min_chunk.y..=max_chunk.y {
                for cz in min_chunk.z..=max_chunk.z {
                    let chunk_coord = ChunkCoord::new(cx, cy, cz);
                    if let Some(solid_coords) = self
                        .spatial_index
                        .chunks
                        .get(&chunk_coord)
                        .map(|c| &c.solid_coords)
                    {
                        for &coord in solid_coords {
                            if coord.x >= min_coord.x
                                && coord.x <= max_coord.x
                                && coord.y >= min_coord.y
                                && coord.y <= max_coord.y
                                && coord.z >= min_coord.z
                                && coord.z <= max_coord.z
                            {
                                let offset = self.get_visual_offset(coord);
                                let pos = Vec3::new(coord.x as f32, coord.y as f32, coord.z as f32)
                                    + offset;
                                results.push((coord, pos));
                            }
                        }
                    }
                }
            }
        }

        results
    }

    pub fn get(&self, coord: WorldCoord) -> Option<&Cell> {
        self.cells.get(&coord)
    }

    pub fn get_mut(&mut self, coord: WorldCoord) -> Option<&mut Cell> {
        self.bump_render_revision();
        self.mark_render_dirty(coord, DirtyReason::Geometry);
        self.dirty_authored_chunks
            .insert(ChunkCoord::from_world_coord(coord));
        self.cells.get_mut(&coord)
    }

    /// Promotes an authored chunk into the resident query cache without
    /// allocating IDs or enqueuing one physics change per cell.
    pub(crate) fn insert_loaded_chunk(
        &mut self,
        cells: impl IntoIterator<Item = (WorldCoord, Cell)>,
    ) {
        for (coord, cell) in cells {
            if let Some(previous) = self.cells.insert(coord, cell.clone()) {
                self.id_to_coord.remove(&previous.id);
                self.runtime_state.remove(&previous.id);
            }
            self.id_to_coord.insert(cell.id, coord);
            self.observe_authored_id(cell.id);
            if cell.cell_type == CellType::AudioEmitter {
                self.audio_emitter_ids.insert(cell.id);
            }
            self.update_spatial_index_at(coord);
            self.mark_render_dirty(coord, DirtyReason::Geometry);
        }
        self.streaming_physics_rebuild_requested = true;
    }

    /// Removes all authored cells belonging to a chunk from RAM. Runtime
    /// cells deliberately remain independent of authored chunk eviction.
    pub(crate) fn evict_authored_chunk(&mut self, chunk_coord: ChunkCoord) -> Vec<(WorldCoord, Cell)> {
        let coords: Vec<WorldCoord> = self.cells.keys().copied()
            .filter(|coord| ChunkCoord::from_world_coord(*coord) == chunk_coord)
            .collect();
        let mut cells = Vec::with_capacity(coords.len());
        for coord in coords {
            if let Some(cell) = self.cells.remove(&coord) {
                self.id_to_coord.remove(&cell.id);
                self.runtime_state.remove(&cell.id);
                self.audio_emitter_ids.remove(&cell.id);
                self.update_spatial_index_at(coord);
                self.mark_render_dirty(coord, DirtyReason::Geometry);
                cells.push((coord, cell));
            }
        }
        self.streaming_physics_rebuild_requested = true;
        cells
    }

    pub(crate) fn request_streaming_physics_rebuild(&mut self) {
        self.streaming_physics_rebuild_requested = true;
    }

    pub(crate) fn take_streaming_physics_rebuild_request(&mut self) -> bool {
        std::mem::take(&mut self.streaming_physics_rebuild_requested)
    }

    pub fn authored_cells_in_chunk(&self, chunk_coord: ChunkCoord) -> Vec<(WorldCoord, Cell)> {
        self.cells.iter()
            .filter(|(coord, _)| ChunkCoord::from_world_coord(**coord) == chunk_coord)
            .map(|(coord, cell)| (*coord, cell.clone()))
            .collect()
    }

    pub(crate) fn authored_chunk_coords(&self) -> HashSet<ChunkCoord> {
        self.cells
            .keys()
            .map(|coord| ChunkCoord::from_world_coord(*coord))
            .collect()
    }

    pub fn is_authored_chunk_dirty(&self, chunk_coord: ChunkCoord) -> bool {
        self.dirty_authored_chunks.contains(&chunk_coord)
    }

    pub(crate) fn mark_authored_chunk_clean(&mut self, chunk_coord: ChunkCoord) {
        self.dirty_authored_chunks.remove(&chunk_coord);
    }

    pub fn resident_authored_cell_count(&self) -> usize {
        self.cells.len()
    }

    /// Finds a cell's current coordinate by its unique ID.
    /// This is an O(1) lookup using the runtime index.
    pub fn resolve_cell_id(&self, id: u64) -> Option<WorldCoord> {
        if let Some(coord) = self.id_to_coord.get(&id) {
            if let Some(rs) = self.runtime_state.get(&id) {
                if rs.is_deleted {
                    return None;
                }
            }
            return Some(*coord);
        }
        self.runtime_id_to_coord.get(&id).cloned()
    }

    /// Marks a cell as needing physics reconciliation.
    pub fn mark_physics_dirty(&mut self, cell_id: u64) {
        self.physics_dirty_cells.insert(cell_id);
    }

    /// Returns the effective value of light_enabled for a coordinate,
    /// accounting for any runtime overrides.
    pub fn is_light_enabled(&self, coord: WorldCoord) -> bool {
        if let Some(cell) = self.get_effective_cell(coord) {
            if let Some(rs) = self.runtime_state.get(&cell.id) {
                if let Some(enabled) = rs.light_enabled {
                    return enabled;
                }
            }
            return cell.light_enabled;
        }
        false
    }

    /// Sets a runtime-only override for light_enabled.
    /// This does not modify the authored Cell data.
    pub fn set_light_enabled_runtime(&mut self, coord: WorldCoord, enabled: bool) {
        self.bump_render_revision();
        let id = self.get_effective_cell(coord).map(|c| c.id);
        if let Some(id) = id {
            self.runtime_state.entry(id).or_default().light_enabled = Some(enabled);
        }
    }

    /// Returns the effective value of visible for a coordinate,
    /// accounting for any runtime overrides.
    pub fn is_cell_visible(&self, coord: WorldCoord) -> bool {
        if let Some(cell) = self.get_effective_cell(coord) {
            if let Some(rs) = self.runtime_state.get(&cell.id) {
                if let Some(visible) = rs.visible {
                    return visible;
                }
            }
            return cell.visible;
        }
        false
    }

    /// Sets a runtime-only override for cell visibility.
    /// This does not modify the authored Cell data.
    pub fn set_cell_visible_runtime(&mut self, coord: WorldCoord, visible: bool) {
        self.bump_render_revision();
        self.mark_render_dirty(coord, DirtyReason::Geometry);
        let id = self.get_effective_cell(coord).map(|c| c.id);
        if let Some(id) = id {
            self.runtime_state.entry(id).or_default().visible = Some(visible);
        }
        self.update_spatial_index_at(coord);
    }

    // --- Audio Runtime Queries & Overrides ---

    pub fn is_audio_playing(&self, cell_id: u64) -> bool {
        if let Some(cell) = self.get_effective_cell_by_id(cell_id) {
            if let Some(rs) = self.runtime_state.get(&cell.id) {
                if let Some(playing) = rs.audio_playing {
                    return playing;
                }
            }
            return cell.playing;
        }
        false
    }

    pub fn is_audio_paused(&self, cell_id: u64) -> bool {
        if let Some(cell) = self.get_effective_cell_by_id(cell_id) {
            if let Some(rs) = self.runtime_state.get(&cell.id) {
                if let Some(paused) = rs.audio_paused {
                    return paused;
                }
            }
        }
        false
    }

    pub fn is_audio_looped(&self, cell_id: u64) -> bool {
        if let Some(cell) = self.get_effective_cell_by_id(cell_id) {
            if let Some(rs) = self.runtime_state.get(&cell.id) {
                if let Some(looped) = rs.audio_looped {
                    return looped;
                }
            }
            return cell.looped;
        }
        false
    }

    pub fn get_audio_volume(&self, cell_id: u64) -> f32 {
        if let Some(cell) = self.get_effective_cell_by_id(cell_id) {
            if let Some(rs) = self.runtime_state.get(&cell.id) {
                if let Some(vol) = rs.audio_volume {
                    return vol;
                }
            }
            return cell.volume;
        }
        1.0
    }

    pub fn get_audio_path(&self, cell_id: u64) -> String {
        if let Some(cell) = self.get_effective_cell_by_id(cell_id) {
            return cell.audio.clone();
        }
        String::new()
    }

    pub fn set_audio_playing_runtime(&mut self, cell_id: u64, playing: bool) {
        if self.get_effective_cell_by_id(cell_id).is_some() {
            let rs = self.runtime_state.entry(cell_id).or_default();
            rs.audio_playing = Some(playing);
            if playing {
                rs.audio_paused = Some(false);
            }
        }
    }

    pub fn set_audio_paused_runtime(&mut self, cell_id: u64, paused: bool) {
        if self.get_effective_cell_by_id(cell_id).is_some() {
            let rs = self.runtime_state.entry(cell_id).or_default();
            rs.audio_paused = Some(paused);
        }
    }

    pub fn set_audio_looped_runtime(&mut self, cell_id: u64, looped: bool) {
        if self.get_effective_cell_by_id(cell_id).is_some() {
            let rs = self.runtime_state.entry(cell_id).or_default();
            rs.audio_looped = Some(looped);
        }
    }

    pub fn set_audio_volume_runtime(&mut self, cell_id: u64, volume: f32) {
        if self.get_effective_cell_by_id(cell_id).is_some() {
            let rs = self.runtime_state.entry(cell_id).or_default();
            rs.audio_volume = Some(volume);
        }
    }

    pub fn audio_play_runtime(&mut self, cell_id: u64) {
        if self.get_effective_cell_by_id(cell_id).is_some() {
            println!("[AUDIO] play requested for emitter {}", cell_id);
            let rs = self.runtime_state.entry(cell_id).or_default();
            rs.audio_playing = Some(true);
            rs.audio_paused = Some(false);
        }
    }

    pub fn audio_stop_runtime(&mut self, cell_id: u64) {
        if self.get_effective_cell_by_id(cell_id).is_some() {
            let rs = self.runtime_state.entry(cell_id).or_default();
            rs.audio_playing = Some(false);
            rs.audio_paused = Some(false);
        }
    }

    pub fn audio_pause_runtime(&mut self, cell_id: u64) {
        if self.get_effective_cell_by_id(cell_id).is_some() {
            let rs = self.runtime_state.entry(cell_id).or_default();
            rs.audio_paused = Some(true);
        }
    }

    /// Returns the effective color of a cell, accounting for runtime overrides.
    pub fn get_effective_color(&self, coord: WorldCoord) -> Vec3 {
        if let Some(cell) = self.get_effective_cell(coord) {
            if let Some(rs) = self.runtime_state.get(&cell.id) {
                if let Some(color) = rs.color_rgb {
                    return color;
                }
            }
            return cell.color_rgb;
        }
        Vec3::ZERO
    }

    /// Sets a runtime-only override for cell color.
    pub fn set_cell_color_runtime(&mut self, coord: WorldCoord, color: Vec3) {
        self.bump_render_revision();
        self.mark_render_dirty(coord, DirtyReason::MaterialOrOffset);
        let id = self.get_effective_cell(coord).map(|c| c.id);
        if let Some(id) = id {
            self.runtime_state.entry(id).or_default().color_rgb = Some(color);
        }
    }

    /// Returns whether a cell is effectively solid.
    pub fn is_cell_solid(&self, coord: WorldCoord) -> bool {
        if let Some(cell) = self.get_effective_cell(coord) {
            if let Some(rs) = self.runtime_state.get(&cell.id) {
                if let Some(solid) = rs.solid {
                    return solid;
                }
            }
            return cell.solid;
        }
        false
    }

    /// Sets a runtime-only override for cell solidity.
    pub fn set_cell_solid_runtime(&mut self, coord: WorldCoord, solid: bool) {
        self.bump_render_revision();
        self.mark_render_dirty(coord, DirtyReason::Geometry);
        let id = self.get_effective_cell(coord).map(|c| c.id);
        if let Some(id) = id {
            self.runtime_state.entry(id).or_default().solid = Some(solid);
            self.mark_physics_dirty(id);
        }
        self.update_spatial_index_at(coord);
    }

    /// Returns whether a cell is effectively anchored.
    pub fn is_cell_anchored(&self, coord: WorldCoord) -> bool {
        if let Some(cell) = self.get_effective_cell(coord) {
            if let Some(rs) = self.runtime_state.get(&cell.id) {
                if let Some(anchored) = rs.anchored {
                    return anchored;
                }
            }
            return cell.anchored;
        }
        false
    }

    /// Sets a runtime-only override for cell anchored state.
    pub fn set_cell_anchored_runtime(&mut self, coord: WorldCoord, anchored: bool) {
        self.bump_render_revision();
        self.mark_render_dirty(coord, DirtyReason::Geometry);
        let id = self.get_effective_cell(coord).map(|c| c.id);
        if let Some(id) = id {
            self.runtime_state.entry(id).or_default().anchored = Some(anchored);
            self.mark_physics_dirty(id);
        }
    }

    /// Returns the effective visual offset of a cell.
    pub fn get_visual_offset(&self, coord: WorldCoord) -> Vec3 {
        if let Some(cell) = self.get_effective_cell(coord) {
            if let Some(rs) = self.runtime_state.get(&cell.id) {
                if let Some(offset) = rs.visual_offset {
                    return offset;
                }
            }
        }
        Vec3::ZERO
    }

    /// Sets a runtime-only visual offset for a cell.
    pub fn set_visual_offset_runtime(&mut self, coord: WorldCoord, offset: Vec3) {
        self.bump_render_revision();
        self.mark_render_dirty(coord, DirtyReason::MaterialOrOffset);
        let id = self.get_effective_cell(coord).map(|c| c.id);
        if let Some(id) = id {
            self.runtime_state.entry(id).or_default().visual_offset = Some(offset);
            self.mark_physics_dirty(id);
        }
    }

    pub fn get_effective_attributes(
        &self,
        coord: WorldCoord,
    ) -> std::collections::BTreeMap<String, super::cell::AttributeValue> {
        let mut result = std::collections::BTreeMap::new();
        if let Some(cell) = self.cells.get(&coord) {
            result.extend(cell.attributes.clone());
            if let Some(rs) = self.runtime_state.get(&cell.id) {
                result.extend(rs.attribute_overrides.clone());
            }
        }
        result
    }

    pub fn get_effective_attribute(
        &self,
        coord: WorldCoord,
        key: &str,
    ) -> Option<super::cell::AttributeValue> {
        if let Some(cell) = self.get_effective_cell(coord) {
            if let Some(rs) = self.runtime_state.get(&cell.id) {
                if let Some(val) = rs.attribute_overrides.get(key) {
                    return Some(val.clone());
                }
            }
            return cell.attributes.get(key).cloned();
        }
        None
    }

    pub fn get_effective_cell(&self, coord: WorldCoord) -> Option<&Cell> {
        if let Some(id) = self.coord_to_runtime_id.get(&coord) {
            return self.runtime_cells.get(id);
        }
        if let Some(cell) = self.cells.get(&coord) {
            if let Some(rs) = self.runtime_state.get(&cell.id) {
                if rs.is_deleted {
                    return None;
                }
            }
            return Some(cell);
        }
        None
    }

    pub fn get_effective_cell_by_id(&self, id: u64) -> Option<&Cell> {
        if let Some(cell) = self.runtime_cells.get(&id) {
            return Some(cell);
        }
        if let Some(coord) = self.id_to_coord.get(&id) {
            if let Some(rs) = self.runtime_state.get(&id) {
                if rs.is_deleted {
                    return None;
                }
            }
            return self.cells.get(coord);
        }
        None
    }

    pub fn active_effective_blocks(&self) -> Vec<WorldCoord> {
        let mut coords: std::collections::HashSet<WorldCoord> = std::collections::HashSet::new();
        for (coord, cell) in &self.cells {
            if let Some(rs) = self.runtime_state.get(&cell.id) {
                if rs.is_deleted {
                    continue;
                }
            }
            coords.insert(*coord);
        }
        coords.extend(self.runtime_id_to_coord.values().cloned());
        coords.into_iter().collect()
    }

    /// Resident-only effective coordinates without allocating a temporary
    /// whole-world vector. Runtime coordinates follow authored coordinates.
    pub fn iter_active_effective_coords(&self) -> impl Iterator<Item = WorldCoord> + '_ {
        self.cells.iter().filter_map(|(coord, cell)| {
            (!self.runtime_state.get(&cell.id).is_some_and(|state| state.is_deleted))
                .then_some(*coord)
        }).chain(self.runtime_id_to_coord.values().copied())
    }

    pub fn iter_audio_emitter_ids(&self) -> impl Iterator<Item = u64> + '_ {
        self.audio_emitter_ids.iter().copied()
    }

    pub fn create_runtime_cell(&mut self, cell_type: CellType) -> u64 {
        self.bump_render_revision();
        let mut cell = match cell_type {
            CellType::Block => Cell::new_block(),
            CellType::Light => Cell::new_light(),
            CellType::SpawnPoint => Cell::new_spawn_point(),
            CellType::AudioEmitter => Cell::new_audio_emitter(),
            _ => {
                let mut c = Cell::default();
                c.cell_type = cell_type;
                c
            }
        };

        let id = self.generate_runtime_unique_id();
        cell.id = id;

        if cell.cell_type == CellType::AudioEmitter {
            self.audio_emitter_ids.insert(id);
        }

        self.runtime_cells.insert(id, cell);
        self.mark_physics_dirty(id);
        id
    }

    pub fn move_runtime_cell(&mut self, id: u64, new_coord: WorldCoord) -> Result<(), String> {
        self.bump_render_revision();
        if !self.runtime_cells.contains_key(&id) {
            return Err("Not a runtime cell".to_string());
        }

        let old_coord_opt = self.runtime_id_to_coord.remove(&id);
        if let Some(old_coord) = old_coord_opt {
            self.coord_to_runtime_id.remove(&old_coord);
            self.mark_physics_dirty(id); // Dirty old position
            self.mark_render_dirty(old_coord, DirtyReason::Geometry);
            self.update_spatial_index_at(old_coord);
        }

        self.runtime_id_to_coord.insert(id, new_coord);
        self.coord_to_runtime_id.insert(new_coord, id);
        self.mark_physics_dirty(id); // Dirty new position
        self.mark_render_dirty(new_coord, DirtyReason::Geometry);
        self.update_spatial_index_at(new_coord);

        Ok(())
    }

    pub fn delete_cell_runtime(&mut self, id: u64) {
        self.bump_render_revision();
        let coord_opt = self.resolve_cell_id(id);
        if let Some(coord) = coord_opt {
            self.mark_render_dirty(coord, DirtyReason::Geometry);
        }
        if self.runtime_cells.contains_key(&id) {
            if let Some(coord) = self.runtime_id_to_coord.remove(&id) {
                self.coord_to_runtime_id.remove(&coord);
                self.mark_physics_dirty(id);
            }
            self.runtime_cells.remove(&id);
            self.runtime_state.remove(&id);
            self.audio_emitter_ids.remove(&id);
        } else if self.id_to_coord.contains_key(&id) {
            self.runtime_state.entry(id).or_default().is_deleted = true;
            self.mark_physics_dirty(id);
        }
        if let Some(coord) = coord_opt {
            self.update_spatial_index_at(coord);
        }
    }

    pub(crate) fn generate_runtime_unique_id(&self) -> u64 {
        use chrono::Local;
        use std::hash::{Hash, Hasher};

        let mut retry_count = 0;

        loop {
            let mut hasher = std::collections::hash_map::DefaultHasher::new();

            let timestamp = Local::now().format("%Y:%m:%d:%S:%f").to_string();

            timestamp.hash(&mut hasher);
            retry_count.hash(&mut hasher);

            let hash = hasher.finish();

            // Map to 9 digit range: 100,000,000 to 999,999,999.
            let id = 100_000_000 + (hash % 900_000_000);

            if !self.runtime_cells.contains_key(&id)
                && !self.cells.values().any(|cell| cell.id == id)
            {
                return id;
            }

            retry_count += 1;
        }
    }

    pub fn set_attribute_runtime(
        &mut self,
        coord: WorldCoord,
        key: String,
        value: super::cell::AttributeValue,
    ) {
        let id = self.get_effective_cell(coord).map(|c| c.id);
        if let Some(id) = id {
            self.runtime_state
                .entry(id)
                .or_default()
                .attribute_overrides
                .insert(key, value);
        }
    }

    pub fn remove_attribute_runtime(&mut self, coord: WorldCoord, key: &str) {
        let id = self.get_effective_cell(coord).map(|c| c.id);
        if let Some(id) = id {
            if let Some(rs) = self.runtime_state.get_mut(&id) {
                rs.attribute_overrides.remove(key);
            }
        }
    }

    /// Discards all runtime state modifications.
    pub fn clear_runtime_state(&mut self) {
        self.bump_render_revision();
        for id in self.runtime_state.keys() {
            self.physics_dirty_cells.insert(*id);
            if let Some(coord) = self.id_to_coord.get(id) {
                self.mark_render_dirty(*coord, DirtyReason::Geometry);
            }
        }
        for (id, coord) in &self.runtime_id_to_coord {
            self.physics_dirty_cells.insert(*id);
            self.mark_render_dirty(*coord, DirtyReason::Geometry);
        }
        self.runtime_state.clear();
        self.runtime_cells.clear();
        self.runtime_id_to_coord.clear();
        self.coord_to_runtime_id.clear();
        self.rebuild_audio_emitter_index();
        self.rebuild_spatial_index();
    }

    pub fn set_cell(&mut self, coord: WorldCoord, cell_type: CellType) -> u64 {
        self.bump_render_revision();
        self.mark_render_dirty(coord, DirtyReason::Geometry);
        self.dirty_authored_chunks
            .insert(ChunkCoord::from_world_coord(coord));
        let id = if cell_type == CellType::Empty {
            if let Some(cell) = self.cells.remove(&coord) {
                self.id_to_coord.remove(&cell.id);
                self.mark_physics_dirty(cell.id);
                self.runtime_state.remove(&cell.id);
                self.audio_emitter_ids.remove(&cell.id);
                cell.id
            } else {
                0
            }
        } else {
            // Default properties for a new cell.
            let mut cell = match cell_type {
                CellType::Block => Cell::new_block(),
                CellType::Light => Cell::new_light(),
                CellType::SpawnPoint => Cell::new_spawn_point(),
                CellType::AudioEmitter => Cell::new_audio_emitter(),
                _ => {
                    let mut c = Cell::default();
                    c.cell_type = cell_type;
                    c
                }
            };

            cell.id = self.generate_unique_id(coord, cell_type);
            let id = cell.id;
            self.id_to_coord.insert(id, coord);
            self.mark_physics_dirty(id);
            if cell_type == CellType::AudioEmitter {
                self.audio_emitter_ids.insert(id);
            }
            self.cells.insert(coord, cell);
            id
        };
        self.update_spatial_index_at(coord);
        id
    }

    /// Internal helper to update the ID index when an ID is changed manually (e.g. during loading).
    pub(crate) fn update_id_mapping(
        &mut self,
        old_id: u64,
        new_id: u64,
        coord: WorldCoord,
    ) {
        self.id_to_coord.remove(&old_id);
        self.id_to_coord.insert(new_id, coord);
        self.observe_authored_id(new_id);
    }

    /// Rebuilds the ID to coordinate index. Call this if the cells map is replaced (e.g. undo/redo).
    pub fn rebuild_id_mapping(&mut self) {
        self.bump_render_revision();
        for &coord in self.cells.keys() {
            self.mark_render_dirty(coord, DirtyReason::Geometry);
        }
        self.id_to_coord.clear();
        for (coord, cell) in &self.cells {
            self.id_to_coord.insert(cell.id, *coord);
        }
        self.rebuild_spatial_index();
        self.rebuild_audio_emitter_index();
        self.dirty_authored_chunks = self.cells.keys()
            .map(|coord| ChunkCoord::from_world_coord(*coord))
            .collect();
    }

    fn rebuild_audio_emitter_index(&mut self) {
        self.audio_emitter_ids.clear();
        self.audio_emitter_ids.extend(
            self.cells.values().chain(self.runtime_cells.values())
                .filter(|cell| cell.cell_type == CellType::AudioEmitter)
                .map(|cell| cell.id),
        );
    }

    pub(crate) fn generate_unique_id(
        &mut self,
        _coord: WorldCoord,
        _cell_type: CellType,
    ) -> u64 {
        let id = self.next_cell_id;

        self.next_cell_id = self
            .next_cell_id
            .checked_add(1)
            .expect("Authored cell ID space exhausted");

        id
    }

        /// Advances the allocator past an existing authored ID.
        ///
        /// Used when loading IDs that were already assigned on disk.
        pub(crate) fn observe_authored_id(&mut self, id: u64) {
            if id >= self.next_cell_id {
                self.next_cell_id = id
                    .checked_add(1)
                    .expect("Authored cell ID space exhausted");
            }
        }

    pub fn active_blocks(&self) -> Vec<WorldCoord> {
        self.cells.keys().cloned().collect()
    }

    /// Moves a collection of authored cells by a given delta offset, preserving their cell IDs.
    /// Returns Err if the movement destination collides with unrelated cells in the world.
    pub fn move_cells(
        &mut self,
        source_coords: &[WorldCoord],
        delta: WorldCoord,
    ) -> Result<(), String> {
        if delta == WorldCoord::new(0, 0, 0) || source_coords.is_empty() {
            return Ok(());
        }

        let source_set: std::collections::HashSet<WorldCoord> =
            source_coords.iter().cloned().collect();

        // Validate that no destination coordinate collides with an unrelated cell.
        for &src in source_coords {
            let dest = WorldCoord::new(src.x + delta.x, src.y + delta.y, src.z + delta.z);
            if self.cells.contains_key(&dest) && !source_set.contains(&dest) {
                return Err("Destination is occupied by an unrelated cell".to_string());
            }
        }

        self.bump_render_revision();

        // 1. Remove all source cells first to avoid self-overwrite when moving onto own positions.
        let mut moved = Vec::new();
        for &src in source_coords {
            if let Some(cell) = self.cells.remove(&src) {
                self.id_to_coord.remove(&cell.id);
                self.mark_physics_dirty(cell.id);
                self.mark_render_dirty(src, DirtyReason::Geometry);
                moved.push((src, cell));
            }
        }

        // 2. Re-insert cells at destination coordinates.
        for (old_coord, cell) in moved {
            let dest = WorldCoord::new(
                old_coord.x + delta.x,
                old_coord.y + delta.y,
                old_coord.z + delta.z,
            );
            let id = cell.id;
            self.cells.insert(dest, cell);
            self.id_to_coord.insert(id, dest);
            self.mark_physics_dirty(id);
            self.mark_render_dirty(dest, DirtyReason::Geometry);
        }

        Ok(())
    }

    /// Pastes a group of copied cells from clipboard at the given target pivot coordinate.
    /// Generates new unique IDs for every pasted cell and remaps script bindings.
    /// Returns Err if any destination coordinate is occupied by an existing cell.
    pub fn paste_cells(
        &mut self,
        clipboard_cells: &[crate::editor::ClipboardCell],
        target_pivot: WorldCoord,
    ) -> Result<Vec<WorldCoord>, String> {
        if clipboard_cells.is_empty() {
            return Ok(Vec::new());
        }

        // Validate that no destination coordinate is occupied.
        for item in clipboard_cells {
            let dest = WorldCoord::new(
                target_pivot.x + item.offset.x,
                target_pivot.y + item.offset.y,
                target_pivot.z + item.offset.z,
            );
            if self.cells.contains_key(&dest) {
                return Err("Destination is occupied".to_string());
            }
        }

        self.bump_render_revision();
        let mut pasted_coords = Vec::new();

        for item in clipboard_cells {
            let dest = WorldCoord::new(
                target_pivot.x + item.offset.x,
                target_pivot.y + item.offset.y,
                target_pivot.z + item.offset.z,
            );

            let mut cell = item.cell.clone();
            let new_id = self.generate_unique_id(dest, cell.cell_type);
            cell.id = new_id;

            self.cells.insert(dest, cell);
            self.id_to_coord.insert(new_id, dest);
            self.mark_physics_dirty(new_id);
            self.mark_render_dirty(dest, DirtyReason::Geometry);

            if let Some(ref binding) = item.script_binding {
                let mut new_binding = binding.clone();
                new_binding.target_identity = new_id;
                self.script_bindings.push(new_binding);
            }

            pasted_coords.push(dest);
        }

        Ok(pasted_coords)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::editor::ClipboardCell;
    use crate::editor::editor::History;
    use crate::scripting::binding::ScriptBinding;
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
                super::super::cell::AttributeValue::Number(100.0),
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
