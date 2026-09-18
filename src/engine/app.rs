use std::time::Instant;

use winit::event::{ElementState, MouseButton, MouseScrollDelta, WindowEvent};
use winit::keyboard::{KeyCode, PhysicalKey};

use crate::editor::{Editor, EditorTool};
use crate::project::ProjectManager;
use crate::renderer::Renderer;
use crate::world::{CellType, World};
use crate::scripting::scene::ScriptScene;
use crate::scripting::api::HostContext;
use crate::engine::entity::EntityManager;

use super::{EditorMode, View, physics};

/// Application state shared by the window, editor, renderer, world, project
/// system, input handling, and runtime physics.
///
/// App is the coordinator for the running application. It does not own the
/// low level rendering implementation, the world storage rules, or the editor
/// implementation itself. Those systems remain in their own modules.
///
/// World remains the authoritative authored scene. PhysicsWorld is temporary
/// runtime state used while the editor is in Play mode.
pub struct App {
    pub view: View,
    pub renderer: Renderer,
    pub editor: Editor,
    pub world: World,
    pub project_manager: ProjectManager,

    pub last_frame_instant: Instant,
    pub physics_clock: physics::PhysicsClock,
    pub physics_world: physics::PhysicsWorld,
    pub last_mode: EditorMode,

    pub show_new_project_dialog: bool,
    pub new_project_name: String,
    pub show_open_project_dialog: bool,
    pub show_unsaved_scripts_dialog: bool,

    pub is_middle_mouse_down: bool,
    pub last_cursor_pos: Option<(f64, f64)>,
    pub mouse_pos: (f64, f64),

    pub is_left_mouse_down: bool,
    pub drag_start_coord: Option<crate::world::WorldCoord>,

    pub keys_down: std::collections::HashSet<KeyCode>,
    pub jump_requested: bool,

    pub character_system: crate::character::CharacterSystem,

    /// Dedicated runtime camera used only during Play mode.
    pub gameplay_camera: crate::renderer::camera::GameplayCamera,

    /// Saved editor camera restored when leaving Play mode.
    pub saved_editor_camera: Option<crate::renderer::camera::CameraController>,

    pub script_scene: Option<ScriptScene>,
    pub entity_manager: EntityManager,
}

impl App {
    pub fn new(width: f32, height: f32) -> Self {
        Self {
            view: View::Home,

            renderer: Renderer::new(width, height),

            editor: Editor::new(),

            world: World::new(),

            project_manager: ProjectManager::new(),

            last_frame_instant: Instant::now(),

            physics_clock: physics::PhysicsClock::new(),

            physics_world: physics::PhysicsWorld::new(),

            last_mode: EditorMode::default(),

            show_new_project_dialog: false,

            new_project_name: String::new(),

            show_open_project_dialog: false,
            show_unsaved_scripts_dialog: false,

            is_middle_mouse_down: false,

            last_cursor_pos: None,

            mouse_pos: (0.0, 0.0),

            is_left_mouse_down: false,

            drag_start_coord: None,

            keys_down: std::collections::HashSet::new(),

            jump_requested: false,

            character_system: crate::character::CharacterSystem::new(),

            gameplay_camera: crate::renderer::camera::GameplayCamera::new(),

            saved_editor_camera: None,

            script_scene: None,
            entity_manager: EntityManager::new(),
        }
    }
}

