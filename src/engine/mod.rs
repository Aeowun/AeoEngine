pub mod app;
pub mod physics;
pub mod entity;

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
