use std::time::Instant;

use winit::event::{ElementState, MouseButton, MouseScrollDelta, WindowEvent};
use winit::keyboard::{KeyCode, PhysicalKey};

use crate::editor::{Editor, EditorTool};
use crate::project::ProjectManager;
use crate::renderer::Renderer;
use crate::world::{CellType, World};
use crate::scripting::scene::ScriptScene;
use crate::scripting::api::HostContext;
use crate::scripting::host::ScriptHostBridge;
use crate::engine::entity::EntityManager;
use egui::RichText;

use super::{EditorMode, View, physics};

const COLOR_VOID: egui::Color32 = egui::Color32::from_rgb(1, 1, 2);
const COLOR_VOID_ELEVATED: egui::Color32 = egui::Color32::from_rgb(6, 7, 9);
const COLOR_VOID_PANEL: egui::Color32 = egui::Color32::from_rgb(10, 12, 15);

const COLOR_BORDER: egui::Color32 =
    egui::Color32::from_rgba_premultiplied(255, 255, 255, 20);
const COLOR_BORDER_BRIGHT: egui::Color32 =
    egui::Color32::from_rgba_premultiplied(255, 255, 255, 51);

const COLOR_TEXT: egui::Color32 = egui::Color32::from_rgb(160, 164, 171);
const COLOR_TEXT_DIM: egui::Color32 = egui::Color32::from_rgb(92, 98, 108);
const COLOR_TEXT_BRIGHT: egui::Color32 =
    egui::Color32::from_rgb(255, 255, 255);

const COLOR_ACCENT_RED: egui::Color32 = egui::Color32::from_rgb(158, 52, 29);
const COLOR_ACCENT_ORANGE: egui::Color32 =
    egui::Color32::from_rgb(228, 91, 36);
const COLOR_ACCENT_LIGHT: egui::Color32 =
    egui::Color32::from_rgb(255, 173, 99);
const COLOR_ACCENT_CREAM: egui::Color32 =
    egui::Color32::from_rgb(255, 225, 194);
const COLOR_ACCENT_COOL: egui::Color32 =
    egui::Color32::from_rgb(96, 115, 125);

const RMB_GUARD_MAX_RADIUS_FACTOR: f32 = 0.45;
const RMB_GUARD_INNER_RADIUS_FACTOR: f32 = 0.38;
const RMB_GUARD_EDGE_SCROLL_SPEED: f32 = 500.0;

