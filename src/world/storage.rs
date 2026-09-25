use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use glam::Vec3;
use serde::{Deserialize, Serialize};

use super::ChunkCoord;
use super::cell::{AttributeValue, Cell, CellType};
use super::coordinate::WorldCoord;

const CHUNK_FILE_VERSION: u32 = 1;
const CHUNK_EXTENSION: &str = "chunk";

/// Persistent on-disk storage for streamed world chunks.
///
/// This layer knows nothing about rendering, physics, cameras, or streaming.
/// It only owns the relationship between ChunkCoord and files on disk.
#[derive(Clone, Debug)]
pub struct WorldStorage {
    root: PathBuf,
}

impl WorldStorage {
    /// Creates a storage backend rooted at:
    ///
    /// <project_root>/chunks/
    pub fn new(project_root: impl AsRef<Path>) -> io::Result<Self> {
        let root = project_root.as_ref().join("chunks");

        fs::create_dir_all(&root)?;

        Ok(Self { root })
    }

    /// Returns the directory containing chunk files.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Returns the path used for a specific chunk.
    pub fn chunk_path(&self, chunk_coord: ChunkCoord) -> PathBuf {
        self.root.join(format!(
            "{}_{}_{}.{}",
            chunk_coord.x, chunk_coord.y, chunk_coord.z, CHUNK_EXTENSION
        ))
    }
    /// Returns every chunk coordinate currently stored on disk.
    pub fn list_chunks(&self) -> io::Result<Vec<ChunkCoord>> {
        let mut chunks = Vec::new();

        if !self.root.exists() {
            return Ok(chunks);
        }

        for entry in fs::read_dir(&self.root)? {
            let entry = entry?;
            let path = entry.path();

            if !path.is_file() {
                continue;
            }

            if path.extension().and_then(|value| value.to_str()) != Some(CHUNK_EXTENSION) {
                continue;
            }

            let Some(stem) = path.file_stem().and_then(|value| value.to_str()) else {
                continue;
            };

            let parts: Vec<&str> = stem.split('_').collect();

            if parts.len() != 3 {
                continue;
            }

            let Ok(x) = parts[0].parse::<i32>() else {
                continue;
            };

            let Ok(y) = parts[1].parse::<i32>() else {
                continue;
            };

            let Ok(z) = parts[2].parse::<i32>() else {
                continue;
            };

            chunks.push(ChunkCoord::new(x, y, z));
        }

        chunks.sort_unstable();
        Ok(chunks)
    }
    /// Returns true if a chunk exists on disk.
    pub fn chunk_exists(&self, chunk_coord: ChunkCoord) -> bool {
        self.chunk_path(chunk_coord).is_file()
    }

    /// Loads one chunk from disk.
    ///
    /// `Ok(None)` means the chunk has never been saved and should therefore
    /// be treated as an empty chunk.
    pub fn load_chunk(&self, chunk_coord: ChunkCoord) -> io::Result<Option<StoredChunk>> {
        let path = self.chunk_path(chunk_coord);

        if !path.exists() {
            return Ok(None);
        }

        let bytes = fs::read(&path)?;

        let chunk: StoredChunk = serde_json::from_slice(&bytes).map_err(|error| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "Failed to decode chunk ({}, {}, {}): {}",
                    chunk_coord.x, chunk_coord.y, chunk_coord.z, error
                ),
            )
        })?;

        if chunk.version != CHUNK_FILE_VERSION {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "Unsupported chunk version {} for ({}, {}, {}); expected {}",
                    chunk.version, chunk_coord.x, chunk_coord.y, chunk_coord.z, CHUNK_FILE_VERSION
                ),
            ));
        }

        if chunk.coord != chunk_coord {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "Chunk coordinate mismatch: requested ({}, {}, {}), file contains ({}, {}, {})",
                    chunk_coord.x,
                    chunk_coord.y,
                    chunk_coord.z,
                    chunk.coord.x,
                    chunk.coord.y,
                    chunk.coord.z
                ),
            ));
        }

        Ok(Some(chunk))
    }

    /// Saves one chunk to disk.
    ///
    /// The temporary file is written beside the final file first so that a
    /// failed serialization/write does not leave a partially written chunk.
    pub fn save_chunk(&self, chunk: &StoredChunk) -> io::Result<()> {
        fs::create_dir_all(&self.root)?;

        let path = self.chunk_path(chunk.coord);

        let bytes = serde_json::to_vec(chunk).map_err(|error| {
            io::Error::new(
                io::ErrorKind::Other,
                format!(
                    "Failed to encode chunk ({}, {}, {}): {}",
                    chunk.coord.x, chunk.coord.y, chunk.coord.z, error
                ),
            )
        })?;

        let temp_path = path.with_extension("chunk.tmp");

        fs::write(&temp_path, bytes)?;

        if path.exists() {
            fs::remove_file(&path)?;
        }

        fs::rename(&temp_path, &path)?;

        Ok(())
    }

    /// Deletes a chunk from persistent storage.
    ///
    /// This is primarily useful for explicitly deleting empty chunks.
    /// Normal streaming should generally leave empty/nonexistent chunks
    /// absent from disk.
    pub fn delete_chunk(&self, chunk_coord: ChunkCoord) -> io::Result<()> {
        let path = self.chunk_path(chunk_coord);

        if path.exists() {
            fs::remove_file(path)?;
        }

        Ok(())
    }

    /// Builds a serializable chunk from runtime Cells.
    ///
    /// Runtime-only state is intentionally not included.
    pub fn create_stored_chunk(
        chunk_coord: ChunkCoord,
        cells: impl IntoIterator<Item = (WorldCoord, Cell)>,
    ) -> io::Result<StoredChunk> {
        let mut stored_cells = Vec::new();

        for (coord, cell) in cells {
            let expected_chunk = ChunkCoord::from_world_coord(coord);

            if expected_chunk != chunk_coord {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!(
                        "Cell ({}, {}, {}) does not belong to chunk ({}, {}, {})",
                        coord.x, coord.y, coord.z, chunk_coord.x, chunk_coord.y, chunk_coord.z
                    ),
                ));
            }

            stored_cells.push(StoredCell::from_cell(coord, &cell));
        }

        Ok(StoredChunk {
            version: CHUNK_FILE_VERSION,
            coord: chunk_coord,
            cells: stored_cells,
        })
    }
}

