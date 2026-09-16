use winit::event::{
    ElementState,
    MouseButton,
    MouseScrollDelta,
    WindowEvent,
};
use winit::keyboard::{
    KeyCode,
    PhysicalKey,
};
use std::time::Instant;

use crate::renderer::Renderer;
use crate::editor::{
    Editor,
    EditorTool,
};
use crate::world::{
    CellType,
    World,
};
use crate::project::ProjectManager;

use super::{
    physics,
    EditorMode,
    View,
};

/// Application state shared by the window, editor, renderer, world, project
/// system, input handling, and runtime physics.
///
/// App is the coordinator for the running application. It does not own the
/// low level rendering implementation, the world storage rules, or the editor
/// implementation itself. Those systems remain in their own modules.
///
/// The important ownership rule is that World remains the authoritative
/// authored scene. PhysicsWorld is temporary runtime state used while the
/// editor is in Play mode. The renderer consumes those systems when drawing.
///
/// The Home view is the project selection and project creation screen.
/// The Editor view contains the actual world editing environment.
pub struct App {
    /// Current high level application screen.
    ///
    /// Home displays the project interface.
    /// Editor displays the world editor.
    pub view: View,

    /// OpenGL renderer responsible for drawing the application.
    pub renderer: Renderer,

    /// Editor state including tools, camera, selection, hover state, and
    /// editor specific interface state.
    pub editor: Editor,

    /// Authoritative authored world.
    ///
    /// Editor operations modify this world directly. During Play mode the
    /// physics system works from a runtime representation of this state.
    pub world: World,

    /// Project management state including the current project and recent
    /// project information.
    pub project_manager: ProjectManager,

    /// Time recorded at the end of the previous application update.
    ///
    /// This is used to calculate frame time for the fixed physics clock.
    pub last_frame_instant: Instant,

    /// Fixed timestep accumulator used by the runtime physics simulation.
    ///
    /// Physics does not depend directly on the variable render frame rate.
    /// The clock converts variable frame time into repeated fixed simulation
    /// steps.
    pub physics_clock: physics::PhysicsClock,

    /// Temporary runtime physics state.
    ///
    /// These bodies represent moving runtime objects while Play mode is
    /// active. They are not the authoritative saved world.
    pub physics_world: physics::PhysicsWorld,

    /// Editor mode used during the previous update.
    ///
    /// This allows App to detect the transition from Editor mode into Play
    /// mode so the authored world can be registered into the physics world
    /// once at the beginning of a play session.
    pub last_mode: EditorMode,

    /// True when the New Project dialog is currently visible.
    pub show_new_project_dialog: bool,

    /// Text currently entered into the New Project name field.
    pub new_project_name: String,

    /// True when the Open Project dialog is currently visible.
    pub show_open_project_dialog: bool,

    /// True while the middle mouse button is held.
    ///
    /// This state is used by the editor camera orbit controls.
    pub is_middle_mouse_down: bool,

    /// Previous cursor position while a middle mouse camera gesture is active.
    pub last_cursor_pos: Option<(f64, f64)>,

    /// Most recently received window cursor position.
    ///
    /// This remains in window coordinates. The picker converts this position
    /// when it needs to calculate a world grid location.
    pub mouse_pos: (f64, f64),

    /// State for left mouse button drag actions (Build/Erase).
    pub is_left_mouse_down: bool,

    /// Starting grid coordinate for a drag operation.
    pub drag_start_coord: Option<crate::world::WorldCoord>,

    /// Keyboard keys currently held by the user.
    ///
    /// App records physical key state so movement can be evaluated during the
    /// regular update instead of depending on individual key press events.
    pub keys_down: std::collections::HashSet<KeyCode>,

    /// True if a jump was requested this frame.
    ///
    /// This is transient and reset after the physics update.
    pub jump_requested: bool,

    /// Management system for all active runtime characters.
    ///
    /// This handles spawning, ownership, and lifecycle of players and NPCs
    /// during Play mode.
    pub character_system: crate::character::CharacterSystem,
}

impl App {
    /// Creates the initial application state.
    ///
    /// The application starts on the Home screen. A new empty World is created
    /// here and is later replaced by loaded project data when a project is
    /// opened or created.
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

