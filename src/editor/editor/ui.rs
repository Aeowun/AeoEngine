use super::{ColorTarget, Editor, EditorTool};
use crate::editor::tools;

impl Editor {
    pub fn show_ui(
        &mut self,
        ctx: &egui::Context,
        world: &mut crate::world::World,
        project_path: &Option<std::path::PathBuf>,
    ) {
        self.draw_menu_bar(ctx);
        self.draw_tool_bar(ctx, project_path);

        if !self.last_show_script_workspace && self.show_script_workspace {
            self.script_editor.refresh_scripts(project_path);
        }
        self.last_show_script_workspace = self.show_script_workspace;

        if self.show_script_workspace {
            self.script_editor.show_ui(ctx, project_path, world);
            return;
        }

        self.draw_status_bar(ctx);

        self.draw_left_panel(ctx, world, project_path);
        self.draw_right_panel(ctx, world, project_path);

        self.draw_dialogs(ctx, world);

        self.render_active_color_picker(ctx, world);

        // Capture the remaining central area for the engine viewport using the remaining
        // screen space after panels are placed. This does not claim pointer interaction.
        self.viewport_rect = ctx.available_rect();
        self.viewport_ppp = ctx.pixels_per_point();
    }

    pub(crate) fn draw_menu_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Save").clicked() {
                        self.needs_save = true;
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("Exit to Home").clicked() {
                        self.needs_exit = true;
                        ui.close_menu();
                    }
                });

                ui.menu_button("Edit", |ui| {
                    if ui.button("Clear Grid").clicked() {
                        self.show_clear_confirmation = true;
                        ui.close_menu();
                    }
                });

                ui.menu_button("View", |ui| {
                    if ui
                        .selectable_label(self.show_script_workspace, "Script Workspace")
                        .clicked()
                    {
                        self.show_script_workspace = !self.show_script_workspace;
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui
                        .selectable_label(self.navigation_window.is_open, "Navigation")
                        .clicked()
                    {
                        self.navigation_window.is_open = !self.navigation_window.is_open;
                        ui.close_menu();
                    }
                    if ui
                        .selectable_label(self.show_properties_window, "Properties")
                        .clicked()
                    {
                        self.show_properties_window = !self.show_properties_window;
                        ui.close_menu();
                    }
                    if ui
                        .selectable_label(self.script_editor.show_output, "Print Output")
                        .clicked()
                    {
                        self.script_editor.show_output = !self.script_editor.show_output;
                        ui.close_menu();
                    }
                    if ui
                        .selectable_label(self.show_world_window, "World")
                        .clicked()
                    {
                        self.show_world_window = !self.show_world_window;
                        ui.close_menu();
                    }
                });

                ui.menu_button("Tools", |ui| {
                    if ui
                        .selectable_label(self.current_tool == EditorTool::Navigate, "Navigate")
                        .clicked()
                    {
                        self.current_tool = EditorTool::Navigate;
                        ui.close_menu();
                    }
                    if ui
                        .selectable_label(self.current_tool == EditorTool::Select, "Select")
                        .clicked()
                    {
                        self.current_tool = EditorTool::Select;
                        ui.close_menu();
                    }
                    if ui
                        .selectable_label(self.current_tool == EditorTool::Build, "Build")
                        .clicked()
                    {
                        self.current_tool = EditorTool::Build;
                        ui.close_menu();
                    }
                    if ui
                        .selectable_label(self.current_tool == EditorTool::Erase, "Erase")
                        .clicked()
                    {
                        self.current_tool = EditorTool::Erase;
                        ui.close_menu();
                    }
                });
            });
        });
    }

    pub(crate) fn draw_tool_bar(
        &mut self,
        ctx: &egui::Context,
        project_path: &Option<std::path::PathBuf>,
    ) {
        egui::TopBottomPanel::top("tool_bar").show(ctx, |ui| {
            ui.add_space(2.0);
            tools::draw_tool_bar(ui, self, project_path);
            ui.add_space(2.0);
        });
    }

    pub(crate) fn draw_status_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(format!(
                    "Anchor: ({}, {}, {})",
                    self.anchor.x, self.anchor.y, self.anchor.z
                ));
                ui.separator();
                let hover_text = if let Some(h) = self.hovered_cell {
                    format!("Hover: ({}, {}, {})", h.x, h.y, h.z)
                } else {
                    "Hover: —".to_string()
                };
                ui.label(hover_text);
            });
        });
    }

    pub(crate) fn draw_dialogs(&mut self, ctx: &egui::Context, _world: &mut crate::world::World) {
        if self.show_clear_confirmation {
            egui::Window::new("Confirm Clear")
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label("Are you sure you want to erase the entire built scene?");
                    ui.add_space(10.0);
                    ui.horizontal(|ui| {
                        if ui.button("Yes, Clear Everything").clicked() {
                            self.needs_clear_world = true;
                            self.show_clear_confirmation = false;
                        }
                        if ui.button("Cancel").clicked() {
                            self.show_clear_confirmation = false;
                        }
                    });
                });
        }
    }

    pub(crate) fn render_active_color_picker(
        &mut self,
        ctx: &egui::Context,
        world: &mut crate::world::World,
    ) {
        if let Some(target) = self.active_color_target {
            let mut still_open = true;
            let title = match target {
                ColorTarget::Build => "Build Color",
                ColorTarget::PropertyColor(_) => "Block Color",
                ColorTarget::PropertyLight(_) => "Light Color",
            };

            egui::Window::new(title)
                .open(&mut still_open)
                .resizable(false)
                .collapsible(false)
                .show(ctx, |ui| {
                    match target {
                        ColorTarget::Build => {
                            let mut edit_color = [
                                self.build_template.color_rgb.x,
                                self.build_template.color_rgb.y,
                                self.build_template.color_rgb.z,
                            ];
                            if egui::color_picker::color_edit_button_rgb(ui, &mut edit_color)
                                .changed()
                            {
                                self.build_template.color_rgb = glam::Vec3::from_array(edit_color);
                            }
                        }
                        ColorTarget::PropertyColor(coord) => {
                            if let Some(cell) = world.get_mut(coord) {
                                let mut edit_color =
                                    [cell.color_rgb.x, cell.color_rgb.y, cell.color_rgb.z];
                                if egui::color_picker::color_edit_button_rgb(ui, &mut edit_color)
                                    .changed()
                                {
                                    let new_color = glam::Vec3::from_array(edit_color);
                                    cell.color_rgb = new_color;
                                    for &c in &self.selected_coords {
                                        if let Some(other_cell) = world.get_mut(c) {
                                            other_cell.color_rgb = new_color;
                                        }
                                    }
                                }
                            } else {
                                ui.label("Target no longer exists.");
                                if ui.button("Close").clicked() {
                                    self.active_color_target = None;
                                }
                            }
                        }
                        ColorTarget::PropertyLight(coord) => {
                            if let Some(cell) = world.get_mut(coord) {
                                let mut edit_color =
                                    [cell.light_color.x, cell.light_color.y, cell.light_color.z];
                                if egui::color_picker::color_edit_button_rgb(ui, &mut edit_color)
                                    .changed()
                                {
                                    let new_color = glam::Vec3::from_array(edit_color);
                                    cell.light_color = new_color;
                                    for &c in &self.selected_coords {
                                        if let Some(other_cell) = world.get_mut(c) {
                                            other_cell.light_color = new_color;
                                        }
                                    }
                                }
                            } else {
                                ui.label("Target no longer exists.");
                                if ui.button("Close").clicked() {
                                    self.active_color_target = None;
                                }
                            }
                        }
                    }

                    ui.add_space(8.0);
                    if ui.button("Done").clicked() {
                        self.active_color_target = None;
                    }
                });

            if !still_open {
                self.active_color_target = None;
            }
        }
    }
}
