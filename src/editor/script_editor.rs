use crate::scripting::ast::Program;
use crate::scripting::diagnostic::Diagnostics;
use crate::scripting::lexer::Lexer;
use crate::scripting::parser::Parser;
use crate::scripting::source::SourceLocation;
use crate::scripting::source_map::SourceMap;
use crate::scripting::token::{Token, TokenKind};
use egui::text::{LayoutJob, TextFormat};
use egui::{Color32, RichText};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum HighlightKind {
    Keyword,
    Type,
    Boolean,
    Number,
    String,
    Function,
    Property,
    Operator,
    Comment,
}

#[derive(Clone, Copy, Debug)]
struct HighlightSpan {
    start: usize,
    end: usize,
    kind: HighlightKind,
}

pub struct ScriptDocument {
    pub path: PathBuf,
    pub source: String,
    pub original_source: String,
    pub dirty: bool,
    pub diagnostics: Diagnostics,
    pub program: Option<Program>,
    pub source_map: SourceMap,

    highlight_spans: Vec<HighlightSpan>,
}

impl ScriptDocument {
    pub fn new(path: PathBuf, source: String) -> Self {
        let source_map = SourceMap::new(&source);

        let mut document = Self {
            path,
            original_source: source.clone(),
            source,
            dirty: false,
            diagnostics: Diagnostics::new(),
            program: None,
            source_map,
            highlight_spans: Vec::new(),
        };

        document.reparse();
        document
    }

