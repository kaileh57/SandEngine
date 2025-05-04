// File: engine/simulation.rs
use rand::prelude::*;
use crate::engine::constants::*;
use crate::engine::material::MaterialType;

pub struct SandSimulation {
    // Use a flat vector for better cache locality
    grid: Vec<u8>,
    pub temp: Vec<f32>,       // Temperature for each cell
    updated: Vec<bool>,
    pub brush_size: usize,
    pub current_material: MaterialType, // Current selected material
    pub cursor_pos: (usize, usize), // Store cursor position for UI
}

impl SandSimulation {
    pub fn new() -> Self {
        let temp = vec![AMBIENT_TEMP; GRID_WIDTH * GRID_HEIGHT];
        
        Self {
            grid: vec![0; GRID_WIDTH * GRID_HEIGHT],
            temp,
            updated: vec![false; GRID_WIDTH * GRID_HEIGHT],
            brush_size: 3,
            current_material: MaterialType::Sand,
            cursor_pos: (0, 0),
        }
    }

    #[inline]
    pub fn get_index(x: usize, y: usize) -> usize {
        y * GRID_WIDTH + x
    }

    #[inline]
    pub fn get(&self, x: usize, y: usize) -> MaterialType {
        if x < GRID_WIDTH && y < GRID_HEIGHT {
            MaterialType::from_u8(self.grid[Self::get_index(x, y)])
        } else {
            MaterialType::Empty // Out of bounds, return EMPTY
        }
    }

    #[inline]
    pub fn set(&mut self, x: usize, y: usize, value: MaterialType) {
        if x < GRID_WIDTH && y < GRID_HEIGHT {
            let idx = Self::get_index(x, y);
            self.grid[idx] = value.to_u8();
        }
    }

    #[inline]
    pub fn get_temp(&self, x: usize, y: usize) -> f32 {
        if x < GRID_WIDTH && y < GRID_HEIGHT {
            self.temp[Self::get_index(x, y)]
        } else {
            AMBIENT_TEMP // Out of bounds, return AMBIENT_TEMP
        }
    }

    #[inline]
    pub fn set_temp(&mut self, x: usize, y: usize, value: f32) {
        if x < GRID_WIDTH && y < GRID_HEIGHT {
            let idx = Self::get_index(x, y);
            self.temp[idx] = value;
        }
    }

    #[inline]
    fn is_updated(&self, x: usize, y: usize) -> bool {
        if x < GRID_WIDTH && y < GRID_HEIGHT {
            self.updated[Self::get_index(x, y)]
        } else {
            true // Out of bounds is treated as already updated
        }
    }

    #[inline]
    fn set_updated(&mut self, x: usize, y: usize, value: bool) {
        if x < GRID_WIDTH && y < GRID_HEIGHT {
            let idx = Self::get_index(x, y);
            self.updated[idx] = value;
        }
    }

    pub fn clear(&mut self) {
        for i in 0..self.grid.len() {
            self.grid[i] = 0; // EMPTY
            self.temp[i] = AMBIENT_TEMP;
        }
    }

    pub fn update(&mut self) {
        // Reset update flags
        for i in 0..self.updated.len() {
            self.updated[i] = false;
        }

        // Update temperature first
        self.update_temperatures();

        // Process from bottom to top, shuffling column order for natural flow
        let mut columns: Vec<usize> = (0..GRID_WIDTH).collect();
        columns.shuffle(&mut rand::thread_rng());

        for y in (0..GRID_HEIGHT).rev() {
            for &x in &columns {
                let material = self.get(x, y);
                if material != MaterialType::Empty && !self.is_updated(x, y) {
                    match material {
                        MaterialType::Sand => self.update_sand(x, y),
                        MaterialType::Water => self.update_water(x, y),
                        MaterialType::Fire => self.update_fire(x, y),
                        MaterialType::Lava => self.update_lava(x, y),
                        // Other materials will be added later
                        _ => {},
                    }
                }
            }
        }
    }

