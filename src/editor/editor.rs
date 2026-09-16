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

        self.draw_properties_panel(ctx, world);

        self.draw_dialogs(ctx, world);
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

    fn draw_properties_panel(&mut self, ctx: &egui::Context, world: &mut crate::world::World) {
        if self.show_properties_window {
            egui::SidePanel::right("properties_panel")
                .resizable(true)
                .default_width(260.0)
                .show(ctx, |ui| {
                    // Enforce square, hard-edged style for this panel
                    ui.style_mut().visuals.window_rounding = egui::Rounding::ZERO;
                    ui.style_mut().visuals.widgets.noninteractive.rounding = egui::Rounding::ZERO;
                    ui.style_mut().visuals.widgets.inactive.rounding = egui::Rounding::ZERO;
                    ui.style_mut().visuals.widgets.hovered.rounding = egui::Rounding::ZERO;
                    ui.style_mut().visuals.widgets.active.rounding = egui::Rounding::ZERO;

                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        ui.heading("PROPERTIES");
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("X").clicked() {
                                self.show_properties_window = false;
                            }
                        });
                    });
                    ui.separator();

                    if let Some(coord) = self.selected_coord {
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            // --- IDENTITY ---
                            egui::CollapsingHeader::new("IDENTITY")
                                .default_open(true)
                                .show(ui, |ui| {
                                    if let Some(cell) = world.get(coord) {
                                        ui.horizontal(|ui| {
                                            ui.label("Type:");
                                            ui.label(format!("{:?}", cell.cell_type));
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

                            if let Some(cell) = world.get_mut(coord) {
                                // --- PHYSICS ---
                                egui::CollapsingHeader::new("PHYSICS")
                                    .default_open(true)
                                    .show(ui, |ui| {
                                        ui.checkbox(&mut cell.solid, "Solid");
                                        ui.checkbox(&mut cell.anchored, "Anchored");
                                    });

                                // --- RENDERING ---
                                egui::CollapsingHeader::new("RENDERING")
                                    .default_open(true)
                                    .show(ui, |ui| {
                                        ui.checkbox(&mut cell.visible, "Visible");
                                        ui.horizontal(|ui| {
                                            ui.label("Texture:");
                                            ui.text_edit_singleline(&mut cell.texture);
                                        });
                                    });

                                // --- WORLD ---
                                egui::CollapsingHeader::new("WORLD")
                                    .default_open(false)
                                    .show(ui, |ui| {
                                        ui.label("Selectable: true");
                                        ui.label("Editable: true");
                                    });
                            }
                        });
                    } else {
                        ui.centered_and_justified(|ui| {
                            ui.label("No cell selected.");
                        });
                    }
                });
        }
    }

    fn draw_dialogs(&mut self, ctx: &egui::Context, _world: &mut crate::world::World) {
        if let Some(new_anchor) = self.navigation_window.show(ctx) {
            self.set_anchor(new_anchor);
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