    pub fn reparse(&mut self) {
        self.diagnostics = Diagnostics::new();
        self.source_map = SourceMap::new(&self.source);
        self.highlight_spans.clear();

        let file_name = self
            .path
            .file_name()
            .map(|name| name.to_string_lossy().into_owned());

        match Lexer::new(&self.source).tokenize() {
            Ok(tokens) => {
                self.highlight_spans = build_highlight_spans(&self.source, &tokens);

                match Parser::new(tokens).parse() {
                    Ok(program) => {
                        self.program = Some(program);
                    }

                    Err(error) => {
                        self.program = None;

                        self.diagnostics.error(
                            error.message,
                            SourceLocation::new(file_name, error.span),
                        );
                    }
                }
            }

            Err(error) => {
                self.program = None;

                let offset = self
                    .source_map
                    .line_start(error.line)
                    .unwrap_or(0)
                    .saturating_add((error.column as u32).saturating_sub(1));

                let span = crate::scripting::source::SourceSpan::single(offset);

                self.diagnostics.error(
                    error.message,
                    SourceLocation::new(file_name, span),
                );
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

        let Some(root) = project_path else {
            return;
        };

        let scripts_dir = root.join("scripts");

        if let Ok(entries) = std::fs::read_dir(scripts_dir) {
            for entry in entries.flatten() {
                let path = entry.path();

                if path.is_file()
                    && path
                        .extension()
                        .is_some_and(|extension| extension == "aeo")
                {
                    self.scripts_list.push(path);
                }
            }
        }

        self.scripts_list.sort();
    }

    pub fn open_file(&mut self, path: PathBuf) {
        if self.open_documents.contains_key(&path) {
            self.active_document = Some(path);
            return;
        }

        match std::fs::read_to_string(&path) {
            Ok(source) => {
                let document = ScriptDocument::new(path.clone(), source);

                self.open_documents.insert(path.clone(), document);
                self.active_document = Some(path);
            }

            Err(error) => {
                self.output_log.push(format!(
                    "Failed to open {:?}: {}",
                    path, error
                ));
            }
        }
    }

    fn next_open_document(&self) -> Option<PathBuf> {
        self.scripts_list
            .iter()
            .find(|path| self.open_documents.contains_key(*path))
            .cloned()
            .or_else(|| self.open_documents.keys().next().cloned())
    }

    pub fn close_document(&mut self, path: &PathBuf) -> bool {
        if self
            .open_documents
            .get(path)
            .is_some_and(|document| document.dirty)
        {
            return false;
        }

        self.open_documents.remove(path);

        if self.active_document.as_ref() == Some(path) {
            self.active_document = self.next_open_document();
        }

        true
    }

    pub fn save_active(&mut self) {
        let Some(path) = self.active_document.clone() else {
            return;
        };

        let Some(document) = self.open_documents.get_mut(&path) else {
            return;
        };

        if let Err(error) = document.save() {
            self.output_log.push(format!(
                "Failed to save {:?}: {}",
                path, error
            ));
        }
    }

    pub fn save_all(&mut self) {
        for document in self.open_documents.values_mut() {
            if !document.dirty {
                continue;
            }

            if let Err(error) = document.save() {
                self.output_log.push(format!(
                    "Failed to save {:?}: {}",
                    document.path, error
                ));
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

        if !valid_script_filename(&filename) {
            self.output_log
                .push("Create failed: Invalid characters in filename.".to_string());
            return;
        }

        let scripts_dir = project_path.join("scripts");

        if let Err(error) = std::fs::create_dir_all(&scripts_dir) {
            self.output_log.push(format!(
                "Failed to create scripts directory: {}",
                error
            ));
            return;
        }

        let path = scripts_dir.join(&filename);

        if path.exists() {
            self.output_log
                .push(format!("File already exists: {:?}", path));
            return;
        }

        let entity_name = sanitize_entity_name(name);

        let template = format!(
            "entity {} {{\n\
\n\
    fn on_spawn() {{\n\
    }}\n\
\n\
    fn on_ready() {{\n\
    }}\n\
\n\
    fn update(dt: number) {{\n\
    }}\n\
\n\
    fn on_destroy() {{\n\
    }}\n\
}}\n",
            entity_name
        );

        match std::fs::write(&path, template) {
            Ok(()) => {
                self.refresh_scripts(&Some(project_path.clone()));
                self.open_file(path);
            }

            Err(error) => {
                self.output_log
                    .push(format!("Failed to create script: {}", error));
            }
        }
    }

    pub fn rename_script(
        &mut self,
        project_path: &PathBuf,
        old_path: PathBuf,
        new_name: &str,
    ) {
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

        if !valid_script_filename(&filename) {
            self.output_log
                .push("Rename failed: Invalid characters in filename.".to_string());
            return;
        }

        let new_path = project_path.join("scripts").join(&filename);

        if new_path.exists() {
            self.output_log.push(format!(
                "Rename failed: {:?} already exists.",
                new_path
            ));
            return;
        }

        match std::fs::rename(&old_path, &new_path) {
            Ok(()) => {
                if let Some(mut document) = self.open_documents.remove(&old_path) {
                    document.path = new_path.clone();
                    self.open_documents.insert(new_path.clone(), document);
                }

                if self.active_document.as_ref() == Some(&old_path) {
                    self.active_document = Some(new_path);
                }

                self.refresh_scripts(&Some(project_path.clone()));

                self.output_log
                    .push(format!("Renamed to {:?}", filename));
            }

            Err(error) => {
                self.output_log
                    .push(format!("Rename failed: {}", error));
            }
        }
    }

    pub fn delete_script(
        &mut self,
        project_path: &Option<PathBuf>,
        path: PathBuf,
    ) {
        match std::fs::remove_file(&path) {
            Ok(()) => {
                self.open_documents.remove(&path);

                if self.active_document.as_ref() == Some(&path) {
                    self.active_document = self.next_open_document();
                }

                self.refresh_scripts(project_path);

                self.output_log.push(format!(
                    "Deleted {:?}",
                    path.file_name().unwrap_or_default()
                ));
            }

            Err(error) => {
                self.output_log
                    .push(format!("Delete failed: {}", error));
            }
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

        if cancel {
            self.closing_path = None;

            if is_deletion {
                self.deleting_path = None;
            }

            return;
        }

        if save {
            let Some(document) = self.open_documents.get_mut(&path) else {
                self.closing_path = None;

                if is_deletion {
                    self.deleting_path = None;
                }

                return;
            };

            if let Err(error) = document.save() {
                self.output_log.push(format!(
                    "Save failed: {}. {} aborted.",
                    error,
                    if is_deletion {
                        "Deletion"
                    } else {
                        "Closure"
                    }
                ));
                return;
            }
        }

        if is_deletion {
            self.delete_script(project_path, path);
            self.deleting_path = None;
        } else {
            self.open_documents.remove(&path);

            if self.active_document.as_ref() == Some(&path) {
                self.active_document = self.next_open_document();
            }
        }

        self.closing_path = None;
    }

    pub fn perform_search(&mut self) {
        self.search_results.clear();

        let query = self.search_query.trim();

        if query.is_empty() {
            return;
        }

        for path in &self.scripts_list {
            let source = match self.open_documents.get(path) {
                Some(document) => document.source.clone(),
                None => match std::fs::read_to_string(path) {
                    Ok(source) => source,
                    Err(_) => continue,
                },
            };

            for (index, line) in source.lines().enumerate() {
                if line.contains(query) {
                    self.search_results.push(SearchResult {
                        path: path.clone(),
                        line: index + 1,
                        text: line.to_string(),
                    });
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

            self.draw_workspace(ui, project_path, world);
        });

        self.draw_new_script_dialog(ctx, project_path);
        self.draw_rename_dialog(ctx, project_path);
        self.draw_delete_dialog(ctx, project_path);
        self.draw_dirty_dialog(ctx, project_path);
    }

    fn draw_new_script_dialog(
        &mut self,
        ctx: &egui::Context,
        project_path: &Option<PathBuf>,
    ) {
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
                            self.create_new_script(
                                root,
                                &self.new_script_name.clone(),
                            );
                            self.show_new_script_dialog = false;
                            self.new_script_name.clear();
                        }
                    }

                    if ui.button("Cancel").clicked()
                        || ui.input(|input| {
                            input.key_pressed(egui::Key::Escape)
                        })
                    {
                        self.show_new_script_dialog = false;
                        self.new_script_name.clear();
                    }
                });
            });
    }