impl App {
    /// Handles window events used by the application and editor.
    ///
    /// Play camera mouse look is intentionally not handled through
    /// WindowEvent::CursorMoved. Play receives relative DeviceEvent mouse
    /// motion through App::on_mouse_motion instead.
    pub fn on_window_event(&mut self, event: &WindowEvent, egui_ctx: &egui::Context) {
        match event {
            WindowEvent::Resized(size) => {
                self.renderer.resize(size.width as f32, size.height as f32);
            }

            WindowEvent::KeyboardInput {
                event:
                    winit::event::KeyEvent {
                        physical_key,
                        state,
                        ..
                    },
                ..
            } => {
                if egui_ctx.wants_keyboard_input() {
                    return;
                }

                if let PhysicalKey::Code(key) = physical_key {
                    let was_pressed = *state == ElementState::Pressed;
                    if was_pressed {
                        self.keys_down.insert(*key);
                    } else {
                        self.keys_down.remove(key);
                    }

                    if (*key == KeyCode::ShiftLeft || *key == KeyCode::ShiftRight)
                        && self.view == View::Editor
                        && self.editor.mode == EditorMode::Editor
                    {
                        let ppp = egui_ctx.pixels_per_point();
                        let mouse_logical = egui::pos2(
                            self.mouse_pos.0 as f32 / ppp,
                            self.mouse_pos.1 as f32 / ppp,
                        );
                        let in_viewport = self.editor.viewport_rect.contains(mouse_logical);

                        if in_viewport && !egui_ctx.wants_pointer_input() {
                            self.update_hover();
                        }
                    }

                    if was_pressed {
                        let ctrl = self.keys_down.contains(&KeyCode::ControlLeft)
                            || self.keys_down.contains(&KeyCode::ControlRight);

                        match key {
                            // G toggles between Home and Editor.
                            KeyCode::KeyG => {
                                self.view = if self.view == View::Home {
                                    View::Editor
                                } else {
                                    View::Home
                                };
                            }

                            KeyCode::KeyS => {
                                if ctrl && self.view == View::Editor {
                                    if self.editor.show_script_workspace {
                                        let shift = self.keys_down.contains(&KeyCode::ShiftLeft) || self.keys_down.contains(&KeyCode::ShiftRight);
                                        if shift {
                                            self.editor.script_editor.save_all();
                                        } else {
                                            self.editor.script_editor.save_active();
                                        }
                                    } else {
                                        self.save_project();
                                    }
                                }
                            }

                            KeyCode::KeyN => {
                                if ctrl && self.view == View::Editor && self.editor.show_script_workspace {
                                    self.editor.script_editor.show_new_script_dialog = true;
                                }
                            }

                            KeyCode::KeyZ => {
                                if ctrl && self.view == View::Editor {
                                    if let Some(prev) = self.editor.history.undo_stack.pop() {
                                        self.editor
                                            .history
                                            .redo_stack
                                            .push(self.world.cells.clone());
                                        self.world.cells = prev;
                                    }
                                }
                            }

                            KeyCode::KeyY => {
                                if ctrl && self.view == View::Editor {
                                    if let Some(next) = self.editor.history.redo_stack.pop() {
                                        self.editor
                                            .history
                                            .undo_stack
                                            .push(self.world.cells.clone());
                                        self.world.cells = next;
                                    }
                                }
                            }

                            KeyCode::Delete => {
                                if self.view == View::Editor
                                    && !self.editor.selected_coords.is_empty()
                                {
                                    self.editor.history.push(self.world.cells.clone());
                                    for coord in &self.editor.selected_coords {
                                        self.world.cells.remove(coord);
                                    }
                                    self.editor.selected_coords.clear();
                                    self.editor.selected_coord = None;
                                }
                            }

                            KeyCode::KeyF => {
                                if self.view == View::Editor
                                    && !self.editor.selected_coords.is_empty()
                                {
                                    let mut min = glam::Vec3::new(f32::MAX, f32::MAX, f32::MAX);
                                    let mut max = glam::Vec3::new(f32::MIN, f32::MIN, f32::MIN);
                                    for coord in &self.editor.selected_coords {
                                        let p = glam::Vec3::new(
                                            coord.x as f32,
                                            coord.y as f32,
                                            coord.z as f32,
                                        );
                                        min = min.min(p);
                                        max = max.max(p);
                                    }
                                    let center = (min + max) / 2.0;
                                    self.editor.camera.target = center;
                                    self.editor.anchor = crate::world::WorldCoord::new(
                                        center.x.round() as i32,
                                        center.y.round() as i32,
                                        center.z.round() as i32,
                                    );
                                }
                            }

                            KeyCode::Digit1 => {
                                if self.view == View::Editor {
                                    self.editor.current_tool = EditorTool::Navigate;
                                }
                            }
                            KeyCode::Digit2 => {
                                if self.view == View::Editor {
                                    self.editor.current_tool = EditorTool::Select;
                                }
                            }
                            KeyCode::Digit3 => {
                                if self.view == View::Editor {
                                    self.editor.current_tool = EditorTool::Build;
                                }
                            }
                            KeyCode::Digit4 => {
                                if self.view == View::Editor {
                                    self.editor.current_tool = EditorTool::Erase;
                                }
                            }

                            KeyCode::Escape => {
                                if self.editor.mode == EditorMode::Play {
                                    self.editor.mode = EditorMode::Editor;
                                } else if self.view == View::Editor {
                                    if self.is_left_mouse_down {
                                        self.is_left_mouse_down = false;
                                        self.drag_start_coord = None;
                                    } else if !self.editor.selected_coords.is_empty() {
                                        self.editor.selected_coords.clear();
                                        self.editor.selected_coord = None;
                                    } else {
                                        self.exit_to_home();
                                    }
                                }
                            }

                            KeyCode::Space => {
                                if self.view == View::Editor && self.editor.mode == EditorMode::Play
                                {
                                    self.jump_requested = true;
                                }
                            }

                            _ => {}
                        }
                    } else {
                        self.keys_down.remove(key);
                    }
                }
            }

            WindowEvent::MouseInput { state, button, .. } => {
                let ppp = egui_ctx.pixels_per_point();

                let mouse_logical =
                    egui::pos2(self.mouse_pos.0 as f32 / ppp, self.mouse_pos.1 as f32 / ppp);

                let in_viewport = self.editor.viewport_rect.contains(mouse_logical);

                let wants_pointer = egui_ctx.wants_pointer_input();

                if *button == MouseButton::Middle {
                    if *state == ElementState::Pressed {
                        if !in_viewport || wants_pointer {
                            return;
                        }
                        self.is_middle_mouse_down = true;
                    } else {
                        self.is_middle_mouse_down = false;
                        self.last_cursor_pos = None;
                    }

                    return;
                }

                if *button == MouseButton::Left {
                    if *state == ElementState::Pressed {
                        if !in_viewport || wants_pointer {
                            return;
                        }

                        self.is_left_mouse_down = true;

                        self.drag_start_coord = self.editor.hovered_cell;

                        if self.editor.current_tool == EditorTool::Navigate
                            || self.editor.current_tool == EditorTool::Select
                        {
                            self.on_click();
                        }
                    } else {
                        if self.is_left_mouse_down {
                            if self.editor.current_tool == EditorTool::Build
                                || self.editor.current_tool == EditorTool::Erase
                                || self.editor.current_tool == EditorTool::Select
                            {
                                if let (Some(start), Some(end)) =
                                    (self.drag_start_coord, self.editor.hovered_cell)
                                {
                                    self.apply_tool_to_range(start, end);
                                }
                            }
                        }

                        self.is_left_mouse_down = false;

                        self.drag_start_coord = None;
                    }

                    return;
                }

                if !in_viewport {
                    return;
                }
            }

            /// CursorMoved remains the editor cursor path.
            ///
            /// Play camera look does not use the cursor's absolute position
            /// or WindowEvent::CursorMoved delta.
            WindowEvent::CursorMoved { position, .. } => {
                let dx = position.x - self.mouse_pos.0;

                let dy = position.y - self.mouse_pos.1;

                self.mouse_pos = (position.x, position.y);

                let ppp = egui_ctx.pixels_per_point();

                let mouse_logical = egui::pos2(position.x as f32 / ppp, position.y as f32 / ppp);

                let in_viewport = self.editor.viewport_rect.contains(mouse_logical);

                if in_viewport && self.editor.mode == crate::engine::EditorMode::Editor {
                    self.update_hover();
                } else {
                    self.editor.hovered_cell = None;
                }

                if self.is_middle_mouse_down && self.editor.mode == EditorMode::Editor {
                    self.editor.camera.orbit(dx as f32, dy as f32);
                }
            }

            WindowEvent::MouseWheel { delta, .. } => {
                let ppp = egui_ctx.pixels_per_point();

                let mouse_logical =
                    egui::pos2(self.mouse_pos.0 as f32 / ppp, self.mouse_pos.1 as f32 / ppp);

                let in_viewport = self.editor.viewport_rect.contains(mouse_logical);

                if !in_viewport || egui_ctx.wants_pointer_input() {
                    return;
                }

                let y = match delta {
                    MouseScrollDelta::LineDelta(_, y) => *y,

                    MouseScrollDelta::PixelDelta(pos) => (pos.y / 100.0) as f32,
                };

                if self.editor.mode == EditorMode::Editor {
                    self.editor.camera.zoom(y);
                    self.update_hover();
                }
            }

            _ => {}
        }
    }

