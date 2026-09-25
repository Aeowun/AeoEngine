use super::Editor;
use crate::world::WorldCoord;

impl Editor {
    pub(crate) fn draw_right_panel(
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
                    ui.style_mut().visuals.window_rounding = egui::Rounding::ZERO;
                    ui.style_mut().visuals.widgets.noninteractive.rounding =
                        egui::Rounding::ZERO;
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
                                ui.output_mut(|o| {
                                    o.cursor_icon = egui::CursorIcon::ResizeVertical
                                });
                            }

                            if sep_resp.dragged() {
                                self.right_panel_split +=
                                    ui.input(|i| i.pointer.delta().y) / total_height;
                            }

                            self.right_panel_split = self.right_panel_split.clamp(0.1, 0.9);
                        }
                    }

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

    pub(crate) fn render_navigation_content(&mut self, ui: &mut egui::Ui) {
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

    pub(crate) fn render_terminal_content(&mut self, ui: &mut egui::Ui) {
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

    pub(crate) fn draw_left_panel(
        &mut self,
        ctx: &egui::Context,
        world: &mut crate::world::World,
        project_path: &Option<std::path::PathBuf>,
    ) {
        if self.show_world_window {
            egui::SidePanel::left("left_panel")
                .resizable(true)
                .default_width(260.0)
                .show(ctx, |ui| {
                    ui.style_mut().visuals.window_rounding = egui::Rounding::ZERO;
                    ui.style_mut().visuals.widgets.noninteractive.rounding =
                        egui::Rounding::ZERO;
                    ui.style_mut().visuals.widgets.inactive.rounding = egui::Rounding::ZERO;
                    ui.style_mut().visuals.widgets.hovered.rounding = egui::Rounding::ZERO;
                    ui.style_mut().visuals.widgets.active.rounding = egui::Rounding::ZERO;

                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        ui.heading("WORLD");
                        ui.with_layout(
                            egui::Layout::right_to_left(egui::Align::Center),
                            |ui| {
                                if ui.button("X").clicked() {
                                    self.show_world_window = false;
                                }
                            },
                        );
                    });
                    ui.separator();

                    self.render_world_content(ui, world, project_path);
                });
        }
    }

    fn ensure_world_hierarchy_cache(&mut self, world: &crate::world::World) {
        let revision = world.render_revision();

        if self.world_hierarchy_revision == revision {
            return;
        }

        self.world_hierarchy_blocks.clear();
        self.world_hierarchy_lights.clear();
        self.world_hierarchy_audio_emitters.clear();
        self.world_hierarchy_spawn_points.clear();
        self.world_hierarchy_fx_blocks.clear();
        self.world_hierarchy_players.clear();
        self.world_hierarchy_npcs.clear();

        for coord in world.active_blocks() {
            let Some(cell) = world.get(coord) else {
                continue;
            };

            match cell.cell_type {
                crate::world::CellType::Block => self.world_hierarchy_blocks.push(coord),
                crate::world::CellType::Light => self.world_hierarchy_lights.push(coord),
                crate::world::CellType::AudioEmitter => {
                    self.world_hierarchy_audio_emitters.push(coord)
                }
                crate::world::CellType::SpawnPoint => {
                    self.world_hierarchy_spawn_points.push(coord)
                }
                crate::world::CellType::FxBlock => self.world_hierarchy_fx_blocks.push(coord),
                crate::world::CellType::Player => self.world_hierarchy_players.push(coord),
                crate::world::CellType::NPC => self.world_hierarchy_npcs.push(coord),
                crate::world::CellType::Empty => {}
            }
        }

        let sort_fn =
            |a: &WorldCoord, b: &WorldCoord| a.x.cmp(&b.x).then(a.y.cmp(&b.y)).then(a.z.cmp(&b.z));

        self.world_hierarchy_blocks.sort_by(sort_fn);
        self.world_hierarchy_lights.sort_by(sort_fn);
        self.world_hierarchy_audio_emitters.sort_by(sort_fn);
        self.world_hierarchy_spawn_points.sort_by(sort_fn);
        self.world_hierarchy_fx_blocks.sort_by(sort_fn);
        self.world_hierarchy_players.sort_by(sort_fn);
        self.world_hierarchy_npcs.sort_by(sort_fn);

        self.world_hierarchy_revision = revision;
    }

    pub(crate) fn render_world_content(
        &mut self,
        ui: &mut egui::Ui,
        world: &mut crate::world::World,
        project_path: &Option<std::path::PathBuf>,
    ) {
        ui.label("STREAMING");

        ui.add(
            egui::DragValue::new(&mut self.editor_max_distance_chunks)
                .range(0..=64)
                .prefix("Editor max: ")
                .suffix(" chunks"),
        );

        ui.add(
            egui::DragValue::new(&mut self.play_max_distance_chunks)
                .range(0..=64)
                .prefix("Play max: ")
                .suffix(" chunks"),
        );

        ui.separator();

        self.ensure_world_hierarchy_cache(world);

        let selected_coord = self.selected_coord;
        let mut clicked_coord = None;

        egui::ScrollArea::vertical().show(ui, |ui| {
            // --- HIERARCHY ---
            //
            // Blocks are intentionally count-only. Rendering one UI row per
            // block makes the editor frame cost scale with world size.
            ui.horizontal(|ui| {
                ui.label(format!("BLOCKS ({})", self.world_hierarchy_blocks.len()));
            });

            ui.add_space(6.0);

            clicked_coord = clicked_coord.or(draw_world_category_navigator(
                ui,
                "LIGHTS",
                &self.world_hierarchy_lights,
                selected_coord,
            ));

            clicked_coord = clicked_coord.or(draw_world_category_navigator(
                ui,
                "AUDIO EMITTERS",
                &self.world_hierarchy_audio_emitters,
                selected_coord,
            ));

            clicked_coord = clicked_coord.or(draw_world_category_navigator(
                ui,
                "SPAWN POINTS",
                &self.world_hierarchy_spawn_points,
                selected_coord,
            ));

            clicked_coord = clicked_coord.or(draw_world_category_navigator(
                ui,
                "FX BLOCKS",
                &self.world_hierarchy_fx_blocks,
                selected_coord,
            ));

            clicked_coord = clicked_coord.or(draw_world_category_navigator(
                ui,
                "PLAYERS",
                &self.world_hierarchy_players,
                selected_coord,
            ));

            clicked_coord = clicked_coord.or(draw_world_category_navigator(
                ui,
                "NPCs",
                &self.world_hierarchy_npcs,
                selected_coord,
            ));

            ui.separator();
        });

        if let Some(coord) = clicked_coord {
            self.selected_coord = Some(coord);
            self.selected_coords = vec![coord];
        }

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

                    egui::ComboBox::from_id_salt("sky_preset_combo")
                        .selected_text(&current_preset)
                        .show_ui(ui, |ui| {
                            for option in &["Temperate", "Tropical", "Desert", "Snowy", "Mars"] {
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

        // --- MOUSE ---
        egui::CollapsingHeader::new("MOUSE")
            .default_open(true)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Cursor:");

                    let mut visible_selected = world.cursor_visible;
                    let mut changed = false;

                    egui::ComboBox::from_id_salt("world_cursor_visible_combo")
                        .selected_text(if visible_selected {
                            "Visible"
                        } else {
                            "Hidden"
                        })
                        .show_ui(ui, |ui| {
                            if ui
                                .selectable_value(&mut visible_selected, false, "Hidden")
                                .clicked()
                            {
                                changed = true;
                            }

                            if ui
                                .selectable_value(&mut visible_selected, true, "Visible")
                                .clicked()
                            {
                                changed = true;
                            }
                        });

                    if changed {
                        world.cursor_visible = visible_selected;
                        self.needs_save = true;
                    }
                });

                ui.horizontal(|ui| {
                    ui.label("Screen:");

                    let mut locked_selected = world.screen_locked;
                    let mut changed = false;

                    egui::ComboBox::from_id_salt("world_screen_locked_combo")
                        .selected_text(if locked_selected {
                            "Locked"
                        } else {
                            "Unlocked"
                        })
                        .show_ui(ui, |ui| {
                            if ui
                                .selectable_value(&mut locked_selected, true, "Locked")
                                .clicked()
                            {
                                changed = true;
                            }

                            if ui
                                .selectable_value(&mut locked_selected, false, "Unlocked")
                                .clicked()
                            {
                                changed = true;
                            }
                        });

                    if changed {
                        world.screen_locked = locked_selected;
                        self.needs_save = true;
                    }
                });
            });

        // --- CHARACTER ---
        egui::CollapsingHeader::new("CHARACTER")
            .default_open(true)
            .show(ui, |ui| {
                let available = if let Some(path) = project_path {
                    crate::project::discover_characters(path)
                } else {
                    vec!["custom".to_string()]
                };

                ui.horizontal(|ui| {
                    ui.label("Character:");

                    let mut current = world.selected_character.clone();
                    let mut changed = false;

                    egui::ComboBox::from_id_salt("world_character_combo")
                        .selected_text(&current)
                        .show_ui(ui, |ui| {
                            for char_name in available {
                                if ui
                                    .selectable_value(
                                        &mut current,
                                        char_name.clone(),
                                        &char_name,
                                    )
                                    .clicked()
                                {
                                    changed = true;
                                }
                            }
                        });

                    if changed {
                        world.selected_character = current;
                        self.needs_save = true;
                    }
                });

                ui.add_space(4.0);

                if ui.button("Import Character...").clicked() {
                    if let Some(folder) = rfd::FileDialog::new().pick_folder() {
                        if let Some(project_dir) = project_path {
                            let char_name = folder
                                .file_name()
                                .and_then(|n| n.to_str())
                                .unwrap_or("imported_char");

                            let dest_dir =
                                project_dir.join("characters").join(char_name);

                            if let Err(e) = copy_dir_all(&folder, &dest_dir) {
                                eprintln!("Failed to import character: {}", e);
                            } else {
                                world.selected_character = char_name.to_string();
                                self.needs_save = true;
                            }
                        }
                    }
                }
            });

        // --- CONTROLLER ---
        egui::CollapsingHeader::new("CONTROLLER")
            .default_open(true)
            .show(ui, |ui| {
                let available = if let Some(path) = project_path {
                    crate::project::discover_controllers(path)
                } else {
                    vec!["thirdPerson_Controller".to_string()]
                };

                ui.horizontal(|ui| {
                    ui.label("Controller:");

                    let mut current = world.selected_controller.clone();
                    let mut changed = false;

                    egui::ComboBox::from_id_salt("world_controller_combo")
                        .selected_text(&current)
                        .show_ui(ui, |ui| {
                            for ctrl_name in available {
                                if ui
                                    .selectable_value(
                                        &mut current,
                                        ctrl_name.clone(),
                                        &ctrl_name,
                                    )
                                    .clicked()
                                {
                                    changed = true;
                                }
                            }
                        });

                    if changed {
                        world.selected_controller = current;
                        self.needs_save = true;
                    }
                });
            });

        // --- CAMERA ---
        egui::CollapsingHeader::new("CAMERA")
            .default_open(true)
            .show(ui, |ui| {
                let available = if let Some(path) = project_path {
                    crate::project::discover_cameras(path)
                } else {
                    vec!["thirdPerson".to_string(), "firstPerson".to_string()]
                };

                ui.horizontal(|ui| {
                    ui.label("Camera:");

                    let mut current = world.selected_camera.clone();
                    let mut changed = false;

                    egui::ComboBox::from_id_salt("world_camera_combo")
                        .selected_text(&current)
                        .show_ui(ui, |ui| {
                            for cam_name in available {
                                if ui
                                    .selectable_value(
                                        &mut current,
                                        cam_name.clone(),
                                        &cam_name,
                                    )
                                    .clicked()
                                {
                                    changed = true;
                                }
                            }
                        });

                    if changed {
                        world.selected_camera = current;
                        self.needs_save = true;
                    }
                });
            });
    }
}

