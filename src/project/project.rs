use super::recent::RecentProjects;
use std::fs;
use std::path::PathBuf;

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
                self.recent_projects = recent
                    .paths
                    .iter()
                    .map(PathBuf::from)
                    .filter(|p| p.exists())
                    .collect();
            }
        }
    }

    pub fn save_recent(&self) {
        let path = self.user_data_dir.join("recent_projects.json");
        let recent = RecentProjects {
            paths: self
                .recent_projects
                .iter()
                .map(|p| p.to_string_lossy().into_owned())
                .collect(),
        };
        if let Ok(content) = serde_json::to_string_pretty(&recent) {
            fs::write(path, content).ok();
        }
    }

    pub fn add_recent(&mut self, path: PathBuf) {
        let path_str = path.to_string_lossy().into_owned();
        self.recent_projects
            .retain(|p| p.to_string_lossy() != path_str);
        self.recent_projects.insert(0, path);
        if self.recent_projects.len() > 10 {
            self.recent_projects.truncate(10);
        }
        self.save_recent();
    }

    pub fn create_project(&mut self, name: &str) -> Option<PathBuf> {
        if name.is_empty() {
            return None;
        }
        let project_path = self.user_data_dir.join(name);
        if !project_path.exists() {
            fs::create_dir_all(&project_path).ok()?;
        }

        let scripts_path = project_path.join("scripts");
        if !scripts_path.exists() {
            fs::create_dir_all(&scripts_path).ok();
        }

        self.add_recent(project_path.clone());
        self.current_project = Some(project_path.clone());
        Some(project_path)
    }

    pub fn open_project(&mut self, path: PathBuf) -> bool {
        if path.exists() && path.is_dir() {
            let scripts_path = path.join("scripts");
            if !scripts_path.exists() {
                fs::create_dir_all(&scripts_path).ok();
            }

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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_create_project_creates_scripts_dir() {
        let test_dir = PathBuf::from("TestUserData_Create");
        if test_dir.exists() {
            fs::remove_dir_all(&test_dir).ok();
        }
        fs::create_dir_all(&test_dir).unwrap();

        let mut manager = ProjectManager {
            user_data_dir: test_dir.clone(),
            current_project: None,
            recent_projects: Vec::new(),
        };

        let project_name = "NewProject";
        let project_path = manager.create_project(project_name).expect("Should create project");

        let scripts_dir = project_path.join("scripts");
        assert!(scripts_dir.exists(), "Scripts directory should be created");
        assert!(scripts_dir.is_dir(), "Scripts path should be a directory");

        // Verify no .aeo files
        let entries = fs::read_dir(&scripts_dir).unwrap();
        assert_eq!(entries.count(), 0, "Scripts directory should be empty");

        fs::remove_dir_all(&test_dir).ok();
    }

    #[test]
    fn test_open_project_ensures_scripts_dir() {
        let test_dir = PathBuf::from("TestUserData_Open");
        if test_dir.exists() {
            fs::remove_dir_all(&test_dir).ok();
        }
        fs::create_dir_all(&test_dir).unwrap();

        let project_path = test_dir.join("ExistingProject");
        fs::create_dir_all(&project_path).unwrap();
        // scripts dir is missing initially

        let mut manager = ProjectManager {
            user_data_dir: test_dir.clone(),
            current_project: None,
            recent_projects: Vec::new(),
        };

        let success = manager.open_project(project_path.clone());
        assert!(success, "Should open existing project");

        let scripts_dir = project_path.join("scripts");
        assert!(scripts_dir.exists(), "Scripts directory should be created on open");
        assert!(scripts_dir.is_dir(), "Scripts path should be a directory");

        fs::remove_dir_all(&test_dir).ok();
    }

    #[test]
    fn test_metadata_persistence_intact() {
        let test_dir = PathBuf::from("TestUserData_Metadata");
        if test_dir.exists() {
            fs::remove_dir_all(&test_dir).ok();
        }
        fs::create_dir_all(&test_dir).unwrap();

        let mut manager = ProjectManager {
            user_data_dir: test_dir.clone(),
            current_project: None,
            recent_projects: Vec::new(),
        };

        manager.create_project("Project1");
        manager.create_project("Project2");

        assert_eq!(manager.recent_projects.len(), 2);

        let recent_path = test_dir.join("recent_projects.json");
        assert!(recent_path.exists(), "recent_projects.json should be saved");

        // Re-load
        let mut new_manager = ProjectManager {
            user_data_dir: test_dir.clone(),
            current_project: None,
            recent_projects: Vec::new(),
        };
        new_manager.load_recent();
        assert_eq!(new_manager.recent_projects.len(), 2);

        fs::remove_dir_all(&test_dir).ok();
    }
}