    /// Applies raw relative mouse movement to the gameplay camera.
    ///
    /// This is the Play mode mouse look path. Absolute cursor position is not
    /// used to determine orientation.
    pub fn on_mouse_motion(&mut self, dx: f64, dy: f64) {
        if self.view == View::Editor && self.editor.mode == EditorMode::Play {
            self.gameplay_camera.orbit(dx as f32, dy as f32);
        }
    }

    fn update_hover(&mut self) {
        if !self.editor.plane_picking {
            let hit = crate::editor::grid::picking::raycast_world(
                self.mouse_pos.0 as f32,
                self.mouse_pos.1 as f32,
                self.renderer.width(),
                self.renderer.height(),
                &self.editor.camera,
                &self.world,
                self.editor.mode == crate::engine::EditorMode::Editor,
            );

            if let Some((coord, normal)) = hit {
                if self.editor.current_tool == EditorTool::Build {
                    let shift_down = self.keys_down.contains(&KeyCode::ShiftLeft)
                        || self.keys_down.contains(&KeyCode::ShiftRight);

                    if !shift_down {
                        // Offset by normal to place on surface
                        self.editor.hovered_cell = Some(crate::world::WorldCoord::new(
                            coord.x + normal.x as i32,
                            coord.y + normal.y as i32,
                            coord.z + normal.z as i32,
                        ));
                    } else {
                        // Overwrite
                        self.editor.hovered_cell = Some(coord);
                    }
                } else {
                    self.editor.hovered_cell = Some(coord);
                }
                return;
            }

            // No ray hit. Select and Erase do not fall back to the plane.
            if self.editor.current_tool == EditorTool::Select
                || self.editor.current_tool == EditorTool::Erase
            {
                self.editor.hovered_cell = None;
                return;
            }
        }

        self.editor.hovered_cell = crate::editor::grid::picking::update_hover(
            self.mouse_pos.0 as f32,
            self.mouse_pos.1 as f32,
            self.renderer.width(),
            self.renderer.height(),
            &self.editor.camera,
            self.editor.anchor,
        );
    }

