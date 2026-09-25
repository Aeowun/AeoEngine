pub mod asset_search;
pub mod audio_tools;
pub mod texture_tools;

pub use audio_tools::{draw_audio_edit, import_audio};
pub use texture_tools::draw_texture_edit;

use super::editor::{ColorTarget, Editor, EditorTool};
use crate::engine::EditorMode;
use egui::{Color32, RichText, Ui};
use glam::Vec3;
use std::path::PathBuf;

/// Draw the shared RGB editor and color-wheel toggle.
pub fn draw_color_edit(ui: &mut Ui, editor: &mut Editor, target: ColorTarget, color: &mut Vec3) {
    ui.horizontal(|ui| {
        let mut text = format!("{:.3}, {:.3}, {:.3}", color.x, color.y, color.z);

        let response = ui.add(
            egui::TextEdit::singleline(&mut text)
                .desired_width(140.0)
                .hint_text("R, G, B"),
        );

        if response.changed() {
            let values: Vec<f32> = text
                .split(|character: char| character == ',' || character.is_whitespace())
                .filter_map(|part| part.parse::<f32>().ok())
                .collect();

            if values.len() == 3 {
                *color = Vec3::new(values[0], values[1], values[2]);
            }
        }

        response.context_menu(|ui| {
            if ui.button("Copy RGB Value").clicked() {
                ui.output_mut(|output| {
                    output.copied_text = format!("{:.3}, {:.3}, {:.3}", color.x, color.y, color.z);
                });

                ui.close_menu();
            }
        });

        ui.label("|");

        let is_open = editor.active_color_target == Some(target);

        let button = if is_open {
            egui::Button::new(RichText::new(" --- ").color(Color32::WHITE))
                .fill(Color32::from_rgb(0, 100, 200))
        } else {
            egui::Button::new(" --- ")
        };

        if ui.add(button).clicked() {
            editor.active_color_target = if is_open { None } else { Some(target) };
        }
    });
}

// -----------------------------------------------------------------------------
// Main toolbar
// -----------------------------------------------------------------------------

pub fn draw_tool_bar(ui: &mut Ui, editor: &mut Editor, project_path: &Option<PathBuf>) {
    ui.horizontal(|ui| {
        let scripts_button = if editor.show_script_workspace {
            egui::Button::new(RichText::new("Scripts").color(Color32::WHITE))
                .fill(Color32::from_rgb(0, 100, 200))
        } else {
            egui::Button::new("Scripts")
        };

        if ui.add(scripts_button).clicked() {
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

            let text = if editor.plane_picking {
                "Plane [ ON ]"
            } else {
                "Plane [ OFF ]"
            };

            if ui.button(text).clicked() {
                editor.plane_picking = !editor.plane_picking;
            }
        });

        if editor.current_tool != EditorTool::Build {
            return;
        }

        ui.separator();

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

                if ui
                    .selectable_label(
                        editor.build_template.cell_type == crate::world::CellType::AudioEmitter,
                        "Audio Emitter",
                    )
                    .clicked()
                {
                    editor.build_template = crate::world::Cell::new_audio_emitter();
                    ui.close_menu();
                }
            },
        );

        ui.separator();

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

        ui.menu_button("Physics ▼", |ui| {
            ui.checkbox(&mut editor.build_template.solid, "Solid");

            ui.checkbox(&mut editor.build_template.anchored, "Anchored");
        });
    });

    // Browsers are drawn after the toolbar so they do not depend on
    // the lifetime of the toolbar's menus.
    texture_tools::draw_texture_browser(ui.ctx(), &mut editor.build_template.texture);

    audio_tools::draw_audio_browser(ui.ctx(), &mut editor.build_template.audio, project_path);
}

fn draw_tool_button(ui: &mut Ui, current: &mut EditorTool, tool: EditorTool, text: &str) {
    let active = *current == tool;

    let button = if active {
        egui::Button::new(RichText::new(text).color(Color32::WHITE))
            .fill(Color32::from_rgb(0, 100, 200))
    } else {
        egui::Button::new(text)
    };

    if ui.add(button).clicked() {
        *current = tool;
    }
}
