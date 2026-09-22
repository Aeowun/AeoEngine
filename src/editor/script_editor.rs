use crate::scripting::ast::Program;
use crate::scripting::diagnostic::Diagnostics;
use crate::scripting::lexer::Lexer;
use crate::scripting::parser::Parser;
use crate::scripting::source::SourceLocation;
use crate::scripting::source_map::SourceMap;
use egui::{Color32, RichText};
use std::collections::HashMap;
use std::path::PathBuf;

pub struct ScriptDocument {
    pub path: PathBuf,
    pub source: String,
    pub original_source: String,
    pub dirty: bool,
    pub diagnostics: Diagnostics,
    pub program: Option<Program>,
    pub source_map: SourceMap,
}

impl ScriptDocument {
    pub fn new(path: PathBuf, source: String) -> Self {
        let source_map = SourceMap::new(&source);
        let mut doc = Self {
            path,
            original_source: source.clone(),
            source,
            dirty: false,
            diagnostics: Diagnostics::new(),
            program: None,
            source_map,
        };
        doc.reparse();
        doc
    }

    pub fn reparse(&mut self) {
        self.diagnostics = Diagnostics::new();
        self.source_map = SourceMap::new(&self.source);

        let lexer = Lexer::new(&self.source);
        match lexer.tokenize() {
            Ok(tokens) => {
                let parser = Parser::new(tokens);
                match parser.parse() {
                    Ok(program) => {
                        self.program = Some(program);
                    }
                    Err(e) => {
                        self.program = None;
                        let file_name = self
                            .path
                            .file_name()
                            .map(|n| n.to_string_lossy().into_owned());
                        self.diagnostics
                            .error(e.message, SourceLocation::new(file_name, e.span));
                    }
                }
            }
            Err(e) => {
                self.program = None;
                let file_name = self
                    .path
                    .file_name()
                    .map(|n| n.to_string_lossy().into_owned());
                // Lexer error doesn't have a span, just line/column. We'll have to approximate or update Lexer.
                // For now, let's assume we can at least report it.
                // Actually, LexerError has line and column. Diagnostic expects a span.
                // We'll use a single-byte span at the calculated offset if possible, or just a dummy span.
                // Since we have SourceMap, we can try to find the offset.
                let offset = self.source_map.line_start(e.line).unwrap_or(0)
                    + (e.column as u32).saturating_sub(1);
                let span = crate::scripting::source::SourceSpan::single(offset);
                self.diagnostics
                    .error(e.message, SourceLocation::new(file_name, span));
            }
        }
    }

    pub fn save(&mut self) -> std::io::Result<()> {
        std::fs::write(&self.path, &self.source)?;
        self.original_source = self.source.clone();
        self.dirty = false;
        Ok(())
    }
}

pub struct ScriptEditor {
    pub open_documents: HashMap<PathBuf, ScriptDocument>,
    pub active_document: Option<PathBuf>,
    pub scripts_list: Vec<PathBuf>,
    pub show_new_script_dialog: bool,
    pub new_script_name: String,
    pub show_rename_dialog: bool,
    pub rename_target: Option<PathBuf>,
    pub rename_new_name: String,
    pub search_query: String,
    pub show_search: bool,
    pub search_results: Vec<SearchResult>,

    pub closing_path: Option<PathBuf>,
    pub deleting_path: Option<PathBuf>,

    // UI state
    pub show_problems: bool,
    pub show_output: bool,
    pub output_log: Vec<String>,
    pub cursor_request: Option<(PathBuf, usize)>,
}

pub struct SearchResult {
    pub path: PathBuf,
    pub line: usize,
    pub text: String,
}

impl ScriptEditor {
    pub fn new() -> Self {
        Self {
            open_documents: HashMap::new(),
            active_document: None,
            scripts_list: Vec::new(),
            show_new_script_dialog: false,
            new_script_name: String::new(),
            show_rename_dialog: false,
            rename_target: None,
            rename_new_name: String::new(),
            search_query: String::new(),
            show_search: false,
            search_results: Vec::new(),
            closing_path: None,
            deleting_path: None,
            show_problems: true,
            show_output: true,
            output_log: Vec::new(),
            cursor_request: None,
        }
    }