    // Temperature update function
    fn update_temperatures(&mut self) {
        // For each cell, calculate new temperature based on neighbors
        let mut new_temps = self.temp.clone();

        for y in 0..GRID_HEIGHT {
            for x in 0..GRID_WIDTH {
                if self.get(x, y) == MaterialType::Empty {
                    // Empty cells tend toward ambient temperature
                    new_temps[Self::get_index(x, y)] = AMBIENT_TEMP;
                    continue;
                }

                let cell_temp = self.get_temp(x, y);
                let material = self.get(x, y);

                // Materials with special heat properties
                if material == MaterialType::Fire {
                    // Fire maintains or increases its temperature
                    new_temps[Self::get_index(x, y)] = (cell_temp + 10.0).min(MAX_TEMP);
                    continue;
                } else if material == MaterialType::Lava {
                    // Lava cools very slowly
                    new_temps[Self::get_index(x, y)] = (cell_temp - 0.1).max(1000.0);
                    continue;
                }

                // Temperature diffusion with neighbors
                let mut neighbor_temp_sum = 0.0;
                let mut neighbor_count = 0;

                for dy in -1..=1 {
                    for dx in -1..=1 {
                        if dx == 0 && dy == 0 {
                            continue;
                        }

                        let nx = x as isize + dx;
                        let ny = y as isize + dy;

                        if nx >= 0 && nx < GRID_WIDTH as isize && ny >= 0 && ny < GRID_HEIGHT as isize {
                            let nx = nx as usize;
                            let ny = ny as usize;
                            neighbor_temp_sum += self.get_temp(nx, ny);
                            neighbor_count += 1;
                        } else {
                            // Boundary conditions: ambient temperature
                            neighbor_temp_sum += AMBIENT_TEMP;
                            neighbor_count += 1;
                        }
                    }
                }

                // Calculate average neighbor temperature
                let avg_neighbor_temp = neighbor_temp_sum / neighbor_count as f32;
                
                // Calculate new temperature with diffusion and cooling
                let conductivity = match material {
                    MaterialType::Stone => 0.2,
                    MaterialType::Water => 0.6,
                    MaterialType::Sand => 0.3,
                    MaterialType::Plant => 0.1,
                    _ => 0.4, // Default conductivity
                };
                
                // Temperature diffusion formula
                let delta_temp = (avg_neighbor_temp - cell_temp) * conductivity;
                
                // Cooling effect
                let cooling = (AMBIENT_TEMP - cell_temp) * COOLING_RATE;
                
                // Update temperature
                new_temps[Self::get_index(x, y)] = (cell_temp + delta_temp + cooling).max(-273.15).min(MAX_TEMP);
            }
        }

        // Update the temperature grid
        self.temp = new_temps;
    }

