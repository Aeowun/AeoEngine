use crate::world::WorldCoord;
use crate::renderer::camera::CameraController;
use crate::engine::EditorMode;
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

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ColorTarget {
    Build,
    PropertyColor(WorldCoord),
    PropertyLight(WorldCoord),
}

pub struct Editor {
    pub mode: EditorMode,
    pub camera: CameraController,
    pub anchor: WorldCoord,
    pub hovered_cell: Option<WorldCoord>,
    pub selected_coord: Option<WorldCoord>,
    pub current_tool: EditorTool,

    // Windows
    pub navigation_window: NavigationWindow,
    pub show_properties_window: bool,
    pub show_world_window: bool,

    // Layout
    pub right_panel_split: f32,

    // signals
    pub show_clear_confirmation: bool,
    pub needs_clear_world: bool,
    pub needs_save: bool,
    pub needs_exit: bool,

    // Build Settings (Template for new blocks)
    pub build_template: crate::world::Cell,

    // Active persistent color picker
    pub active_color_target: Option<ColorTarget>,

    /// The central area of the editor screen not covered by side or top/bottom panels.
    pub viewport_rect: egui::Rect,
}

impl Editor {
    pub fn new() -> Self {
        Self {
            mode: EditorMode::default(),
            camera: CameraController::new(),
            anchor: WorldCoord::new(0, 0, 0),
            hovered_cell: None,
            selected_coord: None,
            current_tool: EditorTool::Navigate,

            navigation_window: NavigationWindow::new(),
            show_properties_window: true,
            show_world_window: true,

            right_panel_split: 0.5,

            show_clear_confirmation: false,
            needs_clear_world: false,
            needs_save: false,
            needs_exit: false,

            build_template: crate::world::Cell::new_block(),
            active_color_target: None,
            viewport_rect: egui::Rect::NOTHING,
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

        self.draw_left_panel(ctx, world);
        self.draw_right_panel(ctx, world);

        self.draw_dialogs(ctx, world);

        self.render_active_color_picker(ctx, world);

        // Capture the remaining central area for the engine viewport using the remaining
        // screen space after panels are placed. This does not claim pointer interaction.
        self.viewport_rect = ctx.available_rect();
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
                    if ui.selectable_label(self.navigation_window.is_open, "Navigation").clicked() {
                        self.navigation_window.is_open = !self.navigation_window.is_open;
                        ui.close_menu();
                    }
                    if ui.selectable_label(self.show_properties_window, "Properties").clicked() {
                        self.show_properties_window = !self.show_properties_window;
                        ui.close_menu();
                    }
                    if ui.selectable_label(self.show_world_window, "World").clicked() {
                        self.show_world_window = !self.show_world_window;
                        ui.close_menu();
                    }
                });

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
            tools::draw_tool_bar(ui, self);
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