            is_middle_mouse_down: false,

            last_cursor_pos: None,

            mouse_pos: (0.0, 0.0),

            is_left_mouse_down: false,

            drag_start_coord: None,

            keys_down: std::collections::HashSet::new(),

            jump_requested: false,

            character_system: crate::character::CharacterSystem::new(),
        }
    }

    /// Receives window input events from winit.
    ///
    /// App is responsible for deciding which application systems should react
    /// to each event. egui has already received the same event before this
    /// function is called.
    ///
    /// Keyboard input is gated when egui is requesting keyboard ownership.
    /// Pointer input is currently gated using egui pointer state for normal
    /// clicks, hover, and wheel input.
    ///
    /// Middle mouse is handled separately so the editor camera can maintain
    /// its drag state across cursor movement.
    pub fn on_window_event(
        &mut self,
        event: &WindowEvent,
        egui_ctx: &egui::Context,
    ) {
        match event {
            // Window size changed.
            //
            // Renderer dimensions must be updated whenever the native window
            // changes size so subsequent rendering uses the new dimensions.
            WindowEvent::Resized(size) => {
                self.renderer.resize(
                    size.width as f32,
                    size.height as f32,
                );
            }

            // Keyboard input event.
            //
            // Physical key codes are used here so the editor movement and
            // shortcut state does not depend on keyboard text layout.
            WindowEvent::KeyboardInput {
                event: winit::event::KeyEvent {
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
                    if *state == ElementState::Pressed {
                        self.keys_down.insert(*key);

                        match key {
                            // G toggles between Home and Editor.
                            KeyCode::KeyG => {
                                self.view = if self.view == View::Home {
                                    View::Editor
                                } else {
                                    View::Home
                                };
                            }

                            // H and Escape return to Home when currently
                            // editing.
                            KeyCode::KeyH | KeyCode::Escape => {
                                if self.view == View::Editor {
                                    self.exit_to_home();
                                } else {
                                    self.view = View::Home;
                                }
                            }

                            KeyCode::Space => {
                                if self.view == View::Editor && self.editor.mode == EditorMode::Play {
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

            // Mouse button input.
            //
            // Middle mouse is tracked separately because it controls the
            // editor camera orbit gesture.
            WindowEvent::MouseInput {
                state,
                button,
                ..
            } => {
                if *button == MouseButton::Middle {
                    self.is_middle_mouse_down =
                        *state == ElementState::Pressed;

                    if !self.is_middle_mouse_down {
                        self.last_cursor_pos = None;
                    }

                    return;
                }

                if *button == MouseButton::Left {
                    if *state == ElementState::Pressed {
                        self.is_left_mouse_down = true;
                        self.drag_start_coord = self.editor.hovered_cell;

                        // Immediate actions (Navigate, Select) trigger on press.
                        if self.editor.current_tool == EditorTool::Navigate
                            || self.editor.current_tool == EditorTool::Select
                        {
                            self.on_click();
                        }
                    } else {
                        self.is_left_mouse_down = false;

                        // Range actions (Build, Erase) trigger on release.
                        if self.editor.current_tool == EditorTool::Build
                            || self.editor.current_tool == EditorTool::Erase
                        {
                            if let (Some(start), Some(end)) =
                                (self.drag_start_coord, self.editor.hovered_cell)
                            {
                                self.apply_tool_to_range(start, end);
                            }
                        }

                        self.drag_start_coord = None;
                    }

                    return;
                }

                // Normal pointer actions are ignored while egui owns the
                // pointer.
                if egui_ctx.wants_pointer_input()
                    || egui_ctx.is_using_pointer()
                {
                    return;
                }
            }

            // Cursor movement event.
            //
            // The raw window cursor position is stored first. The editor
            // hover system can then use that position when calculating the
            // current grid cell.
            WindowEvent::CursorMoved {
                position,
                ..
            } => {
                self.mouse_pos = (
                    position.x,
                    position.y,
                );

                // Hover is only active when the pointer is available to the
                // editor and the application is actually editing.
                if egui_ctx.wants_pointer_input()
                    || egui_ctx.is_using_pointer()
                    || self.editor.mode != crate::engine::EditorMode::Editor
                {
                    self.editor.hovered_cell = None;
                } else {
                    self.update_hover();
                }

                // Once middle mouse is held, mouse movement is converted
                // into camera orbit movement.
                //
                // last_cursor_pos is established on the first movement after
                // the gesture starts so that the first event does not produce
                // a large artificial camera jump.
                if self.is_middle_mouse_down {
                    if let Some((last_x, last_y)) =
                        self.last_cursor_pos
                    {
                        let dx = position.x - last_x;
                        let dy = position.y - last_y;

                        self.editor.camera.orbit(
                            dx as f32,
                            dy as f32,
                        );
                    }

                    self.last_cursor_pos = Some((
                        position.x,
                        position.y,
                    ));
                }
            }

            // Mouse wheel input.
            //
            // Wheel movement controls editor camera zoom. Pointer ownership
            // is checked before changing the camera so controls used by egui
            // do not also change the editor camera.
            WindowEvent::MouseWheel {
                delta,
                ..
            } => {
                if egui_ctx.wants_pointer_input()
                    || egui_ctx.is_using_pointer()
                {
                    return;
                }

                let y = match delta {
                    MouseScrollDelta::LineDelta(_, y) => *y,

                    MouseScrollDelta::PixelDelta(pos) => {
                        (pos.y / 100.0) as f32
                    }
                };

                self.editor.camera.zoom(y);

                // Camera movement can change which grid location lies under
                // the cursor, so hover is refreshed after zoom.
                self.update_hover();
            }

            _ => {}
        }
    }

    /// Recalculates the editor hover cell from the current mouse position.
    ///
    /// The picker receives the renderer dimensions and camera so it can turn
    /// the cursor location into a ray and determine which editor grid cell is
    /// currently under the mouse.
    ///
    /// App does not perform the picking math itself. That responsibility stays
    /// inside the editor grid picking system.
    fn update_hover(&mut self) {
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

    /// Handles immediate editor actions (Navigate, Select).
    ///
    /// These actions trigger on mouse press rather than release.
    fn on_click(&mut self) {
        if self.view == View::Editor
            && self.editor.mode == crate::engine::EditorMode::Editor
        {
            if let Some(hover) = self.editor.hovered_cell {
                match self.editor.current_tool {
                    // Navigate changes the editor anchor.
                    EditorTool::Navigate => {
                        self.editor.set_anchor(hover);
                    }

                    // Select changes the currently inspected world cell.
                    EditorTool::Select => {
                        self.editor.selected_coord = Some(hover);
                        self.editor.show_properties_window = true;
                    }

                    _ => {}
                }
            }
        }
    }

    /// Applies the current tool (Build, Erase) to a range of cells.
    ///
    /// This handles single clicks (start == end), lines (one axis drag),
    /// and rectangular planes (two axis drag).
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
                        // Build writes a new Block cell into the authored World using current settings.
                        EditorTool::Build => {
                            let mut cell = self.editor.build_template.clone();
                            self.world.set_cell(coord, cell.cell_type);

                            if let Some(target) = self.world.get_mut(coord) {
                                *target = cell;
                            }
                        }

                        // Erase replaces the selected grid location with Empty.
                        EditorTool::Erase => {
                            self.world.set_cell(coord, CellType::Empty);
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    /// Performs one variable frame update.
    ///
    /// Physics is handled through the fixed timestep clock rather than by
    /// directly using frame_time as the simulation step.
    ///
    /// Editor camera movement, world management requests, and project save
    /// requests are also processed here.
    pub fn update(
        &mut self,
        egui_ctx: &egui::Context,
    ) {
        let now = Instant::now();

        let frame_time =
            now.duration_since(
                self.last_frame_instant,
            )
            .as_secs_f32();

        self.last_frame_instant = now;

        if self.view == View::Editor {
            // Snapshot world on the transition from Editor to Play.
            if self.editor.mode == EditorMode::Play
                && self.last_mode == EditorMode::Editor
            {
                self.physics_world
                    .register_from_world(&self.world);

                self.character_system.spawn_player(&self.world);
            }

            self.last_mode = self.editor.mode;

            if self.editor.mode == EditorMode::Play {
                let gravity = self.world.gravity;

                let p_world =
                    &mut self.physics_world;

                // Capture horizontal movement input for characters.
                let mut move_input = glam::Vec2::ZERO;
                if self.keys_down.contains(&KeyCode::KeyW) { move_input.y += 1.0; }
                if self.keys_down.contains(&KeyCode::KeyS) { move_input.y -= 1.0; }
                if self.keys_down.contains(&KeyCode::KeyA) { move_input.x -= 1.0; }
                if self.keys_down.contains(&KeyCode::KeyD) { move_input.x += 1.0; }

                let character_system = &mut self.character_system;
                let world = &self.world;

                // Physics uses the fixed simulation clock so the same amount
                // of simulation time produces the same sequence of fixed
                // physics steps regardless of render frame rate.
                self.physics_clock.update(
                    frame_time,
                    |dt| {
                        // Gravity changes velocity first.
                        p_world.apply_gravity(
                            gravity,
                            dt,
                        );

                        p_world.integrate_positions(
                            dt,
                        );

                        p_world.resolve_dynamic_collisions();

                        p_world.resolve_static_collisions();

                        p_world.refresh_dynamic_support();

                        p_world.update_sleeping(
                            gravity,
                        );

                        // --- Character Update ---
                        // Characters interact with the voxel world using fixed simulation steps.
                        character_system.update(world, dt, move_input, self.jump_requested);
                    },
                );

                // Reset transient input flags after the physics simulation.
                self.jump_requested = false;
            } else {
                // Leaving Play clears accumulated simulation time and runtime state.
                self.physics_clock.reset();
                self.character_system.clear();
            }

            // Camera movement is blocked while egui is requesting keyboard
            // interaction.
            if egui_ctx.wants_keyboard_input() {
                return;
            }

            let mut move_vec =
                glam::Vec2::ZERO;

            if self.keys_down.contains(
                &KeyCode::KeyW,
            ) {
                move_vec.y += 1.0;
            }

            if self.keys_down.contains(
                &KeyCode::KeyS,
            ) {
                move_vec.y -= 1.0;
            }

            if self.keys_down.contains(
                &KeyCode::KeyA,
            ) {
                move_vec.x -= 1.0;
            }

            if self.keys_down.contains(
                &KeyCode::KeyD,
            ) {
                move_vec.x += 1.0;
            }

            if move_vec != glam::Vec2::ZERO {
                self.editor.camera.move_target(
                    move_vec.x,
                    move_vec.y,
                );

                /// Navigation coordinate fields follow the camera target so
                /// the navigation UI continues to describe the current editor
                /// focus.
                let target =
                    self.editor.camera.target;

                self.editor.navigation_window.x_buf =
                    (target.x.floor() as i32)
                        .to_string();

                self.editor.navigation_window.y_buf =
                    (target.y.floor() as i32)
                        .to_string();

                self.editor.navigation_window.z_buf =
                    (target.z.floor() as i32)
                        .to_string();
            }

            /// A clear request replaces the current authored world with a new
            /// empty world.
            if self.editor.needs_clear_world {
                self.world = World::new();
                self.editor.needs_clear_world = false;

                println!("World cleared.");
            }

            /// Save requests are processed here so editor UI code can request
            /// a save without directly performing filesystem work from the UI.
            if self.editor.needs_save {
                self.save_project();
                self.editor.needs_save = false;
            }

            /// Exit requests return the application to the Home screen.
            if self.editor.needs_exit {
                self.exit_to_home();
                self.editor.needs_exit = false;
            }
        }
    }

    /// Leaves the current editor session and returns to Home.
    ///
    /// The authored project is saved before the current World and runtime
    /// physics state are discarded.
    ///
    /// The runtime PhysicsWorld is temporary and therefore does not become the
    /// source of truth when leaving the editor.
    pub fn exit_to_home(&mut self) {
        // Reset the editor mode before unloading the editor session.
        self.editor.mode =
            crate::engine::EditorMode::Editor;

        // Save the current authored project before unloading it.
        self.save_project();

        // Discard the in memory authored world currently loaded into the App.
        self.world = World::new();

        // Discard temporary runtime physics bodies and characters.
        self.physics_world.bodies.clear();
        self.character_system.clear();

        // The project manager no longer owns an active project.
        self.project_manager.current_project =
            None;

        // Home becomes the active application view.
        self.view = View::Home;
    }

    /// Builds the egui interface for the current application view.
    ///
    /// The Home view contains the project management interface.
    /// The Editor view delegates UI construction to Editor::show_ui.
    ///
    /// UI actions request state changes during the current frame. next_view is
    /// used so the active view is changed after the current UI hierarchy has
    /// finished being built.
    pub fn update_ui(
        &mut self,
        ctx: &egui::Context,
    ) {
        let mut next_view = None;

        if self.view == View::Home {
            /// Main project window.
            ///
            /// This is the current Home screen interface. It is not the
            /// OpenGL triangle or other renderer background. Those visuals are
            /// produced by Renderer::render_home.
            egui::Window::new("Projects")
                .anchor(
                    egui::Align2::CENTER_CENTER,
                    [0.0, 0.0],
                )
                .collapsible(false)
                .resizable(false)
                .show(
                    ctx,
                    |ui| {
                        /// Starts the New Project flow.
                        if ui
                            .button("New Project")
                            .clicked()
                        {
                            self.show_new_project_dialog =
                                true;

                            self.show_open_project_dialog =
                                false;
                        }

                        /// Starts the Open Project flow.
                        if ui
                            .button("Open Project")
                            .clicked()
                        {
                            self.show_open_project_dialog =
                                true;

                            self.show_new_project_dialog =
                                false;
                        }

                        /// Recent projects are only shown when at least one
                        /// project has been recorded by the project manager.
                        if !self
                            .project_manager
                            .recent_projects
                            .is_empty()
                        {
                            ui.separator();
                            ui.label(
                                "Recent Projects",
                            );

                            let recent =
                                self.project_manager
                                    .recent_projects
                                    .clone();

                            for path in recent {
                                let name = path
                                    .file_name()
                                    .unwrap_or_default()
                                    .to_string_lossy();

                                if ui
                                    .button(
                                        format!(
                                            "{}",
                                            name
                                        ),
                                    )
                                    .clicked()
                                {
                                    if self
                                        .project_manager
                                        .open_project(
                                            path,
                                        )
                                    {
                                        self.load_project();

                                        next_view =
                                            Some(
                                                View::Editor,
                                            );
                                    } else {
                                        self.project_manager
                                            .load_recent();
                                    }
                                }
                            }
                        }
                    },
                );

            /// New Project dialog.
            if self.show_new_project_dialog {
                egui::Window::new("New Project")
                    .anchor(
                        egui::Align2::CENTER_CENTER,
                        [0.0, 100.0],
                    )
                    .collapsible(false)
                    .show(
                        ctx,
                        |ui| {
                            ui.horizontal(
                                |ui| {
                                    ui.label("Name:");

                                    ui.text_edit_singleline(
                                        &mut self
                                            .new_project_name,
                                    );
                                },
                            );

                            ui.horizontal(
                                |ui| {
                                    /// Creates the project and loads it into
                                    /// the editor when successful.
                                    if ui
                                        .button("Create")
                                        .clicked()
                                    {
                                        if self
                                            .project_manager
                                            .create_project(
                                                &self
                                                    .new_project_name,
                                            )
                                            .is_some()
                                        {
                                            self.load_project();

                                            next_view =
                                                Some(
                                                    View::Editor,
                                                );

                                            self
                                                .show_new_project_dialog =
                                                false;

                                            self
                                                .new_project_name
                                                .clear();
                                        }
                                    }

                                    /// Cancels the creation dialog without
                                    /// changing the current project.
                                    if ui
                                        .button("Cancel")
                                        .clicked()
                                    {
                                        self
                                            .show_new_project_dialog =
                                            false;
                                    }
                                },
                            );
                        },
                    );
            }

            /// Open Project dialog.
            if self.show_open_project_dialog {
                egui::Window::new("Open Project")
                    .anchor(
                        egui::Align2::CENTER_CENTER,
                        [0.0, 100.0],
                    )
                    .collapsible(false)
                    .show(
                        ctx,
                        |ui| {
                            let projects =
                                self.project_manager
                                    .list_projects();

                            if projects.is_empty() {
                                ui.label(
                                    "No projects found in UserData.",
                                );
                            } else {
                                for path in projects {
                                    let name = path
                                        .file_name()
                                        .unwrap_or_default()
                                        .to_string_lossy();

                                    if ui
                                        .button(
                                            format!(
                                                "{}",
                                                name
                                            ),
                                        )
                                        .clicked()
                                    {
                                        if self
                                            .project_manager
                                            .open_project(
                                                path,
                                            )
                                        {
                                            self.load_project();

                                            next_view =
                                                Some(
                                                    View::Editor,
                                                );

                                            self
                                                .show_open_project_dialog =
                                                false;
                                        }
                                    }
                                }
                            }

                            /// Closes the project dialog without opening
                            /// anything.
                            if ui
                                .button("Cancel")
                                .clicked()
                            {
                                self
                                    .show_open_project_dialog =
                                    false;
                            }
                        },
                    );
            }
        } else if self.view == View::Editor {
            /// Editor owns the actual editor interface. App only passes the
            /// current egui context and the authoritative World.
            self.editor.show_ui(
                ctx,
                &mut self.world,
            );
        }

        /// Apply any requested screen transition after UI generation.
        if let Some(view) = next_view {
            self.view = view;
        }
    }

    /// Draws the current application view.
    ///
    /// Home rendering is handled by Renderer::render_home.
    /// Editor rendering is handled by Renderer::render_editor.
    ///
    /// The renderer receives immutable world and physics references because
    /// rendering should represent current state rather than modify ownership
    /// of the scene or simulation.
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
                    if self.is_left_mouse_down { self.drag_start_coord } else { None },
                );
            }
        }
    }

    /// Loads the active project World and camera state from disk.
    ///
    /// World persistence owns the world file format. App only selects the file
    /// belonging to the current project and asks the persistence layer to load
    /// it.
    pub fn load_project(&mut self) {
        if let Some(project_path) =
            &self.project_manager.current_project
        {
            /// Load authored world data.
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

            /// Camera state is stored separately from the World because the
            /// camera belongs to the editor session rather than the authored
            /// scene itself.
            let camera_path =
                project_path.join("camera.dat");

            if camera_path.exists() {
                if let Ok(content) =
                    std::fs::read_to_string(
                        &camera_path,
                    )
                {
                    let parts:
                        Vec<&str> =
                        content
                            .split_whitespace()
                            .collect();

                    if parts.len() >= 6 {
                        self.editor.camera.yaw =
                            parts[0]
                                .parse()
                                .unwrap_or(
                                    self.editor
                                        .camera
                                        .yaw,
                                );

                        self.editor.camera.pitch =
                            parts[1]
                                .parse()
                                .unwrap_or(
                                    self.editor
                                        .camera
                                        .pitch,
                                );

                        self.editor.camera.distance =
                            parts[2]
                                .parse()
                                .unwrap_or(
                                    self.editor
                                        .camera
                                        .distance,
                                );

                        self.editor.camera.target.x =
                            parts[3]
                                .parse()
                                .unwrap_or(
                                    self.editor
                                        .camera
                                        .target
                                        .x,
                                );

                        self.editor.camera.target.y =
                            parts[4]
                                .parse()
                                .unwrap_or(
                                    self.editor
                                        .camera
                                        .target
                                        .y,
                                );

                        self.editor.camera.target.z =
                            parts[5]
                                .parse()
                                .unwrap_or(
                                    self.editor
                                        .camera
                                        .target
                                        .z,
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

    /// Saves the current authored project and editor camera state.
    ///
    /// Physics runtime bodies are intentionally not saved here. The authored
    /// World remains the persistent scene representation.
    pub fn save_project(&self) {
        if let Some(project_path) =
            &self.project_manager.current_project
        {
            /// Save the authoritative World.
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

            /// Save the editor camera separately from the authored World.
            let camera_path =
                project_path.join("camera.dat");

            let cam =
                &self.editor.camera;

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