    // Material update functions
    fn update_sand(&mut self, x: usize, y: usize) {
        self.set_updated(x, y, true);
        
        // If we're at the bottom, do nothing
        if y == GRID_HEIGHT - 1 {
            return;
        }
        
        // Check if we can fall down
        let below = self.get(x, y + 1);
        if below == MaterialType::Empty {
            // Fall down
            self.set(x, y + 1, MaterialType::Sand);
            self.set(x, y, MaterialType::Empty);
            
            // Move temperature
            let temp = self.get_temp(x, y);
            self.set_temp(x, y + 1, temp);
            self.set_temp(x, y, AMBIENT_TEMP);
            
            self.set_updated(x, y + 1, true);
            return;
        }
        
        // Check if we can displace water or other lower density materials
        if below == MaterialType::Water {
            // Swap positions
            self.set(x, y + 1, MaterialType::Sand);
            self.set(x, y, MaterialType::Water);
            
            // Swap temperatures
            let temp1 = self.get_temp(x, y);
            let temp2 = self.get_temp(x, y + 1);
            self.set_temp(x, y + 1, temp1);
            self.set_temp(x, y, temp2);
            
            self.set_updated(x, y + 1, true);
            return;
        }
        
        // Try to fall diagonally
        let can_fall_left = x > 0 && self.get(x - 1, y + 1) == MaterialType::Empty;
        let can_fall_right = x < GRID_WIDTH - 1 && self.get(x + 1, y + 1) == MaterialType::Empty;
        
        if can_fall_left && can_fall_right {
            // Choose randomly between left and right
            let go_left = rand::thread_rng().gen_bool(0.5);
            if go_left {
                self.set(x - 1, y + 1, MaterialType::Sand);
                self.set(x, y, MaterialType::Empty);
                
                // Move temperature
                let temp = self.get_temp(x, y);
                self.set_temp(x - 1, y + 1, temp);
                self.set_temp(x, y, AMBIENT_TEMP);
                
                self.set_updated(x - 1, y + 1, true);
            } else {
                self.set(x + 1, y + 1, MaterialType::Sand);
                self.set(x, y, MaterialType::Empty);
                
                // Move temperature
                let temp = self.get_temp(x, y);
                self.set_temp(x + 1, y + 1, temp);
                self.set_temp(x, y, AMBIENT_TEMP);
                
                self.set_updated(x + 1, y + 1, true);
            }
            return;
        } else if can_fall_left {
            self.set(x - 1, y + 1, MaterialType::Sand);
            self.set(x, y, MaterialType::Empty);
            
            // Move temperature
            let temp = self.get_temp(x, y);
            self.set_temp(x - 1, y + 1, temp);
            self.set_temp(x, y, AMBIENT_TEMP);
            
            self.set_updated(x - 1, y + 1, true);
            return;
        } else if can_fall_right {
            self.set(x + 1, y + 1, MaterialType::Sand);
            self.set(x, y, MaterialType::Empty);
            
            // Move temperature
            let temp = self.get_temp(x, y);
            self.set_temp(x + 1, y + 1, temp);
            self.set_temp(x, y, AMBIENT_TEMP);
            
            self.set_updated(x + 1, y + 1, true);
            return;
        }
        
        // Check if we can displace water diagonally
        if x > 0 && self.get(x - 1, y + 1) == MaterialType::Water {
            self.set(x - 1, y + 1, MaterialType::Sand);
            self.set(x, y, MaterialType::Water);
            
            // Swap temperatures
            let temp1 = self.get_temp(x, y);
            let temp2 = self.get_temp(x - 1, y + 1);
            self.set_temp(x - 1, y + 1, temp1);
            self.set_temp(x, y, temp2);
            
            self.set_updated(x - 1, y + 1, true);
            return;
        } else if x < GRID_WIDTH - 1 && self.get(x + 1, y + 1) == MaterialType::Water {
            self.set(x + 1, y + 1, MaterialType::Sand);
            self.set(x, y, MaterialType::Water);
            
            // Swap temperatures
            let temp1 = self.get_temp(x, y);
            let temp2 = self.get_temp(x + 1, y + 1);
            self.set_temp(x + 1, y + 1, temp1);
            self.set_temp(x, y, temp2);
            
            self.set_updated(x + 1, y + 1, true);
            return;
        }
    }

