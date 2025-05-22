//! # Sand Simulation Engine
//!
//! `sand_simulation_engine` is a Rust library that provides the core logic for a falling sand simulation.
//! It includes modules for defining materials, individual particles, the simulation grid, and the main simulation engine
//! that orchestrates the interactions and behaviors of particles.
//!
//! ## Modules
//! - `material`: Defines the types of materials (e.g., Sand, Water, Stone) and their physical properties.
//! - `particle`: Defines the `Particle` struct, representing an individual element in the simulation with state like temperature and lifespan.
//! - `grid`: Defines the `Grid` struct, which manages the 2D array of particles.
//! - `simulation_engine`: Contains the `SimulationEngine`, the main orchestrator that applies simulation rules and updates the grid state.
//!
//! ## Usage
//! Typically, a game or application would create an instance of `SimulationEngine`,
//! then repeatedly call its `update()` method in a game loop. User interactions (like placing particles)
//! are handled through methods like `place_particle_circle()`. The state of the grid can be accessed
//! for rendering.
//!
//! ```no_run
//! use sand_simulation_engine::simulation_engine::SimulationEngine;
//! use sand_simulation_engine::material::MaterialType;
//!
//! fn main() {
//!     let mut engine = SimulationEngine::new(200, 150); // Create a 200x150 grid
//!     
//!     // Place some sand
//!     engine.place_particle_circle(100, 50, 5, MaterialType::SAND, None);
//!
//!     // Game loop
//!     loop {
//!         let delta_time = 0.016; // Assuming roughly 60 FPS
//!         engine.update(delta_time);
//!         
//!         // Render engine.grid.cells here...
//!         // Handle user input here...
//!         
//!         // Break condition for the loop
//!         # break; 
//!     }
//! }
//! ```

pub mod particle;
pub mod material;
pub mod grid;
pub mod simulation_engine;
