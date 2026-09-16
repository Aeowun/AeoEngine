use winit::event::{WindowEvent, MouseButton, MouseScrollDelta, ElementState};
use winit::keyboard::{KeyCode, PhysicalKey};

use crate::renderer::Renderer;
use crate::editor::{Editor, EditorTool};
use crate::world::{World, CellType};
use crate::project::ProjectManager;
use super::View;

pub struct App {
    pub view: View,
    pub renderer: Renderer,
    pub editor: Editor,
    pub world: World,
    pub project_manager: ProjectManager,

    // UI State for Home
    pub show_new_project_dialog: bool,
    pub new_project_name: String,
    pub show_open_project_dialog: bool,

    // Input state
    pub is_middle_mouse_down: bool,
    pub last_cursor_pos: Option<(f64, f64)>,
    pub mouse_pos: (f64, f64),
    pub keys_down: std::collections::HashSet<KeyCode>,
}

impl App {
    pub fn new(width: f32, height: f32) -> Self {
        Self {
            view: View::Home,
            renderer: Renderer::new(width, height),
            editor: Editor::new(),
            world: World::new(),
            project_manager: ProjectManager::new(),

            show_new_project_dialog: false,
            new_project_name: String::new(),
            show_open_project_dialog: false,

            is_middle_mouse_down: false,
            last_cursor_pos: None,
            mouse_pos: (0.0, 0.0),
            keys_down: std::collections::HashSet::new(),
        }
    }

    pub fn on_window_event(&mut self, event: &WindowEvent, egui_ctx: &egui::Context) {
        match event {
            WindowEvent::Resized(size) => {
                self.renderer.resize(size.width as f32, size.height as f32);
            }

            WindowEvent::KeyboardInput { event: winit::event::KeyEvent { physical_key, state, .. }, .. } => {
                if egui_ctx.wants_keyboard_input() {
                    return;
                }

                if let PhysicalKey::Code(key) = physical_key {
                    if *state == ElementState::Pressed {
                        self.keys_down.insert(*key);
                        match key {
                            KeyCode::KeyG => {
                                self.view = if self.view == View::Home { View::Editor } else { View::Home };
                            }
                            KeyCode::KeyH | KeyCode::Escape => {
                                self.view = View::Home;
                            }
                            _ => {}
                        }
                    } else {
                        self.keys_down.remove(key);
                    }
                }
            }

            WindowEvent::MouseInput { state, button, .. } => {
                if egui_ctx.wants_pointer_input() || egui_ctx.is_using_pointer() {
                    return;
                }
                if *button == MouseButton::Middle {
                    self.is_middle_mouse_down = *state == ElementState::Pressed;
                    if !self.is_middle_mouse_down {
                        self.last_cursor_pos = None;
                    }
                } else if *button == MouseButton::Left && *state == ElementState::Pressed {
                    self.on_click();
                }
            }

            WindowEvent::CursorMoved { position, .. } => {
                self.mouse_pos = (position.x, position.y);

                if egui_ctx.wants_pointer_input() || egui_ctx.is_using_pointer() {
                    self.editor.hovered_cell = None;
                } else {
                    self.update_hover();
                }

                if self.is_middle_mouse_down {
                    if let Some((last_x, last_y)) = self.last_cursor_pos {
                        let dx = position.x - last_x;
                        let dy = position.y - last_y;
                        self.editor.camera.orbit(dx as f32, dy as f32);
                    }
                    self.last_cursor_pos = Some((position.x, position.y));
                }
            }

            WindowEvent::MouseWheel { delta, .. } => {
                if egui_ctx.wants_pointer_input() || egui_ctx.is_using_pointer() {
                    return;
                }
                let y = match delta {
                    MouseScrollDelta::LineDelta(_, y) => *y,
                    MouseScrollDelta::PixelDelta(pos) => (pos.y / 100.0) as f32,
                };
                self.editor.camera.zoom(y);
                self.update_hover();
            }

            _ => {}
        }
    }

    fn update_hover(&mut self) {
        self.editor.hovered_cell = crate::editor::grid::picking::update_hover(
            self.mouse_pos.0 as f32,
            self.mouse_pos.1 as f32,
            self.renderer.width(),
            self.renderer.height(),
            &self.editor.camera,
            self.editor.anchor
        );
    }

    fn on_click(&mut self) {
        if self.view == View::Editor {
            if let Some(hover) = self.editor.hovered_cell {
                match self.editor.current_tool {
                    EditorTool::Navigate => {
                        self.editor.set_anchor(hover);
                    }
                    EditorTool::Select => {
                        self.editor.selected_coord = Some(hover);
                        self.editor.show_properties_window = true;
                    }
                    EditorTool::Build => {
                        self.world.set_cell(hover, CellType::Grass);
                        // Ensure it has default grass properties if newly placed
                        if let Some(cell) = self.world.get_mut(hover) {
                            *cell = crate::world::Cell::new_grass();
                        }
                    }
                    EditorTool::Erase => {
                        self.world.set_cell(hover, CellType::Empty);
                    }
                }
            } else {
                // Clicking outside grid might clear selection?
                // Let's keep it for now but maybe user wants it persistent.
            }
        }
    }

