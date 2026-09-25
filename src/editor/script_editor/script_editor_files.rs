use super::{ScriptDocument, ScriptEditor, SearchResult};
use std::path::PathBuf;

impl ScriptEditor {
    pub fn refresh_scripts(&mut self, project_path: &Option<PathBuf>) {
        self.scripts_list.clear();

        let Some(root) = project_path else {
            return;
        };

        let scripts_dir = root.join("scripts");

        if let Ok(entries) = std::fs::read_dir(scripts_dir) {
            for entry in entries.flatten() {
                let path = entry.path();

                if path.is_file() && path.extension().is_some_and(|extension| extension == "aeo") {
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
                self.output_log
                    .push(format!("Failed to open {:?}: {}", path, error));
            }
        }
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
            self.output_log
                .push(format!("Failed to save {:?}: {}", path, error));
        }
    }

    pub fn save_all(&mut self) {
        for document in self.open_documents.values_mut() {
            if !document.dirty {
                continue;
            }

            if let Err(error) = document.save() {
                self.output_log
                    .push(format!("Failed to save {:?}: {}", document.path, error));
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
            self.output_log
                .push(format!("Failed to create scripts directory: {}", error));
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

        if !valid_script_filename(&filename) {
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

                self.output_log.push(format!("Renamed to {:?}", filename));
            }

            Err(error) => {
                self.output_log.push(format!("Rename failed: {}", error));
            }
        }
    }

    pub fn delete_script(&mut self, project_path: &Option<PathBuf>, path: PathBuf) {
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
                self.output_log.push(format!("Delete failed: {}", error));
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
                    if is_deletion { "Deletion" } else { "Closure" }
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
}

pub(crate) fn valid_script_filename(filename: &str) -> bool {
    filename.chars().all(|character| {
        !matches!(
            character,
            '\\' | '/' | ':' | '*' | '?' | '"' | '<' | '>' | '|'
        )
    })
}

pub(crate) fn sanitize_entity_name(name: &str) -> String {
    let base = name
        .strip_suffix(".aeo")
        .unwrap_or(name)
        .split('.')
        .next()
        .unwrap_or("MyScript");

    let mut result = base
        .chars()
        .filter(|character| character.is_ascii_alphanumeric() || *character == '_')
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
