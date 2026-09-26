use std::collections::HashSet;

use winit::keyboard::KeyCode;

use crate::scripting::api::HostContext;
use crate::scripting::host::ScriptHostBridge;
use crate::scripting::scene::ScriptScene;
use crate::scripting::value::{HandleKind, Value};

use super::App;
use super::app_project::get_timestamp;

impl App {
    pub(crate) fn sync_player_entity_from_character(&mut self) {
        self.character_system
            .sync_entity_positions(&mut self.entity_manager);
    }

    pub(crate) fn start_scripting(&mut self) {
        self.script_contacted_last_frame.clear();
        self.runtime_ui.clear();
        self.controller_object = None;
        self.camera_object = None;

        self.authored_disabled_scripts = self.world.disabled_scripts.clone();

        let Some(project_path) = &self.project_manager.current_project else {
            return;
        };

        let controller_name = self.world.selected_controller.clone();
        let camera_name = self.world.selected_camera.clone();

        match ScriptScene::load_from_bindings(
            project_path,
            &self.world,
            &self.world.script_bindings,
            &mut self.entity_manager,
            0.0,
        ) {
            Ok(mut scene) => {
                let mut bridge = ScriptHostBridge {
                    entity_manager: &mut self.entity_manager,
                    world: &mut self.world,
                    mouse: &mut self.mouse,
                    dynamic_properties: &mut self.script_dynamic_properties,
                    pending_events: &mut self.script_pending_events,
                    test_results: &mut self.script_test_results,
                    pending_enable_scripts: &mut self.script_pending_enable,
                    pending_disable_scripts: &mut self.script_pending_disable,
                    runtime_ui: &mut self.runtime_ui,
                    viewport_size: [self.renderer.width(), self.renderer.height()],
                    move_input: glam::Vec2::ZERO,
                    jump_requested: false,
                    orbit_delta: [0.0, 0.0],
                    character_system: Some(&mut self.character_system),
                    gameplay_camera: Some(&mut self.gameplay_camera),
                    valid_entity_declarations: Some(scene.valid_entity_declarations.clone()),
                    pending_spawns: &mut self.script_pending_spawns,
                    project_path: self.project_manager.current_project.as_deref(),
                };

                let mut context = HostContext {
                    delta_time: 0.0,
                    engine: &mut bridge,
                };

                if let Err(error) = scene.start(&mut context) {
                    self.editor.terminal_output.push_str(&format!(
                        "[{}] [ERROR] Scripting startup error: {}\n",
                        get_timestamp(),
                        error
                    ));
                } else {
                    let player_value = Value::Namespace("player".to_string());
                    let camera_value = Value::Namespace("camera".to_string());
                    let input_value = Value::Namespace("input".to_string());

                    if let Ok(object) = scene.instantiate_package_object(
                        "controllers",
                        &controller_name,
                        vec![
                            player_value.clone(),
                            camera_value.clone(),
                            input_value.clone(),
                        ],
                        &mut context,
                    ) {
                        self.controller_object = Some(object);
                    }

                    if let Ok(object) = scene.instantiate_package_object(
                        "cameras",
                        &camera_name,
                        vec![player_value],
                        &mut context,
                    ) {
                        self.camera_object = Some(object);
                    }

                    self.script_scene = Some(scene);
                }
            }
            Err(error) => {
                self.editor.terminal_output.push_str(&format!(
                    "[{}] [ERROR] Scripting load error: {}\n",
                    get_timestamp(),
                    error
                ));
            }
        }
    }

