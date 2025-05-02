// File: simulation.rs
use rand::prelude::*;
use crate::constants::*;
use crate::material::{MaterialType, MaterialProperties};

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
        let mut temp = vec![AMBIENT_TEMP; GRID_WIDTH * GRID_HEIGHT];
        
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

    fn update_sand(&mut self, x: usize, y: usize) {
        // Mark as updated
        self.set_updated(x, y, true);

        // Try to move down
        if y < GRID_HEIGHT - 1 && self.get(x, y + 1) == MaterialType::Empty {
            self.set(x, y + 1, MaterialType::Sand);
            self.set(x, y, MaterialType::Empty);
            self.set_updated(x, y + 1, true);
            return;
        }

        // Try to move diagonally
        if y < GRID_HEIGHT - 1 {
            let left_first = rand::thread_rng().gen_bool(0.5);
            
            if left_first {
                // Try left diagonal first
                if x > 0 && self.get(x - 1, y + 1) == MaterialType::Empty {
                    self.set(x - 1, y + 1, MaterialType::Sand);
                    self.set(x, y, MaterialType::Empty);
                    self.set_updated(x - 1, y + 1, true);
                    return;
                }
                // Then right diagonal
                if x < GRID_WIDTH - 1 && self.get(x + 1, y + 1) == MaterialType::Empty {
                    self.set(x + 1, y + 1, MaterialType::Sand);
                    self.set(x, y, MaterialType::Empty);
                    self.set_updated(x + 1, y + 1, true);
                    return;
                }
            } else {
                // Try right diagonal first
                if x < GRID_WIDTH - 1 && self.get(x + 1, y + 1) == MaterialType::Empty {
                    self.set(x + 1, y + 1, MaterialType::Sand);
                    self.set(x, y, MaterialType::Empty);
                    self.set_updated(x + 1, y + 1, true);
                    return;
                }
                // Then left diagonal
                if x > 0 && self.get(x - 1, y + 1) == MaterialType::Empty {
                    self.set(x - 1, y + 1, MaterialType::Sand);
                    self.set(x, y, MaterialType::Empty);
                    self.set_updated(x - 1, y + 1, true);
                    return;
                }
            }
        }
    }

    fn update_water(&mut self, x: usize, y: usize) {
        // Mark as updated
        self.set_updated(x, y, true);

        // Try to move down
        if y < GRID_HEIGHT - 1 && self.get(x, y + 1) == MaterialType::Empty {
            self.set(x, y + 1, MaterialType::Water);
            self.set(x, y, MaterialType::Empty);
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
                    self.set_updated(x - 1, y + 1, true);
                    return;
                }
                // Then right diagonal
                if x < GRID_WIDTH - 1 && self.get(x + 1, y + 1) == MaterialType::Empty {
                    self.set(x + 1, y + 1, MaterialType::Water);
                    self.set(x, y, MaterialType::Empty);
                    self.set_updated(x + 1, y + 1, true);
                    return;
                }
            } else {
                // Try right diagonal first
                if x < GRID_WIDTH - 1 && self.get(x + 1, y + 1) == MaterialType::Empty {
                    self.set(x + 1, y + 1, MaterialType::Water);
                    self.set(x, y, MaterialType::Empty);
                    self.set_updated(x + 1, y + 1, true);
                    return;
                }
                // Then left diagonal
                if x > 0 && self.get(x - 1, y + 1) == MaterialType::Empty {
                    self.set(x - 1, y + 1, MaterialType::Water);
                    self.set(x, y, MaterialType::Empty);
                    self.set_updated(x - 1, y + 1, true);
                    return;
                }
            }
        }

        // Try to move horizontally (water spreading)
        let spread_left = rand::thread_rng().gen_bool(0.5);
        if spread_left {
            // Try left
            if x > 0 && self.get(x - 1, y) == MaterialType::Empty {
                self.set(x - 1, y, MaterialType::Water);
                self.set(x, y, MaterialType::Empty);
                self.set_updated(x - 1, y, true);
                return;
            }
            // Try right
            if x < GRID_WIDTH - 1 && self.get(x + 1, y) == MaterialType::Empty {
                self.set(x + 1, y, MaterialType::Water);
                self.set(x, y, MaterialType::Empty);
                self.set_updated(x + 1, y, true);
                return;
            }
        } else {
            // Try right
            if x < GRID_WIDTH - 1 && self.get(x + 1, y) == MaterialType::Empty {
                self.set(x + 1, y, MaterialType::Water);
                self.set(x, y, MaterialType::Empty);
                self.set_updated(x + 1, y, true);
                return;
            }
            // Try left
            if x > 0 && self.get(x - 1, y) == MaterialType::Empty {
                self.set(x - 1, y, MaterialType::Water);
                self.set(x, y, MaterialType::Empty);
                self.set_updated(x - 1, y, true);
                return;
            }
        }
    }

    fn update_fire(&mut self, x: usize, y: usize) {
        // Mark as updated
        self.set_updated(x, y, true);

        // Fire goes up and has a chance to disappear
        if rand::thread_rng().gen_bool(0.05) {
            self.set(x, y, MaterialType::Empty);
            return;
        }

        // Try to move up
        if y > 0 && self.get(x, y - 1) == MaterialType::Empty {
            self.set(x, y - 1, MaterialType::Fire);
            self.set(x, y, MaterialType::Empty);
            
            // Move temperature with the fire
            let temp = self.get_temp(x, y);
            self.set_temp(x, y - 1, temp);
            self.set_temp(x, y, AMBIENT_TEMP);
            
            self.set_updated(x, y - 1, true);
            return;
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
            return;
        }

        // Try to move down
        if y < GRID_HEIGHT - 1 && self.get(x, y + 1) == MaterialType::Empty {
            self.set(x, y + 1, MaterialType::Lava);
            self.set(x, y, MaterialType::Empty);
            
            // Move temperature with the lava
            let temp = self.get_temp(x, y);
            self.set_temp(x, y + 1, temp);
            self.set_temp(x, y, AMBIENT_TEMP);
            
            self.set_updated(x, y + 1, true);
            return;
        }

        // Try to move diagonally down
        if y < GRID_HEIGHT - 1 {
            let left_first = rand::thread_rng().gen_bool(0.5);
            
            if left_first {
                // Try left diagonal first
                if x > 0 && self.get(x - 1, y + 1) == MaterialType::Empty {
                    self.set(x - 1, y + 1, MaterialType::Lava);
                    self.set(x, y, MaterialType::Empty);
                    
                    // Move temperature
                    let temp = self.get_temp(x, y);
                    self.set_temp(x - 1, y + 1, temp);
                    self.set_temp(x, y, AMBIENT_TEMP);
                    
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
                    
                    self.set_updated(x - 1, y + 1, true);
                    return;
                }
            }
        }

        // Try to move horizontally (slower spreading than water)
        if rand::thread_rng().gen_bool(0.3) {
            let spread_left = rand::thread_rng().gen_bool(0.5);
            if spread_left {
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
                if x < CELL_SIZE || x >= WIDTH as usize - CELL_SIZE || 
                   y < CELL_SIZE || y >= HEIGHT as usize - CELL_SIZE {
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
}