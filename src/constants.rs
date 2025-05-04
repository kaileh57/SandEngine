// File: constants.rs

// Grid and window dimensions
pub const GRID_WIDTH: usize = 200;
pub const GRID_HEIGHT: usize = 150;
pub const CELL_SIZE: usize = 4;
pub const WIDTH: u32 = (GRID_WIDTH * CELL_SIZE) as u32;
pub const HEIGHT: u32 = (GRID_HEIGHT * CELL_SIZE) as u32;
pub const UI_WIDTH: u32 = 300; // Increased from 200 to 300
pub const WINDOW_WIDTH: u32 = WIDTH + UI_WIDTH;

// Colors
pub const C_EMPTY: [u8; 4] = [0, 0, 0, 255];
pub const C_SAND: [u8; 4] = [194, 178, 128, 255];
pub const C_WATER: [u8; 4] = [50, 100, 200, 255];
pub const C_STONE: [u8; 4] = [100, 100, 100, 255];
pub const C_PLANT: [u8; 4] = [50, 150, 50, 255];
pub const C_FIRE: [u8; 4] = [255, 69, 0, 255];
pub const C_LAVA: [u8; 4] = [200, 50, 0, 255];
pub const C_BORDER: [u8; 4] = [100, 100, 100, 255];
pub const C_ERASER: [u8; 4] = [255, 0, 255, 255];
pub const C_UI_BG: [u8; 4] = [40, 40, 40, 255];
pub const C_UI_TEXT: [u8; 4] = [240, 240, 240, 255];
pub const C_UI_HIGHLIGHT: [u8; 4] = [100, 100, 160, 255];
pub const C_UI_BUTTON: [u8; 4] = [80, 80, 90, 255];
pub const C_UI_BUTTON_SELECTED: [u8; 4] = [100, 100, 120, 255];
pub const C_UI_BUTTON_BORDER: [u8; 4] = [160, 160, 180, 255];
pub const C_UI_CLEAR_BUTTON: [u8; 4] = [180, 60, 60, 255];
pub const C_UI_CLEAR_BUTTON_BORDER: [u8; 4] = [220, 100, 100, 255];

// Temperature constants
pub const AMBIENT_TEMP: f32 = 20.0;
pub const MAX_TEMP: f32 = 3000.0;
pub const COOLING_RATE: f32 = 0.005;

// Physics constants
pub const GRAVITY: f32 = 0.4;  // Base gravity acceleration
pub const MAX_VELOCITY: f32 = 3.0;  // Maximum fall speed for most particles

// Sand physics
pub const SAND_GRAVITY: f32 = 0.4;  // Sand falls at normal gravity
pub const SAND_MAX_VELOCITY: f32 = 3.0;  // Sand has normal terminal velocity

// Water physics
pub const WATER_GRAVITY: f32 = 0.3;  // Water accelerates a bit slower than sand
pub const WATER_MAX_VELOCITY: f32 = 2.5;  // Water flows a bit slower at max
pub const WATER_VISCOSITY: f32 = 0.05;  // Water has low viscosity (flows easily)

// Lava physics
pub const LAVA_GRAVITY: f32 = 0.2;  // Lava is more viscous, falls slower
pub const LAVA_MAX_VELOCITY: f32 = 1.5;  // Lava has lower terminal velocity
pub const LAVA_VISCOSITY: f32 = 0.4;  // Lava has high viscosity (flows slowly)

// Fire physics
pub const FIRE_UPDRAFT: f32 = 0.3;  // Fire rises
pub const FIRE_MAX_VELOCITY: f32 = 2.0;  // Maximum updraft speed

// Stone physics - very rigid
pub const STONE_GRAVITY: f32 = 0.5;  // Stone falls faster than sand
pub const STONE_MAX_VELOCITY: f32 = 3.5;  // Stone has higher terminal velocity
pub const STONE_RIGIDITY: f32 = 0.9;  // Stone has high rigidity (rarely flows)

// Plant physics - somewhat rigid but organic
pub const PLANT_GRAVITY: f32 = 0.35;  // Plant falls a bit slower than sand
pub const PLANT_MAX_VELOCITY: f32 = 2.8;  // Plant has moderate terminal velocity
pub const PLANT_RIGIDITY: f32 = 0.6;  // Plant has moderate rigidity