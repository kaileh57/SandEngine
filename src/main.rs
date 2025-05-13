use minifb::{Key, Window, WindowOptions};
use rand::Rng;
use std::time::Duration;
use std::thread;

const WIDTH: usize = 800;
const HEIGHT: usize = 600;
const MAX_FPS: u64 = 60;
const NUM_THREADS: usize = 8; // Adjust based on your CPU cores

// Material types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Material {
    Air,
    Sand,
    Stone,
    Water,
}

// Color constants
const COLOR_AIR: u32 = 0x000000;       // Black
const COLOR_SAND: u32 = 0xC2B280;      // Sand color
const COLOR_STONE: u32 = 0x808080;     // Grey
const COLOR_WATER: u32 = 0x4040FF;     // Blue

struct World {
    grid: Vec<Material>,
    grid_buffer: Vec<Material>, // Double buffer for parallel updates
    width: usize,
    height: usize,
}

impl World {
    fn new(width: usize, height: usize) -> Self {
        let grid = vec![Material::Air; width * height];
        let grid_buffer = grid.clone();
        Self { grid, grid_buffer, width, height }
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

    fn set_buffer(&mut self, x: usize, y: usize, material: Material) {
        if x < self.width && y < self.height {
            self.grid_buffer[y * self.width + x] = material;
        }
    }

    fn get_buffer(&self, x: usize, y: usize) -> Material {
        if x < self.width && y < self.height {
            self.grid_buffer[y * self.width + x]
        } else {
            Material::Air
        }
    }

    fn clear(&mut self) {
        for cell in &mut self.grid {
            *cell = Material::Air;
        }
    }

    fn swap_buffers(&mut self) {
        std::mem::swap(&mut self.grid, &mut self.grid_buffer);
    }

    fn update(&mut self) {
        let mut rng = rand::thread_rng();
        
        // Copy current grid to buffer
        self.grid_buffer.copy_from_slice(&self.grid);
        
        // Process in parallel by spawning multiple threads
        let mut handles = Vec::new();
        let slice_height = self.height / NUM_THREADS;
        
        for i in 0..NUM_THREADS {
            let start_y = i * slice_height;
            let end_y = if i == NUM_THREADS - 1 {
                self.height
            } else {
                (i + 1) * slice_height
            };
            
            // Clone necessary data for the thread
            let grid_slice = self.grid[start_y * self.width..end_y * self.width].to_vec();
            let width = self.width;
            let height = self.height;
            let thread_rng_seed = rng.gen::<u64>();
            
            // Create a reference to the buffer for this thread to write to
            let buffer_ptr = self.grid_buffer.as_mut_ptr();
            
            let handle = thread::spawn(move || {
                let mut thread_rng = rand::rngs::StdRng::seed_from_u64(thread_rng_seed);
                
                for y in (start_y..end_y).rev() {
                    for x in 0..width {
                        let idx = y * width + x;
                        match grid_slice[idx - start_y * width] {
                            Material::Sand => {
                                // Process sand physics
                                if y + 1 < height {
                                    let below_idx = (y + 1) * width + x;
                                    let below = if y + 1 < end_y {
                                        grid_slice[below_idx - start_y * width]
                                    } else {
                                        Material::Air // Boundary case, assume air
                                    };
                                    
                                    if below == Material::Air || below == Material::Water {
                                        // Move down - safe to write since we're processing bottom to top
                                        unsafe {
                                            *buffer_ptr.add(idx) = if below == Material::Water { Material::Water } else { Material::Air };
                                            *buffer_ptr.add(below_idx) = Material::Sand;
                                        }
                                    } 
                                    else {
                                        // Try diagonal
                                        let left_first = thread_rng.gen_bool(0.5);
                                        
                                        if left_first {
                                            if x > 0 {
                                                let below_left_idx = (y + 1) * width + (x - 1);
                                                let below_left = if y + 1 < end_y && x > 0 {
                                                    grid_slice[below_left_idx - start_y * width]
                                                } else {
                                                    Material::Air
                                                };
                                                
                                                if below_left == Material::Air || below_left == Material::Water {
                                                    unsafe {
                                                        *buffer_ptr.add(idx) = if below_left == Material::Water { Material::Water } else { Material::Air };
                                                        *buffer_ptr.add(below_left_idx) = Material::Sand;
                                                    }
                                                } else if x + 1 < width {
                                                    let below_right_idx = (y + 1) * width + (x + 1);
                                                    let below_right = if y + 1 < end_y && x + 1 < width {
                                                        grid_slice[below_right_idx - start_y * width]
                                                    } else {
                                                        Material::Air
                                                    };
                                                    
                                                    if below_right == Material::Air || below_right == Material::Water {
                                                        unsafe {
                                                            *buffer_ptr.add(idx) = if below_right == Material::Water { Material::Water } else { Material::Air };
                                                            *buffer_ptr.add(below_right_idx) = Material::Sand;
                                                        }
                                                    }
                                                }
                                            }
                                        } else {
                                            if x + 1 < width {
                                                let below_right_idx = (y + 1) * width + (x + 1);
                                                let below_right = if y + 1 < end_y && x + 1 < width {
                                                    grid_slice[below_right_idx - start_y * width]
                                                } else {
                                                    Material::Air
                                                };
                                                
                                                if below_right == Material::Air || below_right == Material::Water {
                                                    unsafe {
                                                        *buffer_ptr.add(idx) = if below_right == Material::Water { Material::Water } else { Material::Air };
                                                        *buffer_ptr.add(below_right_idx) = Material::Sand;
                                                    }
                                                } else if x > 0 {
                                                    let below_left_idx = (y + 1) * width + (x - 1);
                                                    let below_left = if y + 1 < end_y && x > 0 {
                                                        grid_slice[below_left_idx - start_y * width]
                                                    } else {
                                                        Material::Air
                                                    };
                                                    
                                                    if below_left == Material::Air || below_left == Material::Water {
                                                        unsafe {
                                                            *buffer_ptr.add(idx) = if below_left == Material::Water { Material::Water } else { Material::Air };
                                                            *buffer_ptr.add(below_left_idx) = Material::Sand;
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            },
                            Material::Water => {
                                // Process water physics (simplified for performance)
                                if y + 1 < height {
                                    let below_idx = (y + 1) * width + x;
                                    let below = if y + 1 < end_y {
                                        grid_slice[below_idx - start_y * width]
                                    } else {
                                        Material::Air
                                    };
                                    
                                    if below == Material::Air {
                                        // Move down
                                        unsafe {
                                            *buffer_ptr.add(idx) = Material::Air;
                                            *buffer_ptr.add(below_idx) = Material::Water;
                                        }
                                    } else {
                                        // Try to move horizontally
                                        let left_first = thread_rng.gen_bool(0.5);
                                        let should_spread = thread_rng.gen_ratio(3, 4);
                                        
                                        if should_spread {
                                            if left_first {
                                                if x > 0 {
                                                    let left_idx = y * width + (x - 1);
                                                    let left = if x > 0 {
                                                        grid_slice[left_idx - start_y * width]
                                                    } else {
                                                        Material::Air
                                                    };
                                                    
                                                    if left == Material::Air {
                                                        unsafe {
                                                            *buffer_ptr.add(idx) = Material::Air;
                                                            *buffer_ptr.add(left_idx) = Material::Water;
                                                        }
                                                    } else if x + 1 < width {
                                                        let right_idx = y * width + (x + 1);
                                                        let right = if x + 1 < width {
                                                            grid_slice[right_idx - start_y * width]
                                                        } else {
                                                            Material::Air
                                                        };
                                                        
                                                        if right == Material::Air {
                                                            unsafe {
                                                                *buffer_ptr.add(idx) = Material::Air;
                                                                *buffer_ptr.add(right_idx) = Material::Water;
                                                            }
                                                        }
                                                    }
                                                }
                                            } else {
                                                if x + 1 < width {
                                                    let right_idx = y * width + (x + 1);
                                                    let right = if x + 1 < width {
                                                        grid_slice[right_idx - start_y * width]
                                                    } else {
                                                        Material::Air
                                                    };
                                                    
                                                    if right == Material::Air {
                                                        unsafe {
                                                            *buffer_ptr.add(idx) = Material::Air;
                                                            *buffer_ptr.add(right_idx) = Material::Water;
                                                        }
                                                    } else if x > 0 {
                                                        let left_idx = y * width + (x - 1);
                                                        let left = if x > 0 {
                                                            grid_slice[left_idx - start_y * width]
                                                        } else {
                                                            Material::Air
                                                        };
                                                        
                                                        if left == Material::Air {
                                                            unsafe {
                                                                *buffer_ptr.add(idx) = Material::Air;
                                                                *buffer_ptr.add(left_idx) = Material::Water;
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            },
                            _ => {}
                        }
                    }
                }
            });
            
            handles.push(handle);
        }
        
        // Wait for all threads to complete
        for handle in handles {
            handle.join().unwrap();
        }
        
        // Swap buffers
        self.swap_buffers();
    }

    fn render(&self, buffer: &mut [u32]) {
        // Fast rendering using direct indexing
        for (i, &material) in self.grid.iter().enumerate() {
            buffer[i] = match material {
                Material::Air => COLOR_AIR,
                Material::Sand => COLOR_SAND,
                Material::Stone => COLOR_STONE,
                Material::Water => COLOR_WATER,
            };
        }
    }

    fn create_circle(&mut self, center_x: usize, center_y: usize, radius: usize, material: Material) {
        let radius_sq = (radius * radius) as isize;
        let min_x = (center_x as isize - radius as isize).max(0) as usize;
        let max_x = (center_x as isize + radius as isize).min(self.width as isize - 1) as usize;
        let min_y = (center_y as isize - radius as isize).max(0) as usize;
        let max_y = (center_y as isize + radius as isize).min(self.height as isize - 1) as usize;
        
        for y in min_y..=max_y {
            let dy = y as isize - center_y as isize;
            let dy_sq = dy * dy;
            
            for x in min_x..=max_x {
                let dx = x as isize - center_x as isize;
                let dist_sq = dx * dx + dy_sq;
                
                if dist_sq <= radius_sq {
                    self.set(x, y, material);
                }
            }
        }
    }
}

fn main() {
    // Create a new window
    let mut window = Window::new(
        "Fast Falling Sand Simulation",
        WIDTH,
        HEIGHT,
        WindowOptions {
            scale: minifb::Scale::X1,
            ..WindowOptions::default()
        },
    )
    .expect("Failed to create window");

    // No frame rate limit for maximum speed
    window.limit_update_rate(None);

    // Create world
    let mut world = World::new(WIDTH, HEIGHT);
    
    // Create buffer to hold pixels
    let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];
    
    // Track mouse state
    let mut brush_material = Material::Sand;
    let mut brush_radius = 5;
    let mut fps_counter = 0;
    let mut last_time = std::time::Instant::now();
    let mut update_fps = std::time::Instant::now();
    let mut fps = 0;

    // Display controls
    println!("Controls:");
    println!("  1: Sand");
    println!("  2: Stone");
    println!("  3: Water");
    println!("  0: Eraser (Air)");
    println!("  SPACE: Drop sand");
    println!("  C: Clear everything");
    println!("  +/-: Change brush size");
    println!("  ESC: Quit");

    // Main loop
    while window.is_open() && !window.is_key_down(Key::Escape) {
        // Get mouse position and button state
        if let Some((x, y)) = window.get_mouse_pos(minifb::MouseMode::Clamp) {
            let mouse_pressed = window.get_mouse_down(minifb::MouseButton::Left);
            
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
        if window.is_key_pressed(Key::Key3, minifb::KeyRepeat::No) {
            brush_material = Material::Water;
            println!("Selected: Water");
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
        if window.is_key_pressed(Key::C, minifb::KeyRepeat::No) {
            // Clear everything
            world.clear();
            println!("Cleared world");
        }
        
        // Update simulation
        world.update();
        
        // Render
        world.render(&mut buffer);
        
        // Update FPS counter
        fps_counter += 1;
        if update_fps.elapsed().as_secs() >= 1 {
            fps = fps_counter;
            fps_counter = 0;
            update_fps = std::time::Instant::now();
            
            // Update window title with FPS
            window.set_title(&format!("Fast Falling Sand Simulation - {} FPS", fps));
        }
        
        // Update the window with the new buffer
        window.update_with_buffer(&buffer, WIDTH, HEIGHT)
            .expect("Failed to update window");
    }
} 