    fn on_click(&mut self) {
        if self.view == View::Editor && self.editor.mode == crate::engine::EditorMode::Editor {
            if let Some(hover) = self.editor.hovered_cell {
                match self.editor.current_tool {
                    EditorTool::Navigate => {
                        self.editor.set_anchor(hover);
                    }

                    EditorTool::Select => {
                        self.editor.selected_coord = Some(hover);
                        self.editor.selected_coords = vec![hover];

                        self.editor.show_properties_window = true;
                    }

                    _ => {}
                }
            }
        }
    }

    fn apply_tool_to_range(
        &mut self,
        start: crate::world::WorldCoord,
        end: crate::world::WorldCoord,
    ) {
        if self.view != View::Editor || self.editor.mode != crate::engine::EditorMode::Editor {
            return;
        }

        if self.editor.current_tool == EditorTool::Select {
            self.editor.selected_coords.clear();
            self.editor.selected_coord = None;
        } else {
            self.editor.history.push(self.world.cells.clone());
        }

        let x_min = start.x.min(end.x);

        let x_max = start.x.max(end.x);

        let y_min = start.y.min(end.y);

        let y_max = start.y.max(end.y);

        let z_min = start.z.min(end.z);

        let z_max = start.z.max(end.z);

        for x in x_min..=x_max {
            for y in y_min..=y_max {
                for z in z_min..=z_max {
                    let coord = crate::world::WorldCoord::new(x, y, z);

                    match self.editor.current_tool {
                        EditorTool::Build => {
                            let cell = self.editor.build_template.clone();

                            self.world.set_cell(coord, cell.cell_type);

                            if let Some(target) = self.world.get_mut(coord) {
                                *target = cell;
                            }
                        }

                        EditorTool::Erase => {
                            self.world.set_cell(coord, CellType::Empty);
                        }

                        EditorTool::Select => {
                            if self.world.get(coord).is_some() {
                                self.editor.selected_coords.push(coord);
                                // Primary selection is the most recentauthored cell in the range.
                                self.editor.selected_coord = Some(coord);
                            }
                        }

                        _ => {}
                    }
                }
            }
        }
    }

