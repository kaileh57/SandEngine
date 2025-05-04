// Engine module
pub mod simulation;
pub mod material;
pub mod constants;

// Re-export main components
pub use simulation::SandSimulation;
pub use material::{MaterialType, MaterialProperties}; 