    pub(crate) fn stop_scripting(&mut self) {
        let scene_opt = self.script_scene.take();

        if let Some(mut scene) = scene_opt {
            let mut bridge = ScriptHostBridge {
                entity_manager: &mut self.entity_manager,

                world: &mut self.world,

                mouse: &mut self.mouse,

                dynamic_properties: &mut self.script_dynamic_properties,

                pending_events: &mut self.script_pending_events,

                test_results: &mut self.script_test_results,

                pending_enable_scripts: &mut self.script_pending_enable,

                pending_disable_scripts: &mut self.script_pending_disable,

                runtime_ui: &mut self.runtime_ui,

                viewport_size: [self.renderer.width(), self.renderer.height()],

                move_input: glam::Vec2::ZERO,

                jump_requested: false,

                orbit_delta: [0.0, 0.0],

                character_system: Some(&mut self.character_system),

                gameplay_camera: Some(&mut self.gameplay_camera),

                valid_entity_declarations: Some(scene.valid_entity_declarations.clone()),

                pending_spawns: &mut self.script_pending_spawns,

                project_path: self.project_manager.current_project.as_deref(),
            };

            let mut context = HostContext {
                delta_time: 0.0,
                engine: &mut bridge,
            };

            scene.stop(&mut context);
        }

        self.script_contacted_last_frame.clear();
        self.script_pending_events.clear();
        self.script_test_results.clear();
        self.script_pending_enable.clear();
        self.script_pending_disable.clear();
        self.script_pending_spawns.clear();
        self.script_dynamic_properties.clear();

        self.runtime_ui.clear();

        self.controller_object = None;
        self.camera_object = None;

        self.world.disabled_scripts = std::mem::take(&mut self.authored_disabled_scripts);

        self.entity_manager.clear();
    }

    pub(crate) fn fire_player_spawned_event(&mut self, player_entity_id: u64) {
        if let Some(scene) = &mut self.script_scene {
            let mut bridge = ScriptHostBridge {
                entity_manager: &mut self.entity_manager,

                world: &mut self.world,

                mouse: &mut self.mouse,

                viewport_size: [self.renderer.width(), self.renderer.height()],

                dynamic_properties: &mut self.script_dynamic_properties,

                pending_events: &mut self.script_pending_events,

                test_results: &mut self.script_test_results,

                pending_enable_scripts: &mut self.script_pending_enable,

                pending_disable_scripts: &mut self.script_pending_disable,

                runtime_ui: &mut self.runtime_ui,

                move_input: glam::Vec2::ZERO,

                jump_requested: false,

                orbit_delta: [0.0, 0.0],

                character_system: Some(&mut self.character_system),

                gameplay_camera: Some(&mut self.gameplay_camera),

                valid_entity_declarations: Some(scene.valid_entity_declarations.clone()),

                pending_spawns: &mut self.script_pending_spawns,

                project_path: self.project_manager.current_project.as_deref(),
            };

            let mut context = HostContext {
                delta_time: 0.0,
                engine: &mut bridge,
            };

            let arguments = vec![Value::Handle {
                kind: HandleKind::Entity,
                id: player_entity_id,
            }];

            if let Err(error) = scene.dispatch_event("PlayerSpawned", arguments, &mut context) {
                self.editor.terminal_output.push_str(&format!(
                    "[{}] [ERROR] Event PlayerSpawned error: {}\n",
                    get_timestamp(),
                    error
                ));
            }
        }
    }