    pub fn update(&mut self, egui_ctx: &egui::Context) {
        let now = Instant::now();

        let frame_time = now.duration_since(self.last_frame_instant).as_secs_f32();

        self.last_frame_instant = now;

        if self.view == View::Editor {
            if self.editor.mode == EditorMode::Play && self.last_mode == EditorMode::Editor {
                let has_dirty_scripts = self.editor.script_editor.open_documents.values().any(|d| d.dirty);
                if has_dirty_scripts && !self.show_unsaved_scripts_dialog {
                    self.editor.mode = EditorMode::Editor; // Revert until confirmed
                    self.show_unsaved_scripts_dialog = true;
                    return;
                }

                self.saved_editor_camera = Some(self.editor.camera.clone());

                self.physics_world.register_from_world(&self.world);

                self.character_system.spawn_player(&self.world);

                self.start_scripting();
            }

            if self.editor.mode == EditorMode::Editor && self.last_mode == EditorMode::Play {
                if let Some(saved) = self.saved_editor_camera.take() {
                    self.editor.camera = saved;
                }

                self.physics_clock.reset();

                self.character_system.clear();

                self.physics_world.bodies.clear();

                self.stop_scripting();
            }

            self.last_mode = self.editor.mode;

            if self.editor.mode == EditorMode::Play {
                let scene_opt = &mut self.script_scene;
                let em = &mut self.entity_manager;

                if let Some(scene) = scene_opt {
                    let mut context = HostContext {
                        delta_time: frame_time as f64,
                        engine: em,
                    };
                    if let Err(e) = scene.update(frame_time, &mut context) {
                        eprintln!("Scripting error: {}", e);
                    }
                }

                let gravity = self.world.gravity;

                let p_world = &mut self.physics_world;

                let mut raw_input = glam::Vec2::ZERO;

                if self.keys_down.contains(&KeyCode::KeyW) {
                    raw_input.y += 1.0;
                }

                if self.keys_down.contains(&KeyCode::KeyS) {
                    raw_input.y -= 1.0;
                }

                if self.keys_down.contains(&KeyCode::KeyA) {
                    raw_input.x -= 1.0;
                }

                if self.keys_down.contains(&KeyCode::KeyD) {
                    raw_input.x += 1.0;
                }

                let world_move_input = if raw_input.length_squared() > 0.001 {
                    let (cam_fwd, cam_right) = self.gameplay_camera.get_horizontal_basis();

                    let world_vec = cam_right * raw_input.x + cam_fwd * raw_input.y;

                    glam::Vec2::new(world_vec.x, world_vec.z).normalize()
                } else {
                    glam::Vec2::ZERO
                };

                let character_system = &mut self.character_system;

                let world = &self.world;

                let gameplay_camera = &mut self.gameplay_camera;

                self.physics_clock.update(frame_time, |dt| {
                    p_world.apply_gravity(gravity, dt);

                    p_world.integrate_positions(dt);

                    p_world.resolve_dynamic_collisions();

                    p_world.resolve_static_collisions();

                    p_world.refresh_dynamic_support();

                    p_world.update_sleeping(gravity);

                    character_system.update(world, p_world, dt, world_move_input, self.jump_requested);

                    let _ = gameplay_camera;
                });

                if let Some(player) = self.character_system.get_active_characters().next() {
                    self.gameplay_camera.update(player, world);
                }

                self.jump_requested = false;
            } else {
                self.physics_clock.reset();
                self.character_system.clear();
                self.physics_world.bodies.clear();
            }

            if egui_ctx.wants_keyboard_input() {
                return;
            }

            let mut move_vec = glam::Vec2::ZERO;

            if self.keys_down.contains(&KeyCode::KeyW) {
                move_vec.y += 1.0;
            }

            if self.keys_down.contains(&KeyCode::KeyS) {
                move_vec.y -= 1.0;
            }

            if self.keys_down.contains(&KeyCode::KeyA) {
                move_vec.x -= 1.0;
            }

            if self.keys_down.contains(&KeyCode::KeyD) {
                move_vec.x += 1.0;
            }

            if move_vec != glam::Vec2::ZERO {
                self.editor.camera.move_target(move_vec.x, move_vec.y);

                let target = self.editor.camera.target;

                self.editor.navigation_window.x_buf = (target.x.floor() as i32).to_string();

                self.editor.navigation_window.y_buf = (target.y.floor() as i32).to_string();

                self.editor.navigation_window.z_buf = (target.z.floor() as i32).to_string();
            }

            if self.editor.needs_clear_world {
                self.editor.history.push(self.world.cells.clone());
                self.world = World::new();

                self.editor.needs_clear_world = false;

                println!("World cleared.");
            }

            if self.editor.needs_save {
                self.save_project();
                self.editor.needs_save = false;
            }

            if self.editor.needs_exit {
                self.exit_to_home();
                self.editor.needs_exit = false;
            }
        }
    }