/// Persistent representation of one world chunk.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct StoredChunk {
    pub version: u32,
    pub coord: ChunkCoord,
    pub cells: Vec<StoredCell>,
}

impl StoredChunk {
    pub fn new(coord: ChunkCoord) -> Self {
        Self {
            version: CHUNK_FILE_VERSION,
            coord,
            cells: Vec::new(),
        }
    }

    /// Converts every stored cell back into runtime cells and world coordinates.
    pub fn into_cells(self) -> Vec<(WorldCoord, Cell)> {
        self.cells
            .into_iter()
            .map(|cell| cell.into_cell(self.coord))
            .collect()
    }
}

/// Serializable representation of a Cell.
///
/// Coordinates are stored locally inside the chunk. This keeps chunk files
/// self-contained while avoiding duplication of the chunk origin in every cell.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct StoredCell {
    pub local_x: i32,
    pub local_y: i32,
    pub local_z: i32,

    pub id: u64,
    pub cell_type: StoredCellType,

    pub visible: bool,
    pub solid: bool,
    pub anchored: bool,

    pub texture: String,
    pub color_rgb: [f32; 3],

    pub audio: String,
    pub playing: bool,
    pub looped: bool,
    pub volume: f32,

    pub collision_events_enabled: bool,

    pub light_color: [f32; 3],
    pub light_intensity: f32,
    pub light_range: f32,
    pub light_shadows: bool,
    pub light_enabled: bool,

    pub entity_identity: Option<String>,

    pub attributes: BTreeMap<String, AttributeValue>,
}

impl StoredCell {
    pub fn from_cell(coord: WorldCoord, cell: &Cell) -> Self {
        Self {
            local_x: coord.x.rem_euclid(super::CHUNK_SIZE),
            local_y: coord.y.rem_euclid(super::CHUNK_SIZE),
            local_z: coord.z.rem_euclid(super::CHUNK_SIZE),

            id: cell.id,
            cell_type: StoredCellType::from(cell.cell_type),

            visible: cell.visible,
            solid: cell.solid,
            anchored: cell.anchored,

            texture: cell.texture.clone(),
            color_rgb: [cell.color_rgb.x, cell.color_rgb.y, cell.color_rgb.z],

            audio: cell.audio.clone(),
            playing: cell.playing,
            looped: cell.looped,
            volume: cell.volume,

            collision_events_enabled: cell.collision_events_enabled,

            light_color: [cell.light_color.x, cell.light_color.y, cell.light_color.z],
            light_intensity: cell.light_intensity,
            light_range: cell.light_range,
            light_shadows: cell.light_shadows,
            light_enabled: cell.light_enabled,

            entity_identity: cell.entity_identity.clone(),

            attributes: cell.attributes.clone(),
        }
    }

    pub fn into_cell(self, chunk_coord: ChunkCoord) -> (WorldCoord, Cell) {
        let coord = WorldCoord::new(
            chunk_coord.x * super::CHUNK_SIZE + self.local_x,
            chunk_coord.y * super::CHUNK_SIZE + self.local_y,
            chunk_coord.z * super::CHUNK_SIZE + self.local_z,
        );

        let cell = Cell {
            id: self.id,
            cell_type: self.cell_type.into(),

            visible: self.visible,
            solid: self.solid,
            anchored: self.anchored,

            texture: self.texture,
            color_rgb: Vec3::new(self.color_rgb[0], self.color_rgb[1], self.color_rgb[2]),

            audio: self.audio,
            playing: self.playing,
            looped: self.looped,
            volume: self.volume,

            collision_events_enabled: self.collision_events_enabled,

            light_color: Vec3::new(
                self.light_color[0],
                self.light_color[1],
                self.light_color[2],
            ),
            light_intensity: self.light_intensity,
            light_range: self.light_range,
            light_shadows: self.light_shadows,
            light_enabled: self.light_enabled,

            entity_identity: self.entity_identity,

            attributes: self.attributes,
        };

        (coord, cell)
    }
}

