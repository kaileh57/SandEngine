// Game module
pub mod ui;
pub mod ui_components;
pub mod text_renderer;
pub mod material_renderer;
pub mod constants;

// Re-export main components
pub use ui::UI;
pub use material_renderer::MaterialRenderer; 