    pub fn refresh_scripts(&mut self, project_path: &Option<PathBuf>) {
        self.scripts_list.clear();
        if let Some(root) = project_path {
            let scripts_dir = root.join("scripts");
            if let Ok(entries) = std::fs::read_dir(scripts_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_file() && path.extension().map_or(false, |ext| ext == "aeo") {
                        self.scripts_list.push(path);
                    }
                }
            }
        }
        self.scripts_list.sort();
    }

    pub fn open_file(&mut self, path: PathBuf) {
        if !self.open_documents.contains_key(&path) {
            if let Ok(source) = std::fs::read_to_string(&path) {
                let doc = ScriptDocument::new(path.clone(), source);
                self.open_documents.insert(path.clone(), doc);
            }
        }
        self.active_document = Some(path);
    }

    pub fn close_document(&mut self, path: &PathBuf) -> bool {
        if let Some(doc) = self.open_documents.get(path) {
            if doc.dirty {
                // Return false to indicate we need confirmation
                return false;
            }
        }
        self.open_documents.remove(path);
        if self.active_document.as_ref() == Some(path) {
            self.active_document = self.open_documents.keys().next().cloned();
        }
        true
    }

    pub fn save_active(&mut self) {
        if let Some(path) = &self.active_document {
            if let Some(doc) = self.open_documents.get_mut(path) {
                if let Err(e) = doc.save() {
                    self.output_log
                        .push(format!("Failed to save {:?}: {}", path, e));
                }
            }
        }
    }

    pub fn save_all(&mut self) {
        for doc in self.open_documents.values_mut() {
            if doc.dirty {
                if let Err(e) = doc.save() {
                    self.output_log
                        .push(format!("Failed to save {:?}: {}", doc.path, e));
                }
            }
        }
    }

    pub fn create_new_script(&mut self, project_path: &PathBuf, name: &str) {
        let name = name.trim();
        if name.is_empty() {
            return;
        }

        let mut filename = name.to_string();
        if !filename.ends_with(".aeo") {
            filename.push_str(".aeo");
        }

        let path = project_path.join("scripts").join(&filename);
        if path.exists() {
            self.output_log
                .push(format!("File already exists: {:?}", path));
            return;
        }

        let entity_name = name.split('.').next().unwrap_or("MyScript");
        // Sanitize entity name (basic)
        let entity_name = entity_name
            .chars()
            .filter(|c| c.is_alphanumeric())
            .collect::<String>();
        let entity_name = if entity_name.is_empty() {
            "MyScript".to_string()
        } else {
            entity_name
        };

        let template = format!(
            "entity {} {{

    fn on_spawn() {{
    }}

    fn on_ready() {{
    }}

    fn update(dt: number) {{
    }}

    fn on_destroy() {{
    }}
}}
",
            entity_name
        );

        if let Err(e) = std::fs::write(&path, &template) {
            self.output_log
                .push(format!("Failed to create script: {}", e));
        } else {
            self.refresh_scripts(&Some(project_path.clone()));
            self.open_file(path);
        }
    }

    pub fn rename_script(&mut self, project_path: &PathBuf, old_path: PathBuf, new_name: &str) {
        let new_name = new_name.trim();
        if new_name.is_empty() {
            self.output_log
                .push("Rename failed: Name cannot be empty.".to_string());
            return;
        }

        let mut filename = new_name.to_string();
        if !filename.ends_with(".aeo") {
            filename.push_str(".aeo");
        }

        // Windows invalid characters check: \ / : * ? " < > |
        if filename
            .chars()
            .any(|c| matches!(c, '\\' | '/' | ':' | '*' | '?' | '"' | '<' | '>' | '|'))
        {
            self.output_log
                .push("Rename failed: Invalid characters in filename.".to_string());
            return;
        }

        let new_path = project_path.join("scripts").join(&filename);
        if new_path.exists() {
            self.output_log
                .push(format!("Rename failed: {:?} already exists.", new_path));
            return;
        }

        if let Err(e) = std::fs::rename(&old_path, &new_path) {
            self.output_log.push(format!("Rename failed: {}", e));
        } else {
            // Update internal state
            if let Some(mut doc) = self.open_documents.remove(&old_path) {
                doc.path = new_path.clone();
                self.open_documents.insert(new_path.clone(), doc);
            }

            if self.active_document.as_ref() == Some(&old_path) {
                self.active_document = Some(new_path.clone());
            }

            self.refresh_scripts(&Some(project_path.clone()));
            self.output_log.push(format!("Renamed to {:?}", filename));
        }
    }

    pub fn delete_script(&mut self, project_path: &Option<PathBuf>, path: PathBuf) {
        if let Err(e) = std::fs::remove_file(&path) {
            self.output_log.push(format!("Delete failed: {}", e));
        } else {
            self.open_documents.remove(&path);
            if self.active_document.as_ref() == Some(&path) {
                self.active_document = self.open_documents.keys().next().cloned();
            }
            self.refresh_scripts(project_path);
            self.output_log.push(format!(
                "Deleted {:?}",
                path.file_name().unwrap_or_default()
            ));
        }
    }

    pub fn apply_dirty_response(
        &mut self,
        project_path: &Option<PathBuf>,
        path: PathBuf,
        save: bool,
        cancel: bool,
    ) {
        let is_deletion = self.deleting_path.as_ref() == Some(&path);
        let mut close = false;
        let mut delete_after = false;

        if cancel {
            self.closing_path = None;
            if is_deletion {
                self.deleting_path = None;
            }
            return;
        }

        if save {
            if let Some(doc) = self.open_documents.get_mut(&path) {
                if let Err(e) = doc.save() {
                    self.output_log.push(format!(
                        "Save failed: {}. {} aborted.",
                        e,
                        if is_deletion { "Deletion" } else { "Closure" }
                    ));
                    return;
                }
            }
            close = true;
            if is_deletion {
                delete_after = true;
            }
        } else {
            // Don't Save
            close = true;
            if is_deletion {
                delete_after = true;
            }
        }

        if close {
            if delete_after {
                self.delete_script(project_path, path.clone());
                self.deleting_path = None;
            } else {
                self.open_documents.remove(&path);
                if self.active_document.as_ref() == Some(&path) {
                    self.active_document = self.open_documents.keys().next().cloned();
                }
            }
            self.closing_path = None;
        }
    }

    pub fn perform_search(&mut self) {
        self.search_results.clear();
        if self.search_query.is_empty() {
            return;
        }

        for path in &self.scripts_list {
            if let Ok(source) = std::fs::read_to_string(path) {
                for (i, line) in source.lines().enumerate() {
                    if line.contains(&self.search_query) {
                        self.search_results.push(SearchResult {
                            path: path.clone(),
                            line: i + 1,
                            text: line.to_string(),
                        });
                    }
                }
            }
        }
    }

    pub fn show_ui(
        &mut self,
        ctx: &egui::Context,
        project_path: &Option<PathBuf>,
        world: &mut crate::world::World,
    ) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.style_mut().visuals.window_rounding = egui::Rounding::ZERO;

            // Layout: Top bar, then horizontal split (Explorer | Editor | Inspector), then Bottom panel (Problems/Output)

            self.draw_workspace(ui, project_path, world);
        });

        if self.show_new_script_dialog {
            egui::Window::new("New Script")
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .collapsible(false)
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Name:");
                        ui.text_edit_singleline(&mut self.new_script_name);
                    });
                    ui.horizontal(|ui| {
                        if ui.button("Create").clicked()
                            || ui.input(|i| i.key_pressed(egui::Key::Enter))
                        {
                            if let Some(root) = project_path {
                                self.create_new_script(root, &self.new_script_name.clone());
                                self.show_new_script_dialog = false;
                                self.new_script_name.clear();
                            }
                        }
                        if ui.button("Cancel").clicked()
                            || ui.input(|i| i.key_pressed(egui::Key::Escape))
                        {
                            self.show_new_script_dialog = false;
                        }
                    });
                });
        }

        if self.show_rename_dialog {
            egui::Window::new("Rename Script")
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .collapsible(false)
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("New Name:");
                        ui.text_edit_singleline(&mut self.rename_new_name);
                    });
                    ui.horizontal(|ui| {
                        if ui.button("Rename").clicked()
                            || ui.input(|i| i.key_pressed(egui::Key::Enter))
                        {
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
                            || ui.input(|i| i.key_pressed(egui::Key::Escape))
                        {
                            self.show_rename_dialog = false;
                            self.rename_target = None;
                        }
                    });
                });
        }

        if let Some(path) = self.deleting_path.clone() {
            // If we are already showing the dirty dialog for this path, don't show the confirmation
            if self.closing_path.as_ref() != Some(&path) {
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
                            if ui.button("Delete").clicked()
                                || ui.input(|i| i.key_pressed(egui::Key::Enter))
                            {
                                let is_dirty =
                                    self.open_documents.get(&path).map_or(false, |d| d.dirty);
                                if is_dirty {
                                    self.closing_path = Some(path.clone());
                                } else {
                                    self.delete_script(project_path, path.clone());
                                    self.deleting_path = None;
                                }
                            }
                            if ui.button("Cancel").clicked()
                                || ui.input(|i| i.key_pressed(egui::Key::Escape))
                            {
                                self.deleting_path = None;
                            }
                        });
                    });
            }
        }

        if let Some(path) = self.closing_path.clone() {
            // Check if this closure is part of a deletion
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
                        || ui.input(|i| i.key_pressed(egui::Key::Escape))
                    {
                        self.apply_dirty_response(project_path, path.clone(), false, true);
                    }
                });
            });
        }
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
            // Header
            ui.horizontal(|ui| {
                ui.heading(RichText::new("SCRIPT EDITOR").strong());
                ui.separator();

                if ui.button("New Script").on_hover_text("Create a new .aeo script").clicked() {
                    self.show_new_script_dialog = true;
                }
                if ui.button("Save").on_hover_text("Save the active document (Ctrl+S)").clicked() {
                    self.save_active();
                }
                if ui.button("Save All").on_hover_text("Save all open documents (Ctrl+Shift+S)").clicked() {
                    self.save_all();
                }
                if ui.button("Refresh").on_hover_text("Refresh the script list from disk").clicked() {
                    self.refresh_scripts(project_path);
                }

                ui.separator();

                ui.toggle_value(&mut self.show_search, "Search");
                if self.show_search {
                    ui.label("Find:");
                    let search_resp = ui.add(egui::TextEdit::singleline(&mut self.search_query)
                        .hint_text("Search in all scripts...")
                        .desired_width(180.0));
                    if search_resp.changed() {
                        self.perform_search();
                    }
                }
            });
            ui.separator();

            // Calculate heights. If no problems/output, bottom panel is tiny.
            let has_problems = if let Some(path) = &self.active_document {
                self.open_documents.get(path).map_or(false, |d| !d.diagnostics.is_empty())
            } else {
                false
            };

            let bottom_panel_min_height = 28.0;
            let bottom_panel_expanded_height = 150.0;

            let bottom_panel_height = if (self.show_problems && has_problems) || (self.show_output && !self.output_log.is_empty()) {
                bottom_panel_expanded_height
            } else {
                bottom_panel_min_height
            };

            let main_height = ui.available_height() - bottom_panel_height - 10.0;

            ui.horizontal(|ui| {
                // LEFT: Explorer
                let explorer_width = 230.0;
                ui.allocate_ui(egui::vec2(explorer_width, main_height), |ui| {
                    egui::ScrollArea::vertical().id_salt("explorer_scroll").show(ui, |ui| {
                        ui.vertical(|ui| {
                            if self.show_search && !self.search_results.is_empty() {
                                ui.label(RichText::new("SEARCH RESULTS").strong());
                                for result in &self.search_results {
                                    let filename = result.path.file_name().unwrap_or_default().to_string_lossy();
                                    if ui.selectable_label(false, format!("{}:{} {}", filename, result.line, result.text.trim())).clicked() {
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
                                        let filename = path.file_name().unwrap_or_default().to_string_lossy();
                                        let is_active = self.active_document.as_ref() == Some(path);
                                        let is_dirty = self.open_documents.get(path).map_or(false, |d| d.dirty);
                                        let label = if is_dirty {
                                            format!("{} *", filename)
                                        } else {
                                            filename.to_string()
                                        };

                                        ui.horizontal(|ui| {
                                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                if ui.small_button("").on_hover_text("Delete script from disk").clicked() {
                                                    self.deleting_path = Some(path.clone());
                                                }
                                                if ui.small_button("📝").on_hover_text("Rename script").clicked() {
                                                    self.show_rename_dialog = true;
                                                    self.rename_target = Some(path.clone());
                                                    self.rename_new_name = path.file_stem().unwrap_or_default().to_string_lossy().to_string();
                                                }
                                                if self.open_documents.contains_key(path) {
                                                    if ui.small_button("❌").on_hover_text("Close document").clicked() {
                                                        action_close = Some(path.clone());
                                                    }
                                                }

                                                ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                                                    ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Truncate);
                                                    let resp = ui.selectable_label(is_active, label);
                                                    if resp.clicked() {
                                                        action_open = Some(path.clone());
                                                    }
                                                });
                                            });
                                        });
                                    }
                                });
                        });
                    });
                });

                ui.separator();

                // MIDDLE: Editor
                let inspector_width = 240.0;
                let editor_width = (ui.available_width() - inspector_width - 20.0).max(100.0);

                ui.allocate_ui(egui::vec2(editor_width, main_height), |ui| {
                    if let Some(path) = self.active_document.clone() {
                        if let Some(doc) = self.open_documents.get_mut(&path) {
                            ui.vertical(|ui| {
                                ui.horizontal(|ui| {
                                    let filename = path.file_name().unwrap_or_default().to_string_lossy();
                                    ui.label(RichText::new(filename).strong());
                                    if doc.dirty {
                                        ui.label(RichText::new("(modified)").italics().color(egui::Color32::YELLOW));
                                    }
                                });

                                let available_h = ui.available_height();
                                egui::ScrollArea::vertical()
                                    .id_salt("editor_v_scroll")
                                    .max_height(available_h)
                                    .show(ui, |ui| {
                                        ui.horizontal(|ui| {
                                            // Gutter for line numbers
                                            let line_count = doc.source.lines().count().max(1);
                                            let gutter_width = (line_count.to_string().len() as f32 * 8.0).max(24.0);

                                            ui.allocate_ui(egui::vec2(gutter_width, ui.available_height()), |ui| {
                                                let mut gutter_text = String::new();
                                                for i in 1..=line_count {
                                                    gutter_text.push_str(&format!("{}\n", i));
                                                }
                                                ui.add(egui::Label::new(egui::RichText::new(gutter_text)
                                                    .monospace()
                                                    .color(egui::Color32::from_gray(120))
                                                    .size(12.0)));
                                            });

                                            egui::ScrollArea::horizontal()
                                                .id_salt("editor_h_scroll")
                                                .show(ui, |ui| {
                                                    ui.push_id(&path, |ui| {
                                                        let theme = egui::TextEdit::multiline(&mut doc.source)
                                                            .font(egui::TextStyle::Monospace)
                                                            .code_editor()
                                                            .desired_width(f32::INFINITY)
                                                            .desired_rows(line_count)
                                                            .lock_focus(true);

                                                        let response = ui.add(theme);

                                                        let mut clear_req = false;
                                                        if let Some((req_path, offset)) = &self.cursor_request {
                                                            if req_path == &path {
                                                                response.request_focus();
                                                                let mut state = egui::text_edit::TextEditState::load(ui.ctx(), response.id).unwrap_or_default();
                                                                state.set_ccursor_range(Some(egui::text::CCursorRange::one(egui::text::CCursor::new(*offset))));
                                                                state.store(ui.ctx(), response.id);
                                                                clear_req = true;
                                                            }
                                                        }
                                                        if clear_req {
                                                            self.cursor_request = None;
                                                        }

                                                        if response.changed() {
                                                            doc.dirty = doc.source != doc.original_source;
                                                            doc.reparse();
                                                        }
                                                    });
                                                });
                                        });
                                    });
                            });
                        }
                    } else {
                        ui.centered_and_justified(|ui| {
                            ui.label("No script open.");
                        });
                    }
                });

                ui.separator();

                // RIGHT: Inspector
                ui.allocate_ui(egui::vec2(inspector_width, main_height), |ui| {
                    egui::ScrollArea::vertical().id_salt("inspector_scroll").show(ui, |ui| {
                        self.draw_inspector(ui, world, project_path);
                    });
                });
            });

            ui.separator();

            // BOTTOM: Problems / Output
            ui.allocate_ui(egui::vec2(ui.available_width(), bottom_panel_height), |ui| {
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        let problems_label = if has_problems {
                            format!("PROBLEMS ({})", if let Some(path) = &self.active_document {
                                self.open_documents.get(path).map_or(0, |d| d.diagnostics.len())
                            } else { 0 })
                        } else {
                            "PROBLEMS".to_string()
                        };

                        if ui.selectable_label(self.show_problems, problems_label).clicked() {
                            self.show_problems = true;
                            self.show_output = false;
                        }
                        if ui.selectable_label(self.show_output, "OUTPUT").clicked() {
                            self.show_output = true;
                            self.show_problems = false;
                        }

                        if !has_problems && self.output_log.is_empty() {
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                ui.label(RichText::new("No issues found").color(egui::Color32::from_gray(140)).small());
                            });
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
            });
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

    fn draw_inspector(
        &mut self,
        ui: &mut egui::Ui,
        world: &mut crate::world::World,
        project_path: &Option<PathBuf>,
    ) {
        ui.vertical(|ui| {
            ui.label(RichText::new("INSPECTOR").strong());
            ui.separator();
            if let Some(path) = &self.active_document {
                if let Some(doc) = self.open_documents.get(path) {
                    ui.label(RichText::new("SCRIPT").strong());
                    ui.indent("script_info", |ui| {
                        ui.label(format!("File: {}", path.file_name().unwrap_or_default().to_string_lossy()));

                        let relative_path = if let Some(root) = project_path {
                            path.strip_prefix(root).unwrap_or(path).to_string_lossy().to_string()
                        } else {
                            path.to_string_lossy().to_string()
                        };
                        let relative_path = relative_path.replace("\\", "/");

                        let mut enabled = !world.disabled_scripts.contains(&relative_path);
                        if ui.checkbox(&mut enabled, "Script Enabled").on_hover_text("If disabled, this script will not execute lifecycle hooks or events for ANY attached cell.").changed() {
                            if enabled {
                                world.disabled_scripts.retain(|p| p != &relative_path);
                            } else {
                                if !world.disabled_scripts.contains(&relative_path) {
                                    world.disabled_scripts.push(relative_path);
                                }
                            }
                        }
                    });

                    ui.add_space(8.0);
                    ui.label(RichText::new("ENTITIES").strong());
                    ui.indent("entities_info", |ui| {
                        if let Some(program) = &doc.program {
                            for decl in &program.declarations {
                                if let crate::scripting::ast::Declaration::Entity(entity) = decl {
                                    ui.horizontal(|ui| {
                                        ui.label(RichText::new(&entity.name).strong());
                                    });

                                    ui.indent("lifecycle", |ui| {
                                        for member in &entity.members {
                                            if let crate::scripting::ast::EntityMember::Function(f) = member {
                                                let is_lifecycle = matches!(f.name.as_str(), "on_spawn" | "on_ready" | "update" | "on_destroy");

                                                ui.horizontal(|ui| {
                                                    ui.label(if is_lifecycle { "⚡" } else { "ƒ" });
                                                    ui.label(&f.name);
                                                });
                                            }
                                        }
                                    });
                                }
                            }
                        } else {
                            ui.label(RichText::new("(parse failed)").color(egui::Color32::RED));
                        }
                    });
                }
            } else {
                ui.centered_and_justified(|ui| {
                    ui.label("Select a script to view details.");
                });
            }
        });
    }

    fn draw_problems(&mut self, ui: &mut egui::Ui) {
        ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Wrap);

        let mut click_action = None;

        if let Some(path) = &self.active_document {
            if let Some(doc) = self.open_documents.get(path) {
                let mut diags_info = Vec::new();
                for diag in doc.diagnostics.as_slice() {
                    let (start, _) = doc.source_map.span_positions(diag.location.span);
                    diags_info.push((
                        diag.severity.clone(),
                        diag.location.file.clone(),
                        start.line,
                        start.column,
                        diag.message.clone(),
                        diag.location.span.start as usize,
                    ));
                }

                egui::ScrollArea::vertical()
                    .id_salt("problems_scroll")
                    .show(ui, |ui| {
                        for info in diags_info {
                            let (severity, file, line, col, message, offset) = info;
                            let color = match severity {
                                crate::scripting::diagnostic::DiagnosticSeverity::Error => {
                                    egui::Color32::RED
                                }
                                crate::scripting::diagnostic::DiagnosticSeverity::Warning => {
                                    egui::Color32::YELLOW
                                }
                                _ => egui::Color32::WHITE,
                            };

                            ui.horizontal(|ui| {
                                ui.colored_label(color, format!("{:?}", severity));
                                let msg = format!(
                                    "{}:{}:{}  {}",
                                    file.as_deref().unwrap_or("unknown"),
                                    line,
                                    col,
                                    message
                                );

                                let row_resp = ui.selectable_label(false, msg);
                                if row_resp.clicked() {
                                    if let Some(ref f) = file {
                                        if let Some(p) = self
                                            .scripts_list
                                            .iter()
                                            .find(|p| {
                                                p.file_name()
                                                    .map_or(false, |n| n.to_string_lossy() == *f)
                                            })
                                            .cloned()
                                        {
                                            click_action = Some((p, offset));
                                        }
                                    } else if let Some(p) = &self.active_document {
                                        click_action = Some((p.clone(), offset));
                                    }
                                }
                            });
                        }
                    });
            }
        }

        if let Some((p, offset)) = click_action {
            self.open_file(p.clone());
            self.cursor_request = Some((p, offset));
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_script_filenames_sorted() {
        let test_dir = PathBuf::from("TestProject_Scripts");
        let scripts_dir = test_dir.join("scripts");
        if test_dir.exists() {
            fs::remove_dir_all(&test_dir).ok();
        }
        fs::create_dir_all(&scripts_dir).unwrap();

        fs::write(scripts_dir.join("z.aeo"), "").unwrap();
        fs::write(scripts_dir.join("a.aeo"), "").unwrap();
        fs::write(scripts_dir.join("m.aeo"), "").unwrap();
        fs::write(scripts_dir.join("ignored.txt"), "").unwrap(); // Should be ignored

        let mut editor = ScriptEditor::new();
        editor.refresh_scripts(&Some(test_dir.clone()));

        assert_eq!(editor.scripts_list.len(), 3);
        assert_eq!(editor.scripts_list[0].file_name().unwrap(), "a.aeo");
        assert_eq!(editor.scripts_list[1].file_name().unwrap(), "m.aeo");
        assert_eq!(editor.scripts_list[2].file_name().unwrap(), "z.aeo");

        fs::remove_dir_all(&test_dir).ok();
    }

    #[test]
    fn test_script_selection_opens_document() {
        let test_dir = std::path::PathBuf::from("TestProject_Open");
        let scripts_dir = test_dir.join("scripts");
        if test_dir.exists() {
            fs::remove_dir_all(&test_dir).ok();
        }
        fs::create_dir_all(&scripts_dir).unwrap();

        let path = scripts_dir.join("open_test.aeo");
        fs::write(&path, "script content").unwrap();

        let mut editor = ScriptEditor::new();
        editor.refresh_scripts(&Some(test_dir.clone()));

        // Simulate selection (the same logic used by the UI's action_open -> open_file)
        editor.open_file(path.clone());

        assert_eq!(editor.active_document, Some(path.clone()));
        assert!(editor.open_documents.contains_key(&path));
        assert_eq!(
            editor.open_documents.get(&path).unwrap().source,
            "script content"
        );

        fs::remove_dir_all(&test_dir).ok();
    }

    #[test]
    fn test_save_behavior() {
        let test_dir = PathBuf::from("TestProject_Save");
        let scripts_dir = test_dir.join("scripts");
        if test_dir.exists() {
            fs::remove_dir_all(&test_dir).ok();
        }
        fs::create_dir_all(&scripts_dir).unwrap();

        let path = scripts_dir.join("save_test.aeo");
        fs::write(&path, "initial").unwrap();

        let mut doc = ScriptDocument::new(path.clone(), "initial".to_string());
        assert!(!doc.dirty);

        doc.source = "changed".to_string();
        doc.dirty = true;

        doc.save().expect("Save should succeed");
        assert!(!doc.dirty);
        assert_eq!(fs::read_to_string(&path).unwrap(), "changed");

        fs::remove_dir_all(&test_dir).ok();
    }

    #[test]
    fn test_diagnostics_integration() {
        let source = "entity {"; // Invalid entity (missing name)
        let doc = ScriptDocument::new(PathBuf::from("error.aeo"), source.to_string());

        assert!(!doc.diagnostics.is_empty());
        assert!(doc.diagnostics.has_errors());

        let diag = &doc.diagnostics.as_slice()[0];
        assert_eq!(
            diag.severity,
            crate::scripting::diagnostic::DiagnosticSeverity::Error
        );
        assert!(diag.message.contains("Expected"));
    }

    #[test]
    fn test_new_script_template() {
        let test_dir = PathBuf::from("TestProject_Template");
        let scripts_dir = test_dir.join("scripts");
        if test_dir.exists() {
            fs::remove_dir_all(&test_dir).ok();
        }
        fs::create_dir_all(&scripts_dir).unwrap();

        let mut editor = ScriptEditor::new();
        editor.create_new_script(&test_dir, "PlayerScript");

        let path = scripts_dir.join("PlayerScript.aeo");
        assert!(path.exists());
        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("entity PlayerScript"));
        assert!(content.contains("fn on_spawn()"));

        // Verify it parses correctly
        let doc = ScriptDocument::new(path, content);
        assert!(!doc.diagnostics.has_errors());

        fs::remove_dir_all(&test_dir).ok();
    }

    #[test]
    fn test_search_results() {
        let test_dir = PathBuf::from("TestProject_Search");
        let scripts_dir = test_dir.join("scripts");
        if test_dir.exists() {
            fs::remove_dir_all(&test_dir).ok();
        }
        fs::create_dir_all(&scripts_dir).unwrap();

        fs::write(scripts_dir.join("one.aeo"), "hello world").unwrap();
        fs::write(scripts_dir.join("two.aeo"), "goodbye").unwrap();
        fs::write(scripts_dir.join("three.txt"), "hello hidden").unwrap(); // Should be ignored

        let mut editor = ScriptEditor::new();
        editor.refresh_scripts(&Some(test_dir.clone()));
        editor.search_query = "hello".to_string();
        editor.perform_search();

        assert_eq!(editor.search_results.len(), 1);
        assert_eq!(
            editor.search_results[0].path.file_name().unwrap(),
            "one.aeo"
        );

        fs::remove_dir_all(&test_dir).ok();
    }

    #[test]
    fn test_inspector_metadata_updates_after_edit() {
        let source = "entity test { fn on_spawn() {} fn on_destroy() {} }";
        let mut doc = ScriptDocument::new(PathBuf::from("test.aeo"), source.to_string());

        // Initially both functions should be present
        let mut has_spawn = false;
        let mut has_destroy = false;
        if let Some(program) = &doc.program {
            if let crate::scripting::ast::Declaration::Entity(entity) = &program.declarations[0] {
                for member in &entity.members {
                    if let crate::scripting::ast::EntityMember::Function(f) = member {
                        if f.name == "on_spawn" {
                            has_spawn = true;
                        }
                        if f.name == "on_destroy" {
                            has_destroy = true;
                        }
                    }
                }
            }
        }
        assert!(has_spawn);
        assert!(has_destroy);

        // Delete on_destroy
        doc.source = "entity test { fn on_spawn() {} }".to_string();
        doc.reparse();

        // Check again
        let mut has_spawn = false;
        let mut has_destroy = false;
        if let Some(program) = &doc.program {
            if let crate::scripting::ast::Declaration::Entity(entity) = &program.declarations[0] {
                for member in &entity.members {
                    if let crate::scripting::ast::EntityMember::Function(f) = member {
                        if f.name == "on_spawn" {
                            has_spawn = true;
                        }
                        if f.name == "on_destroy" {
                            has_destroy = true;
                        }
                    }
                }
            }
        }
        assert!(has_spawn);
        assert!(
            !has_destroy,
            "on_destroy should be gone from parsed metadata"
        );
    }

    #[test]
    fn test_delete_dirty_save_failure_aborts_deletion() {
        let test_dir = PathBuf::from("TestProject_DeleteFailure");
        let scripts_dir = test_dir.join("scripts");
        if test_dir.exists() {
            fs::remove_dir_all(&test_dir).ok();
        }
        fs::create_dir_all(&scripts_dir).unwrap();

        let path = scripts_dir.join("bug.aeo");
        fs::write(&path, "initial content").unwrap();

        let mut editor = ScriptEditor::new();
        let mut doc = ScriptDocument::new(path.clone(), "initial content".to_string());
        doc.source = "dirty content".to_string();
        doc.dirty = true;
        editor.open_documents.insert(path.clone(), doc);

        // Simulate start of deletion
        editor.deleting_path = Some(path.clone());
        editor.closing_path = Some(path.clone());

        // Break the filesystem for this path so save fails
        fs::remove_file(&path).unwrap();
        fs::create_dir(&path).unwrap(); // Writing to a directory will fail on most OS

        // Trigger "Save" response
        editor.apply_dirty_response(&Some(test_dir.clone()), path.clone(), true, false);

        // Verify:
        // 1. Path still exists (as a directory, but it hasn't been removed by delete_script)
        assert!(path.exists());
        // 2. Document is still open
        assert!(editor.open_documents.contains_key(&path));
        // 3. Deletion state is still active
        assert_eq!(editor.closing_path, Some(path.clone()));
        assert_eq!(editor.deleting_path, Some(path.clone()));
        // 4. Error is logged
        assert!(editor.output_log.iter().any(|l| l.contains("Save failed")));

        fs::remove_dir_all(&test_dir).ok();
    }
}
