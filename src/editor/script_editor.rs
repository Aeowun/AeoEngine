use std::path::PathBuf;
use std::collections::HashMap;
use crate::scripting::ast::Program;
use crate::scripting::diagnostic::Diagnostics;
use crate::scripting::source_map::SourceMap;
use crate::scripting::lexer::Lexer;
use crate::scripting::parser::Parser;
use crate::scripting::source::SourceLocation;

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
                        let file_name = self.path.file_name().map(|n| n.to_string_lossy().into_owned());
                        self.diagnostics.error(e.message, SourceLocation::new(file_name, e.span));
                    }
                }
            }
            Err(e) => {
                self.program = None;
                let file_name = self.path.file_name().map(|n| n.to_string_lossy().into_owned());
                // Lexer error doesn't have a span, just line/column. We'll have to approximate or update Lexer.
                // For now, let's assume we can at least report it.
                // Actually, LexerError has line and column. Diagnostic expects a span.
                // We'll use a single-byte span at the calculated offset if possible, or just a dummy span.
                // Since we have SourceMap, we can try to find the offset.
                let offset = self.source_map.line_start(e.line).unwrap_or(0) + (e.column as u32).saturating_sub(1);
                let span = crate::scripting::source::SourceSpan::single(offset);
                self.diagnostics.error(e.message, SourceLocation::new(file_name, span));
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
    pub search_query: String,
    pub show_search: bool,
    pub search_results: Vec<SearchResult>,

    pub closing_path: Option<PathBuf>,

    // UI state
    pub show_problems: bool,
    pub show_output: bool,
    pub output_log: Vec<String>,
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
            search_query: String::new(),
            show_search: false,
            search_results: Vec::new(),
            closing_path: None,
            show_problems: true,
            show_output: true,
            output_log: Vec::new(),
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
                    self.output_log.push(format!("Failed to save {:?}: {}", path, e));
                }
            }
        }
    }

    pub fn save_all(&mut self) {
        for doc in self.open_documents.values_mut() {
            if doc.dirty {
                if let Err(e) = doc.save() {
                    self.output_log.push(format!("Failed to save {:?}: {}", doc.path, e));
                }
            }
        }
    }

    pub fn create_new_script(&mut self, project_path: &PathBuf, name: &str) {
        let name = name.trim();
        if name.is_empty() { return; }

        let mut filename = name.to_string();
        if !filename.ends_with(".aeo") {
            filename.push_str(".aeo");
        }

        let path = project_path.join("scripts").join(&filename);
        if path.exists() {
            self.output_log.push(format!("File already exists: {:?}", path));
            return;
        }

        let entity_name = name.split('.').next().unwrap_or("MyScript");
        // Sanitize entity name (basic)
        let entity_name = entity_name.chars().filter(|c| c.is_alphanumeric()).collect::<String>();
        let entity_name = if entity_name.is_empty() { "MyScript".to_string() } else { entity_name };

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
", entity_name);

        if let Err(e) = std::fs::write(&path, &template) {
            self.output_log.push(format!("Failed to create script: {}", e));
        } else {
            self.refresh_scripts(&Some(project_path.clone()));
            self.open_file(path);
        }
    }

    pub fn perform_search(&mut self) {
        self.search_results.clear();
        if self.search_query.is_empty() { return; }

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

    pub fn show_ui(&mut self, ctx: &egui::Context, project_path: &Option<PathBuf>) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.style_mut().visuals.window_rounding = egui::Rounding::ZERO;

            // Layout: Top bar, then horizontal split (Explorer | Editor | Inspector), then Bottom panel (Problems/Output)

            self.draw_workspace(ui, project_path);
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
                        if ui.button("Create").clicked() {
                            if let Some(root) = project_path {
                                self.create_new_script(root, &self.new_script_name.clone());
                                self.show_new_script_dialog = false;
                                self.new_script_name.clear();
                            }
                        }
                        if ui.button("Cancel").clicked() {
                            self.show_new_script_dialog = false;
                        }
                    });
                });
        }

        if let Some(path) = self.closing_path.clone() {
            let mut close = false;
            egui::Window::new("Unsaved Changes")
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .collapsible(false)
                .show(ctx, |ui| {
                    ui.label(format!("Save changes to {}?", path.file_name().unwrap_or_default().to_string_lossy()));
                    ui.add_space(10.0);
                    ui.horizontal(|ui| {
                        if ui.button("Save").clicked() {
                            if let Some(doc) = self.open_documents.get_mut(&path) {
                                doc.save().ok();
                            }
                            close = true;
                        }
                        if ui.button("Don't Save").clicked() {
                            close = true;
                        }
                        if ui.button("Cancel").clicked() {
                            self.closing_path = None;
                        }
                    });
                });

            if close {
                self.open_documents.remove(&path);
                if self.active_document.as_ref() == Some(&path) {
                    self.active_document = self.open_documents.keys().next().cloned();
                }
                self.closing_path = None;
            }
        }
    }

    fn draw_workspace(&mut self, ui: &mut egui::Ui, project_path: &Option<PathBuf>) {
        let mut action_open = None;
        let mut action_close = None;

        ui.vertical(|ui| {
            // Header
            ui.horizontal(|ui| {
                ui.heading("SCRIPT EDITOR");
                if ui.button("New Script").clicked() {
                    self.show_new_script_dialog = true;
                }
                if ui.button("Save").clicked() {
                    self.save_active();
                }
                if ui.button("Save All").clicked() {
                    self.save_all();
                }
                if ui.button("Refresh").clicked() {
                    self.refresh_scripts(project_path);
                }
                ui.separator();
                ui.checkbox(&mut self.show_search, "Search");
                if self.show_search {
                    if ui.text_edit_singleline(&mut self.search_query).changed() {
                        self.perform_search();
                    }
                }
            });
            ui.separator();

            let total_height = ui.available_height();
            let bottom_panel_height = 150.0;
            let main_height = total_height - bottom_panel_height - 10.0;

            ui.horizontal(|ui| {
                // LEFT: Explorer
                ui.allocate_ui(egui::vec2(200.0, main_height), |ui| {
                    ui.vertical(|ui| {
                        if self.show_search && !self.search_results.is_empty() {
                            ui.label("SEARCH RESULTS");
                            egui::ScrollArea::vertical().id_source("search_results").show(ui, |ui| {
                                for result in &self.search_results {
                                    let filename = result.path.file_name().unwrap_or_default().to_string_lossy();
                                    if ui.selectable_label(false, format!("{}:{} {}", filename, result.line, result.text.trim())).clicked() {
                                        action_open = Some(result.path.clone());
                                    }
                                }
                            });
                            ui.separator();
                        }

                        ui.label("SCRIPTS");
                        ui.separator();
                        egui::ScrollArea::vertical().id_source("scripts_list").show(ui, |ui| {
                            ui.collapsing("scripts/", |ui| {
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
                                        if ui.selectable_label(is_active, label).clicked() {
                                            action_open = Some(path.clone());
                                        }
                                        if self.open_documents.contains_key(path) {
                                            if ui.small_button("x").clicked() {
                                                action_close = Some(path.clone());
                                            }
                                        }
                                    });
                                }
                            });
                        });
                    });
                });

                ui.separator();

                // MIDDLE: Editor
                let editor_width = ui.available_width() - 250.0;
                ui.allocate_ui(egui::vec2(editor_width, main_height), |ui| {
                    if let Some(path) = self.active_document.clone() {
                        if let Some(doc) = self.open_documents.get_mut(&path) {
                            ui.vertical(|ui| {
                                ui.horizontal(|ui| {
                                    ui.label(path.file_name().unwrap_or_default().to_string_lossy());
                                    if doc.dirty {
                                        ui.label("(unsaved)");
                                    }
                                });

                                let theme = egui::TextEdit::multiline(&mut doc.source)
                                    .font(egui::TextStyle::Monospace)
                                    .code_editor()
                                    .desired_width(f32::INFINITY)
                                    .desired_rows(20)
                                    .lock_focus(true);

                                let response = ui.add(theme);

                                if response.changed() {
                                    doc.dirty = doc.source != doc.original_source;
                                    doc.reparse();
                                }
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
                ui.allocate_ui(egui::vec2(250.0, main_height), |ui| {
                    self.draw_inspector(ui);
                });
            });

            ui.separator();

            // BOTTOM: Problems / Output
            ui.allocate_ui(egui::vec2(ui.available_width(), bottom_panel_height), |ui| {
                ui.horizontal(|ui| {
                    if ui.selectable_label(self.show_problems, "PROBLEMS").clicked() {
                        self.show_problems = true;
                        self.show_output = false;
                    }
                    if ui.selectable_label(self.show_output, "OUTPUT").clicked() {
                        self.show_output = true;
                        self.show_problems = false;
                    }
                });
                ui.separator();
                if self.show_problems {
                    self.draw_problems(ui);
                } else if self.show_output {
                    self.draw_output(ui);
                }
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

    fn draw_inspector(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            ui.label("INSPECTOR");
            ui.separator();
            if let Some(path) = &self.active_document {
                if let Some(doc) = self.open_documents.get(path) {
                    ui.label("SCRIPT");
                    ui.indent("script_info", |ui| {
                        ui.label(format!("File: {}", path.file_name().unwrap_or_default().to_string_lossy()));
                    });

                    ui.add_space(8.0);
                    ui.label("ENTITIES");
                    ui.indent("entities_info", |ui| {
                        if let Some(program) = &doc.program {
                            for decl in &program.declarations {
                                if let crate::scripting::ast::Declaration::Entity(entity) = decl {
                                    ui.label(&entity.name);

                                    ui.indent("lifecycle", |ui| {
                                        let mut has_spawn = false;
                                        let mut has_ready = false;
                                        let mut has_update = false;
                                        let mut has_destroy = false;

                                        for member in &entity.members {
                                            if let crate::scripting::ast::EntityMember::Function(f) = member {
                                                match f.name.as_str() {
                                                    "on_spawn" => has_spawn = true,
                                                    "on_ready" => has_ready = true,
                                                    "update" => has_update = true,
                                                    "on_destroy" => has_destroy = true,
                                                    _ => {}
                                                }
                                            }
                                        }

                                        let check = |ui: &mut egui::Ui, label: &str, present: bool| {
                                            ui.label(format!("{} {}", if present { "✓" } else { "○" }, label));
                                        };

                                        check(ui, "on_spawn", has_spawn);
                                        check(ui, "on_ready", has_ready);
                                        check(ui, "update", has_update);
                                        check(ui, "on_destroy", has_destroy);
                                    });
                                }
                            }
                        } else {
                            ui.label("(parse failed)");
                        }
                    });
                }
            }
        });
    }

    fn draw_problems(&mut self, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            if let Some(path) = &self.active_document {
                if let Some(doc) = self.open_documents.get(path) {
                    for diag in doc.diagnostics.as_slice() {
                        let (start, _) = doc.source_map.span_positions(diag.location.span);
                        ui.horizontal(|ui| {
                            let color = match diag.severity {
                                crate::scripting::diagnostic::DiagnosticSeverity::Error => egui::Color32::RED,
                                crate::scripting::diagnostic::DiagnosticSeverity::Warning => egui::Color32::YELLOW,
                                _ => egui::Color32::WHITE,
                            };
                            ui.colored_label(color, format!("{:?}", diag.severity));
                            ui.label(format!("{}:{}:{}  {}",
                                diag.location.file.as_deref().unwrap_or("unknown"),
                                start.line, start.column,
                                diag.message));
                        });
                    }
                }
            }
        });
    }

    fn draw_output(&mut self, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical().show(ui, |ui| {
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
        if test_dir.exists() { fs::remove_dir_all(&test_dir).ok(); }
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
    fn test_save_behavior() {
        let test_dir = PathBuf::from("TestProject_Save");
        let scripts_dir = test_dir.join("scripts");
        if test_dir.exists() { fs::remove_dir_all(&test_dir).ok(); }
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
        assert_eq!(diag.severity, crate::scripting::diagnostic::DiagnosticSeverity::Error);
        assert!(diag.message.contains("Expected"));
    }

    #[test]
    fn test_new_script_template() {
        let test_dir = PathBuf::from("TestProject_Template");
        let scripts_dir = test_dir.join("scripts");
        if test_dir.exists() { fs::remove_dir_all(&test_dir).ok(); }
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
        if test_dir.exists() { fs::remove_dir_all(&test_dir).ok(); }
        fs::create_dir_all(&scripts_dir).unwrap();

        fs::write(scripts_dir.join("one.aeo"), "hello world").unwrap();
        fs::write(scripts_dir.join("two.aeo"), "goodbye").unwrap();
        fs::write(scripts_dir.join("three.txt"), "hello hidden").unwrap(); // Should be ignored

        let mut editor = ScriptEditor::new();
        editor.refresh_scripts(&Some(test_dir.clone()));
        editor.search_query = "hello".to_string();
        editor.perform_search();

        assert_eq!(editor.search_results.len(), 1);
        assert_eq!(editor.search_results[0].path.file_name().unwrap(), "one.aeo");

        fs::remove_dir_all(&test_dir).ok();
    }
}
