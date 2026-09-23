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
        ensure_project_characters(&project_path);
        ensure_project_controllers(&project_path);
        ensure_project_cameras(&project_path);
        ensure_project_audio(&project_path);

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
            ensure_project_characters(&path);
            ensure_project_controllers(&path);
            ensure_project_cameras(&path);
            ensure_project_audio(&path);

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

pub fn ensure_project_characters(project_path: &std::path::Path) {
    let chars_dir = project_path.join("characters");
    if !chars_dir.exists() {
        let _ = fs::create_dir_all(&chars_dir);
    }

    let custom_dir = chars_dir.join("custom");
    if !custom_dir.exists() {
        let _ = fs::create_dir_all(&custom_dir);
        let pkg = r#"{
  "name": "custom",
  "mesh_type": "robot",
  "has_gun": false,
  "appearance": {
    "skin_color": [0.70, 0.71, 0.70, 1.0],
    "armor_color": [0.46, 0.47, 0.45, 1.0],
    "cloth_color": [0.10, 0.10, 0.10, 1.0],
    "detail_color": [0.12, 0.13, 0.15, 1.0],
    "accessory_color": [0.08, 0.30, 0.34, 1.0],
    "show_accessory": true
  }
}"#;
        let _ = fs::write(custom_dir.join("package.json"), pkg);
    }

    let soldier_dir = chars_dir.join("custom_soldier");
    if !soldier_dir.exists() {
        let _ = fs::create_dir_all(&soldier_dir);
        let pkg = r#"{
  "name": "custom_soldier",
  "mesh_type": "soldier",
  "has_gun": true,
  "appearance": {
    "skin_color": [0.90, 0.70, 0.10, 1.0],
    "armor_color": [0.22, 0.35, 0.20, 1.0],
    "cloth_color": [0.08, 0.08, 0.09, 1.0],
    "detail_color": [0.15, 0.18, 0.16, 1.0],
    "accessory_color": [0.80, 0.30, 0.00, 1.0],
    "show_accessory": true
  }
}"#;
        let _ = fs::write(soldier_dir.join("package.json"), pkg);
    }
}

pub fn discover_characters(project_path: &std::path::Path) -> Vec<String> {
    ensure_project_characters(project_path);
    let mut names = Vec::new();
    let chars_dir = project_path.join("characters");
    if let Ok(entries) = fs::read_dir(&chars_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    names.push(name.to_string());
                }
            }
        }
    }
    names.sort();
    if names.is_empty() {
        names.push("custom".to_string());
    }
    names
}

pub fn ensure_project_controllers(project_path: &std::path::Path) {
    let ctrl_dir = project_path.join("controllers");
    if !ctrl_dir.exists() {
        let _ = fs::create_dir_all(&ctrl_dir);
    }

    let tp_dir = ctrl_dir.join("thirdPerson_Controller");
    if !tp_dir.exists() {
        let _ = fs::create_dir_all(&tp_dir);
        let pkg = r#"{
  "name": "thirdPerson_Controller",
  "type": "third_person"
}"#;
        let _ = fs::write(tp_dir.join("package.json"), pkg);
    }
}

pub fn discover_controllers(project_path: &std::path::Path) -> Vec<String> {
    ensure_project_controllers(project_path);
    let mut names = Vec::new();
    let ctrl_dir = project_path.join("controllers");
    if let Ok(entries) = fs::read_dir(&ctrl_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    names.push(name.to_string());
                }
            }
        }
    }
    names.sort();
    if names.is_empty() {
        names.push("thirdPerson_Controller".to_string());
    }
    names
}

pub fn ensure_project_cameras(project_path: &std::path::Path) {
    let cam_dir = project_path.join("cameras");
    if !cam_dir.exists() {
        let _ = fs::create_dir_all(&cam_dir);
    }

    let tp_dir = cam_dir.join("thirdPerson");
    if !tp_dir.exists() {
        let _ = fs::create_dir_all(&tp_dir);
        let pkg = r#"{
  "name": "thirdPerson",
  "type": "third_person"
}"#;
        let _ = fs::write(tp_dir.join("package.json"), pkg);
    }

    let fp_dir = cam_dir.join("firstPerson");
    if !fp_dir.exists() {
        let _ = fs::create_dir_all(&fp_dir);
        let pkg = r#"{
  "name": "firstPerson",
  "type": "first_person"
}"#;
        let _ = fs::write(fp_dir.join("package.json"), pkg);
    }
}

pub fn discover_cameras(project_path: &std::path::Path) -> Vec<String> {
    ensure_project_cameras(project_path);
    let mut names = Vec::new();
    let cam_dir = project_path.join("cameras");
    if let Ok(entries) = fs::read_dir(&cam_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    names.push(name.to_string());
                }
            }
        }
    }
    names.sort();
    if names.is_empty() {
        names.push("thirdPerson".to_string());
    }
    names
}

pub fn ensure_project_audio(project_path: &std::path::Path) {
    let audio_dir = project_path.join(".assets").join("audio");
    if !audio_dir.exists() {
        let _ = fs::create_dir_all(&audio_dir);
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
        let project_path = manager
            .create_project(project_name)
            .expect("Should create project");

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
        assert!(
            scripts_dir.exists(),
            "Scripts directory should be created on open"
        );
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
