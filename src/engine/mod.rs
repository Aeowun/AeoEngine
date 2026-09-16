pub mod app;

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