    fn update_water(&mut self, x: usize, y: usize) {
        self.set_updated(x, y, true);
        
        // Water evaporates at high temperatures
        let temp = self.get_temp(x, y);
        if temp > 100.0 && rand::thread_rng().gen_bool(0.1) {
            self.set(x, y, MaterialType::Empty);
            return;
        }
        
        // If we're at the bottom, just try to spread
        if y == GRID_HEIGHT - 1 {
            // Try to spread left or right randomly
            let go_left = rand::thread_rng().gen_bool(0.5);
            if go_left {
                if x > 0 && self.get(x - 1, y) == MaterialType::Empty {
                    self.set(x - 1, y, MaterialType::Water);
                    self.set(x, y, MaterialType::Empty);
                    
                    // Move temperature
                    let temp = self.get_temp(x, y);
                    self.set_temp(x - 1, y, temp);
                    self.set_temp(x, y, AMBIENT_TEMP);
                    
                    self.set_updated(x - 1, y, true);
                }
            } else {
                if x < GRID_WIDTH - 1 && self.get(x + 1, y) == MaterialType::Empty {
                    self.set(x + 1, y, MaterialType::Water);
                    self.set(x, y, MaterialType::Empty);
                    
                    // Move temperature
                    let temp = self.get_temp(x, y);
                    self.set_temp(x + 1, y, temp);
                    self.set_temp(x, y, AMBIENT_TEMP);
                    
                    self.set_updated(x + 1, y, true);
                }
            }
            return;
        }
        
        // Check if we can fall down
        let below = self.get(x, y + 1);
        if below == MaterialType::Empty {
            // Fall down
            self.set(x, y + 1, MaterialType::Water);
            self.set(x, y, MaterialType::Empty);
            
            // Move temperature
            let temp = self.get_temp(x, y);
            self.set_temp(x, y + 1, temp);
            self.set_temp(x, y, AMBIENT_TEMP);
            
            self.set_updated(x, y + 1, true);
            return;
        }
        
        // Try to fall diagonally
        let can_fall_left = x > 0 && self.get(x - 1, y + 1) == MaterialType::Empty;
        let can_fall_right = x < GRID_WIDTH - 1 && self.get(x + 1, y + 1) == MaterialType::Empty;
        
        if can_fall_left && can_fall_right {
            // Choose randomly between left and right
            let go_left = rand::thread_rng().gen_bool(0.5);
            if go_left {
                self.set(x - 1, y + 1, MaterialType::Water);
                self.set(x, y, MaterialType::Empty);
                
                // Move temperature
                let temp = self.get_temp(x, y);
                self.set_temp(x - 1, y + 1, temp);
                self.set_temp(x, y, AMBIENT_TEMP);
                
                self.set_updated(x - 1, y + 1, true);
            } else {
                self.set(x + 1, y + 1, MaterialType::Water);
                self.set(x, y, MaterialType::Empty);
                
                // Move temperature
                let temp = self.get_temp(x, y);
                self.set_temp(x + 1, y + 1, temp);
                self.set_temp(x, y, AMBIENT_TEMP);
                
                self.set_updated(x + 1, y + 1, true);
            }
            return;
        } else if can_fall_left {
            self.set(x - 1, y + 1, MaterialType::Water);
            self.set(x, y, MaterialType::Empty);
            
            // Move temperature
            let temp = self.get_temp(x, y);
            self.set_temp(x - 1, y + 1, temp);
            self.set_temp(x, y, AMBIENT_TEMP);
            
            self.set_updated(x - 1, y + 1, true);
            return;
        } else if can_fall_right {
            self.set(x + 1, y + 1, MaterialType::Water);
            self.set(x, y, MaterialType::Empty);
            
            // Move temperature
            let temp = self.get_temp(x, y);
            self.set_temp(x + 1, y + 1, temp);
            self.set_temp(x, y, AMBIENT_TEMP);
            
            self.set_updated(x + 1, y + 1, true);
            return;
        }
        
        // Try to spread horizontally with higher probability (increased from 0.7 to 0.95)
        if rand::thread_rng().gen_bool(0.95) {
            // Try left and right with equal probability 
            let go_left = rand::thread_rng().gen_bool(0.5);
            if go_left {
                if x > 0 && self.get(x - 1, y) == MaterialType::Empty {
                    self.set(x - 1, y, MaterialType::Water);
                    self.set(x, y, MaterialType::Empty);
                    
                    // Move temperature
                    let temp = self.get_temp(x, y);
                    self.set_temp(x - 1, y, temp);
                    self.set_temp(x, y, AMBIENT_TEMP);
                    
                    self.set_updated(x - 1, y, true);
                    return;
                } else if x < GRID_WIDTH - 1 && self.get(x + 1, y) == MaterialType::Empty {
                    // If left is blocked, try right without waiting for next update
                    self.set(x + 1, y, MaterialType::Water);
                    self.set(x, y, MaterialType::Empty);
                    
                    // Move temperature
                    let temp = self.get_temp(x, y);
                    self.set_temp(x + 1, y, temp);
                    self.set_temp(x, y, AMBIENT_TEMP);
                    
                    self.set_updated(x + 1, y, true);
                    return;
                }
            } else {
                if x < GRID_WIDTH - 1 && self.get(x + 1, y) == MaterialType::Empty {
                    self.set(x + 1, y, MaterialType::Water);
                    self.set(x, y, MaterialType::Empty);
                    
                    // Move temperature
                    let temp = self.get_temp(x, y);
                    self.set_temp(x + 1, y, temp);
                    self.set_temp(x, y, AMBIENT_TEMP);
                    
                    self.set_updated(x + 1, y, true);
                    return;
                } else if x > 0 && self.get(x - 1, y) == MaterialType::Empty {
                    // If right is blocked, try left without waiting for next update
                    self.set(x - 1, y, MaterialType::Water);
                    self.set(x, y, MaterialType::Empty);
                    
                    // Move temperature
                    let temp = self.get_temp(x, y);
                    self.set_temp(x - 1, y, temp);
                    self.set_temp(x, y, AMBIENT_TEMP);
                    
                    self.set_updated(x - 1, y, true);
                    return;
                }
            }
        }
    }

