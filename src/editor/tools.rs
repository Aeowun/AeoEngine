use super::editor::{ColorTarget, Editor, EditorTool};
use crate::engine::EditorMode;
use egui::{Color32, RichText, Ui};
use glam::Vec3;
use std::fs;

/// Unified color control with numeric entry and a toggle for a persistent color wheel.
/// Format: [ R, G, B | --- ]
pub fn draw_color_edit(ui: &mut Ui, editor: &mut Editor, target: ColorTarget, color: &mut Vec3) {
    ui.horizontal(|ui| {
        // --- Left side: Editable numeric values ---
        let mut text = format!("{:.3}, {:.3}, {:.3}", color.x, color.y, color.z);
        let text_edit = egui::TextEdit::singleline(&mut text)
            .desired_width(140.0)
            .hint_text("R, G, B");

        let response = ui.add(text_edit);

        if response.changed() {
            let parts: Vec<f32> = text
                .split(|c: char| c == ',' || c.is_whitespace())
                .filter_map(|s| s.parse::<f32>().ok())
                .collect();

            if parts.len() == 3 {
                *color = Vec3::new(parts[0], parts[1], parts[2]);
            }
        }

        // Clipboard Copy
        response.context_menu(|ui| {
            if ui.button("Copy RGB Value").clicked() {
                ui.output_mut(|o| {
                    o.copied_text = format!("{:.3}, {:.3}, {:.3}", color.x, color.y, color.z)
                });
                ui.close_menu();
            }
        });

        ui.label("|");

        // --- Right side: Color Wheel toggle button ---
        // Clicking this toggles the persistent picker in the Editor.
        let is_open = editor.active_color_target == Some(target);

        let button = if is_open {
            egui::Button::new(RichText::new(" --- ").color(Color32::WHITE))
                .fill(Color32::from_rgb(0, 100, 200))
        } else {
            egui::Button::new(" --- ")
        };

        if ui.add(button).clicked() {
            if is_open {
                editor.active_color_target = None;
            } else {
                editor.active_color_target = Some(target);
            }
        }
    });
}

pub fn draw_texture_edit(ui: &mut Ui, texture: &mut String) {
    ui.horizontal(|ui| {
        ui.label("Texture:");

        let mut available_textures = Vec::new();
        if let Ok(entries) = fs::read_dir(".assets/textures") {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("png") {
                    if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                        available_textures.push(stem.to_string());
                    }
                }
            }
        }
        available_textures.sort();
        available_textures.dedup();

        egui::ComboBox::new(ui.next_auto_id(), "")
            .selected_text(texture.as_str())
            .width(120.0)
            .show_ui(ui, |ui| {
                if !available_textures.contains(texture) && !texture.is_empty() {
                    ui.selectable_value(
                        texture,
                        texture.clone(),
                        RichText::new(texture.as_str()).italics(),
                    );
                }
                for t in &available_textures {
                    ui.selectable_value(texture, t.clone(), t.as_str());
                }
            });

        if ui.button("Import...").clicked() {
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("PNG Image", &["png"])
                .pick_file()
            {
                let textures_dir = std::path::Path::new(".assets/textures");
                if !textures_dir.exists() {
                    let _ = std::fs::create_dir_all(textures_dir);
                }

                let file_name = path.file_name().unwrap().to_string_lossy().to_string();
                let stem = path.file_stem().unwrap().to_string_lossy().to_string();
                let mut dest_path = textures_dir.join(&file_name);

                if dest_path.exists() {
                    let mut i = 1;
                    loop {
                        let new_name = format!("{}_{}.png", stem, i);
                        let p = textures_dir.join(&new_name);

                        if !p.exists() {
                            dest_path = p;
                            break;
                        }

                        i += 1;
                    }
                }

                if let Err(e) = std::fs::copy(&path, &dest_path) {
                    eprintln!("Failed to copy texture: {}", e);
                } else {
                    let final_stem = dest_path.file_stem().unwrap().to_string_lossy().to_string();
                    *texture = final_stem;
                }
            }
        }
    });
}

