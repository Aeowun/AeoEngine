use std::path::{PathBuf};
use std::fs;
use super::recent::RecentProjects;

pub struct ProjectManager {
    pub user_data_dir: PathBuf,
    pub current_project: Option<PathBuf>,
    pub recent_projects: Vec<PathBuf>,
}

impl ProjectManager {
    pub fn new() -> Self {
        let user_data_dir = PathBuf::from("UserData");
        if !user_data_dir.exists() {
            fs::create_dir_all(&user_data_dir).ok();
        }

        let mut manager = Self {
            user_data_dir,
            current_project: None,
            recent_projects: Vec::new(),
        };
        manager.load_recent();
        manager
    }

    pub fn load_recent(&mut self) {
        let path = self.user_data_dir.join("recent_projects.json");
        if let Ok(content) = fs::read_to_string(&path) {
            if let Ok(recent) = serde_json::from_str::<RecentProjects>(&content) {
                self.recent_projects = recent.paths.iter()
                    .map(PathBuf::from)
                    .filter(|p| p.exists())
                    .collect();
            }
        }
    }

    pub fn save_recent(&self) {
        let path = self.user_data_dir.join("recent_projects.json");
        let recent = RecentProjects {
            paths: self.recent_projects.iter()
                .map(|p| p.to_string_lossy().into_owned())
                .collect(),
        };
        if let Ok(content) = serde_json::to_string_pretty(&recent) {
            fs::write(path, content).ok();
        }
    }

    pub fn add_recent(&mut self, path: PathBuf) {
        let path_str = path.to_string_lossy().into_owned();
        self.recent_projects.retain(|p| p.to_string_lossy() != path_str);
        self.recent_projects.insert(0, path);
        if self.recent_projects.len() > 10 {
            self.recent_projects.truncate(10);
        }
        self.save_recent();
    }

    pub fn create_project(&mut self, name: &str) -> Option<PathBuf> {
        if name.is_empty() { return None; }
        let project_path = self.user_data_dir.join(name);
        if !project_path.exists() {
            fs::create_dir_all(&project_path).ok()?;
        }
        self.add_recent(project_path.clone());
        self.current_project = Some(project_path.clone());
        Some(project_path)
    }

    pub fn open_project(&mut self, path: PathBuf) -> bool {
        if path.exists() && path.is_dir() {
            self.add_recent(path.clone());
            self.current_project = Some(path);
            true
        } else {
            false
        }
    }

    pub fn list_projects(&self) -> Vec<PathBuf> {
        let mut projects = Vec::new();
        if let Ok(entries) = fs::read_dir(&self.user_data_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    projects.push(path);
                }
            }
        }
        projects
    }
}