    fn draw_rename_dialog(
        &mut self,
        ctx: &egui::Context,
        project_path: &Option<PathBuf>,
    ) {
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
                        if let (Some(root), Some(target)) = (
                            project_path,
                            self.rename_target.clone(),
                        ) {
                            self.rename_script(
                                root,
                                target,
                                &self.rename_new_name.clone(),
                            );

                            self.show_rename_dialog = false;
                            self.rename_new_name.clear();
                            self.rename_target = None;
                        }
                    }

                    if ui.button("Cancel").clicked()
                        || ui.input(|input| {
                            input.key_pressed(egui::Key::Escape)
                        })
                    {
                        self.show_rename_dialog = false;
                        self.rename_new_name.clear();
                        self.rename_target = None;
                    }
                });
            });
    }

    fn draw_delete_dialog(
        &mut self,
        ctx: &egui::Context,
        project_path: &Option<PathBuf>,
    ) {
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
                        || ui.input(|input| {
                            input.key_pressed(egui::Key::Escape)
                        })
                    {
                        self.deleting_path = None;
                    }
                });
            });
    }

    fn draw_dirty_dialog(
        &mut self,
        ctx: &egui::Context,
        project_path: &Option<PathBuf>,
    ) {
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
                path.file_name()
                    .unwrap_or_default()
                    .to_string_lossy(),
                if is_deletion {
                    "deleting"
                } else {
                    "closing"
                }
            ));

            ui.add_space(10.0);

            ui.horizontal(|ui| {
                if ui.button("Save").clicked() {
                    self.apply_dirty_response(
                        project_path,
                        path.clone(),
                        true,
                        false,
                    );
                }

                if ui.button("Don't Save").clicked() {
                    self.apply_dirty_response(
                        project_path,
                        path.clone(),
                        false,
                        false,
                    );
                }

                if ui.button("Cancel").clicked()
                    || ui.input(|input| {
                        input.key_pressed(egui::Key::Escape)
                    })
                {
                    self.apply_dirty_response(
                        project_path,
                        path.clone(),
                        false,
                        true,
                    );
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

            let bottom_panel_height =
                if (self.show_problems && has_problems)
                    || (self.show_output && !self.output_log.is_empty())
                {
                    bottom_panel_expanded_height
                } else {
                    bottom_panel_min_height
                };

            let main_height =
                (ui.available_height() - bottom_panel_height - 10.0).max(100.0);

            ui.horizontal(|ui| {
                let explorer_width = 230.0;

                ui.allocate_ui(
                    egui::vec2(explorer_width, main_height),
                    |ui| {
                        egui::ScrollArea::vertical()
                            .id_salt("explorer_scroll")
                            .show(ui, |ui| {
                                if self.show_search
                                    && !self.search_results.is_empty()
                                {
                                    ui.label(
                                        RichText::new("SEARCH RESULTS").strong(),
                                    );
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

                                        if ui
                                            .selectable_label(false, label)
                                            .clicked()
                                        {
                                            action_open =
                                                Some(result.path.clone());
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
                                            let filename = path
                                                .file_name()
                                                .unwrap_or_default()
                                                .to_string_lossy();

                                            let active =
                                                self.active_document
                                                    .as_ref()
                                                    == Some(path);

                                            let dirty = self
                                                .open_documents
                                                .get(path)
                                                .is_some_and(
                                                    |document| document.dirty,
                                                );

                                            let label = if dirty {
                                                format!("{} *", filename)
                                            } else {
                                                filename.to_string()
                                            };

                                            ui.horizontal(|ui| {
                                                ui.with_layout(
                                                    egui::Layout::right_to_left(
                                                        egui::Align::Center,
                                                    ),
                                                    |ui| {
                                                        if ui
                                                            .small_button("")
                                                            .on_hover_text(
                                                                "Delete script from disk",
                                                            )
                                                            .clicked()
                                                        {
                                                            self.deleting_path =
                                                                Some(path.clone());
                                                        }

                                                        if ui
                                                            .small_button("📝")
                                                            .on_hover_text(
                                                                "Rename script",
                                                            )
                                                            .clicked()
                                                        {
                                                            self.show_rename_dialog =
                                                                true;
                                                            self.rename_target =
                                                                Some(path.clone());
                                                            self.rename_new_name =
                                                                path.file_stem()
                                                                    .unwrap_or_default()
                                                                    .to_string_lossy()
                                                                    .to_string();
                                                        }

                                                        if self
                                                            .open_documents
                                                            .contains_key(path)
                                                            && ui
                                                                .small_button("❌")
                                                                .on_hover_text(
                                                                    "Close document",
                                                                )
                                                                .clicked()
                                                        {
                                                            action_close =
                                                                Some(path.clone());
                                                        }

                                                        ui.with_layout(
                                                            egui::Layout::left_to_right(
                                                                egui::Align::Center,
                                                            ),
                                                            |ui| {
                                                                ui.style_mut()
                                                                    .wrap_mode =
                                                                    Some(
                                                                        egui::TextWrapMode::Truncate,
                                                                    );

                                                                if ui
                                                                    .selectable_label(
                                                                        active,
                                                                        label,
                                                                    )
                                                                    .clicked()
                                                                {
                                                                    action_open =
                                                                        Some(path.clone());
                                                                }
                                                            },
                                                        );
                                                    },
                                                );
                                            });
                                        }
                                    });
                            });
                    },
                );

                ui.separator();

                let inspector_width = 240.0;
                let editor_width =
                    (ui.available_width() - inspector_width - 20.0)
                        .max(100.0);

                ui.allocate_ui(
                    egui::vec2(editor_width, main_height),
                    |ui| {
                        self.draw_editor(ui);
                    },
                );

                ui.separator();

                ui.allocate_ui(
                    egui::vec2(inspector_width, main_height),
                    |ui| {
                        egui::ScrollArea::vertical()
                            .id_salt("inspector_scroll")
                            .show(ui, |ui| {
                                self.draw_inspector(
                                    ui,
                                    world,
                                    project_path,
                                );
                            });
                    },
                );
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
                                .and_then(|path| {
                                    self.open_documents
                                        .get(path)
                                })
                                .map_or(0, |document| {
                                    document.diagnostics.len()
                                });

                            let problem_label = if has_problems {
                                format!("PROBLEMS ({})", problem_count)
                            } else {
                                "PROBLEMS".to_string()
                            };

                            if ui
                                .selectable_label(
                                    self.show_problems,
                                    problem_label,
                                )
                                .clicked()
                            {
                                self.show_problems = true;
                                self.show_output = false;
                            }

                            if ui
                                .selectable_label(
                                    self.show_output,
                                    "OUTPUT",
                                )
                                .clicked()
                            {
                                self.show_output = true;
                                self.show_problems = false;
                            }

                            if !has_problems
                                && self.output_log.is_empty()
                            {
                                ui.with_layout(
                                    egui::Layout::right_to_left(
                                        egui::Align::Center,
                                    ),
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
                let filename = path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy();

                ui.label(RichText::new(filename).strong());

                if document.dirty {
                    ui.label(
                        RichText::new("(modified)")
                            .italics()
                            .color(Color32::YELLOW),
                    );
                }
            });

            let available_height = ui.available_height();

            egui::ScrollArea::vertical()
                .id_salt("editor_v_scroll")
                .max_height(available_height)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        let line_count =
                            document.source.split('\n').count().max(1);

                        let gutter_width =
                            (line_count.to_string().len() as f32 * 8.0)
                                .max(24.0);

                        ui.allocate_ui(
                            egui::vec2(
                                gutter_width,
                                ui.available_height(),
                            ),
                            |ui| {
                                let mut gutter = String::new();

                                for line in 1..=line_count {
                                    gutter.push_str(&format!("{}\n", line));
                                }

                                ui.add(
                                    egui::Label::new(
                                        RichText::new(gutter)
                                            .monospace()
                                            .color(
                                                Color32::from_gray(120),
                                            )
                                            .size(12.0),
                                    ),
                                );
                            },
                        );

                        egui::ScrollArea::horizontal()
                            .id_salt("editor_h_scroll")
                            .show(ui, |ui| {
                                ui.push_id(&path, |ui| {
                                    let highlight_spans =
                                        document.highlight_spans.clone();

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
                                        egui::TextEdit::multiline(
                                            &mut document.source,
                                        )
                                        .font(egui::TextStyle::Monospace)
                                        .code_editor()
                                        .desired_width(f32::INFINITY)
                                        .desired_rows(line_count)
                                        .lock_focus(true)
                                        .layouter(&mut layouter),
                                    );

                                    let mut clear_cursor_request = false;

                                    if let Some((
                                        request_path,
                                        byte_offset,
                                    )) = &self.cursor_request
                                    {
                                        if request_path == &path {
                                            response.request_focus();

                                            let mut state =
                                                egui::text_edit::TextEditState::load(
                                                    ui.ctx(),
                                                    response.id,
                                                )
                                                .unwrap_or_default();

                                            let char_offset =
                                                byte_offset_to_char_index(
                                                    &document.source,
                                                    *byte_offset,
                                                );

                                            state.cursor.set_char_range(
                                                Some(
                                                    egui::text::CCursorRange::one(
                                                        egui::text::CCursor::new(
                                                            char_offset,
                                                        ),
                                                    ),
                                                ),
                                            );

                                            state.store(
                                                ui.ctx(),
                                                response.id,
                                            );

                                            clear_cursor_request = true;
                                        }
                                    }

                                    if clear_cursor_request {
                                        self.cursor_request = None;
                                    }

                                    if response.changed() {
                                        document.dirty =
                                            document.source
                                                != document.original_source;

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
                    path.file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
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

                let mut enabled =
                    !world.disabled_scripts.contains(&relative_path);

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
                    } else if !world
                        .disabled_scripts
                        .contains(&relative_path)
                    {
                        world.disabled_scripts.push(relative_path);
                    }
                }
            });

            ui.add_space(8.0);
            ui.label(RichText::new("ENTITIES").strong());

            ui.indent("entities_info", |ui| {
                let Some(program) = &document.program else {
                    ui.label(
                        RichText::new("(parse failed)")
                            .color(Color32::RED),
                    );
                    return;
                };

                for declaration in &program.declarations {
                    if let crate::scripting::ast::Declaration::Entity(
                        entity,
                    ) = declaration
                    {
                        ui.label(RichText::new(&entity.name).strong());

                        ui.indent("lifecycle", |ui| {
                            for member in &entity.members {
                                let crate::scripting::ast::EntityMember::Function(
                                    function,
                                ) = member
                                else {
                                    continue;
                                };

                                let lifecycle = matches!(
                                    function.name.as_str(),
                                    "on_spawn"
                                        | "on_ready"
                                        | "update"
                                        | "on_destroy"
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
        ui.style_mut().wrap_mode =
            Some(egui::TextWrapMode::Wrap);

        let mut click_action = None;

        if let Some(path) = &self.active_document {
            if let Some(document) = self.open_documents.get(path) {
                let mut diagnostics = Vec::new();

                for diagnostic in document.diagnostics.as_slice() {
                    let (start, _) =
                        document
                            .source_map
                            .span_positions(diagnostic.location.span);

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
                        for (
                            severity,
                            file,
                            line,
                            column,
                            message,
                            offset,
                        ) in diagnostics
                        {
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
                                ui.colored_label(
                                    color,
                                    format!("{:?}", severity),
                                );

                                let message = format!(
                                    "{}:{}:{}  {}",
                                    file.as_deref().unwrap_or("unknown"),
                                    line,
                                    column,
                                    message
                                );

                                if ui
                                    .selectable_label(false, message)
                                    .clicked()
                                {
                                    if let Some(file_name) = &file {
                                        if let Some(path) = self
                                            .scripts_list
                                            .iter()
                                            .find(|path| {
                                                path.file_name().map_or(
                                                    false,
                                                    |name| {
                                                        name.to_string_lossy()
                                                            == *file_name
                                                    },
                                                )
                                            })
                                            .cloned()
                                        {
                                            click_action =
                                                Some((path, offset));
                                        }
                                    } else if let Some(path) =
                                        &self.active_document
                                    {
                                        click_action =
                                            Some((path.clone(), offset));
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
        ui.style_mut().wrap_mode =
            Some(egui::TextWrapMode::Wrap);

        egui::ScrollArea::vertical()
            .id_salt("output_scroll")
            .show(ui, |ui| {
                for line in &self.output_log {
                    ui.label(line);
                }
            });
    }
}

fn valid_script_filename(filename: &str) -> bool {
    filename
        .chars()
        .all(|character| {
            !matches!(
                character,
                '\\'
                    | '/'
                    | ':'
                    | '*'
                    | '?'
                    | '"'
                    | '<'
                    | '>'
                    | '|'
            )
        })
}

fn sanitize_entity_name(name: &str) -> String {
    let base = name
        .strip_suffix(".aeo")
        .unwrap_or(name)
        .split('.')
        .next()
        .unwrap_or("MyScript");

    let mut result = base
        .chars()
        .filter(|character| {
            character.is_ascii_alphanumeric() || *character == '_'
        })
        .collect::<String>();

    if result.is_empty() {
        result = "MyScript".to_string();
    }

    if result
        .chars()
        .next()
        .is_some_and(|character| character.is_ascii_digit())
    {
        result.insert(0, '_');
    }

    result
}

fn build_highlight_spans(
    source: &str,
    tokens: &[Token],
) -> Vec<HighlightSpan> {
    let mut spans = Vec::with_capacity(tokens.len());

    for token in tokens {
        let start = token.span.start as usize;
        let end = token.span.end as usize;

        if start >= end || end > source.len() {
            continue;
        }

        let Some(kind) =
            highlight_kind(source, start, end, &token.kind)
        else {
            continue;
        };

        spans.push(HighlightSpan {
            start,
            end,
            kind,
        });
    }

    for (start, end) in find_comment_spans(source) {
        spans.push(HighlightSpan {
            start,
            end,
            kind: HighlightKind::Comment,
        });
    }

    spans.sort_by_key(|span| (span.start, span.end));
    spans
}

fn highlight_kind(
    source: &str,
    start: usize,
    end: usize,
    token: &TokenKind,
) -> Option<HighlightKind> {
    match token {
        TokenKind::Entity
        | TokenKind::Fn
        | TokenKind::Return
        | TokenKind::If
        | TokenKind::Else
        | TokenKind::While
        | TokenKind::For
        | TokenKind::In
        | TokenKind::Const
        | TokenKind::Import
        | TokenKind::On => Some(HighlightKind::Keyword),

        TokenKind::True | TokenKind::False | TokenKind::Nil => {
            Some(HighlightKind::Boolean)
        }

        TokenKind::Number(_) => Some(HighlightKind::Number),

        TokenKind::String(_) => Some(HighlightKind::String),

        TokenKind::Identifier(name) if matches!(
            name.as_str(),
            "number" | "string" | "bool"
        ) =>
        {
            Some(HighlightKind::Type)
        }

        TokenKind::Identifier(_) => {
            if previous_non_whitespace(source, start) == Some('.') {
                Some(HighlightKind::Property)
            } else if next_non_whitespace(source, end) == Some('(') {
                Some(HighlightKind::Function)
            } else {
                None
            }
        }

        TokenKind::Plus
        | TokenKind::Minus
        | TokenKind::Star
        | TokenKind::Slash
        | TokenKind::Percent
        | TokenKind::Equal
        | TokenKind::EqualEqual
        | TokenKind::Bang
        | TokenKind::BangEqual
        | TokenKind::Less
        | TokenKind::LessEqual
        | TokenKind::Greater
        | TokenKind::GreaterEqual
        | TokenKind::AndAnd
        | TokenKind::OrOr
        | TokenKind::PlusEqual
        | TokenKind::MinusEqual
        | TokenKind::StarEqual
        | TokenKind::SlashEqual => Some(HighlightKind::Operator),

        _ => None,
    }
}

fn highlight_color(
    kind: HighlightKind,
    base: Color32,
) -> Color32 {
    match kind {
        HighlightKind::Keyword => {
            Color32::from_rgb(198, 120, 221)
        }

        HighlightKind::Type => {
            Color32::from_rgb(86, 182, 194)
        }

        HighlightKind::Boolean => {
            Color32::from_rgb(86, 156, 214)
        }

        HighlightKind::Number => {
            Color32::from_rgb(209, 154, 102)
        }

        HighlightKind::String => {
            Color32::from_rgb(152, 195, 121)
        }

        HighlightKind::Function => {
            Color32::from_rgb(97, 175, 239)
        }

        HighlightKind::Property => {
            Color32::from_rgb(220, 220, 170)
        }

        HighlightKind::Operator => {
            Color32::from_rgb(180, 180, 180)
        }

        HighlightKind::Comment => {
            Color32::from_rgb(106, 153, 85)
        }
    }
}

fn build_layout_job(
    ui: &egui::Ui,
    source: &str,
    spans: &[HighlightSpan],
    wrap_width: f32,
) -> LayoutJob {
    let font_id =
        egui::TextStyle::Monospace.resolve(ui.style());
    let base_color = ui.visuals().text_color();

    let mut job = LayoutJob::default();

    job.break_on_newline = true;
    job.wrap.max_width = wrap_width;

    let mut cursor = 0;

    for span in spans {
        if span.start > source.len()
            || span.end > source.len()
            || span.start < cursor
        {
            continue;
        }

        if span.start > cursor {
            job.append(
                &source[cursor..span.start],
                0.0,
                TextFormat::simple(
                    font_id.clone(),
                    base_color,
                ),
            );
        }

        let mut format = TextFormat::simple(
            font_id.clone(),
            highlight_color(span.kind, base_color),
        );

        if span.kind == HighlightKind::Comment {
            format.italics = true;
        }

        job.append(
            &source[span.start..span.end],
            0.0,
            format,
        );

        cursor = span.end;
    }

    if cursor < source.len() {
        job.append(
            &source[cursor..],
            0.0,
            TextFormat::simple(font_id, base_color),
        );
    }

    job
}

fn find_comment_spans(source: &str) -> Vec<(usize, usize)> {
    let bytes = source.as_bytes();
    let mut spans = Vec::new();

    let mut index = 0;
    let mut in_string = false;
    let mut escaped = false;

    while index < bytes.len() {
        let byte = bytes[index];

        if in_string {
            if escaped {
                escaped = false;
                index += 1;
                continue;
            }

            if byte == b'\\' {
                escaped = true;
                index += 1;
                continue;
            }

            if byte == b'"' {
                in_string = false;
            }

            index += 1;
            continue;
        }

        if byte == b'"' {
            in_string = true;
            index += 1;
            continue;
        }

        if byte == b'/'
            && index + 1 < bytes.len()
            && bytes[index + 1] == b'/'
        {
            let start = index;

            index += 2;

            while index < bytes.len()
                && bytes[index] != b'\n'
            {
                index += 1;
            }

            spans.push((start, index));
            continue;
        }

        index += 1;
    }

    spans
}

fn previous_non_whitespace(
    source: &str,
    offset: usize,
) -> Option<char> {
    source
        .get(..offset)
        .and_then(|prefix| {
            prefix
                .chars()
                .rev()
                .find(|character| !character.is_whitespace())
        })
}

fn next_non_whitespace(
    source: &str,
    offset: usize,
) -> Option<char> {
    source
        .get(offset..)
        .and_then(|suffix| {
            suffix
                .chars()
                .find(|character| !character.is_whitespace())
        })
}

fn byte_offset_to_char_index(
    source: &str,
    byte_offset: usize,
) -> usize {
    source
        .get(..byte_offset.min(source.len()))
        .map(|prefix| prefix.chars().count())
        .unwrap_or_else(|| source.chars().count())
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
        fs::write(scripts_dir.join("ignored.txt"), "").unwrap();

        let mut editor = ScriptEditor::new();
        editor.refresh_scripts(&Some(test_dir.clone()));

        assert_eq!(editor.scripts_list.len(), 3);
        assert_eq!(
            editor.scripts_list[0].file_name().unwrap(),
            "a.aeo"
        );
        assert_eq!(
            editor.scripts_list[1].file_name().unwrap(),
            "m.aeo"
        );
        assert_eq!(
            editor.scripts_list[2].file_name().unwrap(),
            "z.aeo"
        );

        fs::remove_dir_all(&test_dir).ok();
    }

    #[test]
    fn test_script_selection_opens_document() {
        let test_dir = PathBuf::from("TestProject_Open");
        let scripts_dir = test_dir.join("scripts");

        if test_dir.exists() {
            fs::remove_dir_all(&test_dir).ok();
        }

        fs::create_dir_all(&scripts_dir).unwrap();

        let path = scripts_dir.join("open_test.aeo");

        fs::write(&path, "script content").unwrap();

        let mut editor = ScriptEditor::new();

        editor.refresh_scripts(&Some(test_dir.clone()));
        editor.open_file(path.clone());

        assert_eq!(
            editor.active_document,
            Some(path.clone())
        );
        assert!(
            editor.open_documents.contains_key(&path)
        );
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

        let mut document =
            ScriptDocument::new(path.clone(), "initial".to_string());

        assert!(!document.dirty);

        document.source = "changed".to_string();
        document.dirty = true;

        document.save().expect("Save should succeed");

        assert!(!document.dirty);
        assert_eq!(
            fs::read_to_string(&path).unwrap(),
            "changed"
        );

        fs::remove_dir_all(&test_dir).ok();
    }

    #[test]
    fn test_diagnostics_integration() {
        let source = "entity {";

        let document = ScriptDocument::new(
            PathBuf::from("error.aeo"),
            source.to_string(),
        );

        assert!(!document.diagnostics.is_empty());
        assert!(document.diagnostics.has_errors());

        let diagnostic =
            &document.diagnostics.as_slice()[0];

        assert_eq!(
            diagnostic.severity,
            crate::scripting::diagnostic::DiagnosticSeverity::Error
        );

        assert!(diagnostic.message.contains("Expected"));
    }

    #[test]
    fn test_new_script_template() {
        let test_dir =
            PathBuf::from("TestProject_Template");
        let scripts_dir = test_dir.join("scripts");

        if test_dir.exists() {
            fs::remove_dir_all(&test_dir).ok();
        }

        fs::create_dir_all(&scripts_dir).unwrap();

        let mut editor = ScriptEditor::new();

        editor.create_new_script(
            &test_dir,
            "PlayerScript",
        );

        let path =
            scripts_dir.join("PlayerScript.aeo");

        assert!(path.exists());

        let content =
            fs::read_to_string(&path).unwrap();

        assert!(content.contains("entity PlayerScript"));
        assert!(content.contains("fn on_spawn()"));

        let document =
            ScriptDocument::new(path, content);

        assert!(!document.diagnostics.has_errors());

        fs::remove_dir_all(&test_dir).ok();
    }

    #[test]
    fn test_search_results() {
        let test_dir =
            PathBuf::from("TestProject_Search");
        let scripts_dir = test_dir.join("scripts");

        if test_dir.exists() {
            fs::remove_dir_all(&test_dir).ok();
        }

        fs::create_dir_all(&scripts_dir).unwrap();

        fs::write(
            scripts_dir.join("one.aeo"),
            "hello world",
        )
        .unwrap();

        fs::write(
            scripts_dir.join("two.aeo"),
            "goodbye",
        )
        .unwrap();

        fs::write(
            scripts_dir.join("three.txt"),
            "hello hidden",
        )
        .unwrap();

        let mut editor = ScriptEditor::new();

        editor.refresh_scripts(&Some(test_dir.clone()));

        editor.search_query = "hello".to_string();
        editor.perform_search();

        assert_eq!(
            editor.search_results.len(),
            1
        );

        assert_eq!(
            editor.search_results[0]
                .path
                .file_name()
                .unwrap(),
            "one.aeo"
        );

        fs::remove_dir_all(&test_dir).ok();
    }

    #[test]
    fn test_inspector_metadata_updates_after_edit() {
        let source =
            "entity test { fn on_spawn() {} fn on_destroy() {} }";

        let mut document = ScriptDocument::new(
            PathBuf::from("test.aeo"),
            source.to_string(),
        );

        let mut has_spawn = false;
        let mut has_destroy = false;

        if let Some(program) = &document.program {
            if let crate::scripting::ast::Declaration::Entity(
                entity,
            ) = &program.declarations[0]
            {
                for member in &entity.members {
                    if let crate::scripting::ast::EntityMember::Function(
                        function,
                    ) = member
                    {
                        if function.name == "on_spawn" {
                            has_spawn = true;
                        }

                        if function.name == "on_destroy" {
                            has_destroy = true;
                        }
                    }
                }
            }
        }

        assert!(has_spawn);
        assert!(has_destroy);

        document.source =
            "entity test { fn on_spawn() {} }".to_string();

        document.reparse();

        let mut has_spawn = false;
        let mut has_destroy = false;

        if let Some(program) = &document.program {
            if let crate::scripting::ast::Declaration::Entity(
                entity,
            ) = &program.declarations[0]
            {
                for member in &entity.members {
                    if let crate::scripting::ast::EntityMember::Function(
                        function,
                    ) = member
                    {
                        if function.name == "on_spawn" {
                            has_spawn = true;
                        }

                        if function.name == "on_destroy" {
                            has_destroy = true;
                        }
                    }
                }
            }
        }

        assert!(has_spawn);
        assert!(!has_destroy);
    }

    #[test]
    fn test_delete_dirty_save_failure_aborts_deletion() {
        let test_dir =
            PathBuf::from("TestProject_DeleteFailure");
        let scripts_dir = test_dir.join("scripts");

        if test_dir.exists() {
            fs::remove_dir_all(&test_dir).ok();
        }

        fs::create_dir_all(&scripts_dir).unwrap();

        let path = scripts_dir.join("bug.aeo");

        fs::write(&path, "initial content").unwrap();

        let mut editor = ScriptEditor::new();

        let mut document = ScriptDocument::new(
            path.clone(),
            "initial content".to_string(),
        );

        document.source = "dirty content".to_string();
        document.dirty = true;

        editor
            .open_documents
            .insert(path.clone(), document);

        editor.deleting_path = Some(path.clone());
        editor.closing_path = Some(path.clone());

        fs::remove_file(&path).unwrap();
        fs::create_dir(&path).unwrap();

        editor.apply_dirty_response(
            &Some(test_dir.clone()),
            path.clone(),
            true,
            false,
        );

        assert!(path.exists());
        assert!(
            editor.open_documents.contains_key(&path)
        );

        assert_eq!(
            editor.closing_path,
            Some(path.clone())
        );

        assert_eq!(
            editor.deleting_path,
            Some(path.clone())
        );

        assert!(
            editor
                .output_log
                .iter()
                .any(|line| line.contains("Save failed"))
        );

        fs::remove_dir_all(&test_dir).ok();
    }

    #[test]
    fn test_highlighting_detects_aeoscript_tokens() {
        let source = r#"
entity Player {
    fn update(dt: number) {
        const speed = 4.0
        player.sound.play()
        // comment
    }
}
"#;

        let tokens =
            Lexer::new(source).tokenize().unwrap();

        let spans = build_highlight_spans(
            source,
            &tokens,
        );

        assert!(spans.iter().any(|span| {
            span.kind == HighlightKind::Keyword
        }));

        assert!(spans.iter().any(|span| {
            span.kind == HighlightKind::Type
        }));

        assert!(spans.iter().any(|span| {
            span.kind == HighlightKind::Number
        }));

        assert!(spans.iter().any(|span| {
            span.kind == HighlightKind::Function
        }));

        assert!(spans.iter().any(|span| {
            span.kind == HighlightKind::Property
        }));

        assert!(spans.iter().any(|span| {
            span.kind == HighlightKind::Comment
        }));
    }

    #[test]
    fn test_byte_offset_to_character_index() {
        let source = "éPlayer";

        assert_eq!(
            byte_offset_to_char_index(source, 0),
            0
        );

        assert_eq!(
            byte_offset_to_char_index(source, 2),
            1
        );

        assert_eq!(
            byte_offset_to_char_index(source, source.len()),
            7
        );
    }

    #[test]
    fn test_entity_name_sanitization() {
        assert_eq!(
            sanitize_entity_name("PlayerScript"),
            "PlayerScript"
        );

        assert_eq!(
            sanitize_entity_name("Player Script"),
            "PlayerScript"
        );

        assert_eq!(
            sanitize_entity_name("123Door"),
            "_123Door"
        );

        assert_eq!(
            sanitize_entity_name(""),
            "MyScript"
        );
    }
}