    pub(crate) fn update_gameplay(&mut self, frame_time: f32) {
        /*
         * ----------------------------------------------------------
         * SCRIPT PHASE
         * ----------------------------------------------------------
         *
         * CharacterSystem is authoritative for the player's current
         * transform. Publish that transform to the script-facing
         * Player entity before scripts run.
         *
         * There is NO reverse position comparison here.
         *
         * When AeoScript explicitly calls:
         *
         *   get_entity("Player").set_position(...)
         *
         * the existing EngineHost::set_position() implementation
         * handles the deliberate teleport immediately.
         */
        self.sync_player_entity_from_character();

        if let Some(scene) = self.script_scene.as_mut() {
            let mut bridge = ScriptHostBridge {
                entity_manager: &mut self.entity_manager,

                world: &mut self.world,

                mouse: &mut self.mouse,

                dynamic_properties: &mut self.script_dynamic_properties,

                pending_events: &mut self.script_pending_events,

                test_results: &mut self.script_test_results,

                pending_enable_scripts: &mut self.script_pending_enable,

                pending_disable_scripts: &mut self.script_pending_disable,

                runtime_ui: &mut self.runtime_ui,

                viewport_size: [self.renderer.width(), self.renderer.height()],

                move_input: glam::Vec2::ZERO,

                jump_requested: false,

                orbit_delta: [0.0, 0.0],

                character_system: Some(&mut self.character_system),

                gameplay_camera: Some(&mut self.gameplay_camera),

                valid_entity_declarations: Some(scene.valid_entity_declarations.clone()),

                pending_spawns: &mut self.script_pending_spawns,

                project_path: self.project_manager.current_project.as_deref(),
            };

            let mut context = HostContext {
                delta_time: frame_time as f64,

                engine: &mut bridge,
            };

            if let Err(error) = scene.update(frame_time, &mut context) {
                self.editor.terminal_output.push_str(&format!(
                    "[{}] [ERROR] Scripting runtime error: {}\n",
                    get_timestamp(),
                    error
                ));
            }

            for record in scene.drain_output() {
                let timestamp = get_timestamp();

                let formatted = if record.script_path.is_none() && record.entity_name.is_none() {
                    let engine_context = match record.severity {
                        crate::scripting::log::LogSeverity::Error => "Engine Error",

                        crate::scripting::log::LogSeverity::Warning => "Engine Warning",

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
                    let script_part = if let Some(path) = &record.script_path {
                        format!("{} | ", path)
                    } else {
                        String::new()
                    };

                    let entity_part = if let Some(name) = &record.entity_name {
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

                    let context_part = if let Some(context_name) = &record.context_name {
                        format!("{} | ", context_name)
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

                self.editor.terminal_output.push_str(&formatted);
            }
        }

        self.physics_world.sync_with_world(&mut self.world);

        let gravity = self.world.gravity;

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

        let mut contacted_this_frame = HashSet::new();

        let scene_opt = &mut self.script_scene;

        let controller_opt = &self.controller_object;

        let camera_opt = &self.camera_object;

        let entity_manager = &mut self.entity_manager;

        let world = &mut self.world;

        let mouse = &mut self.mouse;

        let dynamic_properties = &mut self.script_dynamic_properties;

        let pending_events = &mut self.script_pending_events;

        let test_results = &mut self.script_test_results;

        let pending_enable = &mut self.script_pending_enable;

        let pending_disable = &mut self.script_pending_disable;

        let runtime_ui = &mut self.runtime_ui;

        let character_system = &mut self.character_system;

        let gameplay_camera = &mut self.gameplay_camera;

        let pending_spawns = &mut self.script_pending_spawns;

        let valid_decls = scene_opt
            .as_ref()
            .map(|s| s.valid_entity_declarations.clone());

        /*
         * Input requests are edge-triggered per render frame.
         *
         * A fixed-timestep loop may execute several simulation steps
         * during one render frame, but a single key press should not
         * become several jump requests.
         */
        let mut step_jump = self.jump_requested;

        let mut step_orbit = self.orbit_delta;

        self.orbit_delta = [0.0, 0.0];

        self.physics_clock.update(frame_time, |dt| {
            /*
             * --------------------------------------------------
             * SCRIPTED CONTROLLER
             * --------------------------------------------------
             *
             * The controller writes player movement state
             * directly through EngineHost.
             */
            if let (Some(scene), Some(controller)) = (scene_opt.as_mut(), controller_opt) {
                let mut bridge = ScriptHostBridge {
                    entity_manager,
                    world,
                    mouse,

                    dynamic_properties,
                    pending_events,
                    test_results,

                    pending_enable_scripts: pending_enable,

                    pending_disable_scripts: pending_disable,

                    runtime_ui,

                    move_input: raw_input,

                    jump_requested: step_jump,

                    viewport_size: [self.renderer.width(), self.renderer.height()],

                    orbit_delta: step_orbit,

                    character_system: Some(character_system),

                    gameplay_camera: Some(gameplay_camera),

                    valid_entity_declarations: valid_decls.clone(),

                    pending_spawns,

                    project_path: self.project_manager.current_project.as_deref(),
                };

                let mut context = HostContext {
                    delta_time: dt as f64,

                    engine: &mut bridge,
                };

                let _ = scene.call_object_method(
                    controller,
                    "update",
                    vec![Value::Number(dt as f64)],
                    &mut context,
                );
            }

            // CharacterSystem consumes the horizontal velocity supplied by the
            // scripted controller above. It owns gravity, integration, collision,
            // grounded state, and animation.

            let p_world = &mut self.physics_world;
            p_world.apply_gravity(gravity, dt);
            p_world.integrate_positions(dt);
            p_world.resolve_dynamic_collisions();
            p_world.resolve_static_collisions();
            p_world.refresh_dynamic_support();
            p_world.update_sleeping(gravity);
            p_world.rebuild_dynamic_body_index();

            let contacted = character_system.update_scripted(world, p_world, dt);

            contacted_this_frame.extend(contacted);

            // Update the scripted camera after the character has completed this
            // fixed simulation step so it follows the latest authoritative transform.
            if let (Some(scene), Some(camera)) = (scene_opt.as_mut(), camera_opt) {
                let mut bridge = ScriptHostBridge {
                    entity_manager,
                    world,
                    mouse,
                    dynamic_properties,
                    pending_events,
                    test_results,
                    pending_enable_scripts: pending_enable,
                    pending_disable_scripts: pending_disable,
                    runtime_ui,
                    move_input: raw_input,
                    jump_requested: false,
                    viewport_size: [self.renderer.width(), self.renderer.height()],
                    orbit_delta: step_orbit,
                    character_system: Some(character_system),
                    gameplay_camera: Some(gameplay_camera),
                    valid_entity_declarations: valid_decls.clone(),
                    pending_spawns,
                    project_path: self.project_manager.current_project.as_deref(),
                };

                let mut context = HostContext {
                    delta_time: dt as f64,
                    engine: &mut bridge,
                };

                let _ = scene.call_object_method(
                    camera,
                    "update",
                    vec![Value::Number(dt as f64)],
                    &mut context,
                );
            }

            // These inputs are render-frame requests, not continuous fixed-step state.
            // Consuming them here prevents one key press from becoming multiple jumps
            // or repeated camera orbit operations during catch-up steps.
            step_jump = false;
            step_orbit = [0.0, 0.0];
        });

        /*
         * ----------------------------------------------------------
         * FINAL SCRIPT-FACING PLAYER SYNC
         * ----------------------------------------------------------
         *
         * Physics is authoritative. Publish the final transform
         * after the fixed simulation has completed.
         */
        self.sync_player_entity_from_character();

        /*
         * Collision/event processing remains outside the fixed
         * movement implementation so overlap transitions are evaluated
         * once per rendered frame.
         */
        if let Some(scene) = &mut self.script_scene {
            let mut bridge = ScriptHostBridge {
                entity_manager: &mut self.entity_manager,

                world: &mut self.world,

                mouse: &mut self.mouse,

                viewport_size: [self.renderer.width(), self.renderer.height()],

                dynamic_properties: &mut self.script_dynamic_properties,

                pending_events: &mut self.script_pending_events,

                test_results: &mut self.script_test_results,

                pending_enable_scripts: &mut self.script_pending_enable,

                pending_disable_scripts: &mut self.script_pending_disable,

                runtime_ui: &mut self.runtime_ui,

                move_input: raw_input,

                jump_requested: false,

                orbit_delta: [0.0, 0.0],

                character_system: Some(&mut self.character_system),

                gameplay_camera: Some(&mut self.gameplay_camera),

                valid_entity_declarations: Some(scene.valid_entity_declarations.clone()),

                pending_spawns: &mut self.script_pending_spawns,

                project_path: self.project_manager.current_project.as_deref(),
            };

            let mut context = HostContext {
                delta_time: frame_time as f64,

                engine: &mut bridge,
            };

            for &cell_id in &contacted_this_frame {
                if !self.script_contacted_last_frame.contains(&cell_id) {
                    if let Err(error) = scene.on_player_overlap(cell_id, true, &mut context) {
                        self.editor.terminal_output.push_str(&format!(
                            "[{}] [ERROR] Overlap start error: {}\n",
                            get_timestamp(),
                            error
                        ));
                    }
                }
            }

            for &cell_id in &self.script_contacted_last_frame {
                if !contacted_this_frame.contains(&cell_id) {
                    if let Err(error) = scene.on_player_overlap(cell_id, false, &mut context) {
                        self.editor.terminal_output.push_str(&format!(
                            "[{}] [ERROR] Overlap end error: {}\n",
                            get_timestamp(),
                            error
                        ));
                    }
                }
            }

            if !contacted_this_frame.is_empty() {
                if let Err(error) = scene.on_player_contact(&contacted_this_frame, &mut context) {
                    self.editor.terminal_output.push_str(&format!(
                        "[{}] [ERROR] Scripting contact event error: {}\n",
                        get_timestamp(),
                        error
                    ));
                }
            }
        }

        self.audio_system
            .update(&self.world, &self.project_manager.current_project);

        self.script_contacted_last_frame = contacted_this_frame;

        if self.camera_object.is_none() {
            if let Some(player) = self.character_system.get_active_player() {
                self.gameplay_camera.update(player, &self.world);
            }
        }

        self.jump_requested = false;
    }
}