    pub fn update(&mut self, egui_ctx: &egui::Context) {
        if self.view == View::Editor {
            if egui_ctx.wants_keyboard_input() {
                return;
            }
            let mut move_vec = glam::Vec2::ZERO;
            if self.keys_down.contains(&KeyCode::KeyW) { move_vec.y += 1.0; }
            if self.keys_down.contains(&KeyCode::KeyS) { move_vec.y -= 1.0; }
            if self.keys_down.contains(&KeyCode::KeyA) { move_vec.x -= 1.0; }
            if self.keys_down.contains(&KeyCode::KeyD) { move_vec.x += 1.0; }

            if move_vec != glam::Vec2::ZERO {
                self.editor.camera.move_target(move_vec.x, move_vec.y);

                // Update UI buffers to match current focus
                let target = self.editor.camera.target;
                self.editor.navigation_window.x_buf = (target.x.floor() as i32).to_string();
                self.editor.navigation_window.y_buf = (target.y.floor() as i32).to_string();
                self.editor.navigation_window.z_buf = (target.z.floor() as i32).to_string();
            }

            if self.editor.needs_clear_world {
                self.world = World::new();
                self.editor.needs_clear_world = false;
                println!("World cleared.");
            }

            if self.editor.needs_save {
                self.save_project();
                self.editor.needs_save = false;
            }

            if self.editor.needs_exit {
                self.save_project();
                self.view = View::Home;
                self.editor.needs_exit = false;
            }
        }
    }

    pub fn update_ui(&mut self, ctx: &egui::Context) {
        let mut next_view = None;

        if self.view == View::Home {
            egui::Window::new("Projects")
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    if ui.button("New Project").clicked() {
                        self.show_new_project_dialog = true;
                        self.show_open_project_dialog = false;
                    }
                    if ui.button("Open Project").clicked() {
                        self.show_open_project_dialog = true;
                        self.show_new_project_dialog = false;
                    }

                    if !self.project_manager.recent_projects.is_empty() {
                        ui.separator();
                        ui.label("Recent Projects");
                        let recent = self.project_manager.recent_projects.clone();
                        for path in recent {
                            let name = path.file_name().unwrap_or_default().to_string_lossy();
                            if ui.button(format!("{}", name)).clicked() {
                                if self.project_manager.open_project(path) {
                                    self.load_project();
                                    next_view = Some(View::Editor);
                                } else {
                                    self.project_manager.load_recent();
                                }
                            }
                        }
                    }
                });

            if self.show_new_project_dialog {
                egui::Window::new("New Project")
                    .anchor(egui::Align2::CENTER_CENTER, [0.0, 100.0])
                    .collapsible(false)
                    .show(ctx, |ui| {
                        ui.horizontal(|ui| {
                            ui.label("Name:");
                            ui.text_edit_singleline(&mut self.new_project_name);
                        });
                        ui.horizontal(|ui| {
                            if ui.button("Create").clicked() {
                                if self.project_manager.create_project(&self.new_project_name).is_some() {
                                    self.load_project();
                                    next_view = Some(View::Editor);
                                    self.show_new_project_dialog = false;
                                    self.new_project_name.clear();
                                }
                            }
                            if ui.button("Cancel").clicked() {
                                self.show_new_project_dialog = false;
                            }
                        });
                    });
            }

            if self.show_open_project_dialog {
                egui::Window::new("Open Project")
                    .anchor(egui::Align2::CENTER_CENTER, [0.0, 100.0])
                    .collapsible(false)
                    .show(ctx, |ui| {
                        let projects = self.project_manager.list_projects();
                        if projects.is_empty() {
                            ui.label("No projects found in UserData.");
                        } else {
                            for path in projects {
                                let name = path.file_name().unwrap_or_default().to_string_lossy();
                                if ui.button(format!("{}", name)).clicked() {
                                    if self.project_manager.open_project(path) {
                                        self.load_project();
                                        next_view = Some(View::Editor);
                                        self.show_open_project_dialog = false;
                                    }
                                }
                            }
                        }
                        if ui.button("Cancel").clicked() {
                            self.show_open_project_dialog = false;
                        }
                    });
            }
        } else if self.view == View::Editor {
            self.editor.show_ui(ctx, &mut self.world);
        }

        if let Some(view) = next_view {
            self.view = view;
        }
    }

    pub fn render(&self) {
        match self.view {
            View::Home => self.renderer.render_home(),
            View::Editor => self.renderer.render_editor(&self.editor, &self.world),
        }
    }

    pub fn load_project(&mut self) {
        if let Some(project_path) = &self.project_manager.current_project {
            let world_path = project_path.join("world.dat");
            if let Err(e) = crate::world::persistence::load_world(&mut self.world, &world_path) {
                eprintln!("Failed to load world: {}", e);
            } else {
                println!("World loaded from {:?}", world_path);
            }
        }
    }

    pub fn save_project(&self) {
        if let Some(project_path) = &self.project_manager.current_project {
            let world_path = project_path.join("world.dat");
            if let Err(e) = crate::world::persistence::save_world(&self.world, &world_path) {
                eprintln!("Failed to save world: {}", e);
            } else {
                println!("World saved to {:?}", world_path);
            }
        }
    }
}
