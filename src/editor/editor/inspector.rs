use super::{ColorTarget, Editor};
use crate::editor::tools;
use crate::world::AttributeValue;
use egui::RichText;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AttributeType {
    Number,
    Bool,
    String,
}

#[derive(Clone, Debug, PartialEq)]
pub enum PropertyChange {
    LightColor(glam::Vec3),
    LightIntensity(f32),
    LightRange(f32),
    LightShadows(bool),
    Solid(bool),
    Anchored(bool),
    Visible(bool),
    ColorRgb(glam::Vec3),
    Texture(String),
    Audio(String),
    AudioPlaying(bool),
    AudioLooped(bool),
    AudioVolume(f32),
    CollisionEventsEnabled(bool),
    EntityIdentity(Option<String>),
    AttributeSet(String, AttributeValue),
    AttributeRemove(String),
}

impl Editor {
    pub fn apply_property_changes(
        &self,
        world: &mut crate::world::World,
        changes: &[PropertyChange],
    ) {
        if changes.is_empty() {
            return;
        }
        for &c in &self.selected_coords {
            if let Some(other_cell) = world.get_mut(c) {
                for change in changes {
                    match change {
                        PropertyChange::LightColor(color) => other_cell.light_color = *color,
                        PropertyChange::LightIntensity(intensity) => {
                            other_cell.light_intensity = *intensity
                        }
                        PropertyChange::LightRange(range) => other_cell.light_range = *range,
                        PropertyChange::LightShadows(shadows) => {
                            other_cell.light_shadows = *shadows
                        }
                        PropertyChange::Solid(solid) => other_cell.solid = *solid,
                        PropertyChange::Anchored(anchored) => other_cell.anchored = *anchored,
                        PropertyChange::Visible(visible) => other_cell.visible = *visible,
                        PropertyChange::ColorRgb(color) => other_cell.color_rgb = *color,
                        PropertyChange::Texture(texture) => other_cell.texture = texture.clone(),
                        PropertyChange::Audio(audio) => other_cell.audio = audio.clone(),
                        PropertyChange::AudioPlaying(playing) => other_cell.playing = *playing,
                        PropertyChange::AudioLooped(looped) => other_cell.looped = *looped,
                        PropertyChange::AudioVolume(volume) => other_cell.volume = *volume,
                        PropertyChange::CollisionEventsEnabled(enabled) => {
                            other_cell.collision_events_enabled = *enabled
                        }
                        PropertyChange::EntityIdentity(identity) => {
                            other_cell.entity_identity = identity.clone()
                        }
                        PropertyChange::AttributeSet(name, value) => {
                            other_cell.attributes.insert(name.clone(), value.clone());
                        }
                        PropertyChange::AttributeRemove(name) => {
                            other_cell.attributes.remove(name);
                        }
                    }
                }
            }
        }
    }