    fn start_scripting(&mut self) {
        let Some(project_path) = &self.project_manager.current_project else {
            return;
        };

        match ScriptScene::load_from_bindings(
            project_path,
            &self.world.script_bindings,
            &mut self.entity_manager,
            0.0,
        ) {
            Ok(mut scene) => {
                let em = &mut self.entity_manager;
                let mut host = HostContext {
                    delta_time: 0.0,
                    engine: em,
                };
                if let Err(e) = scene.start(&mut host) {
                    eprintln!("Failed to start script scene: {}", e);
                } else {
                    self.script_scene = Some(scene);
                }
            }
            Err(e) => {
                eprintln!("Scripting failed to load: {}", e);
            }
        }
    }

    fn stop_scripting(&mut self) {
        let scene_opt = self.script_scene.take();
        let em = &mut self.entity_manager;
        if let Some(mut scene) = scene_opt {
            let mut host = HostContext {
                delta_time: 0.0,
                engine: em,
            };
            scene.stop(&mut host);
        }
        em.clear();
    }

    pub fn exit_to_home(&mut self) {
        self.editor.mode = crate::engine::EditorMode::Editor;

        self.save_project();

        self.world = World::new();

        self.physics_world.bodies.clear();

        self.character_system.clear();

        self.project_manager.current_project = None;

        self.editor.history.undo_stack.clear();
        self.editor.history.redo_stack.clear();

        self.view = View::Home;
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
                                if self
                                    .project_manager
                                    .create_project(&self.new_project_name)
                                    .is_some()
                                {
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
            self.editor.show_ui(ctx, &mut self.world, &self.project_manager.current_project);

            if self.show_unsaved_scripts_dialog {
                egui::Window::new("Unsaved Script Changes")
                    .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                    .collapsible(false)
                    .show(ctx, |ui| {
                        ui.label("You have unsaved changes in your scripts.");
                        ui.add_space(10.0);
                        ui.horizontal(|ui| {
                            if ui.button("Save & Play").clicked() {
                                self.editor.script_editor.save_all();
                                self.editor.mode = EditorMode::Play;
                                self.show_unsaved_scripts_dialog = false;
                            }
                            if ui.button("Don't Save").clicked() {
                                self.editor.mode = EditorMode::Play;
                                self.show_unsaved_scripts_dialog = false;
                            }
                            if ui.button("Cancel").clicked() {
                                self.show_unsaved_scripts_dialog = false;
                            }
                        });
                    });
            }
        }

        if let Some(view) = next_view {
            self.view = view;
        }
    }