    fn draw_right_panel(&mut self, ctx: &egui::Context, world: &mut crate::world::World) {
        let show_prop = self.show_properties_window;
        let show_nav = self.navigation_window.is_open;

        if show_prop || show_nav {
            egui::SidePanel::right("right_panel")
                .resizable(true)
                .default_width(260.0)
                .show(ctx, |ui| {
                    // Enforce square, hard-edged style for this panel
                    ui.style_mut().visuals.window_rounding = egui::Rounding::ZERO;
                    ui.style_mut().visuals.widgets.noninteractive.rounding = egui::Rounding::ZERO;
                    ui.style_mut().visuals.widgets.inactive.rounding = egui::Rounding::ZERO;
                    ui.style_mut().visuals.widgets.hovered.rounding = egui::Rounding::ZERO;
                    ui.style_mut().visuals.widgets.active.rounding = egui::Rounding::ZERO;

                    if show_prop && show_nav {
                        let total_height = ui.available_height();
                        let split_height = total_height * self.right_panel_split;

                        // PROPERTIES (Top)
                        egui::TopBottomPanel::top("prop_top")
                            .exact_height(split_height)
                            .show_inside(ui, |ui| {
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
                                self.render_properties_content(ui, world);
                            });

                        // High-vis draggable separator
                        let (sep_rect, sep_resp) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 4.0), egui::Sense::drag());
                        ui.painter().rect_filled(sep_rect, 0.0, egui::Color32::from_rgb(150, 150, 150));
                        if ui.rect_contains_pointer(sep_rect) {
                            ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::ResizeVertical);
                        }

                        if sep_resp.dragged() {
                             self.right_panel_split += ui.input(|i| i.pointer.delta().y) / total_height;
                        }
                        self.right_panel_split = self.right_panel_split.clamp(0.1, 0.9);

                        // NAVIGATION (Bottom)
                        egui::CentralPanel::default().show_inside(ui, |ui| {
                            ui.add_space(4.0);
                            ui.horizontal(|ui| {
                                ui.heading("NAVIGATION");
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    if ui.button("X").clicked() {
                                        self.navigation_window.is_open = false;
                                    }
                                });
                            });
                            ui.separator();
                            self.render_navigation_content(ui);
                        });

                    } else if show_prop {
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
                        self.render_properties_content(ui, world);
                    } else if show_nav {
                        ui.add_space(4.0);
                        ui.horizontal(|ui| {
                            ui.heading("NAVIGATION");
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if ui.button("X").clicked() {
                                    self.navigation_window.is_open = false;
                                }
                            });
                        });
                        ui.separator();
                        self.render_navigation_content(ui);
                    }
                });
        }
    }

    fn render_properties_content(&mut self, ui: &mut egui::Ui, world: &mut crate::world::World) {
        if let Some(coord) = self.selected_coord {
            egui::ScrollArea::vertical().id_source("prop_scroll").show(ui, |ui| {
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
                                    tools::draw_color_edit(ui, self, ColorTarget::PropertyLight(coord), &mut color);
                                    cell.light_color = color;
                                });

                                ui.horizontal(|ui| {
                                    ui.label("Intensity:");
                                    ui.add(egui::DragValue::new(&mut cell.light_intensity).speed(0.1).range(0.0..=f32::MAX));
                                });

                                ui.horizontal(|ui| {
                                    ui.label("Range:");
                                    ui.add(egui::DragValue::new(&mut cell.light_range).speed(0.1).range(0.0..=f32::MAX));
                                });

                                ui.checkbox(&mut cell.light_shadows, "Shadows");
                            });
                    }

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
                                ui.label("Color:");
                                let mut color = cell.color_rgb;
                                tools::draw_color_edit(ui, self, ColorTarget::PropertyColor(coord), &mut color);
                                cell.color_rgb = color;
                            });

                            ui.horizontal(|ui| {
                                ui.label("Texture:");
                                ui.text_edit_singleline(&mut cell.texture);
                            });
                        });
                }
            });
        } else {
            ui.centered_and_justified(|ui| {
                ui.label("No cell selected.");
            });
        }
    }

    fn render_navigation_content(&mut self, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical().id_source("nav_scroll").show(ui, |ui| {
            egui::Grid::new("nav_grid")
                .spacing([10.0, 10.0])
                .show(ui, |ui| {
                    ui.label("X:");
                    ui.text_edit_singleline(&mut self.navigation_window.x_buf);
                    ui.end_row();

                    ui.label("Y:");
                    ui.text_edit_singleline(&mut self.navigation_window.y_buf);
                    ui.end_row();

                    ui.label("Z:");
                    ui.text_edit_singleline(&mut self.navigation_window.z_buf);
                    ui.end_row();
                });

            ui.add_space(10.0);

            if ui.button("Go").clicked() {
                let x = self.navigation_window.x_buf.parse::<i32>().unwrap_or(0);
                let y = self.navigation_window.y_buf.parse::<i32>().unwrap_or(0);
                let z = self.navigation_window.z_buf.parse::<i32>().unwrap_or(0);
                self.set_anchor(WorldCoord::new(x, y, z));
            }
        });
    }

    fn draw_left_panel(&mut self, ctx: &egui::Context, world: &mut crate::world::World) {
        if self.show_world_window {
            egui::SidePanel::left("left_panel")
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
                        ui.heading("WORLD");
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("X").clicked() {
                                self.show_world_window = false;
                            }
                        });
                    });
                    ui.separator();

                    self.render_world_content(ui, world);
                });
        }
    }

    fn render_world_content(&mut self, ui: &mut egui::Ui, world: &mut crate::world::World) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            // --- LIGHTING ---
            egui::CollapsingHeader::new("LIGHTING")
                .default_open(true)
                .show(ui, |ui| {
                    ui.checkbox(&mut world.lighting.shadows_enabled, "Shadows Enabled");
                    ui.checkbox(&mut world.lighting.global_light_enabled, "Global Light Enabled");

                    ui.separator();
                    ui.label("Global Light Direction");
                    ui.horizontal(|ui| {
                        ui.label("X:");
                        ui.add(egui::DragValue::new(&mut world.lighting.global_light_direction.x).speed(0.01));
                        ui.label("Y:");
                        ui.add(egui::DragValue::new(&mut world.lighting.global_light_direction.y).speed(0.01));
                        ui.label("Z:");
                        ui.add(egui::DragValue::new(&mut world.lighting.global_light_direction.z).speed(0.01));
                    });

                    ui.separator();
                    ui.horizontal(|ui| {
                        ui.label("Global Light Color:");
                        let mut color = [
                            world.lighting.global_light_color.x,
                            world.lighting.global_light_color.y,
                            world.lighting.global_light_color.z,
                        ];
                        if ui.color_edit_button_rgb(&mut color).changed() {
                            world.lighting.global_light_color = glam::Vec3::new(color[0], color[1], color[2]);
                        }
                    });

                    ui.horizontal(|ui| {
                        ui.label("Global Light Intensity:");
                        ui.add(egui::DragValue::new(&mut world.lighting.global_light_intensity).speed(0.1).range(0.0..=f32::MAX));
                    });

                    ui.horizontal(|ui| {
                        ui.label("Ambient Intensity:");
                        ui.add(egui::DragValue::new(&mut world.lighting.ambient_intensity).speed(0.01).range(0.0..=1.0));
                    });
                });

            // --- PHYSICS ---
            egui::CollapsingHeader::new("PHYSICS")
                .default_open(true)
                .show(ui, |ui| {
                    ui.label("Gravity");
                    ui.horizontal(|ui| {
                        ui.label("X:");
                        ui.add(egui::DragValue::new(&mut world.gravity.x).speed(0.1));
                        ui.label("Y:");
                        ui.add(egui::DragValue::new(&mut world.gravity.y).speed(0.1));
                        ui.label("Z:");
                        ui.add(egui::DragValue::new(&mut world.gravity.z).speed(0.1));
                    });
                });
        });
    }

    fn draw_dialogs(&mut self, ctx: &egui::Context, _world: &mut crate::world::World) {
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

    fn render_active_color_picker(&mut self, ctx: &egui::Context, world: &mut crate::world::World) {
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
                    let color_ref = match target {
                        ColorTarget::Build => Some(&mut self.build_template.color_rgb),
                        ColorTarget::PropertyColor(coord) => {
                            world.get_mut(coord).map(|c| &mut c.color_rgb)
                        }
                        ColorTarget::PropertyLight(coord) => {
                            world.get_mut(coord).map(|c| &mut c.light_color)
                        }
                    };

                    if let Some(color) = color_ref {
                        let mut edit_color = [color.x, color.y, color.z];
                        if egui::color_picker::color_edit_button_rgb(ui, &mut edit_color).changed() {
                            *color = glam::Vec3::from_array(edit_color);
                        }
                    } else {
                        ui.label("Target no longer exists.");
                        if ui.button("Close").clicked() {
                            self.active_color_target = None;
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
