use egui::{Ui, RichText, Color32};
use super::editor::EditorTool;

pub fn draw_tool_bar(ui: &mut Ui, current_tool: &mut EditorTool) {
    ui.horizontal(|ui| {
        ui.label(RichText::new("Tool").strong());

        draw_tool_button(ui, current_tool, EditorTool::Navigate, "Navigate");
        draw_tool_button(ui, current_tool, EditorTool::Select, "Select");
        draw_tool_button(ui, current_tool, EditorTool::Build, "Build");
        draw_tool_button(ui, current_tool, EditorTool::Erase, "Erase");

        if *current_tool == EditorTool::Build {
            ui.separator();
            ui.label(RichText::new("Block:").strong());
            ui.label("Grass");
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
