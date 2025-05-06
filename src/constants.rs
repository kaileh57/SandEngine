// Grid and window dimensions
pub const GRID_WIDTH: usize = 200;
pub const GRID_HEIGHT: usize = 150;
pub const CELL_SIZE: usize = 4;
pub const WIDTH: u32 = (GRID_WIDTH * CELL_SIZE) as u32;
pub const HEIGHT: u32 = (GRID_HEIGHT * CELL_SIZE) as u32;
pub const UI_WIDTH: u32 = 300; // Width of UI panel
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

// New colors for additional materials
pub const C_GLASS: [u8; 4] = [210, 230, 240, 255];
pub const C_STEAM: [u8; 4] = [180, 180, 190, 255];
pub const C_SMOKE: [u8; 4] = [150, 150, 150, 255];
pub const C_ICE: [u8; 4] = [170, 200, 255, 255];
pub const C_WOOD: [u8; 4] = [139, 69, 19, 255];
pub const C_COAL: [u8; 4] = [40, 40, 40, 255];
pub const C_OIL: [u8; 4] = [80, 70, 20, 255];
pub const C_ACID: [u8; 4] = [100, 255, 100, 255];
pub const C_GUNPOWDER: [u8; 4] = [60, 60, 70, 255];
pub const C_TOXIC_GAS: [u8; 4] = [150, 200, 150, 255];
pub const C_ASH: [u8; 4] = [90, 90, 90, 255];
pub const C_FUSE: [u8; 4] = [100, 80, 60, 255];
pub const C_GENERATOR: [u8; 4] = [255, 0, 0, 255];

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

// New constants for materials and reactions
pub const FIRE_LIFESPAN: f32 = 1.0;  // Fire lasts for ~1 second
pub const STEAM_LIFESPAN: f32 = 10.0;  // Steam lasts for ~10 seconds
pub const SMOKE_LIFESPAN: f32 = 3.0;  // Smoke lasts for ~3 seconds
pub const FUSE_BURN_LIFESPAN: f32 = 4.0;  // Burning fuse lasts for ~4 seconds
pub const TARGET_DT_SCALING: f32 = 60.0;  // Target delta time scaling for 60 FPS
pub const GAS_UPDRAFT: f32 = 0.2;  // How fast gases rise
pub const WATER_COOLING_FACTOR: f32 = 80.0;  // How much water cools fire
pub const PHASE_CHANGE_TEMP_BUFFER: f32 = 5.0;  // Temp buffer for phase changes
pub const MIN_STATE_SECONDS: f32 = 10.0;  // Min time required in a state before certain changes
pub const HIGH_INERTIA_DAMPING: f32 = 0.2;  // Damping factor for high inertia materials
pub const PLANT_GROWTH_CHANCE_PER_SEC: f32 = 0.09;  // Chance for plant to grow per second
pub const GUNPOWDER_YIELD: usize = 4;  // Explosion radius for gunpowder
pub const CONDENSATION_Y_LIMIT: usize = 5;  // Height limit for guaranteed condensation
pub const CONDENSATION_CHANCE_ANYWHERE_PER_SEC: f32 = 0.006;  // Chance for condensation elsewhere
pub const ACID_GAS_TEMP_FACTOR: f32 = 0.8;  // Temperature factor for gas created by acid