use crate::world::WorldCoord;
use crate::renderer::camera::CameraController;
use super::navigation::NavigationWindow;
use super::tools;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum GridPlane {
    Xz,
    Yz,
    Xy,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum EditorTool {
    Navigate,
    Build,
    Erase,
    Select,
}

pub struct Editor {
    pub camera: CameraController,
    pub anchor: WorldCoord,
    pub hovered_cell: Option<WorldCoord>,
    pub selected_coord: Option<WorldCoord>,
    pub current_tool: EditorTool,

    // Windows
    pub navigation_window: NavigationWindow,
    pub show_properties_window: bool,

    // Signals
    pub show_clear_confirmation: bool,
    pub needs_clear_world: bool,
    pub needs_save: bool,
    pub needs_exit: bool,
}

impl Editor {
    pub fn new() -> Self {
        Self {
            camera: CameraController::new(),
            anchor: WorldCoord::new(0, 0, 0),
            hovered_cell: None,
            selected_coord: None,
            current_tool: EditorTool::Navigate,

            navigation_window: NavigationWindow::new(),
            show_properties_window: false,

            show_clear_confirmation: false,
            needs_clear_world: false,
            needs_save: false,
            needs_exit: false,
        }
    }

    pub fn set_anchor(&mut self, coord: WorldCoord) {
        self.anchor = coord;
        self.camera.target = glam::Vec3::new(
            coord.x as f32,
            coord.y as f32,
            coord.z as f32,
        );
        self.navigation_window.sync(coord);
    }

    pub fn show_ui(&mut self, ctx: &egui::Context, world: &mut crate::world::World) {
        self.draw_menu_bar(ctx);
        self.draw_tool_bar(ctx);
        self.draw_status_bar(ctx);
        self.draw_windows(ctx, world);
    }

    fn draw_menu_bar(&mut self, ctx: &egui::Context) {
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
                    if ui.button("Navigation").clicked() {
                        self.navigation_window.is_open = true;
                        ui.close_menu();
                    }
                    if ui.button("Properties").clicked() {
                        self.show_properties_window = true;
                        ui.close_menu();
                    }
                });

                ui.menu_button("World", |_ui| {});

                ui.menu_button("Tools", |ui| {
                    if ui.selectable_label(self.current_tool == EditorTool::Navigate, "Navigate").clicked() {
                        self.current_tool = EditorTool::Navigate;
                        ui.close_menu();
                    }
                    if ui.selectable_label(self.current_tool == EditorTool::Select, "Select").clicked() {
                        self.current_tool = EditorTool::Select;
                        ui.close_menu();
                    }
                    if ui.selectable_label(self.current_tool == EditorTool::Build, "Build").clicked() {
                        self.current_tool = EditorTool::Build;
                        ui.close_menu();
                    }
                    if ui.selectable_label(self.current_tool == EditorTool::Erase, "Erase").clicked() {
                        self.current_tool = EditorTool::Erase;
                        ui.close_menu();
                    }
                });
            });
        });
    }

    fn draw_tool_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("tool_bar").show(ctx, |ui| {
            ui.add_space(2.0);
            tools::draw_tool_bar(ui, &mut self.current_tool);
            ui.add_space(2.0);
        });
    }

    fn draw_status_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(format!("Anchor: ({}, {}, {})", self.anchor.x, self.anchor.y, self.anchor.z));
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

    fn draw_windows(&mut self, ctx: &egui::Context, world: &mut crate::world::World) {
        if let Some(new_anchor) = self.navigation_window.show(ctx) {
            self.set_anchor(new_anchor);
        }

        if self.show_properties_window {
            egui::Window::new("Properties")
                .open(&mut self.show_properties_window)
                .show(ctx, |ui| {
                    if let Some(coord) = self.selected_coord {
                        ui.heading("Cell");
                        ui.label(format!("X: {}", coord.x));
                        ui.label(format!("Y: {}", coord.y));
                        ui.label(format!("Z: {}", coord.z));
                        ui.separator();

                        if let Some(cell) = world.get_mut(coord) {
                            ui.heading("Block");
                            ui.label(format!("{:?}", cell.cell_type));
                            ui.add_space(5.0);

                            ui.checkbox(&mut cell.visible, "Visible");
                            ui.checkbox(&mut cell.solid, "Solid");

                            ui.horizontal(|ui| {
                                ui.label("Texture:");
                                ui.text_edit_singleline(&mut cell.texture);
                            });
                        } else {
                            ui.label("Empty Cell");
                        }
                    } else {
                        ui.label("No cell selected.");
                        ui.label("Use the 'Select' tool to pick a cell.");
                    }
                });
        }

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
}