/// Stable serialized representation of CellType.
///
/// We intentionally do not serialize Rust enum discriminants so adding/reordering
/// CellType variants later cannot silently corrupt old chunk files.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum StoredCellType {
    Block,
    FxBlock,
    Player,
    Npc,
    Light,
    SpawnPoint,
    AudioEmitter,
}

impl From<CellType> for StoredCellType {
    fn from(value: CellType) -> Self {
        match value {
            CellType::Block => Self::Block,
            CellType::FxBlock => Self::FxBlock,
            CellType::Player => Self::Player,
            CellType::NPC => Self::Npc,
            CellType::Light => Self::Light,
            CellType::SpawnPoint => Self::SpawnPoint,
            CellType::AudioEmitter => Self::AudioEmitter,
            CellType::Empty => {
                panic!("Empty cells must not be written to chunk storage")
            }
        }
    }
}

impl From<StoredCellType> for CellType {
    fn from(value: StoredCellType) -> Self {
        match value {
            StoredCellType::Block => CellType::Block,
            StoredCellType::FxBlock => CellType::FxBlock,
            StoredCellType::Player => CellType::Player,
            StoredCellType::Npc => CellType::NPC,
            StoredCellType::Light => CellType::Light,
            StoredCellType::SpawnPoint => CellType::SpawnPoint,
            StoredCellType::AudioEmitter => CellType::AudioEmitter,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unique_test_directory() -> PathBuf {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();

        std::env::temp_dir().join(format!(
            "aeoengine_world_storage_test_{}_{}",
            std::process::id(),
            timestamp
        ))
    }

    #[test]
    fn stored_cell_round_trip_preserves_data() {
        let coord = WorldCoord::new(17, -3, 34);
        let chunk_coord = ChunkCoord::from_world_coord(coord);

        let mut cell = Cell::new_block();
        cell.id = 12345;
        cell.color_rgb = Vec3::new(0.1, 0.2, 0.3);
        cell.texture = "brick".to_string();
        cell.entity_identity = Some("TestEntity".to_string());

        cell.attributes
            .insert("health".to_string(), AttributeValue::Number(100.0));

        let stored = StoredCell::from_cell(coord, &cell);
        let (loaded_coord, loaded_cell) = stored.into_cell(chunk_coord);

        assert_eq!(coord, loaded_coord);
        assert_eq!(cell.id, loaded_cell.id);
        assert_eq!(cell.cell_type, loaded_cell.cell_type);
        assert_eq!(cell.visible, loaded_cell.visible);
        assert_eq!(cell.solid, loaded_cell.solid);
        assert_eq!(cell.anchored, loaded_cell.anchored);
        assert_eq!(cell.texture, loaded_cell.texture);
        assert_eq!(cell.color_rgb, loaded_cell.color_rgb);
        assert_eq!(cell.entity_identity, loaded_cell.entity_identity);
        assert_eq!(cell.attributes, loaded_cell.attributes);
    }

    #[test]
    fn storage_saves_and_loads_chunk() {
        let root = unique_test_directory();
        let storage = WorldStorage::new(&root).unwrap();

        let coord = WorldCoord::new(1, 2, 3);
        let chunk_coord = ChunkCoord::from_world_coord(coord);

        let mut cell = Cell::new_block();
        cell.id = 42;

        let chunk = WorldStorage::create_stored_chunk(chunk_coord, [(coord, cell)]).unwrap();

        storage.save_chunk(&chunk).unwrap();

        assert!(storage.chunk_exists(chunk_coord));

        let loaded = storage
            .load_chunk(chunk_coord)
            .unwrap()
            .expect("chunk should exist");

        assert_eq!(loaded.version, CHUNK_FILE_VERSION);
        assert_eq!(loaded.coord, chunk_coord);
        assert_eq!(loaded.cells.len(), 1);
        assert_eq!(loaded.cells[0].id, 42);

        storage.delete_chunk(chunk_coord).unwrap();

        assert!(!storage.chunk_exists(chunk_coord));

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn missing_chunk_returns_none() {
        let root = unique_test_directory();
        let storage = WorldStorage::new(&root).unwrap();

        let coord = ChunkCoord::new(100, 200, -50);

        assert!(storage.load_chunk(coord).unwrap().is_none());

        let _ = fs::remove_dir_all(root);
    }
}