    pub fn render(&self) {
        match self.view {
            View::Home => {
                self.renderer.render_home();
            }

            View::Editor => {
                self.renderer.render_editor(
                    &self.editor,
                    &self.world,
                    &self.physics_world,
                    &self.character_system,
                    &self.gameplay_camera,
                    if self.is_left_mouse_down {
                        self.drag_start_coord
                    } else {
                        None
                    },
                );
            }
        }
    }

    pub fn load_project(&mut self) {
        if let Some(project_path) = &self.project_manager.current_project {
            self.editor.script_editor.refresh_scripts(&Some(project_path.clone()));

            let world_path = project_path.join("world.dat");

            if let Err(e) = crate::world::persistence::load_world(&mut self.world, &world_path) {
                eprintln!("Failed to load world: {}", e);
            } else {
                println!("World loaded from {:?}", world_path);
            }

            let camera_path = project_path.join("camera.dat");

            if camera_path.exists() {
                if let Ok(content) = std::fs::read_to_string(&camera_path) {
                    let parts: Vec<&str> = content.split_whitespace().collect();

                    if parts.len() >= 6 {
                        self.editor.camera.yaw = parts[0].parse().unwrap_or(self.editor.camera.yaw);

                        self.editor.camera.pitch =
                            parts[1].parse().unwrap_or(self.editor.camera.pitch);

                        self.editor.camera.distance =
                            parts[2].parse().unwrap_or(self.editor.camera.distance);

                        self.editor.camera.target.x =
                            parts[3].parse().unwrap_or(self.editor.camera.target.x);

                        self.editor.camera.target.y =
                            parts[4].parse().unwrap_or(self.editor.camera.target.y);

                        self.editor.camera.target.z =
                            parts[5].parse().unwrap_or(self.editor.camera.target.z);

                        println!("Camera loaded from {:?}", camera_path);
                    }
                }
            }
        }
    }

    pub fn save_project(&mut self) {
        if let Some(project_path) = &self.project_manager.current_project {
            let world_path = project_path.join("world.dat");

            if let Err(e) = crate::world::persistence::save_world(&self.world, &world_path) {
                eprintln!("Failed to save world: {}", e);
            } else {
                println!("World saved to {:?}", world_path);
            }

            let camera_path = project_path.join("camera.dat");

            let cam = &self.editor.camera;

            let content = format!(
                "{} {} {} {} {} {}",
                cam.yaw, cam.pitch, cam.distance, cam.target.x, cam.target.y, cam.target.z,
            );

            if let Err(e) = std::fs::write(&camera_path, content) {
                eprintln!("Failed to save camera: {}", e);
            } else {
                println!("Camera saved to {:?}", camera_path);
            }
        }
    }
}
