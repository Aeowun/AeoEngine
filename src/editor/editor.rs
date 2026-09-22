use super::navigation::NavigationWindow;
use super::tools;
use crate::engine::EditorMode;
use crate::renderer::camera::CameraController;
use crate::world::{AttributeValue, Cell, WorldCoord};
use egui::RichText;
use std::collections::HashMap;

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

pub struct History {
    pub undo_stack: Vec<HashMap<WorldCoord, Cell>>,
    pub redo_stack: Vec<HashMap<WorldCoord, Cell>>,
}

impl History {
    pub fn new() -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        }
    }

    pub fn push(&mut self, cells: HashMap<WorldCoord, Cell>) {
        self.undo_stack.push(cells);
        self.redo_stack.clear();
    }
}

pub struct Editor {
    pub mode: EditorMode,
    pub camera: CameraController,
    pub anchor: WorldCoord,
    pub hovered_cell: Option<WorldCoord>,
    pub selected_coord: Option<WorldCoord>,
    pub selected_coords: Vec<WorldCoord>,
    pub current_tool: EditorTool,

    // Windows
    pub navigation_window: NavigationWindow,
    pub show_properties_window: bool,
    pub show_world_window: bool,
    pub plane_picking: bool,

    // Layout
    pub right_panel_split: f32,

    pub show_script_workspace: bool,
    pub last_show_script_workspace: bool,
    pub script_editor: super::script_editor::ScriptEditor,

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

    pub history: History,

    pub terminal_output: String,
    pub terminal_auto_scroll: bool,

    // Attribute add state
    pub attribute_add_name: String,
    pub attribute_add_type: AttributeType,
    pub attribute_add_value: AttributeValue,
    pub attribute_add_error: Option<String>,
    pub last_selected_coord: Option<WorldCoord>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AttributeType {
    Number,
    Bool,
    String,
}

impl Editor {
    pub fn new() -> Self {
        Self {
            mode: EditorMode::default(),
            camera: CameraController::new(),
            anchor: WorldCoord::new(0, 0, 0),
            hovered_cell: None,
            selected_coord: None,
            selected_coords: Vec::new(),
            current_tool: EditorTool::Select,

            navigation_window: NavigationWindow::new(),
            show_properties_window: true,
            show_world_window: true,
            plane_picking: false,

            right_panel_split: 0.5,

            show_script_workspace: false,
            last_show_script_workspace: false,
            script_editor: super::script_editor::ScriptEditor::new(),

            show_clear_confirmation: false,
            needs_clear_world: false,
            needs_save: false,
            needs_exit: false,

            build_template: crate::world::Cell::new_block(),
            active_color_target: None,
            viewport_rect: egui::Rect::NOTHING,
            history: History::new(),

            terminal_output: String::new(),
            terminal_auto_scroll: true,

            attribute_add_name: String::new(),
            attribute_add_type: AttributeType::String,
            attribute_add_value: AttributeValue::String(String::new()),
            attribute_add_error: None,
            last_selected_coord: None,
        }
    }

    pub fn set_anchor(&mut self, coord: WorldCoord) {
        self.anchor = coord;
        self.camera.target = glam::Vec3::new(coord.x as f32, coord.y as f32, coord.z as f32);
        self.navigation_window.sync(coord);
    }