/// Application state shared by the window, editor, renderer, world, project
/// system, input handling, and runtime physics.
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

    pub is_right_mouse_down: bool,
    pub is_middle_mouse_down: bool,
    pub last_cursor_pos: Option<(f64, f64)>,
    pub mouse_pos: (f64, f64),

    pub rmb_edge_scroll_velocity: glam::Vec2,
    pub last_set_cursor_pos: Option<(f64, f64)>,

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
            is_right_mouse_down: false,
            is_middle_mouse_down: false,
            last_cursor_pos: None,
            mouse_pos: (0.0, 0.0),
            rmb_edge_scroll_velocity: glam::Vec2::ZERO,
            last_set_cursor_pos: None,
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

    pub fn on_window_event(
        &mut self,
        event: &WindowEvent,
        egui_ctx: &egui::Context,
        window: &winit::window::Window,
    ) {
        match event {
            WindowEvent::Resized(size) => {
                self.renderer
                    .resize(size.width as f32, size.height as f32);
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
                if let PhysicalKey::Code(key) = physical_key {
                    let was_pressed = *state == ElementState::Pressed;

                    if was_pressed {
                        self.keys_down.insert(*key);
                    } else {
                        self.keys_down.remove(key);
                    }

                    if egui_ctx.wants_keyboard_input() {
                        let ctrl = self.keys_down.contains(&KeyCode::ControlLeft)
                            || self.keys_down.contains(&KeyCode::ControlRight);

                        if !(*key == KeyCode::KeyS && ctrl) && !(*key == KeyCode::KeyG) {
                            return;
                        }
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
                        let in_viewport =
                            self.editor.viewport_rect.contains(mouse_logical);

                        if in_viewport && !egui_ctx.wants_pointer_input() {
                            self.update_hover();
                        }
                    }

                    if was_pressed {
                        let ctrl = self.keys_down.contains(&KeyCode::ControlLeft)
                            || self.keys_down.contains(&KeyCode::ControlRight);

                        match key {
                            KeyCode::F5 => {
                                if self.view == View::Editor
                                    && self.editor.mode == EditorMode::Editor
                                {
                                    self.editor.mode = EditorMode::Play;
                                }
                            }

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
                                        let shift = self.keys_down.contains(&KeyCode::ShiftLeft)
                                            || self.keys_down.contains(&KeyCode::ShiftRight);

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
                                if ctrl
                                    && self.view == View::Editor
                                    && self.editor.show_script_workspace
                                {
                                    self.editor
                                        .script_editor
                                        .show_new_script_dialog = true;
                                }
                            }

                            KeyCode::KeyZ => {
                                if ctrl && self.view == View::Editor {
                                    if let Some(prev) =
                                        self.editor.history.undo_stack.pop()
                                    {
                                        self.editor
                                            .history
                                            .redo_stack
                                            .push(self.world.cells.clone());

                                        self.world.cells = prev;
                                        self.world.rebuild_id_mapping();
                                    }
                                }
                            }

                            KeyCode::KeyY => {
                                if ctrl && self.view == View::Editor {
                                    if let Some(next) =
                                        self.editor.history.redo_stack.pop()
                                    {
                                        self.editor
                                            .history
                                            .undo_stack
                                            .push(self.world.cells.clone());

                                        self.world.cells = next;
                                        self.world.rebuild_id_mapping();
                                    }
                                }
                            }

                            KeyCode::Delete => {
                                if self.view == View::Editor
                                    && !self.editor.selected_coords.is_empty()
                                {
                                    self.editor
                                        .history
                                        .push(self.world.cells.clone());

                                    for coord in &self.editor.selected_coords {
                                        self.world.set_cell(*coord, CellType::Empty);
                                    }

                                    self.editor.selected_coords.clear();
                                    self.editor.selected_coord = None;
                                }
                            }

                            KeyCode::KeyF => {
                                if self.view == View::Editor
                                    && !self.editor.selected_coords.is_empty()
                                {
                                    let mut min =
                                        glam::Vec3::new(
                                            f32::MAX,
                                            f32::MAX,
                                            f32::MAX,
                                        );

                                    let mut max =
                                        glam::Vec3::new(
                                            f32::MIN,
                                            f32::MIN,
                                            f32::MIN,
                                        );

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

                                    self.editor.anchor =
                                        crate::world::WorldCoord::new(
                                            center.x.round() as i32,
                                            center.y.round() as i32,
                                            center.z.round() as i32,
                                        );
                                }
                            }

                            KeyCode::Digit1 => {
                                if self.view == View::Editor {
                                    self.editor.current_tool =
                                        EditorTool::Navigate;
                                }
                            }

                            KeyCode::Digit2 => {
                                if self.view == View::Editor {
                                    self.editor.current_tool =
                                        EditorTool::Select;
                                }
                            }

                            KeyCode::Digit3 => {
                                if self.view == View::Editor {
                                    self.editor.current_tool =
                                        EditorTool::Build;
                                }
                            }

                            KeyCode::Digit4 => {
                                if self.view == View::Editor {
                                    self.editor.current_tool =
                                        EditorTool::Erase;
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
                                if self.view == View::Editor
                                    && self.editor.mode == EditorMode::Play
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

                let mouse_logical = egui::pos2(
                    self.mouse_pos.0 as f32 / ppp,
                    self.mouse_pos.1 as f32 / ppp,
                );

                let in_viewport =
                    self.editor.viewport_rect.contains(mouse_logical);

                let wants_pointer = egui_ctx.wants_pointer_input();

                if *button == MouseButton::Right {
                    if *state == ElementState::Pressed {
                        if !in_viewport || wants_pointer {
                            return;
                        }

                        self.is_right_mouse_down = true;
                    } else {
                        self.is_right_mouse_down = false;
                        self.rmb_edge_scroll_velocity = glam::Vec2::ZERO;
                        self.last_cursor_pos = None;
                    }

                    return;
                }

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
            }

            WindowEvent::CursorMoved { position, .. } => {
                let dx = position.x - self.mouse_pos.0;
                let dy = position.y - self.mouse_pos.1;

                self.mouse_pos = (position.x, position.y);

                let ppp = egui_ctx.pixels_per_point();

                let mouse_logical = egui::pos2(
                    position.x as f32 / ppp,
                    position.y as f32 / ppp,
                );

                let in_viewport =
                    self.editor.viewport_rect.contains(mouse_logical);

                if in_viewport
                    && self.editor.mode == crate::engine::EditorMode::Editor
                {
                    self.update_hover();
                } else {
                    self.editor.hovered_cell = None;
                }

                if self.view == View::Editor
                    && self.editor.mode == EditorMode::Editor
                    && self.is_right_mouse_down
                {
                    // Radial mouse guard logic
                    if let Some((lx, ly)) = self.last_set_cursor_pos {
                        if (position.x - lx).abs() < 0.1
                            && (position.y - ly).abs() < 0.1
                        {
                            self.last_set_cursor_pos = None;
                            return;
                        }
                    }

                    let ppp = egui_ctx.pixels_per_point();
                    let viewport_rect = self.editor.viewport_rect;
                    let center_logical = viewport_rect.center();
                    let center_physical = egui::pos2(
                        center_logical.x * ppp,
                        center_logical.y * ppp,
                    );

                    let min_side = viewport_rect
                        .width()
                        .min(viewport_rect.height())
                        * ppp;
                    let max_radius =
                        min_side * RMB_GUARD_MAX_RADIUS_FACTOR;
                    let inner_radius =
                        min_side * RMB_GUARD_INNER_RADIUS_FACTOR;

                    let offset = egui::pos2(
                        position.x as f32 - center_physical.x,
                        position.y as f32 - center_physical.y,
                    );
                    let distance =
                        (offset.x * offset.x + offset.y * offset.y).sqrt();

                    // Apply normal mouse-look
                    self.editor.camera.look(dx as f32, dy as f32);

                    // Calculate edge-scroll velocity
                    if distance > inner_radius {
                        let strength = ((distance - inner_radius)
                            / (max_radius - inner_radius))
                            .clamp(0.0, 1.0);
                        let dir = egui::vec2(
                            offset.x / distance,
                            offset.y / distance,
                        );
                        self.rmb_edge_scroll_velocity =
                            glam::Vec2::new(dir.x, dir.y)
                                * strength
                                * RMB_GUARD_EDGE_SCROLL_SPEED;
                    } else {
                        self.rmb_edge_scroll_velocity = glam::Vec2::ZERO;
                    }

                    // Enforce MAX_RADIUS boundary
                    if distance > max_radius {
                        let clamped_offset = egui::vec2(
                            offset.x / distance * max_radius,
                            offset.y / distance * max_radius,
                        );
                        let clamped_pos = egui::pos2(
                            center_physical.x + clamped_offset.x,
                            center_physical.y + clamped_offset.y,
                        );

                        let _ = window.set_cursor_position(
                            winit::dpi::PhysicalPosition::new(
                                clamped_pos.x as f64,
                                clamped_pos.y as f64,
                            ),
                        );
                        self.last_set_cursor_pos =
                            Some((clamped_pos.x as f64, clamped_pos.y as f64));
                        self.mouse_pos =
                            (clamped_pos.x as f64, clamped_pos.y as f64);
                    }
                } else if self.is_middle_mouse_down
                    && self.view == View::Editor
                    && self.editor.mode == EditorMode::Editor
                {
                    self.editor.camera.orbit(dx as f32, dy as f32);
                } else {
                    self.rmb_edge_scroll_velocity = glam::Vec2::ZERO;
                }
            }

            WindowEvent::MouseWheel { delta, .. } => {
                let ppp = egui_ctx.pixels_per_point();

                let mouse_logical = egui::pos2(
                    self.mouse_pos.0 as f32 / ppp,
                    self.mouse_pos.1 as f32 / ppp,
                );

                let in_viewport =
                    self.editor.viewport_rect.contains(mouse_logical);

                if !in_viewport || egui_ctx.wants_pointer_input() {
                    return;
                }

                let y = match delta {
                    MouseScrollDelta::LineDelta(_, y) => *y,
                    MouseScrollDelta::PixelDelta(pos) => {
                        (pos.y / 100.0) as f32
                    }
                };

                if self.editor.mode == EditorMode::Editor {
                    self.editor.camera.zoom(y);
                    self.update_hover();
                }
            }

            _ => {}
        }
    }

    pub fn on_mouse_motion(&mut self, dx: f64, dy: f64) {
        if self.view == View::Editor
            && self.editor.mode == EditorMode::Play
        {
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
                self.editor.mode
                    == crate::engine::EditorMode::Editor,
            );

            if let Some((coord, normal)) = hit {
                if self.editor.current_tool == EditorTool::Build {
                    let shift_down =
                        self.keys_down.contains(&KeyCode::ShiftLeft)
                            || self.keys_down.contains(&KeyCode::ShiftRight);

                    if !shift_down {
                        self.editor.hovered_cell =
                            Some(crate::world::WorldCoord::new(
                                coord.x + normal.x as i32,
                                coord.y + normal.y as i32,
                                coord.z + normal.z as i32,
                            ));
                    } else {
                        self.editor.hovered_cell = Some(coord);
                    }
                } else {
                    self.editor.hovered_cell = Some(coord);
                }

                return;
            }

            if self.editor.current_tool == EditorTool::Select
                || self.editor.current_tool == EditorTool::Erase
            {
                self.editor.hovered_cell = None;
                return;
            }
        }

        self.editor.hovered_cell =
            crate::editor::grid::picking::update_hover(
                self.mouse_pos.0 as f32,
                self.mouse_pos.1 as f32,
                self.renderer.width(),
                self.renderer.height(),
                &self.editor.camera,
                self.editor.anchor,
            );
    }

    fn on_click(&mut self) {
        if self.view == View::Editor
            && self.editor.mode == crate::engine::EditorMode::Editor
        {
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
        if self.view != View::Editor
            || self.editor.mode != crate::engine::EditorMode::Editor
        {
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

                            self.world
                                .set_cell(coord, cell.cell_type);

                            if let Some(target) =
                                self.world.get_mut(coord)
                            {
                                let id = target.id;
                                *target = cell;
                                target.id = id;
                            }
                        }

                        EditorTool::Erase => {
                            self.world
                                .set_cell(coord, CellType::Empty);
                        }

                        EditorTool::Select => {
                            if self.world.get(coord).is_some() {
                                self.editor.selected_coords.push(coord);
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
        let frame_time =
            now.duration_since(self.last_frame_instant).as_secs_f32();

        self.last_frame_instant = now;

        if self.view == View::Editor {
            if self.editor.mode == EditorMode::Play
                && self.last_mode == EditorMode::Editor
            {
                let has_dirty_scripts =
                    self.editor
                        .script_editor
                        .open_documents
                        .values()
                        .any(|d| d.dirty);

                if has_dirty_scripts
                    && !self.show_unsaved_scripts_dialog
                {
                    self.editor.mode = EditorMode::Editor;
                    self.show_unsaved_scripts_dialog = true;
                    return;
                }

                self.saved_editor_camera =
                    Some(self.editor.camera.clone());

                self.physics_world.register_from_world(&self.world);

                let spawned_id =
                    self.character_system.spawn_player(&self.world);

                self.start_scripting();

                if let Some(_char_id) = spawned_id {
                    let em_id =
                        self.entity_manager.create_entity("Player");

                    if let Some(player) =
                        self.character_system
                            .get_active_characters()
                            .next()
                    {
                        self.entity_manager.set_position(
                            em_id,
                            player.transform.position,
                        );
                    }

                    self.fire_player_spawned_event(em_id.0);
                }
            }

            if self.editor.mode == EditorMode::Editor
                && self.last_mode == EditorMode::Play
            {
                if let Some(saved) =
                    self.saved_editor_camera.take()
                {
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
                let world = &mut self.world;

                if let Some(scene) = scene_opt {
                    let mut bridge = ScriptHostBridge {
                        entity_manager: em,
                        world,
                    };

                    let mut context = HostContext {
                        delta_time: frame_time as f64,
                        engine: &mut bridge,
                    };

                    if let Err(e) =
                        scene.update(frame_time, &mut context)
                    {
                        eprintln!("Scripting error: {}", e);

                        self.editor.terminal_output.push_str(
                            &format!(
                                "[{}] [ERROR] Scripting runtime error: {}\n",
                                get_timestamp(),
                                e
                            ),
                        );
                    }

                    for record in scene.drain_output() {
                        let timestamp = get_timestamp();

                        let formatted =
                            if record.script_path.is_none()
                                && record.entity_name.is_none()
                            {
                                let engine_context =
                                    match record.severity {
                                        crate::scripting::log::LogSeverity::Error => {
                                            "Engine Error"
                                        }
                                        crate::scripting::log::LogSeverity::Warning => {
                                            "Engine Warning"
                                        }
                                        _ => "Engine",
                                    };

                                format!(
                                    "[{}] [{}] {} | {}\n",
                                    timestamp,
                                    record.severity.name(),
                                    engine_context,
                                    record.message
                                )
                            } else {
                                let script_part =
                                    if let Some(path) =
                                        &record.script_path
                                    {
                                        format!("{} | ", path)
                                    } else {
                                        String::new()
                                    };

                                let entity_part =
                                    if let Some(name) =
                                        &record.entity_name
                                    {
                                        if name == "Global" {
                                            String::new()
                                        } else {
                                            format!(
                                                "Entity: {} | ID: {} | ",
                                                name,
                                                record.entity_id.unwrap_or(0)
                                            )
                                        }
                                    } else {
                                        String::new()
                                    };

                                let context_part =
                                    if let Some(ctx) =
                                        &record.context_name
                                    {
                                        format!("{} | ", ctx)
                                    } else {
                                        String::new()
                                    };

                                format!(
                                    "[{}] [{}] {}{}{}{}\n",
                                    timestamp,
                                    record.severity.name(),
                                    script_part,
                                    entity_part,
                                    context_part,
                                    record.message
                                )
                            };

                        self.editor.terminal_output.push_str(
                            &formatted,
                        );
                    }
                }

                self.physics_world.sync_with_world(world);

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

                let world_move_input =
                    if raw_input.length_squared() > 0.001 {
                        let (cam_fwd, cam_right) =
                            self.gameplay_camera
                                .get_horizontal_basis();

                        let world_vec =
                            cam_right * raw_input.x
                                + cam_fwd * raw_input.y;

                        glam::Vec2::new(
                            world_vec.x,
                            world_vec.z,
                        )
                        .normalize()
                    } else {
                        glam::Vec2::ZERO
                    };

                let character_system =
                    &mut self.character_system;

                let world = &self.world;

                self.physics_clock.update(
                    frame_time,
                    |dt| {
                        p_world.apply_gravity(gravity, dt);
                        p_world.integrate_positions(dt);
                        p_world.resolve_dynamic_collisions();
                        p_world.resolve_static_collisions();
                        p_world.refresh_dynamic_support();
                        p_world.update_sleeping(gravity);

                        character_system.update(
                            world,
                            p_world,
                            dt,
                            world_move_input,
                            self.jump_requested,
                        );
                    },
                );

                if let Some(player) =
                    self.character_system
                        .get_active_characters()
                        .next()
                {
                    self.gameplay_camera
                        .update(player, world);
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

            let mut move_vec = glam::Vec3::ZERO;

            if self.is_right_mouse_down
                && self.view == View::Editor
                && self.editor.mode == EditorMode::Editor
            {
                if self.keys_down.contains(&KeyCode::KeyW) {
                    move_vec.z += 1.0;
                }

                if self.keys_down.contains(&KeyCode::KeyS) {
                    move_vec.z -= 1.0;
                }

                if self.keys_down.contains(&KeyCode::KeyA) {
                    move_vec.x -= 1.0;
                }

                if self.keys_down.contains(&KeyCode::KeyD) {
                    move_vec.x += 1.0;
                }

                if self.keys_down.contains(&KeyCode::KeyE) {
                    move_vec.y += 1.0;
                }

                if self.keys_down.contains(&KeyCode::KeyQ) {
                    move_vec.y -= 1.0;
                }
            }

            if move_vec != glam::Vec3::ZERO {
                let speed = self.editor.camera.distance * 0.6 * frame_time;

                self.editor
                    .camera
                    .translate_free(move_vec.normalize() * speed);

                let target = self.editor.camera.target;

                self.editor.navigation_window.x_buf =
                    (target.x.floor() as i32).to_string();

                self.editor.navigation_window.y_buf =
                    (target.y.floor() as i32).to_string();

                self.editor.navigation_window.z_buf =
                    (target.z.floor() as i32).to_string();
            }

            // Apply RMB edge-scroll contribution
            if self.is_right_mouse_down
                && self.view == View::Editor
                && self.editor.mode == EditorMode::Editor
                && self.rmb_edge_scroll_velocity != glam::Vec2::ZERO
            {
                self.editor.camera.look(
                    self.rmb_edge_scroll_velocity.x * frame_time,
                    self.rmb_edge_scroll_velocity.y * frame_time,
                );
            }

            if self.editor.needs_clear_world {
                self.editor
                    .history
                    .push(self.world.cells.clone());

                self.world = World::new();
                self.editor.needs_clear_world = false;
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
        let Some(project_path) =
            &self.project_manager.current_project
        else {
            return;
        };

        match ScriptScene::load_from_bindings(
            project_path,
            &self.world,
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
                    eprintln!(
                        "Failed to start script scene: {}",
                        e
                    );

                    self.editor.terminal_output.push_str(
                        &format!(
                            "[{}] [ERROR] Scripting startup error: {}\n",
                            get_timestamp(),
                            e
                        ),
                    );
                } else {
                    self.script_scene = Some(scene);
                }
            }

            Err(e) => {
                eprintln!("Scripting failed to load: {}", e);

                self.editor.terminal_output.push_str(
                    &format!(
                        "[{}] [ERROR] Scripting load error: {}\n",
                        get_timestamp(),
                        e
                    ),
                );
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
        self.world.clear_runtime_state();
    }

    fn fire_player_spawned_event(
        &mut self,
        player_em_id: u64,
    ) {
        if let Some(scene) = &mut self.script_scene {
            let em = &mut self.entity_manager;
            let world = &mut self.world;

            let mut bridge = ScriptHostBridge {
                entity_manager: em,
                world,
            };

            let mut context = HostContext {
                delta_time: 0.0,
                engine: &mut bridge,
            };

            let args = vec![
                crate::scripting::value::Value::Handle {
                    kind: crate::scripting::value::HandleKind::Entity,
                    id: player_em_id,
                },
            ];

            if let Err(e) =
                scene.dispatch_event(
                    "PlayerSpawned",
                    args,
                    &mut context,
                )
            {
                eprintln!(
                    "Failed to dispatch PlayerSpawned: {}",
                    e
                );

                self.editor.terminal_output.push_str(
                    &format!(
                        "[{}] [ERROR] Event PlayerSpawned error: {}\n",
                        get_timestamp(),
                        e
                    ),
                );
            }
        }
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
            self.draw_home_screen(ctx, &mut next_view);
        } else if self.view == View::Editor {
            self.editor.show_ui(
                ctx,
                &mut self.world,
                &self.project_manager.current_project,
            );

            if self.show_unsaved_scripts_dialog {
                egui::Window::new("Unsaved Script Changes")
                    .anchor(
                        egui::Align2::CENTER_CENTER,
                        [0.0, 0.0],
                    )
                    .collapsible(false)
                    .show(ctx, |ui| {
                        ui.label(
                            "You have unsaved changes in your scripts.",
                        );

                        ui.add_space(10.0);

                        ui.horizontal(|ui| {
                            if ui.button("Save & Play").clicked() {
                                self.editor
                                    .script_editor
                                    .save_all();

                                self.editor.mode = EditorMode::Play;
                                self.show_unsaved_scripts_dialog =
                                    false;
                            }

                            if ui.button("Don't Save").clicked() {
                                self.editor.mode = EditorMode::Play;
                                self.show_unsaved_scripts_dialog =
                                    false;
                            }

                            if ui.button("Cancel").clicked() {
                                self.show_unsaved_scripts_dialog =
                                    false;
                            }
                        });
                    });
            }
        }

        if let Some(view) = next_view {
            self.view = view;
        }
    }

    fn apply_home_style(ctx: &egui::Context) {
        let mut visuals = egui::Visuals::dark();

        visuals.window_rounding = 4.0.into();

        visuals.widgets.noninteractive.rounding = 4.0.into();
        visuals.widgets.inactive.rounding = 4.0.into();
        visuals.widgets.hovered.rounding = 4.0.into();
        visuals.widgets.active.rounding = 4.0.into();
        visuals.widgets.open.rounding = 4.0.into();

        visuals.extreme_bg_color = COLOR_VOID;
        visuals.window_fill = COLOR_VOID_ELEVATED;
        visuals.panel_fill = COLOR_VOID;
        visuals.selection.bg_fill = COLOR_ACCENT_ORANGE;

        visuals.widgets.inactive.bg_fill = COLOR_VOID_PANEL;
        visuals.widgets.inactive.fg_stroke =
            egui::Stroke::new(1.0_f32, COLOR_TEXT);

        visuals.widgets.hovered.bg_fill =
            COLOR_VOID_ELEVATED;
        visuals.widgets.hovered.fg_stroke =
            egui::Stroke::new(1.0_f32, COLOR_ACCENT_ORANGE);

        visuals.widgets.active.bg_fill = COLOR_VOID_PANEL;
        visuals.widgets.active.fg_stroke =
            egui::Stroke::new(1.0_f32, COLOR_ACCENT_ORANGE);

        ctx.set_visuals(visuals);
    }

    fn draw_home_screen(
        &mut self,
        ctx: &egui::Context,
        next_view: &mut Option<View>,
    ) {
        Self::apply_home_style(ctx);

        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(COLOR_VOID))
            .show(ctx, |ui| {
                self.draw_home_background(ui);

                egui::ScrollArea::vertical()
                    .id_source("home_scroll")
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        let available_width =
                            ui.available_width();

                        let content_width = if available_width >= 320.0 {
                            (available_width - 48.0)
                                .min(1200.0)
                        } else {
                            available_width
                        };

                        ui.vertical_centered(|ui| {
                            ui.set_width(content_width);

                            ui.add_space(24.0);

                            self.draw_home_header(ui);

                            ui.add_space(40.0);

                            self.draw_home_hero(ui);

                            ui.add_space(40.0);
                            Self::draw_home_divider(ui);
                            ui.add_space(28.0);

                            self.draw_home_templates(ui);

                            ui.add_space(40.0);
                            Self::draw_home_divider(ui);
                            ui.add_space(28.0);

                            self.draw_home_recent(
                                ui,
                                next_view,
                            );

                            ui.add_space(40.0);
                            Self::draw_home_divider(ui);
                            ui.add_space(28.0);

                            self.draw_home_tutorials(ui);

                            ui.add_space(40.0);
                            Self::draw_home_divider(ui);
                            ui.add_space(28.0);

                            self.draw_home_explore(ui);

                            ui.add_space(32.0);

                            self.draw_home_footer(ui);

                            ui.add_space(24.0);
                        });
                    });
            });

        self.draw_home_dialogs(ctx, next_view);
    }

    fn draw_home_divider(ui: &mut egui::Ui) {
        let y = ui.cursor().top();
        let left = ui.cursor().left();
        let right = ui.cursor().right();

        ui.painter().line_segment(
            [
                egui::pos2(left, y),
                egui::pos2(right, y),
            ],
            egui::Stroke::new(
                1.0_f32,
                COLOR_BORDER,
            ),
        );

        ui.add_space(1.0);
    }

    fn draw_home_background(&self, ui: &mut egui::Ui) {
        let rect = ui.max_rect();
        let painter = ui.painter();

        let top_band = egui::Rect::from_min_max(
            rect.left_top(),
            egui::pos2(
                rect.right(),
                rect.top() + 96.0,
            ),
        );

        painter.rect_filled(
            top_band,
            0.0,
            egui::Color32::from_rgba_unmultiplied(
                158,
                52,
                29,
                3,
            ),
        );

        painter.line_segment(
            [
                egui::pos2(rect.left(), rect.top()),
                egui::pos2(rect.right(), rect.top()),
            ],
            egui::Stroke::new(
                1.0_f32,
                egui::Color32::from_rgba_unmultiplied(
                    228,
                    91,
                    36,
                    18,
                ),
            ),
        );
    }

    fn draw_home_header(&self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label(
                RichText::new("AEOENGINE")
                    .monospace()
                    .strong()
                    .color(COLOR_TEXT_BRIGHT)
                    .size(15.0),
            );

            ui.with_layout(
                egui::Layout::right_to_left(
                    egui::Align::Center,
                ),
                |ui| {
                    if self
                        .draw_home_link(ui, "GITHUB")
                        .clicked()
                    {
                        ui.ctx().open_url(
                            egui::OpenUrl::new_tab(
                                "https://github.com/Aeowun/AeoEngine",
                            ),
                        );
                    }

                    ui.add_space(20.0);

                    if self
                        .draw_home_link(ui, "DOCUMENTATION")
                        .clicked()
                    {
                        ui.ctx().open_url(
                            egui::OpenUrl::new_tab(
                                "https://www.aeowun.com/docs/",
                            ),
                        );
                    }
                },
            );
        });
    }

    fn draw_home_link(
        &self,
        ui: &mut egui::Ui,
        text: &str,
    ) -> egui::Response {
        let font_id = egui::FontId::monospace(11.0);

        let galley = ui.painter().layout_no_wrap(
            text.to_owned(),
            font_id.clone(),
            COLOR_TEXT,
        );

        let size = galley.size()
            + egui::vec2(6.0, 6.0);

        let (rect, response) =
            ui.allocate_exact_size(
                size,
                egui::Sense::click(),
            );

        let color = if response.hovered() {
            COLOR_TEXT_BRIGHT
        } else {
            COLOR_TEXT
        };

        ui.painter().text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            text,
            font_id,
            color,
        );

        if response.hovered() {
            ui.painter().line_segment(
                [
                    egui::pos2(
                        rect.left(),
                        rect.bottom() - 1.0,
                    ),
                    egui::pos2(
                        rect.right(),
                        rect.bottom() - 1.0,
                    ),
                ],
                egui::Stroke::new(
                    1.0_f32,
                    COLOR_ACCENT_ORANGE,
                ),
            );
        }

        response
    }

    fn draw_home_hero(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            ui.label(
                RichText::new("AEOENGINE")
                    .monospace()
                    .color(COLOR_TEXT_DIM)
                    .size(11.0),
            );

            ui.add_space(10.0);

            ui.label(
                RichText::new("BUILD WORLDS.")
                    .strong()
                    .color(COLOR_TEXT_BRIGHT)
                    .size(40.0),
            );

            ui.label(
                RichText::new("WRITE LOGIC.")
                    .strong()
                    .color(COLOR_TEXT_BRIGHT)
                    .size(40.0),
            );

            ui.label(
                RichText::new("SHAPE GAMES.")
                    .strong()
                    .color(COLOR_TEXT_BRIGHT)
                    .size(40.0),
            );

            ui.add_space(26.0);

            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 12.0;

                let new_button = egui::Button::new(
                    RichText::new("NEW PROJECT")
                        .monospace()
                        .strong()
                        .color(COLOR_VOID),
                )
                .min_size(egui::vec2(154.0, 34.0))
                .fill(COLOR_TEXT_BRIGHT)
                .stroke(egui::Stroke::new(
                    1.0_f32,
                    COLOR_TEXT_BRIGHT,
                ));

                if ui.add(new_button).clicked() {
                    self.show_new_project_dialog = true;
                    self.show_open_project_dialog = false;
                }

                let open_button = egui::Button::new(
                    RichText::new("OPEN PROJECT")
                        .monospace()
                        .strong()
                        .color(COLOR_TEXT_BRIGHT),
                )
                .min_size(egui::vec2(154.0, 34.0))
                .fill(COLOR_VOID_PANEL)
                .stroke(egui::Stroke::new(
                    1.0_f32,
                    COLOR_BORDER_BRIGHT,
                ));

                if ui.add(open_button).clicked() {
                    self.show_open_project_dialog = true;
                    self.show_new_project_dialog = false;
                }
            });
        });
    }

    fn draw_home_templates(&mut self, ui: &mut egui::Ui) {
        ui.label(
            RichText::new("START WITH A WORLD")
                .monospace()
                .color(COLOR_TEXT_DIM)
                .size(11.0),
        );

        ui.add_space(14.0);

        let available_width = ui.available_width();

        let columns = if available_width >= 900.0 {
            3usize
        } else if available_width >= 600.0 {
            2usize
        } else {
            1usize
        };

        let spacing = 16.0_f32;

        let card_width = (
            available_width
                - spacing * (columns as f32 - 1.0)
        ) / columns as f32;

        let templates = [
            ("BLANK", "Empty World", false),
            ("FIELD", "Open World", true),
            ("CASTLE", "Stone World", true),
            ("SPACE", "Space World", true),
        ];

        for chunk in templates.chunks(columns) {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = spacing;

                for (title, description, coming_soon) in chunk {
                    let clicked = self.draw_template_card(
                        ui,
                        title,
                        description,
                        card_width,
                        *coming_soon,
                    );

                    if clicked && !*coming_soon {
                        self.show_new_project_dialog = true;
                        self.show_open_project_dialog = false;
                    }
                }
            });

            ui.add_space(spacing);
        }
    }

    fn draw_template_card(
        &self,
        ui: &mut egui::Ui,
        title: &str,
        description: &str,
        width: f32,
        coming_soon: bool,
    ) -> bool {
        let height = 164.0_f32;

        let sense = if coming_soon {
            egui::Sense::hover()
        } else {
            egui::Sense::click()
        };

        let (rect, response) =
            ui.allocate_exact_size(
                egui::vec2(width, height),
                sense,
            );

        let hovered = response.hovered();

        let background = if hovered && !coming_soon {
            COLOR_VOID_ELEVATED
        } else {
            COLOR_VOID_PANEL
        };

        let border = if hovered {
            COLOR_BORDER_BRIGHT
        } else {
            COLOR_BORDER
        };

        ui.painter().rect(
            rect,
            4.0,
            background,
            egui::Stroke::new(
                1.0_f32,
                border,
            ),
        );

        if hovered && !coming_soon {
            ui.painter().rect_stroke(
                rect,
                4.0,
                egui::Stroke::new(
                    1.0_f32,
                    COLOR_ACCENT_ORANGE,
                ),
            );

            let glow_rect =
                egui::Rect::from_min_max(
                    egui::pos2(
                        rect.left(),
                        rect.top(),
                    ),
                    egui::pos2(
                        rect.right(),
                        rect.top() + 2.0,
                    ),
                );

            ui.painter().rect_filled(
                glow_rect,
                0.0,
                COLOR_ACCENT_ORANGE,
            );
        }

        let inner = rect.shrink(16.0);

        ui.painter().text(
            egui::pos2(
                inner.left(),
                inner.top(),
            ),
            egui::Align2::LEFT_TOP,
            title,
            egui::FontId::monospace(14.0),
            COLOR_TEXT_BRIGHT,
        );

        ui.painter().text(
            egui::pos2(
                inner.left(),
                inner.top() + 26.0,
            ),
            egui::Align2::LEFT_TOP,
            description,
            egui::FontId::proportional(12.0),
            COLOR_TEXT,
        );

        let start_rect = egui::Rect::from_min_size(
            egui::pos2(
                inner.left(),
                inner.bottom() - 28.0,
            ),
            egui::vec2(
                inner.width(),
                26.0,
            ),
        );

        let start_fill = if coming_soon {
            COLOR_VOID_ELEVATED
        } else {
            COLOR_VOID
        };

        let start_border = if hovered && !coming_soon {
            COLOR_ACCENT_ORANGE
        } else {
            COLOR_BORDER_BRIGHT
        };

        ui.painter().rect(
            start_rect,
            3.0,
            start_fill,
            egui::Stroke::new(
                1.0_f32,
                start_border,
            ),
        );

        ui.painter().text(
            start_rect.center(),
            egui::Align2::CENTER_CENTER,
            "START",
            egui::FontId::monospace(10.0),
            if coming_soon {
                COLOR_TEXT_DIM
            } else {
                COLOR_TEXT_BRIGHT
            },
        );

        if coming_soon && hovered {
            ui.painter().rect_filled(
                rect,
                4.0,
                egui::Color32::from_rgba_unmultiplied(
                    1,
                    1,
                    2,
                    218,
                ),
            );

            ui.painter().text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                "COMING SOON",
                egui::FontId::monospace(11.0),
                COLOR_TEXT_BRIGHT,
            );
        }

        response.clicked()
    }

    fn draw_home_recent(
        &mut self,
        ui: &mut egui::Ui,
        next_view: &mut Option<View>,
    ) {
        ui.label(
            RichText::new("RECENT PROJECTS")
                .monospace()
                .color(COLOR_TEXT_DIM)
                .size(11.0),
        );

        ui.add_space(14.0);

        if self.project_manager.recent_projects.is_empty() {
            ui.label(
                RichText::new("No recent projects.")
                    .color(COLOR_TEXT)
                    .size(12.0),
            );

            ui.add_space(4.0);

            ui.label(
                RichText::new(
                    "Create a new project or open an existing project.",
                )
                .color(COLOR_TEXT_DIM)
                .size(11.0),
            );

            return;
        }

        let recent =
            self.project_manager.recent_projects.clone();

        for path in recent {
            let name = path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy();

            let path_text = path.to_string_lossy();

            let row_height = 58.0_f32;

            let (rect, response) =
                ui.allocate_exact_size(
                    egui::vec2(
                        ui.available_width(),
                        row_height,
                    ),
                    egui::Sense::click(),
                );

            let hovered = response.hovered();

            ui.painter().rect(
                rect,
                4.0,
                if hovered {
                    COLOR_VOID_ELEVATED
                } else {
                    egui::Color32::TRANSPARENT
                },
                egui::Stroke::new(
                    1.0_f32,
                    if hovered {
                        COLOR_BORDER_BRIGHT
                    } else {
                        COLOR_BORDER
                    },
                ),
            );

            if hovered {
                ui.painter().line_segment(
                    [
                        egui::pos2(
                            rect.left(),
                            rect.top(),
                        ),
                        egui::pos2(
                            rect.left(),
                            rect.bottom(),
                        ),
                    ],
                    egui::Stroke::new(
                        2.0_f32,
                        COLOR_ACCENT_ORANGE,
                    ),
                );
            }

            let inner =
                rect.shrink2(egui::vec2(16.0, 8.0));

            ui.painter().text(
                egui::pos2(
                    inner.left(),
                    inner.top(),
                ),
                egui::Align2::LEFT_TOP,
                name.as_ref(),
                egui::FontId::proportional(13.0),
                COLOR_TEXT_BRIGHT,
            );

            ui.painter().text(
                egui::pos2(
                    inner.left(),
                    inner.top() + 22.0,
                ),
                egui::Align2::LEFT_TOP,
                path_text.as_ref(),
                egui::FontId::monospace(10.0),
                COLOR_TEXT_DIM,
            );

            ui.painter().text(
                egui::pos2(
                    inner.right(),
                    rect.center().y,
                ),
                egui::Align2::RIGHT_CENTER,
                ">",
                egui::FontId::proportional(16.0),
                if hovered {
                    COLOR_ACCENT_ORANGE
                } else {
                    COLOR_TEXT_DIM
                },
            );

            if response.clicked() {
                if self.project_manager.open_project(path) {
                    self.load_project();
                    *next_view = Some(View::Editor);
                } else {
                    self.project_manager.load_recent();
                }
            }

            ui.add_space(8.0);
        }
    }

    fn draw_home_tutorials(&self, ui: &mut egui::Ui) {
        ui.label(
            RichText::new("LEARN AEOENGINE")
                .monospace()
                .color(COLOR_TEXT_DIM)
                .size(11.0),
        );

        ui.add_space(14.0);

        let available_width = ui.available_width();

        let columns = if available_width >= 760.0 {
            3usize
        } else if available_width >= 500.0 {
            2usize
        } else {
            1usize
        };

        let spacing = 24.0_f32;

        let item_width = (
            available_width
                - spacing * (columns as f32 - 1.0)
        ) / columns as f32;

        let tutorials = [
            (
                "FIRST WORLD",
                "Build a world.",
                "https://www.aeowun.com/docs/tutorials/first-world/",
            ),
            (
                "FIRST SCRIPT",
                "Bind a script.",
                "https://www.aeowun.com/docs/tutorials/first-script/",
            ),
            (
                "FLASH BEHAVIOR",
                "Make it behave.",
                "https://www.aeowun.com/docs/tutorials/flash-behavior/",
            ),
        ];

        for chunk in tutorials.chunks(columns) {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = spacing;

                for (title, description, url) in chunk {
                    self.draw_tutorial_link(
                        ui,
                        item_width,
                        title,
                        description,
                        url,
                    );
                }
            });

            ui.add_space(12.0);
        }

        ui.add_space(8.0);

        if self
            .draw_home_utility_button(
                ui,
                "VIEW ALL TUTORIALS",
            )
            .clicked()
        {
            ui.ctx().open_url(
                egui::OpenUrl::new_tab(
                    "https://www.aeowun.com/docs/tutorials/",
                ),
            );
        }
    }

    fn draw_tutorial_link(
        &self,
        ui: &mut egui::Ui,
        width: f32,
        title: &str,
        description: &str,
        url: &str,
    ) {
        let (rect, response) =
            ui.allocate_exact_size(
                egui::vec2(width, 54.0),
                egui::Sense::click(),
            );

        let hovered = response.hovered();

        ui.painter().text(
            egui::pos2(
                rect.left(),
                rect.top(),
            ),
            egui::Align2::LEFT_TOP,
            title,
            egui::FontId::monospace(11.0),
            if hovered {
                COLOR_TEXT_BRIGHT
            } else {
                COLOR_TEXT
            },
        );

        ui.painter().text(
            egui::pos2(
                rect.left(),
                rect.top() + 23.0,
            ),
            egui::Align2::LEFT_TOP,
            description,
            egui::FontId::proportional(11.0),
            COLOR_TEXT_DIM,
        );

        if hovered {
            ui.painter().line_segment(
                [
                    egui::pos2(
                        rect.left(),
                        rect.bottom() - 2.0,
                    ),
                    egui::pos2(
                        (rect.left() + 64.0)
                            .min(rect.right()),
                        rect.bottom() - 2.0,
                    ),
                ],
                egui::Stroke::new(
                    1.0_f32,
                    COLOR_ACCENT_ORANGE,
                ),
            );
        }

        if response.clicked() {
            ui.ctx().open_url(
                egui::OpenUrl::new_tab(url),
            );
        }
    }

    fn draw_home_explore(&self, ui: &mut egui::Ui) {
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing.x = 28.0;
            ui.spacing_mut().item_spacing.y = 8.0;

            let links = [
                (
                    "AEOENGINE",
                    "https://www.aeowun.com/aeoengine/",
                ),
                (
                    "AEOSCRIPT",
                    "https://www.aeowun.com/aeoscript/",
                ),
                (
                    "DOCS",
                    "https://www.aeowun.com/docs/",
                ),
                (
                    "SOURCE",
                    "https://github.com/Aeowun/AeoEngine",
                ),
            ];

            for (label, url) in links {
                if self
                    .draw_home_link(ui, label)
                    .clicked()
                {
                    ui.ctx().open_url(
                        egui::OpenUrl::new_tab(url),
                    );
                }
            }
        });
    }

    fn draw_home_utility_button(
        &self,
        ui: &mut egui::Ui,
        label: &str,
    ) -> egui::Response {
        let button = egui::Button::new(
            RichText::new(label)
                .monospace()
                .color(COLOR_TEXT),
        )
        .min_size(egui::vec2(0.0, 28.0))
        .fill(COLOR_VOID)
        .stroke(egui::Stroke::new(
            1.0_f32,
            COLOR_BORDER,
        ));

        ui.add(button)
    }

    fn draw_home_footer(&self, ui: &mut egui::Ui) {
        ui.label(
            RichText::new(format!(
                "AeoEngine {}",
                env!("CARGO_PKG_VERSION")
            ))
            .color(COLOR_TEXT_DIM)
            .monospace()
            .size(10.0),
        );
    }

    fn draw_home_dialogs(
        &mut self,
        ctx: &egui::Context,
        next_view: &mut Option<View>,
    ) {
        if self.show_new_project_dialog {
            egui::Window::new("New Project")
                .anchor(
                    egui::Align2::CENTER_CENTER,
                    [0.0, 0.0],
                )
                .collapsible(false)
                .resizable(false)
                .frame(
                    egui::Frame::window(&ctx.style())
                        .fill(COLOR_VOID_ELEVATED)
                        .stroke(
                            egui::Stroke::new(
                                1.0_f32,
                                COLOR_BORDER,
                            ),
                        )
                )
                .show(ctx, |ui| {
                    ui.add_space(8.0);

                    ui.label(
                        RichText::new("PROJECT NAME")
                            .monospace()
                            .color(COLOR_TEXT_DIM)
                            .size(10.0),
                    );

                    ui.add_space(6.0);

                    ui.add_sized(
                        egui::vec2(320.0, 28.0),
                        egui::TextEdit::singleline(
                            &mut self.new_project_name,
                        ),
                    );

                    ui.add_space(16.0);

                    ui.horizontal(|ui| {
                        if ui.add(
                            egui::Button::new(
                                RichText::new("CREATE PROJECT")
                                    .monospace()
                                    .strong()
                                    .color(COLOR_VOID),
                            )
                            .min_size(
                                egui::vec2(
                                    140.0,
                                    30.0,
                                ),
                            )
                            .fill(COLOR_TEXT_BRIGHT),
                        ).clicked()
                        {
                            if self
                                .project_manager
                                .create_project(
                                    &self.new_project_name,
                                )
                                .is_some()
                            {
                                self.load_project();
                                *next_view =
                                    Some(View::Editor);

                                self.show_new_project_dialog =
                                    false;

                                self.new_project_name.clear();
                            }
                        }

                        if ui.add(
                            egui::Button::new(
                                RichText::new("CANCEL")
                                    .monospace()
                                    .color(COLOR_TEXT),
                            )
                            .min_size(
                                egui::vec2(
                                    90.0,
                                    30.0,
                                ),
                            )
                            .fill(COLOR_VOID_PANEL)
                            .stroke(
                                egui::Stroke::new(
                                    1.0_f32,
                                    COLOR_BORDER_BRIGHT,
                                ),
                            ),
                        ).clicked()
                        {
                            self.show_new_project_dialog =
                                false;
                        }
                    });

                    ui.add_space(8.0);
                });
        }

        if self.show_open_project_dialog {
            egui::Window::new("Open Project")
                .anchor(
                    egui::Align2::CENTER_CENTER,
                    [0.0, 0.0],
                )
                .collapsible(false)
                .resizable(false)
                .frame(
                    egui::Frame::window(&ctx.style())
                        .fill(COLOR_VOID_ELEVATED)
                        .stroke(
                            egui::Stroke::new(
                                1.0_f32,
                                COLOR_BORDER,
                            ),
                        )
                )
                .show(ctx, |ui| {
                    ui.add_space(8.0);

                    let projects =
                        self.project_manager.list_projects();

                    if projects.is_empty() {
                        ui.label(
                            RichText::new(
                                "No projects found in UserData.",
                            )
                            .color(COLOR_TEXT_DIM)
                            .size(12.0),
                        );
                    } else {
                        egui::ScrollArea::vertical()
                            .max_height(300.0)
                            .show(ui, |ui| {
                                for path in projects {
                                    let name = path
                                        .file_name()
                                        .unwrap_or_default()
                                        .to_string_lossy();

                                    if ui.add(
                                        egui::Button::new(
                                            RichText::new(
                                                name.to_string(),
                                            )
                                            .strong()
                                            .color(
                                                COLOR_TEXT_BRIGHT,
                                            ),
                                        )
                                        .min_size(
                                            egui::vec2(
                                                320.0,
                                                32.0,
                                            ),
                                        )
                                        .fill(COLOR_VOID_PANEL)
                                        .stroke(
                                            egui::Stroke::new(
                                                1.0_f32,
                                                COLOR_BORDER,
                                            ),
                                        ),
                                    ).clicked()
                                    {
                                        if self
                                            .project_manager
                                            .open_project(path)
                                        {
                                            self.load_project();

                                            *next_view =
                                                Some(View::Editor);

                                            self.show_open_project_dialog =
                                                false;
                                        }
                                    }

                                    ui.add_space(6.0);
                                }
                            });
                    }

                    ui.add_space(16.0);

                    if ui.add(
                        egui::Button::new(
                            RichText::new("CANCEL")
                                .monospace()
                                .color(COLOR_TEXT),
                        )
                        .min_size(
                            egui::vec2(
                                90.0,
                                30.0,
                            ),
                        )
                        .fill(COLOR_VOID_PANEL)
                        .stroke(
                            egui::Stroke::new(
                                1.0_f32,
                                COLOR_BORDER_BRIGHT,
                            ),
                        ),
                    ).clicked()
                    {
                        self.show_open_project_dialog = false;
                    }

                    ui.add_space(8.0);
                });
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
        if let Some(project_path) =
            &self.project_manager.current_project
        {
            self.editor
                .script_editor
                .refresh_scripts(
                    &Some(project_path.clone()),
                );

            let world_path =
                project_path.join("world.dat");

            if let Err(e) =
                crate::world::persistence::load_world(
                    &mut self.world,
                    &world_path,
                )
            {
                eprintln!(
                    "Failed to load world: {}",
                    e
                );
            } else {
                println!(
                    "World loaded from {:?}",
                    world_path
                );
            }

            let camera_path =
                project_path.join("camera.dat");

            if camera_path.exists() {
                if let Ok(content) =
                    std::fs::read_to_string(&camera_path)
                {
                    let parts: Vec<&str> =
                        content.split_whitespace().collect();

                    if parts.len() >= 6 {
                        self.editor.camera.yaw =
                            parts[0]
                                .parse()
                                .unwrap_or(
                                    self.editor.camera.yaw,
                                );

                        self.editor.camera.pitch =
                            parts[1]
                                .parse()
                                .unwrap_or(
                                    self.editor.camera.pitch,
                                );

                        self.editor.camera.distance =
                            parts[2]
                                .parse()
                                .unwrap_or(
                                    self.editor.camera.distance,
                                );

                        self.editor.camera.target.x =
                            parts[3]
                                .parse()
                                .unwrap_or(
                                    self.editor.camera.target.x,
                                );

                        self.editor.camera.target.y =
                            parts[4]
                                .parse()
                                .unwrap_or(
                                    self.editor.camera.target.y,
                                );

                        self.editor.camera.target.z =
                            parts[5]
                                .parse()
                                .unwrap_or(
                                    self.editor.camera.target.z,
                                );

                        println!(
                            "Camera loaded from {:?}",
                            camera_path
                        );
                    }
                }
            }
        }
    }

    pub fn save_project(&mut self) {
        if let Some(project_path) =
            &self.project_manager.current_project
        {
            let world_path =
                project_path.join("world.dat");

            if let Err(e) =
                crate::world::persistence::save_world(
                    &self.world,
                    &world_path,
                )
            {
                eprintln!(
                    "Failed to save world: {}",
                    e
                );
            } else {
                println!(
                    "World saved to {:?}",
                    world_path
                );
            }

            let camera_path =
                project_path.join("camera.dat");

            let cam = &self.editor.camera;

            let content = format!(
                "{} {} {} {} {} {}",
                cam.yaw,
                cam.pitch,
                cam.distance,
                cam.target.x,
                cam.target.y,
                cam.target.z,
            );

            if let Err(e) =
                std::fs::write(
                    &camera_path,
                    content,
                )
            {
                eprintln!(
                    "Failed to save camera: {}",
                    e
                );
            } else {
                println!(
                    "Camera saved to {:?}",
                    camera_path
                );
            }
        }
    }
}

fn get_timestamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();

    let secs = now.as_secs();

    let hours = (secs / 3600) % 24;
    let mins = (secs / 60) % 60;
    let secs = secs % 60;

    format!("{:02}:{:02}:{:02}", hours, mins, secs)
}