fn draw_world_category_navigator(
    ui: &mut egui::Ui,
    name: &str,
    list: &[WorldCoord],
    selected_coord: Option<WorldCoord>,
) -> Option<WorldCoord> {
    let mut clicked_coord = None;

    ui.horizontal(|ui| {
        ui.label(format!("{} ({})", name, list.len()));

        let has_entries = !list.is_empty();

        let current_index = if has_entries {
            selected_coord.and_then(|coord| list.iter().position(|entry| *entry == coord))
        } else {
            None
        };

        if ui
            .add_enabled(has_entries, egui::Button::new("< PREV"))
            .clicked()
        {
            let index = match current_index {
                Some(index) => {
                    if index == 0 {
                        list.len() - 1
                    } else {
                        index - 1
                    }
                }
                None => list.len() - 1,
            };

            clicked_coord = Some(list[index]);
        }

        if ui
            .add_enabled(has_entries, egui::Button::new("NEXT >"))
            .clicked()
        {
            let index = match current_index {
                Some(index) => (index + 1) % list.len(),
                None => 0,
            };

            clicked_coord = Some(list[index]);
        }
    });

    clicked_coord
}

fn copy_dir_all(
    src: impl AsRef<std::path::Path>,
    dst: impl AsRef<std::path::Path>,
) -> std::io::Result<()> {
    std::fs::create_dir_all(&dst)?;

    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;

        if ty.is_dir() {
            copy_dir_all(entry.path(), dst.as_ref().join(entry.file_name()))?;
        } else {
            std::fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
        }
    }

    Ok(())
}