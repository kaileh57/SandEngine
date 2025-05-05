// ui/mod.rs - UI module definition

// Declare submodules
pub mod components;
pub mod text_renderer;

// Re-export main UI implementation and key components
pub use components::{Button, Panel, Rect, Slider, StatusBar, ButtonAction};
pub use text_renderer::TextRenderer;

// Import the main UI struct directly here
mod ui_impl;
pub use ui_impl::UI;