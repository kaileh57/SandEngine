use minifb::{Key, Window, WindowOptions};
use rand::Rng;
use std::time::Duration;

const WIDTH: usize = 400;
const HEIGHT: usize = 300;
const MAX_FPS: u64 = 240;

// Material types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Material {
    Air,
    Sand,
    Stone,
}

// Color constants
const COLOR_AIR: u32 = 0x000000;       // Black
const COLOR_SAND: u32 = 0xC2B280;      // Sand color
const COLOR_STONE: u32 = 0x808080;     // Grey

struct World {
    grid: Vec<Material>,
    width: usize,
    height: usize,
}

impl World {
    fn new(width: usize, height: usize) -> Self {
        let grid = vec![Material::Air; width * height];
        Self { grid, width, height }
    }

    fn get(&self, x: usize, y: usize) -> Material {
        if x < self.width && y < self.height {
            self.grid[y * self.width + x]
        } else {
            Material::Air
        }
    }

    fn set(&mut self, x: usize, y: usize, material: Material) {
        if x < self.width && y < self.height {
            self.grid[y * self.width + x] = material;
        }
    }

    fn update(&mut self) {
        let mut rng = rand::thread_rng();

        // Process from bottom to top, left to right
        for y in (0..self.height).rev() {
            for x in 0..self.width {
                match self.get(x, y) {
                    Material::Sand => {
                        // Try to move down
                        if y + 1 < self.height && self.get(x, y + 1) == Material::Air {
                            self.set(x, y, Material::Air);
                            self.set(x, y + 1, Material::Sand);
                        }
                        // Try to move diagonally
                        else if y + 1 < self.height {
                            let left_first = rng.gen_bool(0.5);
                            
                            if left_first {
                                // Try left-down first
                                if x > 0 && self.get(x - 1, y + 1) == Material::Air {
                                    self.set(x, y, Material::Air);
                                    self.set(x - 1, y + 1, Material::Sand);
                                }
                                // Then try right-down
                                else if x + 1 < self.width && self.get(x + 1, y + 1) == Material::Air {
                                    self.set(x, y, Material::Air);
                                    self.set(x + 1, y + 1, Material::Sand);
                                }
                            } else {
                                // Try right-down first
                                if x + 1 < self.width && self.get(x + 1, y + 1) == Material::Air {
                                    self.set(x, y, Material::Air);
                                    self.set(x + 1, y + 1, Material::Sand);
                                }
                                // Then try left-down
                                else if x > 0 && self.get(x - 1, y + 1) == Material::Air {
                                    self.set(x, y, Material::Air);
                                    self.set(x - 1, y + 1, Material::Sand);
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    fn render(&self, buffer: &mut [u32]) {
        for y in 0..self.height {
            for x in 0..self.width {
                let index = y * self.width + x;
                buffer[index] = match self.get(x, y) {
                    Material::Air => COLOR_AIR,
                    Material::Sand => COLOR_SAND,
                    Material::Stone => COLOR_STONE,
                };
            }
        }
    }

    fn create_circle(&mut self, center_x: usize, center_y: usize, radius: usize, material: Material) {
        for dy in -(radius as isize)..=(radius as isize) {
            for dx in -(radius as isize)..=(radius as isize) {
                let dist_sq = dx*dx + dy*dy;
                if dist_sq <= (radius as isize)*(radius as isize) {
                    let x = center_x as isize + dx;
                    let y = center_y as isize + dy;
                    
                    if x >= 0 && y >= 0 && x < self.width as isize && y < self.height as isize {
                        self.set(x as usize, y as usize, material);
                    }
                }
            }
        }
    }
}

fn main() {
    // Create a new window
    let mut window = Window::new(
        "Falling Sand Simulation",
        WIDTH,
        HEIGHT,
        WindowOptions::default(),
    )
    .expect("Failed to create window");

    // Limit to max FPS
    window.limit_update_rate(Some(Duration::from_micros(1_000_000 / MAX_FPS)));

    // Create world
    let mut world = World::new(WIDTH, HEIGHT);
    
    // Create buffer to hold pixels
    let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];
    
    // Track mouse state
    let mut mouse_pressed = false;
    let mut brush_material = Material::Sand;
    let mut brush_radius = 5;

    // Main loop
    while window.is_open() && !window.is_key_down(Key::Escape) {
        // Get mouse position and button state
        if let Some((x, y)) = window.get_mouse_pos(minifb::MouseMode::Clamp) {
            mouse_pressed = window.get_mouse_down(minifb::MouseButton::Left);
            
            if mouse_pressed {
                world.create_circle(x as usize, y as usize, brush_radius, brush_material);
            }
        }

        // Handle keyboard input
        if window.is_key_pressed(Key::Key1, minifb::KeyRepeat::No) {
            brush_material = Material::Sand;
            println!("Selected: Sand");
        }
        if window.is_key_pressed(Key::Key2, minifb::KeyRepeat::No) {
            brush_material = Material::Stone;
            println!("Selected: Stone");
        }
        if window.is_key_pressed(Key::Key0, minifb::KeyRepeat::No) {
            brush_material = Material::Air;
            println!("Selected: Eraser");
        }
        if window.is_key_pressed(Key::Equal, minifb::KeyRepeat::No) {
            brush_radius = (brush_radius + 1).min(20);
            println!("Brush size: {}", brush_radius);
        }
        if window.is_key_pressed(Key::Minus, minifb::KeyRepeat::No) {
            brush_radius = (brush_radius - 1).max(1);
            println!("Brush size: {}", brush_radius);
        }
        if window.is_key_pressed(Key::Space, minifb::KeyRepeat::No) {
            // Drop a big pile of sand
            world.create_circle(
                WIDTH / 2,
                HEIGHT / 3,
                30,
                Material::Sand
            );
        }
        
        // Update simulation
        world.update();
        
        // Render
        world.render(&mut buffer);
        
        // Update the window with the new buffer
        window.update_with_buffer(&buffer, WIDTH, HEIGHT)
            .expect("Failed to update window");
    }
}
