use egui::{Ui, RichText, Color32};
use crate::engine::EditorMode;
use super::editor::{Editor, EditorTool, ColorTarget};
use glam::Vec3;

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
                ui.output_mut(|o| o.copied_text = format!("{:.3}, {:.3}, {:.3}", color.x, color.y, color.z));
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

pub fn draw_tool_bar(ui: &mut Ui, editor: &mut Editor) {
    ui.horizontal(|ui| {
        ui.label(RichText::new("Mode").strong());
        if ui.selectable_label(editor.mode == EditorMode::Editor, "Editor").clicked() {
            editor.mode = EditorMode::Editor;
        }
        if ui.selectable_label(editor.mode == EditorMode::Play, "Play").clicked() {
            editor.mode = EditorMode::Play;
        }

        ui.separator();

        ui.label(RichText::new("Tool").strong());

        draw_tool_button(ui, &mut editor.current_tool, EditorTool::Navigate, "Navigate");
        draw_tool_button(ui, &mut editor.current_tool, EditorTool::Select, "Select");
        draw_tool_button(ui, &mut editor.current_tool, EditorTool::Build, "Build");
        draw_tool_button(ui, &mut editor.current_tool, EditorTool::Erase, "Erase");

        ui.separator();

        ui.horizontal(|ui| {
            ui.label(RichText::new("Picking:").strong());
            let pick_text = if editor.plane_picking { "Plane [ ON ]" } else { "Plane [ OFF ]" };
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

                ui.horizontal(|ui| {
                    ui.label("Texture:");
                    ui.text_edit_singleline(&mut editor.build_template.texture);
                });
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
