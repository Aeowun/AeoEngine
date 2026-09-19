use glam::Vec3;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Hash)]
pub enum CellType {
    #[default]
    Empty,
    Block,
    FxBlock,
    Player,
    NPC,
    Light,
    SpawnPoint,
}

impl CellType {
    pub fn is_entity(self) -> bool {
        matches!(self, Self::Player | Self::NPC)
    }
}

#[derive(Clone, Debug)]
pub struct Cell {
    pub id: u64,
    pub cell_type: CellType,
    pub visible: bool,
    pub solid: bool,
    pub anchored: bool,
    pub texture: String,
    pub color_rgb: Vec3,

    // Authored light properties. Only used when cell_type is Light.
    pub light_color: Vec3,
    pub light_intensity: f32,
    pub light_range: f32,
    pub light_shadows: bool,
    pub light_enabled: bool,

    /// Persistent identity for cells that represent scripted entities (e.g. Player, NPC).
    pub entity_identity: Option<String>,
}

/// Temporary runtime-only modifications to a cell's state.
///
/// This allows runtime systems like AeoScript to override authored properties
/// without corrupting the project data. When the game stops, this state is
/// simply discarded.
#[derive(Clone, Debug, Default)]
pub struct RuntimeCellState {
    pub light_enabled: Option<bool>,
    pub visible: Option<bool>,
    pub color_rgb: Option<Vec3>,
    pub solid: Option<bool>,
    pub anchored: Option<bool>,
    pub visual_offset: Option<Vec3>,
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            id: 0,
            cell_type: CellType::Empty,
            visible: true,
            solid: true,
            anchored: true,
            texture: "None".to_string(),
            color_rgb: Vec3::new(0.5, 0.5, 0.5),
            light_color: Vec3::ONE,
            light_intensity: 5.0,
            light_range: 10.0,
            light_shadows: true,
            light_enabled: true,
            entity_identity: None,
        }
    }
}

impl Cell {
    pub fn new_block() -> Self {
        Self {
            id: 0,
            cell_type: CellType::Block,
            visible: true,
            solid: true,
            anchored: true,
            texture: "Block_tx".to_string(),
            ..Default::default()
        }
    }

    /// Creates a new point light cell with standard defaults.
    pub fn new_light() -> Self {
        Self {
            id: 0,
            cell_type: CellType::Light,
            // Lights are authored markers and don't render as blocks themselves.
            visible: false,
            solid: false,
            anchored: true,
            texture: "Light_tx".to_string(),
            color_rgb: Vec3::new(1.0, 1.0, 0.0), // Visual indicator color
            light_color: Vec3::ONE,              // White emitted light
            light_intensity: 5.0,
            light_range: 10.0,
            light_shadows: true,
            light_enabled: true,
            entity_identity: None,
        }
    }

    pub fn new_spawn_point() -> Self {
        Self {
            id: 0,
            cell_type: CellType::SpawnPoint,
            visible: true,
            solid: true,
            anchored: true,
            texture: "Block_tx".to_string(),
            color_rgb: Vec3::new(0.5, 0.5, 0.5),
            ..Default::default()
        }
    }
}
