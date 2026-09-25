pub mod script_editor_documents;
pub mod script_editor_files;
pub mod script_editor_highlighting;
pub mod script_editor_ui;

pub use script_editor_documents::ScriptDocument;
pub use script_editor_ui::SearchResult;

use std::collections::HashMap;
use std::path::PathBuf;

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
}

#[cfg(test)]
mod tests {
    use super::script_editor_files::sanitize_entity_name;
    use super::script_editor_highlighting::{
        HighlightKind, build_highlight_spans, byte_offset_to_char_index,
    };
    use super::*;
    use crate::scripting::lexer::Lexer;
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
        assert_eq!(editor.scripts_list[0].file_name().unwrap(), "a.aeo");
        assert_eq!(editor.scripts_list[1].file_name().unwrap(), "m.aeo");
        assert_eq!(editor.scripts_list[2].file_name().unwrap(), "z.aeo");

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

        let mut document = ScriptDocument::new(path.clone(), "initial".to_string());

        assert!(!document.dirty);

        document.source = "changed".to_string();
        document.dirty = true;

        document.save().expect("Save should succeed");

        assert!(!document.dirty);
        assert_eq!(fs::read_to_string(&path).unwrap(), "changed");

        fs::remove_dir_all(&test_dir).ok();
    }

    #[test]
    fn test_diagnostics_integration() {
        let source = "entity {";

        let document = ScriptDocument::new(PathBuf::from("error.aeo"), source.to_string());

        assert!(!document.diagnostics.is_empty());
        assert!(document.diagnostics.has_errors());

        let diagnostic = &document.diagnostics.as_slice()[0];

        assert_eq!(
            diagnostic.severity,
            crate::scripting::diagnostic::DiagnosticSeverity::Error
        );

        assert!(diagnostic.message.contains("Expected"));
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

        let document = ScriptDocument::new(path, content);

        assert!(!document.diagnostics.has_errors());

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

        fs::write(scripts_dir.join("three.txt"), "hello hidden").unwrap();

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

        let mut document = ScriptDocument::new(PathBuf::from("test.aeo"), source.to_string());

        let mut has_spawn = false;
        let mut has_destroy = false;

        if let Some(program) = &document.program {
            if let crate::scripting::ast::Declaration::Entity(entity) = &program.declarations[0] {
                for member in &entity.members {
                    if let crate::scripting::ast::EntityMember::Function(function) = member {
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

        document.source = "entity test { fn on_spawn() {} }".to_string();

        document.reparse();

        let mut has_spawn = false;
        let mut has_destroy = false;

        if let Some(program) = &document.program {
            if let crate::scripting::ast::Declaration::Entity(entity) = &program.declarations[0] {
                for member in &entity.members {
                    if let crate::scripting::ast::EntityMember::Function(function) = member {
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
        let test_dir = PathBuf::from("TestProject_DeleteFailure");
        let scripts_dir = test_dir.join("scripts");

        if test_dir.exists() {
            fs::remove_dir_all(&test_dir).ok();
        }

        fs::create_dir_all(&scripts_dir).unwrap();

        let path = scripts_dir.join("bug.aeo");

        fs::write(&path, "initial content").unwrap();

        let mut editor = ScriptEditor::new();

        let mut document = ScriptDocument::new(path.clone(), "initial content".to_string());

        document.source = "dirty content".to_string();
        document.dirty = true;

        editor.open_documents.insert(path.clone(), document);

        editor.deleting_path = Some(path.clone());
        editor.closing_path = Some(path.clone());

        fs::remove_file(&path).unwrap();
        fs::create_dir(&path).unwrap();

        editor.apply_dirty_response(&Some(test_dir.clone()), path.clone(), true, false);

        assert!(path.exists());
        assert!(editor.open_documents.contains_key(&path));

        assert_eq!(editor.closing_path, Some(path.clone()));

        assert_eq!(editor.deleting_path, Some(path.clone()));

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

        let tokens = Lexer::new(source).tokenize().unwrap();

        let spans = build_highlight_spans(source, &tokens);

        assert!(
            spans
                .iter()
                .any(|span| { span.kind == HighlightKind::Keyword })
        );

        assert!(
            spans
                .iter()
                .any(|span| { span.kind == HighlightKind::Type })
        );

        assert!(
            spans
                .iter()
                .any(|span| { span.kind == HighlightKind::Number })
        );

        assert!(
            spans
                .iter()
                .any(|span| { span.kind == HighlightKind::Function })
        );

        assert!(
            spans
                .iter()
                .any(|span| { span.kind == HighlightKind::Property })
        );

        assert!(
            spans
                .iter()
                .any(|span| { span.kind == HighlightKind::Comment })
        );
    }

    #[test]
    fn test_byte_offset_to_character_index() {
        let source = "éPlayer";

        assert_eq!(byte_offset_to_char_index(source, 0), 0);

        assert_eq!(byte_offset_to_char_index(source, 2), 1);

        assert_eq!(byte_offset_to_char_index(source, source.len()), 7);
    }

    #[test]
    fn test_entity_name_sanitization() {
        assert_eq!(sanitize_entity_name("PlayerScript"), "PlayerScript");

        assert_eq!(sanitize_entity_name("Player Script"), "PlayerScript");

        assert_eq!(sanitize_entity_name("123Door"), "_123Door");

        assert_eq!(sanitize_entity_name(""), "MyScript");
    }
}
