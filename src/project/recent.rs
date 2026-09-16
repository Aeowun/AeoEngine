use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Default)]
pub struct RecentProjects {
    pub paths: Vec<String>,
}
