#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MaterialSlot {
    Skin,
    Armor,
    Cloth,
    Detail,
    Accessory,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AppearanceCustomization {
    pub skin_color: [f32; 4],
    pub armor_color: [f32; 4],
    pub cloth_color: [f32; 4],
    pub detail_color: [f32; 4],
    pub accessory_color: [f32; 4],
    pub show_accessory: bool,
}

impl Default for AppearanceCustomization {
    fn default() -> Self {
        Self {
            skin_color: [0.70, 0.71, 0.70, 1.0],   // Light industrial metal
            armor_color: [0.46, 0.47, 0.45, 1.0],  // Neutral gray metal
            cloth_color: [0.10, 0.10, 0.10, 1.0],  // Black
            detail_color: [0.12, 0.13, 0.15, 1.0], // Brass/gold
            accessory_color: [0.08, 0.30, 0.34, 1.0], // Deep teal
            show_accessory: true,
        }
    }
}

impl AppearanceCustomization {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_color(&self, slot: MaterialSlot) -> [f32; 4] {
        match slot {
            MaterialSlot::Skin => self.skin_color,
            MaterialSlot::Armor => self.armor_color,
            MaterialSlot::Cloth => self.cloth_color,
            MaterialSlot::Detail => self.detail_color,
            MaterialSlot::Accessory => self.accessory_color,
        }
    }
}