    pub fn show_ui(
        &mut self,
        ctx: &egui::Context,
        world: &mut crate::world::World,
        project_path: &Option<std::path::PathBuf>,
    ) {
        self.draw_menu_bar(ctx);
        self.draw_tool_bar(ctx);

        if !self.last_show_script_workspace && self.show_script_workspace {
            self.script_editor.refresh_scripts(project_path);
        }
        self.last_show_script_workspace = self.show_script_workspace;

        if self.show_script_workspace {
            self.script_editor.show_ui(ctx, project_path, world);
            return;
        }

        self.draw_status_bar(ctx);

        self.draw_left_panel(ctx, world);
        self.draw_right_panel(ctx, world, project_path);

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

    fn draw_right_panel(
        &mut self,
        ctx: &egui::Context,
        world: &mut crate::world::World,
        project_path: &Option<std::path::PathBuf>,
    ) {
        let show_prop = self.show_properties_window;
        let show_nav = self.navigation_window.is_open;
        let show_terminal = self.script_editor.show_output;

        if show_prop || show_nav || show_terminal {
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

                    if show_prop {
                        let total_height = ui.available_height();
                        let split_height = if show_nav || show_terminal {
                            total_height * self.right_panel_split
                        } else {
                            total_height
                        };

                        // PROPERTIES (Top)
                        egui::TopBottomPanel::top("prop_top")
                            .exact_height(split_height)
                            .show_inside(ui, |ui| {
                                ui.add_space(4.0);
                                ui.horizontal(|ui| {
                                    ui.heading("PROPERTIES");
                                    ui.with_layout(
                                        egui::Layout::right_to_left(egui::Align::Center),
                                        |ui| {
                                            if ui.button("X").clicked() {
                                                self.show_properties_window = false;
                                            }
                                        },
                                    );
                                });
                                ui.separator();
                                self.render_properties_content(ui, world, project_path);
                            });

                        if show_nav || show_terminal {
                            // High-vis draggable separator
                            let (sep_rect, sep_resp) = ui.allocate_exact_size(
                                egui::vec2(ui.available_width(), 4.0),
                                egui::Sense::drag(),
                            );
                            ui.painter().rect_filled(
                                sep_rect,
                                0.0,
                                egui::Color32::from_rgb(150, 150, 150),
                            );
                            if ui.rect_contains_pointer(sep_rect) {
                                ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::ResizeVertical);
                            }

                            if sep_resp.dragged() {
                                self.right_panel_split +=
                                    ui.input(|i| i.pointer.delta().y) / total_height;
                            }
                            self.right_panel_split = self.right_panel_split.clamp(0.1, 0.9);
                        }
                    }

                    // Remaining space (Bottom) for Navigation and Terminal
                    egui::CentralPanel::default().show_inside(ui, |ui| {
                        if show_nav {
                            ui.add_space(4.0);
                            ui.horizontal(|ui| {
                                ui.heading("NAVIGATION");
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        if ui.button("X").clicked() {
                                            self.navigation_window.is_open = false;
                                        }
                                    },
                                );
                            });
                            ui.separator();
                            self.render_navigation_content(ui);
                            ui.add_space(8.0);
                        }

                        if show_terminal {
                            ui.add_space(4.0);
                            ui.horizontal(|ui| {
                                ui.heading("PRINT OUTPUT");
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        if ui.button("X").clicked() {
                                            self.script_editor.show_output = false;
                                        }
                                    },
                                );
                            });
                            ui.separator();
                            self.render_terminal_content(ui);
                        }
                    });
                });
        }
    }
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

    fn render_properties_content(
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
                                    egui::ComboBox::from_id_source("attach_script_combo")
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
                                    egui::ComboBox::from_id_source("attr_type_combo")
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

    fn render_navigation_content(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("X:");
            ui.add(
                egui::TextEdit::singleline(&mut self.navigation_window.x_buf).desired_width(40.0),
            );
            ui.add_space(4.0);
            ui.label("Y:");
            ui.add(
                egui::TextEdit::singleline(&mut self.navigation_window.y_buf).desired_width(40.0),
            );
            ui.add_space(4.0);
            ui.label("Z:");
            ui.add(
                egui::TextEdit::singleline(&mut self.navigation_window.z_buf).desired_width(40.0),
            );
            ui.add_space(8.0);

            if ui.button("Go").clicked() {
                let x = self.navigation_window.x_buf.parse::<i32>().unwrap_or(0);
                let y = self.navigation_window.y_buf.parse::<i32>().unwrap_or(0);
                let z = self.navigation_window.z_buf.parse::<i32>().unwrap_or(0);
                self.set_anchor(WorldCoord::new(x, y, z));
            }
        });
    }

    fn render_terminal_content(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            let mut job = egui::text::LayoutJob::default();
            job.wrap.max_width = ui.available_width();

            for line in self.terminal_output.split_inclusive('\n') {
                let color = if line.contains("[ERROR]") {
                    egui::Color32::from_rgb(255, 80, 80)
                } else if line.contains("[WARNING]") {
                    egui::Color32::from_rgb(255, 220, 0)
                } else {
                    ui.visuals().text_color()
                };

                job.append(
                    line,
                    0.0,
                    egui::TextFormat {
                        font_id: egui::TextStyle::Monospace.resolve(ui.style()),
                        color,
                        ..Default::default()
                    },
                );
            }

            let available_h = ui.available_height();
            let controls_h = 30.0;
            let scroll_h = (available_h - controls_h).max(40.0);

            egui::ScrollArea::vertical()
                .id_salt("terminal_scroll")
                .max_height(scroll_h)
                .stick_to_bottom(self.terminal_auto_scroll)
                .show(ui, |ui| {
                    ui.add(egui::Label::new(job).selectable(true));
                });

            ui.horizontal(|ui| {
                if ui.button("Clear").clicked() {
                    self.terminal_output.clear();
                }
                ui.checkbox(&mut self.terminal_auto_scroll, "Auto-scroll");
            });
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
        let active = world.active_blocks();

        let mut blocks = Vec::new();
        let mut lights = Vec::new();
        let mut spawn_points = Vec::new();
        let mut fx_blocks = Vec::new();
        let mut players = Vec::new();
        let mut npcs = Vec::new();

        for coord in active {
            if let Some(cell) = world.get(coord) {
                match cell.cell_type {
                    crate::world::CellType::Block => blocks.push(coord),
                    crate::world::CellType::Light => lights.push(coord),
                    crate::world::CellType::SpawnPoint => spawn_points.push(coord),
                    crate::world::CellType::FxBlock => fx_blocks.push(coord),
                    crate::world::CellType::Player => players.push(coord),
                    crate::world::CellType::NPC => npcs.push(coord),
                    crate::world::CellType::Empty => {}
                }
            }
        }

        let sort_fn =
            |a: &WorldCoord, b: &WorldCoord| a.x.cmp(&b.x).then(a.y.cmp(&b.y)).then(a.z.cmp(&b.z));

        blocks.sort_by(sort_fn);
        lights.sort_by(sort_fn);
        spawn_points.sort_by(sort_fn);
        fx_blocks.sort_by(sort_fn);
        players.sort_by(sort_fn);
        npcs.sort_by(sort_fn);

        egui::ScrollArea::vertical().show(ui, |ui| {
            // --- HIERARCHY ---
            let tree_data = [
                ("BLOCKS", &blocks),
                ("LIGHTS", &lights),
                ("SPAWN POINTS", &spawn_points),
                ("FX BLOCKS", &fx_blocks),
                ("PLAYERS", &players),
                ("NPCs", &npcs),
            ];

            for (name, list) in tree_data {
                egui::CollapsingHeader::new(format!("{} ({})", name, list.len())).show(ui, |ui| {
                    for &coord in list {
                        let label = format!("({}, {}, {})", coord.x, coord.y, coord.z);
                        let is_selected = self.selected_coord == Some(coord);
                        if ui.selectable_label(is_selected, label).clicked() {
                            self.selected_coord = Some(coord);
                            self.selected_coords = vec![coord];
                        }
                    }
                });
            }

            ui.separator();

            // --- LIGHTING ---
            egui::CollapsingHeader::new("LIGHTING")
                .default_open(true)
                .show(ui, |ui| {
                    ui.checkbox(&mut world.lighting.shadows_enabled, "Shadows Enabled");
                    ui.checkbox(
                        &mut world.lighting.global_light_enabled,
                        "Global Light Enabled",
                    );

                    ui.separator();
                    ui.label("Global Light Direction");
                    ui.horizontal(|ui| {
                        ui.label("X:");
                        ui.add(
                            egui::DragValue::new(&mut world.lighting.global_light_direction.x)
                                .speed(0.01),
                        );
                        ui.label("Y:");
                        ui.add(
                            egui::DragValue::new(&mut world.lighting.global_light_direction.y)
                                .speed(0.01),
                        );
                        ui.label("Z:");
                        ui.add(
                            egui::DragValue::new(&mut world.lighting.global_light_direction.z)
                                .speed(0.01),
                        );
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
                            world.lighting.global_light_color =
                                glam::Vec3::new(color[0], color[1], color[2]);
                        }
                    });

                    ui.horizontal(|ui| {
                        ui.label("Global Light Intensity:");
                        ui.add(
                            egui::DragValue::new(&mut world.lighting.global_light_intensity)
                                .speed(0.1)
                                .range(0.0..=f32::MAX),
                        );
                    });

                    ui.horizontal(|ui| {
                        ui.label("Ambient Intensity:");
                        ui.add(
                            egui::DragValue::new(&mut world.lighting.ambient_intensity)
                                .speed(0.01)
                                .range(0.0..=1.0),
                        );
                    });

                    ui.separator();
                    ui.checkbox(&mut world.sky.enabled, "Sky Enabled");
                    ui.horizontal(|ui| {
                        ui.label("Sky:");
                        let mut current_preset = world.sky.preset.clone();
                        egui::ComboBox::from_id_source("sky_preset_combo")
                            .selected_text(&current_preset)
                            .show_ui(ui, |ui| {
                                for option in &["Temperate", "Tropical", "Desert", "Snowy", "Mars"]
                                {
                                    ui.selectable_value(
                                        &mut current_preset,
                                        option.to_string(),
                                        *option,
                                    );
                                }
                            });
                        if current_preset != world.sky.preset {
                            world.sky.preset = current_preset;
                            world.sky.update_preset_textures();
                        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::{CellType, World, WorldCoord};
    use glam::Vec3;

    #[test]
    fn test_editor_script_workspace_toggle() {
        let mut editor = Editor::new();
        assert!(!editor.show_script_workspace);

        editor.show_script_workspace = true;
        assert!(editor.show_script_workspace);

        editor.show_script_workspace = false;
        assert!(!editor.show_script_workspace);
    }

    #[test]
    fn test_inspector_script_binding_lookup() {
        let mut world = crate::world::World::new();
        let mut editor = Editor::new();

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
    fn test_editor_script_refresh_on_transition() {
        let mut editor = Editor::new();
        let test_dir = std::path::PathBuf::from("TestProject_RefreshTransition");
        let scripts_dir = test_dir.join("scripts");
        if test_dir.exists() {
            let _ = std::fs::remove_dir_all(&test_dir);
        }
        std::fs::create_dir_all(&scripts_dir).unwrap();
        std::fs::write(scripts_dir.join("test.aeo"), "").unwrap();

        let ctx = egui::Context::default();
        let mut world = crate::world::World::new();

        // 1. Initial state: not showing, list empty
        assert!(!editor.show_script_workspace);
        assert!(editor.script_editor.scripts_list.is_empty());

        // 2. Toggle show_script_workspace to true
        editor.show_script_workspace = true;

        // 3. Call show_ui inside context.run, which should trigger refresh_scripts
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            editor.show_ui(ctx, &mut world, &Some(test_dir.clone()));
        });

        // 4. Verify list is now populated
        assert_eq!(editor.script_editor.scripts_list.len(), 1);

        let _ = std::fs::remove_dir_all(&test_dir);
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
}
