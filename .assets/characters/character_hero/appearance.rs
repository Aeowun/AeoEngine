#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MaterialSlot {
    Skin,
    Armor,
    Cloth,
    Detail,
    Accessory,
}

#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
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

#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct CharacterPackageConfig {
    pub name: String,
    pub mesh_type: String,
    pub has_gun: bool,
    pub appearance: AppearanceCustomization,
}

impl CharacterPackageConfig {
    pub fn load_from_dir(dir: &std::path::Path) -> Self {
        let pkg_path = dir.join("package.json");
        if let Ok(content) = std::fs::read_to_string(&pkg_path) {
            if let Ok(cfg) = serde_json::from_str::<CharacterPackageConfig>(&content) {
                return cfg;
            }
        }

        let is_soldier = dir
            .file_name()
            .and_then(|n| n.to_str())
            .map_or(false, |n| n.contains("soldier"));
        if is_soldier {
            Self {
                name: "custom_soldier".to_string(),
                mesh_type: "soldier".to_string(),
                has_gun: true,
                appearance: AppearanceCustomization {
                    skin_color: [0.90, 0.70, 0.10, 1.0],
                    armor_color: [0.22, 0.35, 0.20, 1.0],
                    cloth_color: [0.08, 0.08, 0.09, 1.0],
                    detail_color: [0.15, 0.18, 0.16, 1.0],
                    accessory_color: [0.80, 0.30, 0.00, 1.0],
                    show_accessory: true,
                },
            }
        } else {
            Self {
                name: "custom".to_string(),
                mesh_type: "robot".to_string(),
                has_gun: false,
                appearance: AppearanceCustomization::default(),
            }
        }
    }
}
