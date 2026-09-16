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
            skin_color: [0.9, 0.7, 0.6, 1.0],      // Light peach
            armor_color: [0.4, 0.4, 0.45, 1.0],    // Steel blue-gray
            cloth_color: [0.7, 0.2, 0.2, 1.0],     // Crimson red
            detail_color: [0.9, 0.8, 0.2, 1.0],    // Gold details
            accessory_color: [0.2, 0.6, 0.3, 1.0], // Emerald cape/shield
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