pub fn draw_tool_bar(ui: &mut Ui, editor: &mut Editor) {
    ui.horizontal(|ui| {
        let scripts_btn = if editor.show_script_workspace {
            egui::Button::new(RichText::new("Scripts").color(Color32::WHITE))
                .fill(Color32::from_rgb(0, 100, 200))
        } else {
            egui::Button::new("Scripts")
        };

        if ui.add(scripts_btn).clicked() {
            editor.show_script_workspace = !editor.show_script_workspace;
        }

        ui.separator();

        if editor.show_script_workspace {
            ui.label(RichText::new("Script Workspace Active").italics());
            return;
        }

        ui.label(RichText::new("Mode").strong());
        if ui
            .selectable_label(editor.mode == EditorMode::Editor, "Editor")
            .clicked()
        {
            editor.mode = EditorMode::Editor;
        }
        if ui
            .selectable_label(editor.mode == EditorMode::Play, "Play")
            .clicked()
        {
            editor.mode = EditorMode::Play;
        }

        ui.separator();

        ui.label(RichText::new("Tool").strong());

        draw_tool_button(
            ui,
            &mut editor.current_tool,
            EditorTool::Navigate,
            "Navigate",
        );
        draw_tool_button(ui, &mut editor.current_tool, EditorTool::Select, "Select");
        draw_tool_button(ui, &mut editor.current_tool, EditorTool::Build, "Build");
        draw_tool_button(ui, &mut editor.current_tool, EditorTool::Erase, "Erase");

        ui.separator();

        ui.horizontal(|ui| {
            ui.label(RichText::new("Picking:").strong());
            let pick_text = if editor.plane_picking {
                "Plane [ ON ]"
            } else {
                "Plane [ OFF ]"
            };
            if ui.button(pick_text).clicked() {
                editor.plane_picking = !editor.plane_picking;
            }
        });

        if editor.current_tool == EditorTool::Build {
            ui.separator();

            // Block Type Dropdown
            ui.menu_button(
                RichText::new(format!("Block: {:?} ▼", editor.build_template.cell_type)).strong(),
                |ui| {
                    if ui
                        .selectable_label(
                            editor.build_template.cell_type == crate::world::CellType::Block,
                            "Cube",
                        )
                        .clicked()
                    {
                        editor.build_template = crate::world::Cell::new_block();
                        ui.close_menu();
                    }

                    if ui
                        .selectable_label(
                            editor.build_template.cell_type == crate::world::CellType::SpawnPoint,
                            "Spawn point",
                        )
                        .clicked()
                    {
                        editor.build_template = crate::world::Cell::new_spawn_point();
                        ui.close_menu();
                    }

                    if ui
                        .selectable_label(
                            editor.build_template.cell_type == crate::world::CellType::Light,
                            "Light",
                        )
                        .clicked()
                    {
                        editor.build_template = crate::world::Cell::new_light();
                        ui.close_menu();
                    }
                },
            );

            ui.separator();

            // Rendering Settings Dropdown
            ui.menu_button("Rendering ▼", |ui| {
                ui.horizontal(|ui| {
                    ui.label("Color:");
                    let mut color = editor.build_template.color_rgb;
                    draw_color_edit(ui, editor, ColorTarget::Build, &mut color);
                    editor.build_template.color_rgb = color;
                });

                ui.checkbox(&mut editor.build_template.visible, "Visible");

                draw_texture_edit(ui, &mut editor.build_template.texture);
            });

            ui.separator();

            // Physics Settings Dropdown
            ui.menu_button("Physics ▼", |ui| {
                ui.checkbox(&mut editor.build_template.solid, "Solid");
                ui.checkbox(&mut editor.build_template.anchored, "Anchored");
            });
        }
    });
}

fn draw_tool_button(ui: &mut Ui, current: &mut EditorTool, tool: EditorTool, text: &str) {
    let is_active = *current == tool;

    let button = if is_active {
        egui::Button::new(RichText::new(text).color(Color32::WHITE))
            .fill(Color32::from_rgb(0, 100, 200)) // Highlighted blue
    } else {
        egui::Button::new(text)
    };

    if ui.add(button).clicked() {
        *current = tool;
    }
}
