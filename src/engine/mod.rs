pub mod app;
pub mod entity;
pub mod physics;
pub mod ui;

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum View {
    Home,
    Editor,
}

#[derive(PartialEq, Clone, Copy, Debug, Default)]
pub enum EditorMode {
    #[default]
    Editor,
    Play,
}
