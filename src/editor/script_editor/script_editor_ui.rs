use super::ScriptEditor;
use super::script_editor_highlighting::{build_layout_job, byte_offset_to_char_index};
use egui::{Color32, RichText};
use std::path::PathBuf;

pub struct SearchResult {
    pub path: PathBuf,
    pub line: usize,
    pub text: String,
}

impl ScriptEditor {
    pub fn show_ui(
        &mut self,
        ctx: &egui::Context,
        project_path: &Option<PathBuf>,
        world: &mut crate::world::World,
    ) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.style_mut().visuals.window_rounding = egui::Rounding::ZERO;

            self.draw_workspace(ui, project_path, world);
        });

        self.draw_new_script_dialog(ctx, project_path);
        self.draw_rename_dialog(ctx, project_path);
        self.draw_delete_dialog(ctx, project_path);
        self.draw_dirty_dialog(ctx, project_path);
    }

    fn draw_new_script_dialog(&mut self, ctx: &egui::Context, project_path: &Option<PathBuf>) {
        if !self.show_new_script_dialog {
            return;
        }

        egui::Window::new("New Script")
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .collapsible(false)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Name:");
                    ui.text_edit_singleline(&mut self.new_script_name);
                });

                ui.horizontal(|ui| {
                    let create = ui.button("Create").clicked()
                        || ui.input(|input| input.key_pressed(egui::Key::Enter));

                    if create {
                        if let Some(root) = project_path {
                            self.create_new_script(root, &self.new_script_name.clone());
                            self.show_new_script_dialog = false;
                            self.new_script_name.clear();
                        }
                    }

                    if ui.button("Cancel").clicked()
                        || ui.input(|input| input.key_pressed(egui::Key::Escape))
                    {
                        self.show_new_script_dialog = false;
                        self.new_script_name.clear();
                    }
                });
            });
    }

    fn draw_rename_dialog(&mut self, ctx: &egui::Context, project_path: &Option<PathBuf>) {
        if !self.show_rename_dialog {
            return;
        }

        egui::Window::new("Rename Script")
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .collapsible(false)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label("New Name:");
                    ui.text_edit_singleline(&mut self.rename_new_name);
                });

                ui.horizontal(|ui| {
                    let rename = ui.button("Rename").clicked()
                        || ui.input(|input| input.key_pressed(egui::Key::Enter));

                    if rename {
                        if let (Some(root), Some(target)) =
                            (project_path, self.rename_target.clone())
                        {
                            self.rename_script(root, target, &self.rename_new_name.clone());

                            self.show_rename_dialog = false;
                            self.rename_new_name.clear();
                            self.rename_target = None;
                        }
                    }

                    if ui.button("Cancel").clicked()
                        || ui.input(|input| input.key_pressed(egui::Key::Escape))
                    {
                        self.show_rename_dialog = false;
                        self.rename_new_name.clear();
                        self.rename_target = None;
                    }
                });
            });
    }

    fn draw_delete_dialog(&mut self, ctx: &egui::Context, project_path: &Option<PathBuf>) {
        let Some(path) = self.deleting_path.clone() else {
            return;
        };

        if self.closing_path.as_ref() == Some(&path) {
            return;
        }

        egui::Window::new("Confirm Delete")
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .collapsible(false)
            .show(ctx, |ui| {
                ui.label(format!(
                    "Are you sure you want to delete {:?}?",
                    path.file_name().unwrap_or_default()
                ));

                ui.add_space(10.0);

                ui.horizontal(|ui| {
                    let delete = ui.button("Delete").clicked()
                        || ui.input(|input| input.key_pressed(egui::Key::Enter));

                    if delete {
                        let dirty = self
                            .open_documents
                            .get(&path)
                            .is_some_and(|document| document.dirty);

                        if dirty {
                            self.closing_path = Some(path.clone());
                        } else {
                            self.delete_script(project_path, path.clone());
                            self.deleting_path = None;
                        }
                    }

                    if ui.button("Cancel").clicked()
                        || ui.input(|input| input.key_pressed(egui::Key::Escape))
                    {
                        self.deleting_path = None;
                    }
                });
            });
    }

    fn draw_dirty_dialog(&mut self, ctx: &egui::Context, project_path: &Option<PathBuf>) {
        let Some(path) = self.closing_path.clone() else {
            return;
        };

        let is_deletion = self.deleting_path.as_ref() == Some(&path);

        egui::Window::new(if is_deletion {
            "Delete: Unsaved Changes"
        } else {
            "Unsaved Changes"
        })
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .collapsible(false)
        .show(ctx, |ui| {
            ui.label(format!(
                "Save changes to {} before {}?",
                path.file_name().unwrap_or_default().to_string_lossy(),
                if is_deletion { "deleting" } else { "closing" }
            ));

            ui.add_space(10.0);

            ui.horizontal(|ui| {
                if ui.button("Save").clicked() {
                    self.apply_dirty_response(project_path, path.clone(), true, false);
                }

                if ui.button("Don't Save").clicked() {
                    self.apply_dirty_response(project_path, path.clone(), false, false);
                }

                if ui.button("Cancel").clicked()
                    || ui.input(|input| input.key_pressed(egui::Key::Escape))
                {
                    self.apply_dirty_response(project_path, path.clone(), false, true);
                }
            });
        });
    }

    fn draw_workspace(
        &mut self,
        ui: &mut egui::Ui,
        project_path: &Option<PathBuf>,
        world: &mut crate::world::World,
    ) {
        let mut action_open = None;
        let mut action_close = None;

        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.heading(RichText::new("SCRIPT EDITOR").strong());
                ui.separator();

                if ui
                    .button("New Script")
                    .on_hover_text("Create a new .aeo script")
                    .clicked()
                {
                    self.show_new_script_dialog = true;
                }

                if ui
                    .button("Save")
                    .on_hover_text("Save the active document (Ctrl+S)")
                    .clicked()
                {
                    self.save_active();
                }

                if ui
                    .button("Save All")
                    .on_hover_text("Save all open documents (Ctrl+Shift+S)")
                    .clicked()
                {
                    self.save_all();
                }

                if ui
                    .button("Refresh")
                    .on_hover_text("Refresh the script list from disk")
                    .clicked()
                {
                    self.refresh_scripts(project_path);
                }

                ui.separator();

                ui.toggle_value(&mut self.show_search, "Search");

                if self.show_search {
                    ui.label("Find:");

                    let response = ui.add(
                        egui::TextEdit::singleline(&mut self.search_query)
                            .hint_text("Search in all scripts...")
                            .desired_width(180.0),
                    );

                    if response.changed() {
                        self.perform_search();
                    }
                }
            });

            ui.separator();

            let has_problems = self
                .active_document
                .as_ref()
                .and_then(|path| self.open_documents.get(path))
                .is_some_and(|document| !document.diagnostics.is_empty());

            let bottom_panel_min_height = 28.0;
            let bottom_panel_expanded_height = 150.0;

            let bottom_panel_height = if (self.show_problems && has_problems)
                || (self.show_output && !self.output_log.is_empty())
            {
                bottom_panel_expanded_height
            } else {
                bottom_panel_min_height
            };

            let main_height = (ui.available_height() - bottom_panel_height - 10.0).max(100.0);

            ui.horizontal(|ui| {
                let explorer_width = 230.0;

                ui.allocate_ui(egui::vec2(explorer_width, main_height), |ui| {
                    egui::ScrollArea::vertical()
                        .id_salt("explorer_scroll")
                        .show(ui, |ui| {
                            if self.show_search && !self.search_results.is_empty() {
                                ui.label(RichText::new("SEARCH RESULTS").strong());
                                ui.separator();

                                for result in &self.search_results {
                                    let filename = result
                                        .path
                                        .file_name()
                                        .unwrap_or_default()
                                        .to_string_lossy();

                                    let label = format!(
                                        "{}:{} {}",
                                        filename,
                                        result.line,
                                        result.text.trim()
                                    );

                                    if ui.selectable_label(false, label).clicked() {
                                        action_open = Some(result.path.clone());
                                    }
                                }

                                ui.separator();
                            }

                            ui.label(RichText::new("SCRIPTS").strong());
                            ui.separator();

                            egui::CollapsingHeader::new("scripts/")
                                .default_open(true)
                                .show(ui, |ui| {
                                    for path in &self.scripts_list {
                                        let filename =
                                            path.file_name().unwrap_or_default().to_string_lossy();

                                        let active = self.active_document.as_ref() == Some(path);

                                        let dirty = self
                                            .open_documents
                                            .get(path)
                                            .is_some_and(|document| document.dirty);

                                        let label = if dirty {
                                            format!("{} *", filename)
                                        } else {
                                            filename.to_string()
                                        };

                                        ui.horizontal(|ui| {
                                            ui.with_layout(
                                                egui::Layout::right_to_left(egui::Align::Center),
                                                |ui| {
                                                    if ui
                                                        .small_button("")
                                                        .on_hover_text("Delete script from disk")
                                                        .clicked()
                                                    {
                                                        self.deleting_path = Some(path.clone());
                                                    }

                                                    if ui
                                                        .small_button("📝")
                                                        .on_hover_text("Rename script")
                                                        .clicked()
                                                    {
                                                        self.show_rename_dialog = true;
                                                        self.rename_target = Some(path.clone());
                                                        self.rename_new_name = path
                                                            .file_stem()
                                                            .unwrap_or_default()
                                                            .to_string_lossy()
                                                            .to_string();
                                                    }

                                                    if self.open_documents.contains_key(path)
                                                        && ui
                                                            .small_button("❌")
                                                            .on_hover_text("Close document")
                                                            .clicked()
                                                    {
                                                        action_close = Some(path.clone());
                                                    }

                                                    ui.with_layout(
                                                        egui::Layout::left_to_right(
                                                            egui::Align::Center,
                                                        ),
                                                        |ui| {
                                                            ui.style_mut().wrap_mode =
                                                                Some(egui::TextWrapMode::Truncate);

                                                            if ui
                                                                .selectable_label(active, label)
                                                                .clicked()
                                                            {
                                                                action_open = Some(path.clone());
                                                            }
                                                        },
                                                    );
                                                },
                                            );
                                        });
                                    }
                                });
                        });
                });

                ui.separator();

                let inspector_width = 240.0;
                let editor_width = (ui.available_width() - inspector_width - 20.0).max(100.0);

                ui.allocate_ui(egui::vec2(editor_width, main_height), |ui| {
                    self.draw_editor(ui);
                });

                ui.separator();

                ui.allocate_ui(egui::vec2(inspector_width, main_height), |ui| {
                    egui::ScrollArea::vertical()
                        .id_salt("inspector_scroll")
                        .show(ui, |ui| {
                            self.draw_inspector(ui, world, project_path);
                        });
                });
            });

            ui.separator();

            ui.allocate_ui(
                egui::vec2(ui.available_width(), bottom_panel_height),
                |ui| {
                    ui.vertical(|ui| {
                        ui.horizontal(|ui| {
                            let problem_count = self
                                .active_document
                                .as_ref()
                                .and_then(|path| self.open_documents.get(path))
                                .map_or(0, |document| document.diagnostics.len());

                            let problem_label = if has_problems {
                                format!("PROBLEMS ({})", problem_count)
                            } else {
                                "PROBLEMS".to_string()
                            };

                            if ui
                                .selectable_label(self.show_problems, problem_label)
                                .clicked()
                            {
                                self.show_problems = true;
                                self.show_output = false;
                            }

                            if ui.selectable_label(self.show_output, "OUTPUT").clicked() {
                                self.show_output = true;
                                self.show_problems = false;
                            }

                            if !has_problems && self.output_log.is_empty() {
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        ui.label(
                                            RichText::new("No issues found")
                                                .color(Color32::from_gray(140))
                                                .small(),
                                        );
                                    },
                                );
                            }
                        });

                        if bottom_panel_height > bottom_panel_min_height {
                            ui.separator();

                            if self.show_problems {
                                self.draw_problems(ui);
                            } else if self.show_output {
                                self.draw_output(ui);
                            }
                        }
                    });
                },
            );
        });

        if let Some(path) = action_open {
            self.open_file(path);
        }

        if let Some(path) = action_close {
            if !self.close_document(&path) {
                self.closing_path = Some(path);
            }
        }
    }

    fn draw_editor(&mut self, ui: &mut egui::Ui) {
        let Some(path) = self.active_document.clone() else {
            ui.centered_and_justified(|ui| {
                ui.label("No script open.");
            });
            return;
        };

        let Some(document) = self.open_documents.get_mut(&path) else {
            ui.centered_and_justified(|ui| {
                ui.label("No script open.");
            });
            return;
        };

        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                let filename = path.file_name().unwrap_or_default().to_string_lossy();

                ui.label(RichText::new(filename).strong());

                if document.dirty {
                    ui.label(RichText::new("(modified)").italics().color(Color32::YELLOW));
                }
            });

            let available_height = ui.available_height();

            egui::ScrollArea::vertical()
                .id_salt("editor_v_scroll")
                .max_height(available_height)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        let line_count = document.source.split('\n').count().max(1);

                        let gutter_width = (line_count.to_string().len() as f32 * 8.0).max(24.0);

                        ui.allocate_ui(egui::vec2(gutter_width, ui.available_height()), |ui| {
                            let mut gutter = String::new();

                            for line in 1..=line_count {
                                gutter.push_str(&format!("{}\n", line));
                            }

                            ui.add(egui::Label::new(
                                RichText::new(gutter)
                                    .monospace()
                                    .color(Color32::from_gray(120))
                                    .size(12.0),
                            ));
                        });

                        egui::ScrollArea::horizontal()
                            .id_salt("editor_h_scroll")
                            .show(ui, |ui| {
                                ui.push_id(&path, |ui| {
                                    let highlight_spans = document.highlight_spans.clone();

                                    let mut layouter =
                                        move |ui: &egui::Ui, text: &str, wrap_width: f32| {
                                            let job = build_layout_job(
                                                ui,
                                                text,
                                                &highlight_spans,
                                                wrap_width,
                                            );

                                            ui.fonts(|fonts| fonts.layout_job(job))
                                        };

                                    let response = ui.add(
                                        egui::TextEdit::multiline(&mut document.source)
                                            .font(egui::TextStyle::Monospace)
                                            .code_editor()
                                            .desired_width(f32::INFINITY)
                                            .desired_rows(line_count)
                                            .lock_focus(true)
                                            .layouter(&mut layouter),
                                    );

                                    let mut clear_cursor_request = false;

                                    if let Some((request_path, byte_offset)) = &self.cursor_request
                                    {
                                        if request_path == &path {
                                            response.request_focus();

                                            let mut state = egui::text_edit::TextEditState::load(
                                                ui.ctx(),
                                                response.id,
                                            )
                                            .unwrap_or_default();

                                            let char_offset = byte_offset_to_char_index(
                                                &document.source,
                                                *byte_offset,
                                            );

                                            state.cursor.set_char_range(Some(
                                                egui::text::CCursorRange::one(
                                                    egui::text::CCursor::new(char_offset),
                                                ),
                                            ));

                                            state.store(ui.ctx(), response.id);

                                            clear_cursor_request = true;
                                        }
                                    }

                                    if clear_cursor_request {
                                        self.cursor_request = None;
                                    }

                                    if response.changed() {
                                        document.dirty =
                                            document.source != document.original_source;

                                        document.reparse();
                                    }
                                });
                            });
                    });
                });
        });
    }

    fn draw_inspector(
        &mut self,
        ui: &mut egui::Ui,
        world: &mut crate::world::World,
        project_path: &Option<PathBuf>,
    ) {
        ui.vertical(|ui| {
            ui.label(RichText::new("INSPECTOR").strong());
            ui.separator();

            let Some(path) = self.active_document.as_ref() else {
                ui.centered_and_justified(|ui| {
                    ui.label("Select a script to view details.");
                });
                return;
            };

            let Some(document) = self.open_documents.get(path) else {
                return;
            };

            ui.label(RichText::new("SCRIPT").strong());

            ui.indent("script_info", |ui| {
                ui.label(format!(
                    "File: {}",
                    path.file_name().unwrap_or_default().to_string_lossy()
                ));

                let relative_path = if let Some(root) = project_path {
                    path.strip_prefix(root)
                        .unwrap_or(path)
                        .to_string_lossy()
                        .to_string()
                } else {
                    path.to_string_lossy().to_string()
                };

                let relative_path = relative_path.replace('\\', "/");

                let mut enabled = !world.disabled_scripts.contains(&relative_path);

                if ui
                    .checkbox(&mut enabled, "Script Enabled")
                    .on_hover_text(
                        "If disabled, this script will not execute \
                         lifecycle hooks or events for ANY attached cell.",
                    )
                    .changed()
                {
                    if enabled {
                        world
                            .disabled_scripts
                            .retain(|entry| entry != &relative_path);
                    } else if !world.disabled_scripts.contains(&relative_path) {
                        world.disabled_scripts.push(relative_path);
                    }
                }
            });

            ui.add_space(8.0);
            ui.label(RichText::new("ENTITIES").strong());

            ui.indent("entities_info", |ui| {
                let Some(program) = &document.program else {
                    ui.label(RichText::new("(parse failed)").color(Color32::RED));
                    return;
                };

                for declaration in &program.declarations {
                    if let crate::scripting::ast::Declaration::Entity(entity) = declaration {
                        ui.label(RichText::new(&entity.name).strong());

                        ui.indent("lifecycle", |ui| {
                            for member in &entity.members {
                                let crate::scripting::ast::EntityMember::Function(function) =
                                    member
                                else {
                                    continue;
                                };

                                let lifecycle = matches!(
                                    function.name.as_str(),
                                    "on_spawn" | "on_ready" | "update" | "on_destroy"
                                );

                                ui.horizontal(|ui| {
                                    ui.label(if lifecycle { "⚡" } else { "ƒ" });
                                    ui.label(&function.name);
                                });
                            }
                        });
                    }
                }
            });
        });
    }

    fn draw_problems(&mut self, ui: &mut egui::Ui) {
        ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Wrap);

        let mut click_action = None;

        if let Some(path) = &self.active_document {
            if let Some(document) = self.open_documents.get(path) {
                let mut diagnostics = Vec::new();

                for diagnostic in document.diagnostics.as_slice() {
                    let (start, _) = document.source_map.span_positions(diagnostic.location.span);

                    diagnostics.push((
                        diagnostic.severity.clone(),
                        diagnostic.location.file.clone(),
                        start.line,
                        start.column,
                        diagnostic.message.clone(),
                        diagnostic.location.span.start as usize,
                    ));
                }

                egui::ScrollArea::vertical()
                    .id_salt("problems_scroll")
                    .show(ui, |ui| {
                        for (severity, file, line, column, message, offset) in diagnostics {
                            let color = match severity {
                                crate::scripting::diagnostic::DiagnosticSeverity::Error => {
                                    Color32::RED
                                }
                                crate::scripting::diagnostic::DiagnosticSeverity::Warning => {
                                    Color32::YELLOW
                                }
                                _ => Color32::WHITE,
                            };

                            ui.horizontal(|ui| {
                                ui.colored_label(color, format!("{:?}", severity));

                                let message = format!(
                                    "{}:{}:{}  {}",
                                    file.as_deref().unwrap_or("unknown"),
                                    line,
                                    column,
                                    message
                                );

                                if ui.selectable_label(false, message).clicked() {
                                    if let Some(file_name) = &file {
                                        if let Some(path) = self
                                            .scripts_list
                                            .iter()
                                            .find(|path| {
                                                path.file_name().map_or(false, |name| {
                                                    name.to_string_lossy() == *file_name
                                                })
                                            })
                                            .cloned()
                                        {
                                            click_action = Some((path, offset));
                                        }
                                    } else if let Some(path) = &self.active_document {
                                        click_action = Some((path.clone(), offset));
                                    }
                                }
                            });
                        }
                    });
            }
        }

        if let Some((path, offset)) = click_action {
            self.open_file(path.clone());
            self.cursor_request = Some((path, offset));
        }
    }

    fn draw_output(&mut self, ui: &mut egui::Ui) {
        ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Wrap);

        egui::ScrollArea::vertical()
            .id_salt("output_scroll")
            .show(ui, |ui| {
                for line in &self.output_log {
                    ui.label(line);
                }
            });
    }
}
