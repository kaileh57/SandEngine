// File: simulation.rs
use rand::prelude::*;
use crate::constants::*;
use crate::material::MaterialType;

pub struct SandSimulation {
    // Use a flat vector for better cache locality
    grid: Vec<u8>,
    pub temp: Vec<f32>,       // Temperature for each cell
    vel_x: Vec<f32>,          // Horizontal velocity component
    vel_y: Vec<f32>,          // Vertical velocity component
    updated: Vec<bool>,
    pub brush_size: usize,
    pub current_material: MaterialType, // Current selected material
    pub cursor_pos: (usize, usize), // Store cursor position for UI
}

impl SandSimulation {
    pub fn new() -> Self {
        let grid_size = GRID_WIDTH * GRID_HEIGHT;
        let temp = vec![AMBIENT_TEMP; grid_size];
        
        Self {
            grid: vec![0; grid_size],
            temp,
            vel_x: vec![0.0; grid_size],
            vel_y: vec![0.0; grid_size],
            updated: vec![false; grid_size],
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
    pub fn get_vel_x(&self, x: usize, y: usize) -> f32 {
        if x < GRID_WIDTH && y < GRID_HEIGHT {
            self.vel_x[Self::get_index(x, y)]
        } else {
            0.0 // Out of bounds, return 0
        }
    }

    #[inline]
    pub fn set_vel_x(&mut self, x: usize, y: usize, value: f32) {
        if x < GRID_WIDTH && y < GRID_HEIGHT {
            let idx = Self::get_index(x, y);
            self.vel_x[idx] = value;
        }
    }

    #[inline]
    pub fn get_vel_y(&self, x: usize, y: usize) -> f32 {
        if x < GRID_WIDTH && y < GRID_HEIGHT {
            self.vel_y[Self::get_index(x, y)]
        } else {
            0.0 // Out of bounds, return 0
        }
    }

    #[inline]
    pub fn set_vel_y(&mut self, x: usize, y: usize, value: f32) {
        if x < GRID_WIDTH && y < GRID_HEIGHT {
            let idx = Self::get_index(x, y);
            self.vel_y[idx] = value;
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
            self.vel_x[i] = 0.0;
            self.vel_y[i] = 0.0;
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
                        MaterialType::Stone => self.update_stone(x, y),
                        MaterialType::Plant => self.update_plant(x, y),
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

    fn update_sand(&mut self, x: usize, y: usize) {
        // Mark as updated
        self.set_updated(x, y, true);
        
        // Apply gravity (acceleration)
        let mut vx = self.get_vel_x(x, y);
        let mut vy = self.get_vel_y(x, y) + GRAVITY;
        
        // Apply terminal velocity limit
        if vy > MAX_VELOCITY {
            vy = MAX_VELOCITY;
        }
        
        // Minimal horizontal drift to keep sand behavior
        if rand::thread_rng().gen_bool(0.03) {
            vx += (rand::thread_rng().gen::<f32>() - 0.5) * 0.08;
        }
        
        // Apply horizontal dampening (but keep some movement for natural feel)
        vx *= 0.8;
        
        // Calculate potential new position based on velocity
        let new_x = (x as f32 + vx).round() as usize;
        let new_y = (y as f32 + vy).round() as usize;
        
        // Boundary checks
        let new_x = new_x.min(GRID_WIDTH - 1);
        let new_y = new_y.min(GRID_HEIGHT - 1);
        
        // Check if we're in free fall (no support below)
        let free_fall = y < GRID_HEIGHT - 1 && self.get(x, y + 1) == MaterialType::Empty;
        
        // In free fall, use physics-based movement
        if free_fall {
            // If calculated position is different and free
            if (new_x != x || new_y != y) && new_y < GRID_HEIGHT && 
               self.get(new_x, new_y) == MaterialType::Empty {
                // Move according to velocity
                self.set(new_x, new_y, MaterialType::Sand);
                self.set(x, y, MaterialType::Empty);
                
                // Transfer velocity for continued acceleration
                self.set_vel_x(new_x, new_y, vx);
                self.set_vel_y(new_x, new_y, vy);
                
                self.set_updated(new_x, new_y, true);
                return;
            }
            
            // If calculated position isn't free, try straight down with current velocity
            if y < GRID_HEIGHT - 1 && self.get(x, y + 1) == MaterialType::Empty {
                self.set(x, y + 1, MaterialType::Sand);
                self.set(x, y, MaterialType::Empty);
                
                // Transfer velocity for continued acceleration
                self.set_vel_x(x, y + 1, vx);
                self.set_vel_y(x, y + 1, vy);
                
                self.set_updated(x, y + 1, true);
                return;
            }
        }
        
        // Not in free fall or previous move failed, now focus on pyramid formation

        // Check for supported structure (classic sand pyramid pattern)
        let is_supported = y < GRID_HEIGHT - 1 && 
            self.get(x, y + 1) != MaterialType::Empty;
            
        // Check if both diagonals below are supported
        let left_support = x > 0 && 
            y < GRID_HEIGHT - 1 && 
            self.get(x - 1, y + 1) != MaterialType::Empty;
            
        let right_support = x < GRID_WIDTH - 1 && 
            y < GRID_HEIGHT - 1 && 
            self.get(x + 1, y + 1) != MaterialType::Empty;
        
        // If both sides are supported or partially supported, high chance to stay put
        if is_supported && (left_support || right_support) && rand::thread_rng().gen_bool(0.95) {
            // Form a stable pyramid by staying in place
            // Retain a tiny bit of momentum for future movement possibility
            self.set_vel_x(x, y, vx * 0.05);
            self.set_vel_y(x, y, vy * 0.05);
            return;
        }
        
        // Only try diagonal movement if supported directly below but not on at least one side
        if is_supported && y < GRID_HEIGHT - 1 {
            let go_left = if !right_support && left_support {
                // Go right if right side is unsupported
                false
            } else if right_support && !left_support {
                // Go left if left side is unsupported
                true
            } else {
                // Random choice if both unsupported or both supported
                rand::thread_rng().gen_bool(0.5)
            };
            
            if go_left {
                // Try left diagonal
                if x > 0 && self.get(x - 1, y + 1) == MaterialType::Empty {
                    self.set(x - 1, y + 1, MaterialType::Sand);
                    self.set(x, y, MaterialType::Empty);
                    
                    // Transfer minimal velocity for pyramid formation
                    // But add a bit of the current velocity for natural behavior
                    self.set_vel_x(x - 1, y + 1, -0.1 + vx * 0.2);
                    self.set_vel_y(x - 1, y + 1, 0.3 + vy * 0.1);
                    
                    self.set_updated(x - 1, y + 1, true);
                    return;
                }
                // Then try right diagonal if left failed
                if x < GRID_WIDTH - 1 && self.get(x + 1, y + 1) == MaterialType::Empty {
                    self.set(x + 1, y + 1, MaterialType::Sand);
                    self.set(x, y, MaterialType::Empty);
                    
                    // Transfer minimal velocity for pyramid formation
                    // But add a bit of the current velocity for natural behavior
                    self.set_vel_x(x + 1, y + 1, 0.1 + vx * 0.2);
                    self.set_vel_y(x + 1, y + 1, 0.3 + vy * 0.1);
                    
                    self.set_updated(x + 1, y + 1, true);
                    return;
                }
            } else {
                // Try right diagonal first
                if x < GRID_WIDTH - 1 && self.get(x + 1, y + 1) == MaterialType::Empty {
                    self.set(x + 1, y + 1, MaterialType::Sand);
                    self.set(x, y, MaterialType::Empty);
                    
                    // Transfer minimal velocity for pyramid formation
                    // But add a bit of the current velocity for natural behavior
                    self.set_vel_x(x + 1, y + 1, 0.1 + vx * 0.2);
                    self.set_vel_y(x + 1, y + 1, 0.3 + vy * 0.1);
                    
                    self.set_updated(x + 1, y + 1, true);
                    return;
                }
                // Then try left diagonal if right failed
                if x > 0 && self.get(x - 1, y + 1) == MaterialType::Empty {
                    self.set(x - 1, y + 1, MaterialType::Sand);
                    self.set(x, y, MaterialType::Empty);
                    
                    // Transfer minimal velocity for pyramid formation
                    // But add a bit of the current velocity for natural behavior
                    self.set_vel_x(x - 1, y + 1, -0.1 + vx * 0.2);
                    self.set_vel_y(x - 1, y + 1, 0.3 + vy * 0.1);
                    
                    self.set_updated(x - 1, y + 1, true);
                    return;
                }
            }
        }
        
        // If we couldn't move, lose most but not all momentum (for potential future movement)
        self.set_vel_x(x, y, vx * 0.1);
        self.set_vel_y(x, y, vy * 0.1);
    }

    fn update_water(&mut self, x: usize, y: usize) {
        // Mark as updated
        self.set_updated(x, y, true);
        
        // Apply gravity (acceleration) - water accelerates faster than sand
        let mut vx = self.get_vel_x(x, y);
        let mut vy = self.get_vel_y(x, y) + WATER_GRAVITY;
        
        // Apply terminal velocity limit
        if vy > WATER_MAX_VELOCITY {
            vy = WATER_MAX_VELOCITY;
        }
        
        // Water keeps some horizontal momentum
        vx *= 0.95;
        
        // Calculate potential new position based on velocity
        let new_x = (x as f32 + vx).round() as usize;
        let new_y = (y as f32 + vy).round() as usize;
        
        // Boundary checks
        let new_x = new_x.min(GRID_WIDTH - 1);
        let new_y = new_y.min(GRID_HEIGHT - 1);
        
        // If new position is the same as current, try basic movement
        if new_x == x && new_y == y {
            // Try to move down
            if y < GRID_HEIGHT - 1 && self.get(x, y + 1) == MaterialType::Empty {
                self.set(x, y + 1, MaterialType::Water);
                self.set(x, y, MaterialType::Empty);
                // Transfer velocity
                self.set_vel_x(x, y + 1, vx);
                self.set_vel_y(x, y + 1, vy);
                self.set_updated(x, y + 1, true);
                return;
            }

            // Try to move diagonally down
            if y < GRID_HEIGHT - 1 {
                let left_first = rand::thread_rng().gen_bool(0.5);
                
                if left_first {
                    // Try left diagonal first
                    if x > 0 && self.get(x - 1, y + 1) == MaterialType::Empty {
                        self.set(x - 1, y + 1, MaterialType::Water);
                        self.set(x, y, MaterialType::Empty);
                        // Transfer velocity with horizontal component
                        self.set_vel_x(x - 1, y + 1, vx - 0.5);
                        self.set_vel_y(x - 1, y + 1, vy * 0.9);
                        self.set_updated(x - 1, y + 1, true);
                        return;
                    }
                    // Then right diagonal
                    if x < GRID_WIDTH - 1 && self.get(x + 1, y + 1) == MaterialType::Empty {
                        self.set(x + 1, y + 1, MaterialType::Water);
                        self.set(x, y, MaterialType::Empty);
                        // Transfer velocity with horizontal component
                        self.set_vel_x(x + 1, y + 1, vx + 0.5);
                        self.set_vel_y(x + 1, y + 1, vy * 0.9);
                        self.set_updated(x + 1, y + 1, true);
                        return;
                    }
                } else {
                    // Try right diagonal first
                    if x < GRID_WIDTH - 1 && self.get(x + 1, y + 1) == MaterialType::Empty {
                        self.set(x + 1, y + 1, MaterialType::Water);
                        self.set(x, y, MaterialType::Empty);
                        // Transfer velocity with horizontal component
                        self.set_vel_x(x + 1, y + 1, vx + 0.5);
                        self.set_vel_y(x + 1, y + 1, vy * 0.9);
                        self.set_updated(x + 1, y + 1, true);
                        return;
                    }
                    // Then left diagonal
                    if x > 0 && self.get(x - 1, y + 1) == MaterialType::Empty {
                        self.set(x - 1, y + 1, MaterialType::Water);
                        self.set(x, y, MaterialType::Empty);
                        // Transfer velocity with horizontal component
                        self.set_vel_x(x - 1, y + 1, vx - 0.5);
                        self.set_vel_y(x - 1, y + 1, vy * 0.9);
                        self.set_updated(x - 1, y + 1, true);
                        return;
                    }
                }
            }

            // Try to move horizontally with good momentum
            // Choose direction based on current horizontal velocity
            let spread_left = vx < 0.0 || (vx == 0.0 && rand::thread_rng().gen_bool(0.5));
            let horizontal_speed = vx.abs() + 0.5; // Add minimum flow speed
            
            if spread_left {
                // Try left with increasing horizontal velocity
                if x > 0 && self.get(x - 1, y) == MaterialType::Empty {
                    self.set(x - 1, y, MaterialType::Water);
                    self.set(x, y, MaterialType::Empty);
                    // Increase horizontal velocity for better flow
                    self.set_vel_x(x - 1, y, -horizontal_speed);
                    self.set_vel_y(x - 1, y, vy * 0.2); // Reduce vertical component
                    self.set_updated(x - 1, y, true);
                    return;
                }
                // Try right
                if x < GRID_WIDTH - 1 && self.get(x + 1, y) == MaterialType::Empty {
                    self.set(x + 1, y, MaterialType::Water);
                    self.set(x, y, MaterialType::Empty);
                    self.set_vel_x(x + 1, y, horizontal_speed); 
                    self.set_vel_y(x + 1, y, vy * 0.2);
                    self.set_updated(x + 1, y, true);
                    return;
                }
            } else {
                // Try right with increasing horizontal velocity
                if x < GRID_WIDTH - 1 && self.get(x + 1, y) == MaterialType::Empty {
                    self.set(x + 1, y, MaterialType::Water);
                    self.set(x, y, MaterialType::Empty);
                    self.set_vel_x(x + 1, y, horizontal_speed);
                    self.set_vel_y(x + 1, y, vy * 0.2);
                    self.set_updated(x + 1, y, true);
                    return;
                }
                // Try left
                if x > 0 && self.get(x - 1, y) == MaterialType::Empty {
                    self.set(x - 1, y, MaterialType::Water);
                    self.set(x, y, MaterialType::Empty);
                    self.set_vel_x(x - 1, y, -horizontal_speed);
                    self.set_vel_y(x - 1, y, vy * 0.2);
                    self.set_updated(x - 1, y, true);
                    return;
                }
            }
            
            // If we couldn't move, reduce momentum but keep some for when path clears
            self.set_vel_x(x, y, vx * 0.8);
            self.set_vel_y(x, y, vy * 0.5);
            return;
        }
        
        // Try moving to the calculated position based on velocity
        if new_x < GRID_WIDTH && new_y < GRID_HEIGHT && self.get(new_x, new_y) == MaterialType::Empty {
            self.set(new_x, new_y, MaterialType::Water);
            self.set(x, y, MaterialType::Empty);
            // Transfer velocity
            self.set_vel_x(new_x, new_y, vx);
            self.set_vel_y(new_x, new_y, vy);
            self.set_updated(new_x, new_y, true);
            return;
        } else {
            // Handle collision - try to find nearby empty spaces
            // For water, prioritize horizontal movement when blocked vertically
            
            // If blocked below, increase horizontal velocity for better flow
            if y < GRID_HEIGHT - 1 && self.get(x, y + 1) != MaterialType::Empty {
                vx += (rand::thread_rng().gen::<f32>() - 0.5) * 1.5; // Substantial random horizontal force
            }
            
            let order = [
                (0, 1),                      // Down
                (1, 1), (-1, 1),             // Down-right, down-left
                (1, 0), (-1, 0),             // Right, left
                (2, 0), (-2, 0),             // Further right, further left (allows faster flow)
                (0, -1), (1, -1), (-1, -1)   // Up, up-right, up-left
            ];
            
            for (dx, dy) in order.iter() {
                let nx = (x as isize + dx) as usize;
                let ny = (y as isize + dy) as usize;
                
                if nx < GRID_WIDTH && ny < GRID_HEIGHT && self.get(nx, ny) == MaterialType::Empty {
                    self.set(nx, ny, MaterialType::Water);
                    self.set(x, y, MaterialType::Empty);
                    
                    // Adjust velocity based on movement direction
                    let new_vx = if *dx > 0 { vx.max(0.5) } else if *dx < 0 { vx.min(-0.5) } else { vx * 0.8 };
                    let new_vy = if *dy > 0 { vy * 0.9 } else { vy * 0.4 };
                    
                    self.set_vel_x(nx, ny, new_vx);
                    self.set_vel_y(nx, ny, new_vy);
                    
                    self.set_updated(nx, ny, true);
                    return;
                }
            }
            
            // If we're here, we couldn't move at all
            // Water retains more momentum than sand when blocked
            self.set_vel_x(x, y, vx * 0.7);
            self.set_vel_y(x, y, vy * 0.3);
        }
    }

    fn update_fire(&mut self, x: usize, y: usize) {
        // Mark as updated
        self.set_updated(x, y, true);

        // Fire goes up and has a chance to disappear
        if rand::thread_rng().gen_bool(0.05) {
            self.set(x, y, MaterialType::Empty);
            self.set_temp(x, y, AMBIENT_TEMP);
            return;
        }

        // Apply updraft (negative gravity - fire rises)
        let mut vx = self.get_vel_x(x, y);
        let mut vy = self.get_vel_y(x, y) - FIRE_UPDRAFT;
        
        // Apply terminal velocity limit
        if vy < -FIRE_MAX_VELOCITY {
            vy = -FIRE_MAX_VELOCITY;
        }
        
        // Add some random horizontal movement to make fire look more realistic
        vx += (rand::thread_rng().gen::<f32>() - 0.5) * 0.4;
        
        // Calculate potential new position based on velocity
        let new_x = (x as f32 + vx).round() as usize;
        let new_y = (y as f32 + vy).round() as isize;
        
        // Check for boundaries - fire can move up and out of the grid
        if new_y < 0 {
            // Fire moves off the top of the grid
            self.set(x, y, MaterialType::Empty);
            self.set_temp(x, y, AMBIENT_TEMP);
            return;
        }
        
        let new_y = new_y as usize;
        let new_x = new_x.min(GRID_WIDTH - 1);
        
        // Try moving to the calculated position
        if new_x < GRID_WIDTH && new_y < GRID_HEIGHT && self.get(new_x, new_y) == MaterialType::Empty {
            self.set(new_x, new_y, MaterialType::Fire);
            self.set(x, y, MaterialType::Empty);
            
            // Move temperature with the fire
            let temp = self.get_temp(x, y);
            self.set_temp(new_x, new_y, temp);
            self.set_temp(x, y, AMBIENT_TEMP);
            
            // Transfer velocity
            self.set_vel_x(new_x, new_y, vx);
            self.set_vel_y(new_x, new_y, vy);
            
            self.set_updated(new_x, new_y, true);
            return;
        } else {
            // If we couldn't move to the calculated position, try basic movement
            // Try to move up
            if y > 0 && self.get(x, y - 1) == MaterialType::Empty {
                self.set(x, y - 1, MaterialType::Fire);
                self.set(x, y, MaterialType::Empty);
                
                // Move temperature with the fire
                let temp = self.get_temp(x, y);
                self.set_temp(x, y - 1, temp);
                self.set_temp(x, y, AMBIENT_TEMP);
                
                // Transfer velocity with boost upward
                self.set_vel_x(x, y - 1, vx);
                self.set_vel_y(x, y - 1, vy - 0.5); // More upward movement
                
                self.set_updated(x, y - 1, true);
                return;
            }
            
            // Try to move diagonally up
            if y > 0 {
                let left_first = rand::thread_rng().gen_bool(0.5);
                
                if left_first {
                    // Try left diagonal up
                    if x > 0 && self.get(x - 1, y - 1) == MaterialType::Empty {
                        self.set(x - 1, y - 1, MaterialType::Fire);
                        self.set(x, y, MaterialType::Empty);
                        
                        // Move temperature
                        let temp = self.get_temp(x, y);
                        self.set_temp(x - 1, y - 1, temp);
                        self.set_temp(x, y, AMBIENT_TEMP);
                        
                        // Transfer velocity with directional boost
                        self.set_vel_x(x - 1, y - 1, vx - 0.3);
                        self.set_vel_y(x - 1, y - 1, vy - 0.3);
                        
                        self.set_updated(x - 1, y - 1, true);
                        return;
                    }
                    // Try right diagonal up
                    if x < GRID_WIDTH - 1 && self.get(x + 1, y - 1) == MaterialType::Empty {
                        self.set(x + 1, y - 1, MaterialType::Fire);
                        self.set(x, y, MaterialType::Empty);
                        
                        // Move temperature
                        let temp = self.get_temp(x, y);
                        self.set_temp(x + 1, y - 1, temp);
                        self.set_temp(x, y, AMBIENT_TEMP);
                        
                        // Transfer velocity with directional boost
                        self.set_vel_x(x + 1, y - 1, vx + 0.3);
                        self.set_vel_y(x + 1, y - 1, vy - 0.3);
                        
                        self.set_updated(x + 1, y - 1, true);
                        return;
                    }
                } else {
                    // Same pattern but right first
                    // Try right diagonal up
                    if x < GRID_WIDTH - 1 && self.get(x + 1, y - 1) == MaterialType::Empty {
                        self.set(x + 1, y - 1, MaterialType::Fire);
                        self.set(x, y, MaterialType::Empty);
                        
                        // Move temperature
                        let temp = self.get_temp(x, y);
                        self.set_temp(x + 1, y - 1, temp);
                        self.set_temp(x, y, AMBIENT_TEMP);
                        
                        // Transfer velocity with directional boost
                        self.set_vel_x(x + 1, y - 1, vx + 0.3);
                        self.set_vel_y(x + 1, y - 1, vy - 0.3);
                        
                        self.set_updated(x + 1, y - 1, true);
                        return;
                    }
                    // Try left diagonal up
                    if x > 0 && self.get(x - 1, y - 1) == MaterialType::Empty {
                        self.set(x - 1, y - 1, MaterialType::Fire);
                        self.set(x, y, MaterialType::Empty);
                        
                        // Move temperature
                        let temp = self.get_temp(x, y);
                        self.set_temp(x - 1, y - 1, temp);
                        self.set_temp(x, y, AMBIENT_TEMP);
                        
                        // Transfer velocity with directional boost
                        self.set_vel_x(x - 1, y - 1, vx - 0.3);
                        self.set_vel_y(x - 1, y - 1, vy - 0.3);
                        
                        self.set_updated(x - 1, y - 1, true);
                        return;
                    }
                }
            }
            
            // If we still can't move, try horizontal movement
            let move_left = rand::thread_rng().gen_bool(0.5);
            if move_left {
                if x > 0 && self.get(x - 1, y) == MaterialType::Empty {
                    self.set(x - 1, y, MaterialType::Fire);
                    self.set(x, y, MaterialType::Empty);
                    
                    // Move temperature
                    let temp = self.get_temp(x, y);
                    self.set_temp(x - 1, y, temp);
                    self.set_temp(x, y, AMBIENT_TEMP);
                    
                    // Transfer velocity with slight upward boost
                    self.set_vel_x(x - 1, y, vx - 0.2);
                    self.set_vel_y(x - 1, y, vy - 0.1);
                    
                    self.set_updated(x - 1, y, true);
                    return;
                }
            } else {
                if x < GRID_WIDTH - 1 && self.get(x + 1, y) == MaterialType::Empty {
                    self.set(x + 1, y, MaterialType::Fire);
                    self.set(x, y, MaterialType::Empty);
                    
                    // Move temperature
                    let temp = self.get_temp(x, y);
                    self.set_temp(x + 1, y, temp);
                    self.set_temp(x, y, AMBIENT_TEMP);
                    
                    // Transfer velocity with slight upward boost
                    self.set_vel_x(x + 1, y, vx + 0.2);
                    self.set_vel_y(x + 1, y, vy - 0.1);
                    
                    self.set_updated(x + 1, y, true);
                    return;
                }
            }
        }

        // Heat up neighbors
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
                    
                    // Increase temperature of neighbors
                    let current_temp = self.get_temp(nx, ny);
                    self.set_temp(nx, ny, (current_temp + 5.0).min(MAX_TEMP));
                    
                    let neighbor = self.get(nx, ny);
                    
                    // Check if neighbor is flammable
                    let props = neighbor.get_properties();
                    
                    // Higher chance to ignite at higher temperatures
                    let ignite_chance = if props.flammable {
                        let base_chance = 0.1;
                        let temp_factor = (self.get_temp(nx, ny) - AMBIENT_TEMP) / 100.0;
                        (base_chance + temp_factor * 0.2).min(0.5)
                    } else {
                        0.0
                    };
                    
                    if props.flammable && rand::thread_rng().gen_bool(ignite_chance as f64) {
                        self.set(nx, ny, MaterialType::Fire);
                        self.set_temp(nx, ny, self.get_temp(nx, ny) + 200.0);
                        
                        // Set initial velocity for new fire - upward bias
                        self.set_vel_x(nx, ny, (rand::thread_rng().gen::<f32>() - 0.5) * 0.4);
                        self.set_vel_y(nx, ny, -0.5); // Initial upward movement
                        
                        self.set_updated(nx, ny, true);
                    }
                    
                    // Water extinguishes fire
                    if neighbor == MaterialType::Water && rand::thread_rng().gen_bool(0.3) {
                        self.set(x, y, MaterialType::Empty);
                        self.set_temp(x, y, AMBIENT_TEMP);
                        return;
                    }
                }
            }
        }
    }

    fn update_lava(&mut self, x: usize, y: usize) {
        // Mark as updated
        self.set_updated(x, y, true);

        // Check if lava is cooling to stone
        let current_temp = self.get_temp(x, y);
        if current_temp < 1000.0 && rand::thread_rng().gen_bool(0.05) {
            self.set(x, y, MaterialType::Stone);
            self.set_vel_x(x, y, 0.0);
            self.set_vel_y(x, y, 0.0);
            return;
        }

        // Apply gravity (acceleration) - lava moves slower than water
        let mut vx = self.get_vel_x(x, y);
        let mut vy = self.get_vel_y(x, y) + LAVA_GRAVITY;
        
        // Apply terminal velocity limit - lava is more viscous
        if vy > LAVA_MAX_VELOCITY {
            vy = LAVA_MAX_VELOCITY;
        }
        
        // Lava has higher friction, so horizontal momentum decays slower to promote flow
        vx *= 0.95;
        
        // Calculate potential new position based on velocity
        let new_x = (x as f32 + vx).round() as usize;
        let new_y = (y as f32 + vy).round() as usize;
        
        // Boundary checks
        let new_x = new_x.min(GRID_WIDTH - 1);
        let new_y = new_y.min(GRID_HEIGHT - 1);

        // Check if we're in free fall
        let free_fall = y < GRID_HEIGHT - 1 && self.get(x, y + 1) == MaterialType::Empty;
        
        if free_fall {
            // If calculated position is different and free
            if (new_x != x || new_y != y) && new_y < GRID_HEIGHT && 
               self.get(new_x, new_y) == MaterialType::Empty {
                // Move according to velocity
                self.set(new_x, new_y, MaterialType::Lava);
                self.set(x, y, MaterialType::Empty);
                
                // Move temperature
                let temp = self.get_temp(x, y);
                self.set_temp(new_x, new_y, temp);
                self.set_temp(x, y, AMBIENT_TEMP);
                
                // Transfer velocity
                self.set_vel_x(new_x, new_y, vx);
                self.set_vel_y(new_x, new_y, vy);
                
                self.set_updated(new_x, new_y, true);
                return;
            }
            
            // If calculated position isn't free, try straight down
            if y < GRID_HEIGHT - 1 && self.get(x, y + 1) == MaterialType::Empty {
                self.set(x, y + 1, MaterialType::Lava);
                self.set(x, y, MaterialType::Empty);
                
                // Move temperature
                let temp = self.get_temp(x, y);
                self.set_temp(x, y + 1, temp);
                self.set_temp(x, y, AMBIENT_TEMP);
                
                // Transfer velocity with some dampening
                self.set_vel_x(x, y + 1, vx * 0.95);
                self.set_vel_y(x, y + 1, vy);
                
                self.set_updated(x, y + 1, true);
                return;
            }
        }
        
        // Try to move diagonally down (with lower probability)
        if y < GRID_HEIGHT - 1 && rand::thread_rng().gen_bool(0.8) {
            let left_first = rand::thread_rng().gen_bool(0.5);
            
            if left_first {
                // Try left diagonal
                if x > 0 && self.get(x - 1, y + 1) == MaterialType::Empty {
                    self.set(x - 1, y + 1, MaterialType::Lava);
                    self.set(x, y, MaterialType::Empty);
                    
                    // Move temperature
                    let temp = self.get_temp(x, y);
                    self.set_temp(x - 1, y + 1, temp);
                    self.set_temp(x, y, AMBIENT_TEMP);
                    
                    // Slower diagonal movement for lava
                    self.set_vel_x(x - 1, y + 1, vx * 0.7 - 0.2);
                    self.set_vel_y(x - 1, y + 1, vy * 0.7);
                    
                    self.set_updated(x - 1, y + 1, true);
                    return;
                }
                // Then right diagonal
                if x < GRID_WIDTH - 1 && self.get(x + 1, y + 1) == MaterialType::Empty {
                    self.set(x + 1, y + 1, MaterialType::Lava);
                    self.set(x, y, MaterialType::Empty);
                    
                    // Move temperature
                    let temp = self.get_temp(x, y);
                    self.set_temp(x + 1, y + 1, temp);
                    self.set_temp(x, y, AMBIENT_TEMP);
                    
                    // Slower diagonal movement for lava
                    self.set_vel_x(x + 1, y + 1, vx * 0.7 + 0.2);
                    self.set_vel_y(x + 1, y + 1, vy * 0.7);
                    
                    self.set_updated(x + 1, y + 1, true);
                    return;
                }
            } else {
                // Try right diagonal first
                if x < GRID_WIDTH - 1 && self.get(x + 1, y + 1) == MaterialType::Empty {
                    self.set(x + 1, y + 1, MaterialType::Lava);
                    self.set(x, y, MaterialType::Empty);
                    
                    // Move temperature
                    let temp = self.get_temp(x, y);
                    self.set_temp(x + 1, y + 1, temp);
                    self.set_temp(x, y, AMBIENT_TEMP);
                    
                    // Slower diagonal movement for lava
                    self.set_vel_x(x + 1, y + 1, vx * 0.7 + 0.2);
                    self.set_vel_y(x + 1, y + 1, vy * 0.7);
                    
                    self.set_updated(x + 1, y + 1, true);
                    return;
                }
                // Then left diagonal
                if x > 0 && self.get(x - 1, y + 1) == MaterialType::Empty {
                    self.set(x - 1, y + 1, MaterialType::Lava);
                    self.set(x, y, MaterialType::Empty);
                    
                    // Move temperature
                    let temp = self.get_temp(x, y);
                    self.set_temp(x - 1, y + 1, temp);
                    self.set_temp(x, y, AMBIENT_TEMP);
                    
                    // Slower diagonal movement for lava
                    self.set_vel_x(x - 1, y + 1, vx * 0.7 - 0.2);
                    self.set_vel_y(x - 1, y + 1, vy * 0.7);
                    
                    self.set_updated(x - 1, y + 1, true);
                    return;
                }
            }
        }

        // Try to move horizontally (lava spreading) - increased probability for better flow
        // Always try horizontal movement when blocked below
        let should_spread = y >= GRID_HEIGHT - 1 || 
                            self.get(x, y + 1) != MaterialType::Empty || 
                            rand::thread_rng().gen_bool(0.7);
        
        if should_spread {
            // Choose direction based on velocity or random
            let spread_left = if vx.abs() > 0.1 {
                vx < 0.0 
            } else {
                rand::thread_rng().gen_bool(0.5)
            };
            
            // Calculate flow speed - lava should flow slowly but consistently
            // Higher temperature = more fluid
            let temp_factor = ((current_temp - 1000.0) / 800.0).max(0.0).min(1.0);
            let base_speed = 0.2 + temp_factor * 0.3; // 0.2-0.5 range based on temperature
            let flow_speed = base_speed + vx.abs() * 0.3;
            
            if spread_left {
                // Try left
                if x > 0 && self.get(x - 1, y) == MaterialType::Empty {
                    self.set(x - 1, y, MaterialType::Lava);
                    self.set(x, y, MaterialType::Empty);
                    
                    // Move temperature
                    let temp = self.get_temp(x, y);
                    self.set_temp(x - 1, y, temp);
                    self.set_temp(x, y, AMBIENT_TEMP);
                    
                    // Slow horizontal movement for lava
                    self.set_vel_x(x - 1, y, -flow_speed);
                    self.set_vel_y(x - 1, y, vy * 0.3);
                    
                    self.set_updated(x - 1, y, true);
                    return;
                }
                // Try right if left failed
                if x < GRID_WIDTH - 1 && self.get(x + 1, y) == MaterialType::Empty {
                    self.set(x + 1, y, MaterialType::Lava);
                    self.set(x, y, MaterialType::Empty);
                    
                    // Move temperature
                    let temp = self.get_temp(x, y);
                    self.set_temp(x + 1, y, temp);
                    self.set_temp(x, y, AMBIENT_TEMP);
                    
                    // Slow horizontal movement for lava
                    self.set_vel_x(x + 1, y, flow_speed);
                    self.set_vel_y(x + 1, y, vy * 0.3);
                    
                    self.set_updated(x + 1, y, true);
                    return;
                }
            } else {
                // Try right first
                if x < GRID_WIDTH - 1 && self.get(x + 1, y) == MaterialType::Empty {
                    self.set(x + 1, y, MaterialType::Lava);
                    self.set(x, y, MaterialType::Empty);
                    
                    // Move temperature
                    let temp = self.get_temp(x, y);
                    self.set_temp(x + 1, y, temp);
                    self.set_temp(x, y, AMBIENT_TEMP);
                    
                    // Slow horizontal movement for lava
                    self.set_vel_x(x + 1, y, flow_speed);
                    self.set_vel_y(x + 1, y, vy * 0.3);
                    
                    self.set_updated(x + 1, y, true);
                    return;
                }
                // Try left if right failed
                if x > 0 && self.get(x - 1, y) == MaterialType::Empty {
                    self.set(x - 1, y, MaterialType::Lava);
                    self.set(x, y, MaterialType::Empty);
                    
                    // Move temperature
                    let temp = self.get_temp(x, y);
                    self.set_temp(x - 1, y, temp);
                    self.set_temp(x, y, AMBIENT_TEMP);
                    
                    // Slow horizontal movement for lava
                    self.set_vel_x(x - 1, y, -flow_speed);
                    self.set_vel_y(x - 1, y, vy * 0.3);
                    
                    self.set_updated(x - 1, y, true);
                    return;
                }
            }
        }
        
        // Try to move up diagonally (rare, but can happen for very hot lava or when blocked)
        // This helps lava flow over small obstacles
        if y > 0 && current_temp > 1400.0 && rand::thread_rng().gen_bool(0.05) {
            let left_first = rand::thread_rng().gen_bool(0.5);
            
            if left_first {
                // Try left up diagonal
                if x > 0 && self.get(x - 1, y - 1) == MaterialType::Empty {
                    self.set(x - 1, y - 1, MaterialType::Lava);
                    self.set(x, y, MaterialType::Empty);
                    
                    // Move temperature
                    let temp = self.get_temp(x, y);
                    self.set_temp(x - 1, y - 1, temp - 50.0); // Cooling from upward movement
                    self.set_temp(x, y, AMBIENT_TEMP);
                    
                    // Very slow upward movement
                    self.set_vel_x(x - 1, y - 1, -0.2);
                    self.set_vel_y(x - 1, y - 1, -0.1);
                    
                    self.set_updated(x - 1, y - 1, true);
                    return;
                }
                // Try right up diagonal
                if x < GRID_WIDTH - 1 && self.get(x + 1, y - 1) == MaterialType::Empty {
                    self.set(x + 1, y - 1, MaterialType::Lava);
                    self.set(x, y, MaterialType::Empty);
                    
                    // Move temperature
                    let temp = self.get_temp(x, y);
                    self.set_temp(x + 1, y - 1, temp - 50.0); // Cooling from upward movement
                    self.set_temp(x, y, AMBIENT_TEMP);
                    
                    // Very slow upward movement
                    self.set_vel_x(x + 1, y - 1, 0.2);
                    self.set_vel_y(x + 1, y - 1, -0.1);
                    
                    self.set_updated(x + 1, y - 1, true);
                    return;
                }
            } else {
                // Same checks but in opposite order
                if x < GRID_WIDTH - 1 && self.get(x + 1, y - 1) == MaterialType::Empty {
                    self.set(x + 1, y - 1, MaterialType::Lava);
                    self.set(x, y, MaterialType::Empty);
                    
                    // Move temperature
                    let temp = self.get_temp(x, y);
                    self.set_temp(x + 1, y - 1, temp - 50.0);
                    self.set_temp(x, y, AMBIENT_TEMP);
                    
                    // Very slow upward movement
                    self.set_vel_x(x + 1, y - 1, 0.2);
                    self.set_vel_y(x + 1, y - 1, -0.1);
                    
                    self.set_updated(x + 1, y - 1, true);
                    return;
                }
                if x > 0 && self.get(x - 1, y - 1) == MaterialType::Empty {
                    self.set(x - 1, y - 1, MaterialType::Lava);
                    self.set(x, y, MaterialType::Empty);
                    
                    // Move temperature
                    let temp = self.get_temp(x, y);
                    self.set_temp(x - 1, y - 1, temp - 50.0);
                    self.set_temp(x, y, AMBIENT_TEMP);
                    
                    // Very slow upward movement
                    self.set_vel_x(x - 1, y - 1, -0.2);
                    self.set_vel_y(x - 1, y - 1, -0.1);
                    
                    self.set_updated(x - 1, y - 1, true);
                    return;
                }
            }
        }
        
        // If we couldn't move, lava retains momentum for potential future flow
        self.set_vel_x(x, y, vx * 0.8);
        self.set_vel_y(x, y, vy * 0.2);

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
                    let props = neighbor.get_properties();
                    if props.flammable && rand::thread_rng().gen_bool(0.3) {
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

    fn update_stone(&mut self, x: usize, y: usize) {
        // Mark as updated
        self.set_updated(x, y, true);
        
        // Stone is very rigid but still affected by gravity
        let mut vx = self.get_vel_x(x, y);
        let mut vy = self.get_vel_y(x, y) + STONE_GRAVITY;
        
        // Apply terminal velocity limit - stone falls fast
        if vy > STONE_MAX_VELOCITY {
            vy = STONE_MAX_VELOCITY;
        }
        
        // Stone barely moves horizontally
        vx *= 0.7;
        
        // Calculate potential new position based on velocity
        let new_x = (x as f32 + vx).round() as usize;
        let new_y = (y as f32 + vy).round() as usize;
        
        // Boundary checks
        let new_x = new_x.min(GRID_WIDTH - 1);
        let new_y = new_y.min(GRID_HEIGHT - 1);
        
        // Check if we're in free fall (no support below)
        let free_fall = y < GRID_HEIGHT - 1 && self.get(x, y + 1) == MaterialType::Empty;
        
        if free_fall {
            // If calculated position is different and free
            if (new_x != x || new_y != y) && new_y < GRID_HEIGHT && 
               self.get(new_x, new_y) == MaterialType::Empty {
                // Move according to velocity
                self.set(new_x, new_y, MaterialType::Stone);
                self.set(x, y, MaterialType::Empty);
                
                // Transfer velocity for continued acceleration
                self.set_vel_x(new_x, new_y, vx);
                self.set_vel_y(new_x, new_y, vy);
                
                self.set_updated(new_x, new_y, true);
                return;
            }
            
            // If calculated position isn't free, try straight down
            if y < GRID_HEIGHT - 1 && self.get(x, y + 1) == MaterialType::Empty {
                self.set(x, y + 1, MaterialType::Stone);
                self.set(x, y, MaterialType::Empty);
                
                // Transfer velocity with some dampening (stone is heavy)
                self.set_vel_x(x, y + 1, vx * 0.9);
                self.set_vel_y(x, y + 1, vy);
                
                self.set_updated(x, y + 1, true);
                return;
            }
        }
        
        // Not in free fall or failed to move, now focus on stable pile formation
        // Stone is more rigid than sand, so it requires more momentum to move diagonally
        
        // If blocked below, check if we have enough momentum to move diagonally
        // Stone needs substantial momentum to move diagonally (higher threshold)
        if y < GRID_HEIGHT - 1 {
            let momentum = vy.abs();
            
            if momentum > 1.5 { // High momentum threshold for stone
                let left_first = rand::thread_rng().gen_bool(0.5);
                
                if left_first {
                    // Try left diagonal
                    if x > 0 && self.get(x - 1, y + 1) == MaterialType::Empty {
                        self.set(x - 1, y + 1, MaterialType::Stone);
                        self.set(x, y, MaterialType::Empty);
                        
                        // Significant momentum reduction after diagonal movement
                        self.set_vel_x(x - 1, y + 1, vx * 0.3 - 0.1);
                        self.set_vel_y(x - 1, y + 1, vy * 0.3);
                        
                        self.set_updated(x - 1, y + 1, true);
                        return;
                    }
                    // Try right diagonal
                    if x < GRID_WIDTH - 1 && self.get(x + 1, y + 1) == MaterialType::Empty {
                        self.set(x + 1, y + 1, MaterialType::Stone);
                        self.set(x, y, MaterialType::Empty);
                        
                        // Significant momentum reduction after diagonal movement
                        self.set_vel_x(x + 1, y + 1, vx * 0.3 + 0.1);
                        self.set_vel_y(x + 1, y + 1, vy * 0.3);
                        
                        self.set_updated(x + 1, y + 1, true);
                        return;
                    }
                } else {
                    // Try right diagonal first
                    if x < GRID_WIDTH - 1 && self.get(x + 1, y + 1) == MaterialType::Empty {
                        self.set(x + 1, y + 1, MaterialType::Stone);
                        self.set(x, y, MaterialType::Empty);
                        
                        // Significant momentum reduction after diagonal movement
                        self.set_vel_x(x + 1, y + 1, vx * 0.3 + 0.1);
                        self.set_vel_y(x + 1, y + 1, vy * 0.3);
                        
                        self.set_updated(x + 1, y + 1, true);
                        return;
                    }
                    // Try left diagonal
                    if x > 0 && self.get(x - 1, y + 1) == MaterialType::Empty {
                        self.set(x - 1, y + 1, MaterialType::Stone);
                        self.set(x, y, MaterialType::Empty);
                        
                        // Significant momentum reduction after diagonal movement
                        self.set_vel_x(x - 1, y + 1, vx * 0.3 - 0.1);
                        self.set_vel_y(x - 1, y + 1, vy * 0.3);
                        
                        self.set_updated(x - 1, y + 1, true);
                        return;
                    }
                }
            }
        }
        
        // If we couldn't move, retain very little momentum
        self.set_vel_x(x, y, vx * 0.05);
        self.set_vel_y(x, y, vy * 0.05);
    }

    fn update_plant(&mut self, x: usize, y: usize) {
        // Mark as updated
        self.set_updated(x, y, true);
        
        // Plant is rigid but still affected by gravity
        let mut vx = self.get_vel_x(x, y);
        let mut vy = self.get_vel_y(x, y) + PLANT_GRAVITY;
        
        // Apply terminal velocity limit
        if vy > PLANT_MAX_VELOCITY {
            vy = PLANT_MAX_VELOCITY;
        }
        
        // Plant has minimal horizontal movement
        vx *= 0.6;
        
        // Calculate potential new position based on velocity
        let new_x = (x as f32 + vx).round() as usize;
        let new_y = (y as f32 + vy).round() as usize;
        
        // Boundary checks
        let new_x = new_x.min(GRID_WIDTH - 1);
        let new_y = new_y.min(GRID_HEIGHT - 1);
        
        // Check if we're in free fall (no support below)
        let free_fall = y < GRID_HEIGHT - 1 && self.get(x, y + 1) == MaterialType::Empty;
        
        if free_fall {
            // If calculated position is different and free
            if (new_x != x || new_y != y) && new_y < GRID_HEIGHT && 
               self.get(new_x, new_y) == MaterialType::Empty {
                // Move according to velocity
                self.set(new_x, new_y, MaterialType::Plant);
                self.set(x, y, MaterialType::Empty);
                
                // Transfer velocity for continued acceleration
                self.set_vel_x(new_x, new_y, vx);
                self.set_vel_y(new_x, new_y, vy);
                
                self.set_updated(new_x, new_y, true);
                return;
            }
            
            // If calculated position isn't free, try straight down
            if y < GRID_HEIGHT - 1 && self.get(x, y + 1) == MaterialType::Empty {
                self.set(x, y + 1, MaterialType::Plant);
                self.set(x, y, MaterialType::Empty);
                
                // Transfer velocity for continued acceleration
                self.set_vel_x(x, y + 1, vx * 0.9);
                self.set_vel_y(x, y + 1, vy);
                
                self.set_updated(x, y + 1, true);
                return;
            }
        }
        
        // Not in free fall or failed to move, now focus on stable pile formation
        // Plant is more organic than stone, forming looser piles

        // Plant can move diagonally more easily than stone, but less than sand
        if y < GRID_HEIGHT - 1 {
            // This creates piles that are somewhat less structured than stone
            let momentum = vy.abs();
            
            if momentum > 0.8 || rand::thread_rng().gen_bool(0.15) { // Lower threshold, plus random chance
                let left_first = rand::thread_rng().gen_bool(0.5);
                
                if left_first {
                    // Try left diagonal
                    if x > 0 && self.get(x - 1, y + 1) == MaterialType::Empty {
                        self.set(x - 1, y + 1, MaterialType::Plant);
                        self.set(x, y, MaterialType::Empty);
                        
                        // Mild momentum reduction after diagonal movement
                        self.set_vel_x(x - 1, y + 1, vx * 0.5 - 0.2);
                        self.set_vel_y(x - 1, y + 1, vy * 0.5);
                        
                        self.set_updated(x - 1, y + 1, true);
                        return;
                    }
                    // Try right diagonal
                    if x < GRID_WIDTH - 1 && self.get(x + 1, y + 1) == MaterialType::Empty {
                        self.set(x + 1, y + 1, MaterialType::Plant);
                        self.set(x, y, MaterialType::Empty);
                        
                        // Mild momentum reduction after diagonal movement
                        self.set_vel_x(x + 1, y + 1, vx * 0.5 + 0.2);
                        self.set_vel_y(x + 1, y + 1, vy * 0.5);
                        
                        self.set_updated(x + 1, y + 1, true);
                        return;
                    }
                } else {
                    // Same pattern with right first
                    // Try right diagonal first
                    if x < GRID_WIDTH - 1 && self.get(x + 1, y + 1) == MaterialType::Empty {
                        self.set(x + 1, y + 1, MaterialType::Plant);
                        self.set(x, y, MaterialType::Empty);
                        
                        // Mild momentum reduction after diagonal movement
                        self.set_vel_x(x + 1, y + 1, vx * 0.5 + 0.2);
                        self.set_vel_y(x + 1, y + 1, vy * 0.5);
                        
                        self.set_updated(x + 1, y + 1, true);
                        return;
                    }
                    // Try left diagonal
                    if x > 0 && self.get(x - 1, y + 1) == MaterialType::Empty {
                        self.set(x - 1, y + 1, MaterialType::Plant);
                        self.set(x, y, MaterialType::Empty);
                        
                        // Mild momentum reduction after diagonal movement
                        self.set_vel_x(x - 1, y + 1, vx * 0.5 - 0.2);
                        self.set_vel_y(x - 1, y + 1, vy * 0.5);
                        
                        self.set_updated(x - 1, y + 1, true);
                        return;
                    }
                }
            }
        }
        
        // If we couldn't move, lose momentum gradually (plants retain more than stone)
        self.set_vel_x(x, y, vx * 0.2);
        self.set_vel_y(x, y, vy * 0.1);
    }

    // Helper method to get temperature color modification
    pub fn get_temp_color_modifier(&self, x: usize, y: usize) -> [i16; 3] {
        let temp = self.get_temp(x, y);
        let material = self.get(x, y);
        
        // No temperature visualization for empty cells
        if material == MaterialType::Empty {
            return [0, 0, 0];
        }
        
        // Special cases for fire/lava which have their own visualization
        if material == MaterialType::Fire || material == MaterialType::Lava {
            return [0, 0, 0];
        }
        
        // Calculate temperature factor (-1.0 to 1.0)
        let temp_factor = ((temp - AMBIENT_TEMP) / 150.0).max(-1.0).min(1.0);
        
        // Red increases with temperature, blue decreases
        let r_mod = (temp_factor * 50.0) as i16;
        let g_mod = (temp_factor * 20.0) as i16;
        let b_mod = (-temp_factor * 30.0) as i16;
        
        [r_mod, g_mod, b_mod]
    }

    pub fn draw(&self, frame: &mut [u8]) {
        for y in 0..GRID_HEIGHT {
            for x in 0..GRID_WIDTH {
                let material = self.get(x, y);
                let props = material.get_properties();
                let base_color = props.color;
                
                // Apply temperature color modification
                let temp_mod = self.get_temp_color_modifier(x, y);
                
                let r = (base_color[0] as i16 + temp_mod[0]).max(0).min(255) as u8;
                let g = (base_color[1] as i16 + temp_mod[1]).max(0).min(255) as u8;
                let b = (base_color[2] as i16 + temp_mod[2]).max(0).min(255) as u8;
                let a = base_color[3];
                
                // Draw the cell (scaled by CELL_SIZE)
                for dy in 0..CELL_SIZE {
                    for dx in 0..CELL_SIZE {
                        let px = x * CELL_SIZE + dx;
                        let py = y * CELL_SIZE + dy;
                        
                        // Calculate index in frame buffer
                        let idx = ((py * WINDOW_WIDTH as usize) + px) * 4;
                        
                        if idx + 3 < frame.len() {
                            frame[idx] = r;
                            frame[idx + 1] = g;
                            frame[idx + 2] = b;
                            frame[idx + 3] = a;
                        }
                    }
                }
            }
        }
        
        // Draw a border around the simulation area
        for y in 0..HEIGHT as usize {
            for x in 0..WIDTH as usize {
                // Draw border (1 pixel width)
                if x < 1 || x >= WIDTH as usize - 1 || 
                   y < 1 || y >= HEIGHT as usize - 1 {
                    let idx = (y * WINDOW_WIDTH as usize + x) * 4;
                    if idx + 3 < frame.len() {
                        frame[idx..idx+4].copy_from_slice(&C_BORDER);
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
                        self.set_vel_x(cx, cy, 0.0);
                        self.set_vel_y(cx, cy, 0.0);
                    } else {
                        self.set(cx, cy, material);
                        
                        // Add some initial velocity variation based on material
                        let random_vx = (rand::thread_rng().gen::<f32>() - 0.5) * 0.4;
                        let random_vy = rand::thread_rng().gen::<f32>() * 0.2;
                        
                        match material {
                            MaterialType::Fire => {
                                self.set_temp(cx, cy, 800.0);
                                // Fire starts with upward velocity
                                self.set_vel_x(cx, cy, random_vx);
                                self.set_vel_y(cx, cy, -0.5 + random_vy);
                            },
                            MaterialType::Lava => {
                                self.set_temp(cx, cy, 1800.0);
                                // Lava starts with small downward velocity
                                self.set_vel_x(cx, cy, random_vx * 0.5);
                                self.set_vel_y(cx, cy, 0.2 + random_vy * 0.5);
                            },
                            MaterialType::Water => {
                                self.set_temp(cx, cy, AMBIENT_TEMP);
                                // Water starts with more horizontal spread
                                self.set_vel_x(cx, cy, random_vx * 1.5);
                                self.set_vel_y(cx, cy, 0.3 + random_vy);
                            },
                            MaterialType::Sand => {
                                self.set_temp(cx, cy, AMBIENT_TEMP);
                                // Sand has some initial downward velocity
                                self.set_vel_x(cx, cy, random_vx * 0.8);
                                self.set_vel_y(cx, cy, 0.5 + random_vy);
                            },
                            MaterialType::Stone => {
                                self.set_temp(cx, cy, AMBIENT_TEMP);
                                // Stone has higher initial downward velocity
                                self.set_vel_x(cx, cy, random_vx * 0.4); // Less horizontal movement
                                self.set_vel_y(cx, cy, 0.6 + random_vy * 0.8);
                            },
                            MaterialType::Plant => {
                                self.set_temp(cx, cy, AMBIENT_TEMP);
                                // Plant has moderate initial velocity
                                self.set_vel_x(cx, cy, random_vx * 0.6);
                                self.set_vel_y(cx, cy, 0.4 + random_vy * 0.7);
                            },
                            _ => {
                                self.set_temp(cx, cy, AMBIENT_TEMP);
                                self.set_vel_x(cx, cy, 0.0);
                                self.set_vel_y(cx, cy, 0.0);
                            }
                        }
                    }
                }
            }
        }
    }
}