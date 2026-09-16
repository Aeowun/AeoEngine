#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum CellType {
    #[default]
    Empty,
    Grass,
    Block,
    FxBlock,
    Player,
    NPC,
}

#[derive(Clone, Debug)]
pub struct Cell {
    pub cell_type: CellType,
    pub visible: bool,
    pub solid: bool,
    pub texture: String,
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            cell_type: CellType::Empty,
            visible: true,
            solid: true,
            texture: "None".to_string(),
        }
    }
}

impl Cell {
    pub fn new_grass() -> Self {
        Self {
            cell_type: CellType::Grass,
            visible: true,
            solid: true,
            texture: "Grass_tx".to_string(),
        }
    }
}