    pub(crate) fn render_properties_content(
        &mut self,
        ui: &mut egui::Ui,
        world: &mut crate::world::World,
        project_path: &Option<std::path::PathBuf>,
    ) {
        if self.selected_coord != self.last_selected_coord {
            self.attribute_add_name.clear();
            self.attribute_add_type = AttributeType::String;
            self.attribute_add_value = AttributeValue::String(String::new());
            self.attribute_add_error = None;
            self.last_selected_coord = self.selected_coord;
        }

        if let Some(coord) = self.selected_coord {
            let mut cell_opt = world.get(coord).cloned();
            let mut changes = Vec::new();

            egui::ScrollArea::vertical()
                .id_source("prop_scroll")
                .show(ui, |ui| {
                    // --- IDENTITY ---
                    egui::CollapsingHeader::new("IDENTITY")
                        .default_open(true)
                        .show(ui, |ui| {
                            if let Some(ref mut cell) = cell_opt {
                                ui.horizontal(|ui| {
                                    ui.label("ID:");
                                    ui.label(RichText::new(format!("{:08}", cell.id)).monospace());
                                });
                                ui.horizontal(|ui| {
                                    ui.label("Type:");
                                    ui.label(format!("{:?}", cell.cell_type));
                                });

                                ui.horizontal(|ui| {
                                    ui.label("Identity:");
                                    let mut identity = cell.entity_identity.clone().unwrap_or_default();
                                    if ui.add(egui::TextEdit::singleline(&mut identity).desired_width(120.0)).changed() {
                                        let new_identity = if identity.trim().is_empty() {
                                            None
                                        } else {
                                            Some(identity)
                                        };
                                        cell.entity_identity = new_identity.clone();
                                        changes.push(PropertyChange::EntityIdentity(new_identity));
                                    }
                                });
                            } else {
                                ui.label("Type: Empty");
                            }
                        });

                    // --- TRANSFORM ---
                    egui::CollapsingHeader::new("TRANSFORM")
                        .default_open(true)
                        .show(ui, |ui| {
                            ui.label("Position");
                            ui.indent("pos_indent", |ui| {
                                ui.label(format!("X: {}", coord.x));
                                ui.label(format!("Y: {}", coord.y));
                                ui.label(format!("Z: {}", coord.z));
                            });
                        });

                    // --- SCRIPT ---
                    egui::CollapsingHeader::new("SCRIPT")
                        .default_open(true)
                        .show(ui, |ui| {
                            if let Some(ref cell) = cell_opt {
                                let identity = if let Some(ref id) = cell.entity_identity {
                                    id.clone()
                                } else {
                                    format!("{:?}", cell.cell_type)
                                };
                                let binding_index = world.script_bindings.iter().position(|b| b.target_identity == cell.id);

                                ui.horizontal(|ui| {
                                    ui.label("Current Identity:");
                                    ui.label(RichText::new(&identity).monospace());
                                });

                                ui.horizontal(|ui| {
                                    ui.label("Bound Script:");
                                    if let Some(idx) = binding_index {
                                        let script_path = &world.script_bindings[idx].script_path;
                                        let label = RichText::new(script_path).strong();

                                        let mut exists = true;
                                        if let Some(root) = project_path {
                                            if !root.join(script_path).exists() {
                                                exists = false;
                                            }
                                        }

                                        if !exists {
                                            ui.label(label.color(egui::Color32::from_rgb(255, 100, 100)));
                                            ui.label(RichText::new("[missing/stale]").color(egui::Color32::from_rgb(255, 100, 100)).small());
                                        } else {
                                            ui.label(label);
                                        }

                                        ui.add_space(8.0);
                                        let mut enabled = world.script_bindings[idx].enabled;
                                        if ui.checkbox(&mut enabled, "Enabled").changed() {
                                            world.script_bindings[idx].enabled = enabled;
                                            self.needs_save = true;
                                        }
                                    } else {
                                        ui.label("None");
                                    }
                                });

                                ui.add_space(4.0);

                                ui.horizontal(|ui| {
                                    egui::ComboBox::from_id_salt("attach_script_combo")
                                        .selected_text("Attach Script")
                                        .show_ui(ui, |ui| {
                                            let scripts = &self.script_editor.scripts_list;
                                            if scripts.is_empty() {
                                                ui.label("No .aeo scripts found.");
                                            }
                                            for script_path in scripts {
                                                let filename = script_path.file_name().unwrap_or_default().to_string_lossy();
                                                if ui.selectable_label(false, filename.clone()).clicked() {
                                                    let relative_path = if let Some(root) = project_path {
                                                        script_path.strip_prefix(root).unwrap_or(script_path).to_string_lossy().to_string()
                                                    } else {
                                                        script_path.to_string_lossy().to_string()
                                                    };

                                                    let new_binding = crate::scripting::binding::ScriptBinding::new(cell.id, relative_path);

                                                    let binding_index = world.script_bindings.iter().position(|b| b.target_identity == cell.id);
                                                    if let Some(idx) = binding_index {
                                                        world.script_bindings[idx] = new_binding;
                                                    } else {
                                                        world.script_bindings.push(new_binding);
                                                    }
                                                    self.needs_save = true;
                                                }
                                            }
                                        });

                                    let binding_index = world.script_bindings.iter().position(|b| b.target_identity == cell.id);
                                    if binding_index.is_some() {
                                        if ui.button("Remove Script").clicked() {
                                            if let Some(idx) = world.script_bindings.iter().position(|b| b.target_identity == cell.id) {
                                                world.script_bindings.remove(idx);
                                                self.needs_save = true;
                                            }
                                        }
                                    }
                                });
                            }

                            ui.add_space(8.0);
                            ui.label(RichText::new("All Script Bindings:").small().heading());

                            let mut remove_idx = None;
                            for (idx, b) in world.script_bindings.iter().enumerate() {
                                let cell_at_coord = world.resolve_cell_id(b.target_identity)
                                    .and_then(|coord| world.get(coord));

                                let identity_display = if let Some(cell) = cell_at_coord {
                                    let name = if let Some(ref id) = cell.entity_identity {
                                        id.clone()
                                    } else {
                                        format!("{:?}", cell.cell_type)
                                    };
                                    format!("{} [ID {}]", name, b.target_identity)
                                } else {
                                    format!("ID {}", b.target_identity)
                                };

                                ui.horizontal(|ui| {
                                    if cell_at_coord.is_none() {
                                        ui.label(RichText::new(format!("⚠️ [STALE] {} -> {}", identity_display, b.script_path))
                                            .color(egui::Color32::from_rgb(255, 140, 0))
                                            .small());
                                    } else {
                                        ui.label(RichText::new(format!("{} -> {}", identity_display, b.script_path))
                                            .color(egui::Color32::GRAY)
                                            .small());
                                    }

                                    if ui.small_button("").on_hover_text("Remove this authored binding").clicked() {
                                        remove_idx = Some(idx);
                                    }
                                });
                            }

                            if let Some(idx) = remove_idx {
                                world.script_bindings.remove(idx);
                                self.needs_save = true;
                            }
                        });

                    if let Some(ref mut cell) = cell_opt {
                        // --- LIGHT ---
                        if cell.cell_type == crate::world::CellType::Light {
                            egui::CollapsingHeader::new("LIGHT")
                                .default_open(true)
                                .show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        ui.label("Type:");
                                        ui.label("Point");
                                    });

                                    ui.horizontal(|ui| {
                                        ui.label("Color:");
                                        let mut color = cell.light_color;
                                        tools::draw_color_edit(
                                            ui,
                                            self,
                                            ColorTarget::PropertyLight(coord),
                                            &mut color,
                                        );
                                        if color != cell.light_color {
                                            cell.light_color = color;
                                            changes.push(PropertyChange::LightColor(color));
                                        }
                                    });

                                    ui.horizontal(|ui| {
                                        ui.label("Intensity:");
                                        let mut intensity = cell.light_intensity;
                                        if ui.add(
                                            egui::DragValue::new(&mut intensity)
                                                .speed(0.1)
                                                .range(0.0..=f32::MAX),
                                        ).changed() {
                                            cell.light_intensity = intensity;
                                            changes.push(PropertyChange::LightIntensity(intensity));
                                        }
                                    });

                                    ui.horizontal(|ui| {
                                        ui.label("Range:");
                                        let mut range = cell.light_range;
                                        if ui.add(
                                            egui::DragValue::new(&mut range)
                                                .speed(0.1)
                                                .range(0.0..=f32::MAX),
                                        ).changed() {
                                            cell.light_range = range;
                                            changes.push(PropertyChange::LightRange(range));
                                        }
                                    });

                                    let mut shadows = cell.light_shadows;
                                    if ui.checkbox(&mut shadows, "Shadows").changed() {
                                        cell.light_shadows = shadows;
                                        changes.push(PropertyChange::LightShadows(shadows));
                                    }
                                });
                        }

                        // --- AUDIO ---
                        if cell.cell_type == crate::world::CellType::AudioEmitter {
                            egui::CollapsingHeader::new("AUDIO")
                                .default_open(true)
                                .show(ui, |ui| {
                                    let mut audio = cell.audio.clone();
                                    tools::draw_audio_edit(ui, &mut audio, project_path);
                                    if audio != cell.audio {
                                        cell.audio = audio.clone();
                                        changes.push(PropertyChange::Audio(audio));
                                    }

                                    let mut playing = cell.playing;
                                    if ui.checkbox(&mut playing, "Playing").changed() {
                                        cell.playing = playing;
                                        changes.push(PropertyChange::AudioPlaying(playing));
                                    }

                                    let mut looped = cell.looped;
                                    if ui.checkbox(&mut looped, "Looped").changed() {
                                        cell.looped = looped;
                                        changes.push(PropertyChange::AudioLooped(looped));
                                    }

                                    let mut volume = cell.volume;
                                    ui.horizontal(|ui| {
                                        ui.label("Volume:");
                                        if ui
                                            .add(
                                                egui::DragValue::new(&mut volume)
                                                    .speed(0.05)
                                                    .range(0.0..=2.0),
                                            )
                                            .changed()
                                        {
                                            cell.volume = volume;
                                            changes.push(PropertyChange::AudioVolume(volume));
                                        }
                                    });
                                });
                        }

                        // --- PHYSICS ---
                        egui::CollapsingHeader::new("PHYSICS")
                            .default_open(true)
                            .show(ui, |ui| {
                                let mut solid = cell.solid;
                                if ui.checkbox(&mut solid, "Solid").changed() {
                                    cell.solid = solid;
                                    changes.push(PropertyChange::Solid(solid));
                                }
                                let mut anchored = cell.anchored;
                                if ui.checkbox(&mut anchored, "Anchored").changed() {
                                    cell.anchored = anchored;
                                    changes.push(PropertyChange::Anchored(anchored));
                                }
                                let mut collision_events = cell.collision_events_enabled;
                                if ui.checkbox(&mut collision_events, "Collision Events").changed() {
                                    cell.collision_events_enabled = collision_events;
                                    changes.push(PropertyChange::CollisionEventsEnabled(collision_events));
                                }
                            });

                        // --- RENDERING ---
                        egui::CollapsingHeader::new("RENDERING")
                            .default_open(true)
                            .show(ui, |ui| {
                                let mut visible = cell.visible;
                                if ui.checkbox(&mut visible, "Visible").changed() {
                                    cell.visible = visible;
                                    changes.push(PropertyChange::Visible(visible));
                                }

                                ui.horizontal(|ui| {
                                    ui.label("Color:");
                                    let mut color = cell.color_rgb;
                                    tools::draw_color_edit(
                                        ui,
                                        self,
                                        ColorTarget::PropertyColor(coord),
                                        &mut color,
                                    );
                                    if color != cell.color_rgb {
                                        cell.color_rgb = color;
                                        changes.push(PropertyChange::ColorRgb(color));
                                    }
                                });

                                let mut texture = cell.texture.clone();
                                tools::draw_texture_edit(ui, &mut texture);
                                if texture != cell.texture {
                                    cell.texture = texture.clone();
                                    changes.push(PropertyChange::Texture(texture));
                                }
                            });

                        // --- ATTRIBUTES ---
                        egui::CollapsingHeader::new("ATTRIBUTES")
                            .default_open(true)
                            .show(ui, |ui| {
                                // Add Attribute workflow
                                ui.horizontal(|ui| {
                                    if ui.button("+ Add Attribute").clicked() {
                                        let name = self.attribute_add_name.trim().to_string();
                                        if name.is_empty() {
                                            self.attribute_add_error = Some("Name cannot be empty.".to_string());
                                        } else if cell.attributes.contains_key(&name) {
                                            self.attribute_add_error = Some(format!("'{}' already exists.", name));
                                        } else {
                                            changes.push(PropertyChange::AttributeSet(name, self.attribute_add_value.clone()));
                                            self.attribute_add_name.clear();
                                            self.attribute_add_type = AttributeType::String;
                                            self.attribute_add_value = AttributeValue::String(String::new());
                                            self.attribute_add_error = None;
                                        }
                                    }

                                    ui.add(egui::TextEdit::singleline(&mut self.attribute_add_name)
                                        .desired_width(100.0)
                                        .hint_text("Name"));

                                    let prev_type = self.attribute_add_type;
                                    egui::ComboBox::from_id_salt("attr_type_combo")
                                        .selected_text(format!("{:?}", self.attribute_add_type))
                                        .width(80.0)
                                        .show_ui(ui, |ui| {
                                            ui.selectable_value(&mut self.attribute_add_type, AttributeType::Number, "Number");
                                            ui.selectable_value(&mut self.attribute_add_type, AttributeType::Bool, "Bool");
                                            ui.selectable_value(&mut self.attribute_add_type, AttributeType::String, "String");
                                        });

                                    if self.attribute_add_type != prev_type {
                                        self.attribute_add_value = match self.attribute_add_type {
                                            AttributeType::Number => AttributeValue::Number(0.0),
                                            AttributeType::Bool => AttributeValue::Bool(false),
                                            AttributeType::String => AttributeValue::String(String::new()),
                                        };
                                    }
                                });

                                // Initial Value editor for the add form
                                ui.horizontal(|ui| {
                                    ui.label("Initial Value:");
                                    match &mut self.attribute_add_value {
                                        AttributeValue::Number(n) => {
                                            ui.add(egui::DragValue::new(n).speed(0.1));
                                        }
                                        AttributeValue::Bool(b) => {
                                            ui.checkbox(b, "");
                                        }
                                        AttributeValue::String(s) => {
                                            ui.add(egui::TextEdit::singleline(s).desired_width(100.0));
                                        }
                                    }
                                });

                                if let Some(ref err) = self.attribute_add_error {
                                    ui.colored_label(egui::Color32::LIGHT_RED, err);
                                }

                                ui.separator();

                                // List existing attributes
                                let mut attr_to_remove = None;
                                for (name, value) in &cell.attributes {
                                    ui.horizontal(|ui| {
                                        ui.label(RichText::new(name).strong());
                                        ui.add_space(4.0);
                                        ui.label("|");
                                        ui.add_space(4.0);

                                        let mut new_value = value.clone();
                                        let changed = match &mut new_value {
                                            AttributeValue::Number(n) => {
                                                ui.add(egui::DragValue::new(n).speed(0.1)).changed()
                                            }
                                            AttributeValue::Bool(b) => {
                                                ui.checkbox(b, "").changed()
                                            }
                                            AttributeValue::String(s) => {
                                                ui.text_edit_singleline(s).changed()
                                            }
                                        };

                                        if changed {
                                            changes.push(PropertyChange::AttributeSet(name.clone(), new_value));
                                        }

                                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                            if ui.button("Remove").clicked() {
                                                attr_to_remove = Some(name.clone());
                                            }
                                        });
                                    });
                                }

                                if let Some(name) = attr_to_remove {
                                    changes.push(PropertyChange::AttributeRemove(name));
                                }
                            });
                    }
                });

            // Apply all changes to all selected coordinates
            self.apply_property_changes(world, &changes);
        } else {
            ui.centered_and_justified(|ui| {
                ui.label("No cell selected.");
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::{CellType, World, WorldCoord};
    use glam::Vec3;

    #[test]
    fn test_inspector_script_binding_lookup() {
        let mut world = crate::world::World::new();

        let coord = crate::world::WorldCoord::new(1, 1, 1);
        let cell_id = world.set_cell(coord, crate::world::CellType::Player);

        // Initially no binding
        let binding = world
            .script_bindings
            .iter()
            .find(|b| b.target_identity == cell_id);
        assert!(binding.is_none());

        // Add a binding
        world
            .script_bindings
            .push(crate::scripting::binding::ScriptBinding::new(
                cell_id,
                "scripts/test.aeo",
            ));

        // Lookup again
        let binding = world
            .script_bindings
            .iter()
            .find(|b| b.target_identity == cell_id);
        assert!(binding.is_some());
        assert_eq!(binding.unwrap().script_path, "scripts/test.aeo");
    }

    #[test]
    fn test_script_binding_removal() {
        let mut world = crate::world::World::new();
        let cell_id = 12345678; // Dummy ID for test

        // 1. Setup binding
        world
            .script_bindings
            .push(crate::scripting::binding::ScriptBinding::new(
                cell_id,
                "scripts/nonexistent.aeo",
            ));
        assert!(
            world
                .script_bindings
                .iter()
                .any(|b| b.target_identity == cell_id)
        );

        // 2. Perform removal (emulating the UI button logic)
        if let Some(idx) = world
            .script_bindings
            .iter()
            .position(|b| b.target_identity == cell_id)
        {
            world.script_bindings.remove(idx);
        }

        // 3. Verify it's gone
        assert!(
            !world
                .script_bindings
                .iter()
                .any(|b| b.target_identity == cell_id)
        );
    }

    #[test]
    fn test_multi_select_property_application_regression() {
        // 1. Setup a World and place cells
        let mut world = World::new();

        let coord_a = WorldCoord::new(1, 0, 0);
        let coord_b = WorldCoord::new(2, 0, 0);
        let coord_c = WorldCoord::new(3, 0, 0);
        let coord_d = WorldCoord::new(4, 0, 0);
        let coord_x = WorldCoord::new(9, 9, 9); // control cell

        world.set_cell(coord_a, CellType::Block);
        world.set_cell(coord_b, CellType::Block);
        world.set_cell(coord_c, CellType::Block);
        world.set_cell(coord_d, CellType::Block);
        world.set_cell(coord_x, CellType::Block);

        // Initialize distinct values across all selected cells
        if let Some(cell) = world.get_mut(coord_a) {
            cell.texture = "tex_a".to_string();
            cell.color_rgb = Vec3::new(0.1, 0.1, 0.1);
            cell.solid = true;
            cell.anchored = false;
        }
        if let Some(cell) = world.get_mut(coord_b) {
            cell.texture = "tex_b".to_string();
            cell.color_rgb = Vec3::new(0.2, 0.2, 0.2);
            cell.solid = false;
            cell.anchored = true;
        }
        if let Some(cell) = world.get_mut(coord_c) {
            cell.texture = "tex_c".to_string();
            cell.color_rgb = Vec3::new(0.3, 0.3, 0.3);
            cell.solid = true;
            cell.anchored = true;
        }
        if let Some(cell) = world.get_mut(coord_d) {
            cell.texture = "tex_d".to_string();
            cell.color_rgb = Vec3::new(0.4, 0.4, 0.4);
            cell.solid = false;
            cell.anchored = false;
        }
        if let Some(cell) = world.get_mut(coord_x) {
            cell.texture = "tex_x".to_string();
            cell.color_rgb = Vec3::new(0.9, 0.9, 0.9);
            cell.solid = true;
            cell.anchored = true;
        }

        // 2. Create Editor and select multiple cells in a non-trivial order
        let mut editor = Editor::new();
        editor.selected_coords = vec![coord_d, coord_b, coord_a, coord_c];
        editor.selected_coord = Some(coord_c); // Primary selection is C (the last one)

        // Verify initial preconditions to ensure everything is distinct
        assert_ne!(world.get(coord_a).unwrap().texture, "target_tex");
        assert_ne!(world.get(coord_b).unwrap().texture, "target_tex");
        assert_ne!(world.get(coord_c).unwrap().texture, "target_tex");
        assert_ne!(world.get(coord_d).unwrap().texture, "target_tex");
        assert_ne!(world.get(coord_x).unwrap().texture, "target_tex");

        // 3. Apply target changes through the production mutation logic path
        let changes = vec![
            PropertyChange::Texture("target_tex".to_string()),
            PropertyChange::ColorRgb(Vec3::new(0.7, 0.7, 0.7)),
            PropertyChange::Solid(true),
            PropertyChange::Anchored(true),
        ];

        editor.apply_property_changes(&mut world, &changes);

        // 4. Explicitly verify ALL selected cells changed to target values individually
        let cell_a = world.get(coord_a).unwrap();
        assert_eq!(cell_a.texture, "target_tex");
        assert_eq!(cell_a.color_rgb, Vec3::new(0.7, 0.7, 0.7));
        assert_eq!(cell_a.solid, true);
        assert_eq!(cell_a.anchored, true);

        let cell_b = world.get(coord_b).unwrap();
        assert_eq!(cell_b.texture, "target_tex");
        assert_eq!(cell_b.color_rgb, Vec3::new(0.7, 0.7, 0.7));
        assert_eq!(cell_b.solid, true);
        assert_eq!(cell_b.anchored, true);

        let cell_c = world.get(coord_c).unwrap();
        assert_eq!(cell_c.texture, "target_tex");
        assert_eq!(cell_c.color_rgb, Vec3::new(0.7, 0.7, 0.7));
        assert_eq!(cell_c.solid, true);
        assert_eq!(cell_c.anchored, true);

        let cell_d = world.get(coord_d).unwrap();
        assert_eq!(cell_d.texture, "target_tex");
        assert_eq!(cell_d.color_rgb, Vec3::new(0.7, 0.7, 0.7));
        assert_eq!(cell_d.solid, true);
        assert_eq!(cell_d.anchored, true);

        // 5. Verify the control cell was NOT modified
        let cell_x = world.get(coord_x).unwrap();
        assert_eq!(cell_x.texture, "tex_x");
        assert_eq!(cell_x.color_rgb, Vec3::new(0.9, 0.9, 0.9));
        assert_eq!(cell_x.solid, true);
        assert_eq!(cell_x.anchored, true);

        // 6. Single Selection Check: verify single selection case works perfectly
        editor.selected_coords = vec![coord_a];
        editor.selected_coord = Some(coord_a);

        let single_changes = vec![PropertyChange::Texture("single_tex".to_string())];
        editor.apply_property_changes(&mut world, &single_changes);

        assert_eq!(world.get(coord_a).unwrap().texture, "single_tex");
        assert_eq!(world.get(coord_b).unwrap().texture, "target_tex"); // remains target_tex
        assert_eq!(world.get(coord_c).unwrap().texture, "target_tex");
        assert_eq!(world.get(coord_d).unwrap().texture, "target_tex");
    }

    #[test]
    fn test_additive_multi_select() {
        let mut world = World::new();
        let c1 = WorldCoord::new(1, 0, 0);
        let c2 = WorldCoord::new(2, 0, 0);
        let c3 = WorldCoord::new(3, 0, 0);

        world.set_cell(c1, CellType::Block);
        world.set_cell(c2, CellType::Block);
        world.set_cell(c3, CellType::Block);

        let mut editor = Editor::new();
        editor.selected_coords = vec![c1];
        editor.selected_coord = Some(c1);

        // Simulate additive selection (Ctrl + click c2)
        if !editor.selected_coords.contains(&c2) {
            editor.selected_coords.push(c2);
            editor.selected_coord = Some(c2);
        }
        assert_eq!(editor.selected_coords, vec![c1, c2]);

        // Simulate additive selection (Ctrl + click c3)
        if !editor.selected_coords.contains(&c3) {
            editor.selected_coords.push(c3);
            editor.selected_coord = Some(c3);
        }
        assert_eq!(editor.selected_coords, vec![c1, c2, c3]);

        // Simulate toggle deselect (Ctrl + click c2)
        editor.selected_coords.retain(|c| *c != c2);
        assert_eq!(editor.selected_coords, vec![c1, c3]);
    }
}