    fn update_fire(&mut self, x: usize, y: usize) {
        self.set_updated(x, y, true);
        
        // Fire has a chance to burn out
        if rand::thread_rng().gen_bool(0.1) {
            self.set(x, y, MaterialType::Empty);
            self.set_temp(x, y, AMBIENT_TEMP);
            return;
        }
        
        // Fire rises
        if y > 0 && self.get(x, y - 1) == MaterialType::Empty {
            self.set(x, y - 1, MaterialType::Fire);
            self.set(x, y, MaterialType::Empty);
            
            // Move temperature
            let temp = self.get_temp(x, y);
            self.set_temp(x, y - 1, temp);
            self.set_temp(x, y, AMBIENT_TEMP);
            
            self.set_updated(x, y - 1, true);
            return;
        }
        
        // Fire can spread in all directions
        let directions = [
            (0, -1),  // Up
            (1, -1),  // Up-right
            (1, 0),   // Right
            (1, 1),   // Down-right
            (0, 1),   // Down
            (-1, 1),  // Down-left
            (-1, 0),  // Left
            (-1, -1), // Up-left
        ];
        
        // Heat neighbors and potentially ignite flammable materials
        for (dx, dy) in directions.iter() {
            let nx = x as isize + dx;
            let ny = y as isize + dy;
            
            if nx >= 0 && nx < GRID_WIDTH as isize && ny >= 0 && ny < GRID_HEIGHT as isize {
                let nx = nx as usize;
                let ny = ny as usize;
                
                // Increase temperature of neighbors
                let current_temp = self.get_temp(nx, ny);
                self.set_temp(nx, ny, (current_temp + 5.0).min(MAX_TEMP));
                
                let neighbor = self.get(nx, ny);
                
                // If the neighboring cell is flammable, chance to ignite it
                if neighbor != MaterialType::Empty {
                    let is_flammable = match neighbor {
                        MaterialType::Plant => true, // Plant is flammable
                        _ => false,
                    };
                    
                    if is_flammable && rand::thread_rng().gen_bool(0.1) {
                        self.set(nx, ny, MaterialType::Fire);
                        self.set_temp(nx, ny, 800.0);
                        self.set_updated(nx, ny, true);
                    }
                }
            }
        }
    }

