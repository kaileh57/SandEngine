// File: engine/constants.rs

// Grid and window dimensions
pub const GRID_WIDTH: usize = 200;
pub const GRID_HEIGHT: usize = 150;
pub const CELL_SIZE: usize = 4;
pub const WIDTH: u32 = (GRID_WIDTH * CELL_SIZE) as u32;
pub const HEIGHT: u32 = (GRID_HEIGHT * CELL_SIZE) as u32;

// Temperature constants
pub const AMBIENT_TEMP: f32 = 20.0;
pub const MAX_TEMP: f32 = 3000.0;
pub const COOLING_RATE: f32 = 0.005; 