    fn update_lava(&mut self, x: usize, y: usize) {
        self.set_updated(x, y, true);
        
        // Check if lava should turn into stone due to cooling
        let temp = self.get_temp(x, y);
        if temp < 700.0 && rand::thread_rng().gen_bool(0.05) {
            self.set(x, y, MaterialType::Stone);
            return;
        }
        
        // Lava falls down quickly like water
        if y < GRID_HEIGHT - 1 && self.get(x, y + 1) == MaterialType::Empty {
            self.set(x, y + 1, MaterialType::Lava);
            self.set(x, y, MaterialType::Empty);
            
            // Move temperature
            let temp = self.get_temp(x, y);
            self.set_temp(x, y + 1, temp);
            self.set_temp(x, y, AMBIENT_TEMP);
            
            self.set_updated(x, y + 1, true);
            return;
        }
        
        // Lava can replace water
        let directions = [
            (0, 1),   // Down
            (1, 0),   // Right
            (-1, 0),  // Left
            (0, -1),  // Up
        ];
        
        for (dx, dy) in directions.iter() {
            let nx = x as isize + dx;
            let ny = y as isize + dy;
            
            if nx >= 0 && nx < GRID_WIDTH as isize && ny >= 0 && ny < GRID_HEIGHT as isize {
                let nx = nx as usize;
                let ny = ny as usize;
                
                if self.get(nx, ny) == MaterialType::Water {
                    // Water turns to stone instantly when touching lava
                    self.set(nx, ny, MaterialType::Stone);
                    self.set_temp(nx, ny, 100.0);
                    self.set_updated(nx, ny, true);
                }
            }
        }
        
        // Lava falls diagonally quickly like water
        let can_fall_left = x > 0 && y < GRID_HEIGHT - 1 && self.get(x - 1, y + 1) == MaterialType::Empty;
        let can_fall_right = x < GRID_WIDTH - 1 && y < GRID_HEIGHT - 1 && self.get(x + 1, y + 1) == MaterialType::Empty;
        
        if can_fall_left && can_fall_right {
            // Choose randomly between left and right
            let go_left = rand::thread_rng().gen_bool(0.5);
            if go_left {
                self.set(x - 1, y + 1, MaterialType::Lava);
                self.set(x, y, MaterialType::Empty);
                
                // Move temperature
                let temp = self.get_temp(x, y);
                self.set_temp(x - 1, y + 1, temp);
                self.set_temp(x, y, AMBIENT_TEMP);
                
                self.set_updated(x - 1, y + 1, true);
            } else {
                self.set(x + 1, y + 1, MaterialType::Lava);
                self.set(x, y, MaterialType::Empty);
                
                // Move temperature
                let temp = self.get_temp(x, y);
                self.set_temp(x + 1, y + 1, temp);
                self.set_temp(x, y, AMBIENT_TEMP);
                
                self.set_updated(x + 1, y + 1, true);
            }
            return;
        } else if can_fall_left {
            self.set(x - 1, y + 1, MaterialType::Lava);
            self.set(x, y, MaterialType::Empty);
            
            // Move temperature
            let temp = self.get_temp(x, y);
            self.set_temp(x - 1, y + 1, temp);
            self.set_temp(x, y, AMBIENT_TEMP);
            
            self.set_updated(x - 1, y + 1, true);
            return;
        } else if can_fall_right {
            self.set(x + 1, y + 1, MaterialType::Lava);
            self.set(x, y, MaterialType::Empty);
            
            // Move temperature
            let temp = self.get_temp(x, y);
            self.set_temp(x + 1, y + 1, temp);
            self.set_temp(x, y, AMBIENT_TEMP);
            
            self.set_updated(x + 1, y + 1, true);
            return;
        }
        
        // Lava spreads horizontally very slowly (keep this slow)
        if rand::thread_rng().gen_bool(0.1) {
            // Pick a direction randomly
            let go_left = rand::thread_rng().gen_bool(0.5);
            if go_left {
                if x > 0 && self.get(x - 1, y) == MaterialType::Empty {
                    self.set(x - 1, y, MaterialType::Lava);
                    self.set(x, y, MaterialType::Empty);
                    
                    // Move temperature
                    let temp = self.get_temp(x, y);
                    self.set_temp(x - 1, y, temp);
                    self.set_temp(x, y, AMBIENT_TEMP);
                    
                    self.set_updated(x - 1, y, true);
                    return;
                }
                // Try right
                if x < GRID_WIDTH - 1 && self.get(x + 1, y) == MaterialType::Empty {
                    self.set(x + 1, y, MaterialType::Lava);
                    self.set(x, y, MaterialType::Empty);
                    
                    // Move temperature
                    let temp = self.get_temp(x, y);
                    self.set_temp(x + 1, y, temp);
                    self.set_temp(x, y, AMBIENT_TEMP);
                    
                    self.set_updated(x + 1, y, true);
                    return;
                }
            } else {
                // Try right
                if x < GRID_WIDTH - 1 && self.get(x + 1, y) == MaterialType::Empty {
                    self.set(x + 1, y, MaterialType::Lava);
                    self.set(x, y, MaterialType::Empty);
                    
                    // Move temperature
                    let temp = self.get_temp(x, y);
                    self.set_temp(x + 1, y, temp);
                    self.set_temp(x, y, AMBIENT_TEMP);
                    
                    self.set_updated(x + 1, y, true);
                    return;
                }
                // Try left
                if x > 0 && self.get(x - 1, y) == MaterialType::Empty {
                    self.set(x - 1, y, MaterialType::Lava);
                    self.set(x, y, MaterialType::Empty);
                    
                    // Move temperature
                    let temp = self.get_temp(x, y);
                    self.set_temp(x - 1, y, temp);
                    self.set_temp(x, y, AMBIENT_TEMP);
                    
                    self.set_updated(x - 1, y, true);
                    return;
                }
            }
        }

        // Heat neighbors and potentially ignite flammable materials
        for dy in -1..=1 {
            for dx in -1..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }

                let nx = x as isize + dx;
                let ny = y as isize + dy;

                if nx >= 0 && nx < GRID_WIDTH as isize && ny >= 0 && ny < GRID_HEIGHT as isize {
                    let nx = nx as usize;
                    let ny = ny as usize;
                    
                    // Increase temperature of neighbors significantly
                    let current_temp = self.get_temp(nx, ny);
                    self.set_temp(nx, ny, (current_temp + 10.0).min(MAX_TEMP));
                    
                    let neighbor = self.get(nx, ny);
                    
                    // Check if neighbor is flammable
                    let is_flammable = match neighbor {
                        MaterialType::Plant => true,
                        _ => false,
                    };
                    
                    if is_flammable && rand::thread_rng().gen_bool(0.3) {
                        self.set(nx, ny, MaterialType::Fire);
                        self.set_temp(nx, ny, 800.0); // High initial fire temperature
                        self.set_updated(nx, ny, true);
                    }
                    
                    // Water touching lava produces steam (not implemented yet)
                    // Would need a STEAM particle type
                    if neighbor == MaterialType::Water && rand::thread_rng().gen_bool(0.5) {
                        self.set(nx, ny, MaterialType::Empty);
                    }
                }
            }
        }
    }

    pub fn add_material(&mut self, x: usize, y: usize, brush_size: usize, material: MaterialType) {
        let start_x = x.saturating_sub(brush_size);
        let end_x = (x + brush_size).min(GRID_WIDTH - 1);
        let start_y = y.saturating_sub(brush_size);
        let end_y = (y + brush_size).min(GRID_HEIGHT - 1);
        
        let brush_size_squared = (brush_size * brush_size) as isize;
        
        for cy in start_y..=end_y {
            for cx in start_x..=end_x {
                let dx = cx as isize - x as isize;
                let dy = cy as isize - y as isize;
                if dx * dx + dy * dy <= brush_size_squared {
                    if material == MaterialType::Eraser {
                        self.set(cx, cy, MaterialType::Empty);
                        self.set_temp(cx, cy, AMBIENT_TEMP);
                    } else {
                        self.set(cx, cy, material);
                        
                        // Set appropriate initial temperatures
                        match material {
                            MaterialType::Fire => self.set_temp(cx, cy, 800.0),
                            MaterialType::Lava => self.set_temp(cx, cy, 1800.0),
                            _ => self.set_temp(cx, cy, AMBIENT_TEMP)
                        }
                    }
                }
            }
        }
    }
    
    pub fn get_width() -> usize {
        GRID_WIDTH
    }
    
    pub fn get_height() -> usize {
        GRID_HEIGHT
    }
    
    pub fn get_cell_size() -> usize {
        CELL_SIZE
    }
} 