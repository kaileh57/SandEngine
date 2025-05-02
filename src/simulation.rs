use std::collections::HashSet;
use std::sync::Mutex;
use crate::constants::*;
use crate::material::{MaterialType}; // Removed unused MaterialProperties

pub struct SandSimulation {
    pub grid: Vec<u8>,
    pub temp: Vec<f32>,
    pub next_grid: Vec<u8>,
    pub next_temp: Vec<f32>,
    pub active_cells: HashSet<(usize, usize)>,
    pub next_active_cells: Mutex<HashSet<(usize, usize)>>,
    pub rng: FastRand,
    // Added fields needed by UI/main
    pub current_material: MaterialType,
    pub brush_size: usize,
    pub cursor_pos: (usize, usize), 
}

impl SandSimulation {
    pub fn new() -> Self {
        let size = GRID_WIDTH * GRID_HEIGHT;
        Self {
            grid: vec![MaterialType::Empty.to_u8(); size],
            temp: vec![AMBIENT_TEMP; size],
            next_grid: vec![MaterialType::Empty.to_u8(); size],
            next_temp: vec![AMBIENT_TEMP; size],
            active_cells: HashSet::new(),
            next_active_cells: Mutex::new(HashSet::new()),
            rng: FastRand::new(12345), // Use a fixed seed 
            // Initialize new fields
            current_material: MaterialType::Sand, // Default to Sand
            brush_size: 5, // Default brush size
            cursor_pos: (0, 0), // Default cursor position
        }
    }

    pub fn get_index(x: usize, y: usize) -> usize {
        y * GRID_WIDTH + x
    }
    
    // Placeholder/Basic implementations for potentially missing methods called by update logic
    // These might need proper implementation based on your original code.
    
    fn mark_active_next(&self, x: usize, y: usize) {
        // Implementation depends on how active cells are managed
        let mut next_active = self.next_active_cells.lock().unwrap();
        next_active.insert((x, y));
        // Mark neighbors?
        for dy in -1..=1 {
            for dx in -1..=1 {
                let nx = x as isize + dx;
                let ny = y as isize + dy;
                if nx >= 0 && nx < GRID_WIDTH as isize && ny >= 0 && ny < GRID_HEIGHT as isize {
                     next_active.insert((nx as usize, ny as usize));
                }
            }
        }
    }

    // Made public for UI
    pub fn get(&self, x: usize, y: usize) -> MaterialType {
        // Assuming direct grid access is okay here, might need boundary checks
        if x < GRID_WIDTH && y < GRID_HEIGHT {
            MaterialType::from_u8(self.grid[Self::get_index(x, y)])
        } else {
            MaterialType::Stone // Boundary material
        }
    }
    
    // Added placeholder needed by UI
    pub fn get_temp(&self, x: usize, y: usize) -> f32 {
        if x < GRID_WIDTH && y < GRID_HEIGHT {
            self.temp[Self::get_index(x, y)]
        } else {
            AMBIENT_TEMP // Boundary temperature
        }
    }

    fn swap_and_mark(&mut self, x1: usize, y1: usize, x2: usize, y2: usize) {
        let idx1 = Self::get_index(x1, y1);
        let idx2 = Self::get_index(x2, y2);
        
        // Swap in the *next* grid/temp buffers
        self.next_grid.swap(idx1, idx2);
        self.next_temp.swap(idx1, idx2);

        // Mark involved cells and neighbors as active for the next frame
        self.mark_active_next(x1, y1);
        self.mark_active_next(x2, y2);
    }

    fn update_temperatures_active(&mut self) {
        // Needs implementation - likely iterates active_cells, reads self.temp, writes self.next_temp
        // For now, just copy current temps to next temps for the active cells
         let current_active_clone = self.active_cells.clone(); // Clone to avoid borrow issues
        for &(x,y) in current_active_clone.iter() {
             let idx = Self::get_index(x,y);
             self.next_temp[idx] = self.temp[idx]; // Simplified placeholder
         }
        //println!("Warning: update_temperatures_active is not fully implemented.");
    }

    fn update_water(&mut self, x: usize, y: usize, rng: &mut FastRand) {
        let below_y = y + 1;
        let props_self = MaterialType::Water.get_properties();

        // Always mark this cell as active, regardless of movement
        self.mark_active_next(x, y);
        
        // Try to move down first
        if below_y < GRID_HEIGHT {
            // Always mark below cell as active
            self.mark_active_next(x, below_y);
            
            let props_below = self.get(x, below_y).get_properties();
            if props_below.density < props_self.density {
                self.swap_and_mark(x, y, x, below_y);
                return;
            }
        } else {
            // Hit bottom boundary, already marked active
            return;
        }

        // Try to move diagonally down
        let dir = if rng.rand() & 1 == 0 { -1 } else { 1 };
        for d in [dir, -dir] {
            let diag_x = x as isize + d;
            if diag_x >= 0 && diag_x < GRID_WIDTH as isize {
                let diag_x = diag_x as usize;
                
                // Always mark diagonal cells as active
                self.mark_active_next(diag_x, below_y);
                
                let props_diag = self.get(diag_x, below_y).get_properties();
                if props_diag.density < props_self.density {
                    self.swap_and_mark(x, y, diag_x, below_y);
                    return;
                }
            }
        }

        // Try to spread sideways
        for d in [dir, -dir] {
            let side_x = x as isize + d;
            if side_x >= 0 && side_x < GRID_WIDTH as isize {
                let side_x = side_x as usize;
                
                // Always mark side cells as active
                self.mark_active_next(side_x, y);
                
                let props_side = self.get(side_x, y).get_properties();
                if props_side.density < props_self.density {
                    // Spread horizontally with higher chance for water (0.8)
                    if rng.rand_bool(0.8) {
                        self.swap_and_mark(x, y, side_x, y);
                        return;
                    }
                }
            }
        }
        
        // Even if no movement occurred, this cell is already marked active at the beginning
    }

    fn update_fire(&mut self, x: usize, y: usize, rng: &mut FastRand) {
        self.mark_active_next(x, y); // Placeholder: just mark as active
        //println!("Warning: update_fire({}, {}) called but not implemented.", x, y);
    }
    
    // For the update_lava function - adapted to use your existing methods and constants
    fn update_lava(&mut self, x: usize, y: usize, rng: &mut FastRand) {
        let current_temp = self.get_temp(x, y);
        let props_self = MaterialType::Lava.get_properties();

        // Always mark this cell as active, regardless of movement
        self.mark_active_next(x, y);
        
        // 1. Solidify if too cold (using your existing constants)
        // Note: assuming a lava solidification temperature of 950°C if constant not defined
        if current_temp < 950.0 { // Using a literal since LAVA_SOLIDIFY_TEMP isn't defined
            let stone_idx = Self::get_index(x, y);
            self.next_grid[stone_idx] = MaterialType::Stone.to_u8(); // Turn into stone in next grid
            // Keep the temperature (it will naturally cool through diffusion)
            
            // Already marked active earlier
            return;
        }

        // 2. Heat up neighbors with simpler heat transfer
        for dy in -1..=1 {
            for dx in -1..=1 {
                if dx == 0 && dy == 0 { continue; }
                let nx = x as isize + dx;
                let ny = y as isize + dy;
                if nx >= 0 && nx < GRID_WIDTH as isize && ny >= 0 && ny < GRID_HEIGHT as isize {
                    let nx = nx as usize;
                    let ny = ny as usize;
                    
                    // Always mark neighbors as active
                    self.mark_active_next(nx, ny);
                    
                    // Heat transfer and flammable material ignition
                    let neighbor_mat = self.get(nx, ny);
                    let neighbor_idx = Self::get_index(nx, ny);
                    
                    // Simple heat transfer
                    let neighbor_temp = self.temp[neighbor_idx];
                    let heat_transfer = (current_temp - neighbor_temp) * 0.15 * 0.1; // Simplify calculation
                    
                    // Apply heat to neighbor in next_temp
                    self.next_temp[neighbor_idx] = (neighbor_temp + heat_transfer).min(MAX_TEMP);
                    
                    // Check if the neighbor is flammable and hot enough to ignite
                    let neighbor_props = neighbor_mat.get_properties();
                    if neighbor_props.flammable && neighbor_temp > 200.0 && rng.rand_bool(0.05) {
                        // Set on fire in next grid
                        self.next_grid[neighbor_idx] = MaterialType::Fire.to_u8();
                        self.next_temp[neighbor_idx] = 800.0; // Fire temperature
                    }
                }
            }
        }

        // 3. Flow like a viscous liquid (similar to water but slower/denser)
        let below_y = y + 1;
        
        // Try to move down first
        if below_y < GRID_HEIGHT {
            // Always mark below cell as active
            self.mark_active_next(x, below_y);
            
            let props_below = self.get(x, below_y).get_properties();
            if props_below.density < props_self.density {
                if rng.rand_bool(1.0 / (props_self.viscosity + 1.0)) { // Slower flow based on viscosity
                    self.swap_and_mark(x, y, x, below_y);
                    return;
                }
            }
        } else {
            // Hit bottom boundary, already marked active
            return;
        }

        // Try to move diagonally down
        let dir = if rng.rand() & 1 == 0 { -1 } else { 1 };
        for d in [dir, -dir] {
            let diag_x = x as isize + d;
            if diag_x >= 0 && diag_x < GRID_WIDTH as isize {
                let diag_x = diag_x as usize;
                
                // Always mark diagonal cells as active
                self.mark_active_next(diag_x, below_y);
                
                let props_diag = self.get(diag_x, below_y).get_properties();
                if props_diag.density < props_self.density {
                    if rng.rand_bool(1.0 / (props_self.viscosity + 1.0)) {
                        self.swap_and_mark(x, y, diag_x, below_y);
                        return;
                    }
                }
            }
        }

        // Try to spread sideways (less likely than falling)
        if rng.rand_bool(0.3 / (props_self.viscosity + 1.0)) { // Reduced chance based on viscosity
            for d in [dir, -dir] {
                let side_x = x as isize + d;
                if side_x >= 0 && side_x < GRID_WIDTH as isize {
                    let side_x = side_x as usize;
                    
                    // Always mark side cells as active
                    self.mark_active_next(side_x, y);
                    
                    let props_side = self.get(side_x, y).get_properties();
                    if props_side.density < props_self.density {
                        self.swap_and_mark(x, y, side_x, y);
                        return;
                    }
                }
            }
        }
        
        // Even if no movement occurred, this cell is already marked active at the beginning
    }

    fn update_plant(&mut self, x: usize, y: usize, rng: &mut FastRand) {
        self.mark_active_next(x, y); // Placeholder: just mark as active
        //println!("Warning: update_plant({}, {}) called but not implemented.", x, y);
    }

    // Added placeholder needed by main/UI
    pub fn clear(&mut self) {
        let size = GRID_WIDTH * GRID_HEIGHT;
        self.grid.fill(MaterialType::Empty.to_u8());
        self.temp.fill(AMBIENT_TEMP);
        self.next_grid.fill(MaterialType::Empty.to_u8());
        self.next_temp.fill(AMBIENT_TEMP);
        self.active_cells.clear();
        self.next_active_cells.lock().unwrap().clear();
        // Mark all cells active initially?
        for y in 0..GRID_HEIGHT {
            for x in 0..GRID_WIDTH {
                self.active_cells.insert((x, y));
            }
        }
        println!("Simulation cleared (placeholder implementation).");
    }
    
    // Added placeholder needed by main
    pub fn draw(&self, frame: &mut [u8]) {
        // Basic implementation: Draw grid based on material color
        for y in 0..GRID_HEIGHT {
            for x in 0..GRID_WIDTH {
                let idx = Self::get_index(x, y);
                let material = MaterialType::from_u8(self.grid[idx]);
                let color = material.get_color();
                
                // Draw a CELL_SIZE x CELL_SIZE square for each grid cell
                for py in 0..CELL_SIZE {
                    for px in 0..CELL_SIZE {
                        let frame_x = x * CELL_SIZE + px;
                        let frame_y = y * CELL_SIZE + py;
                        let frame_idx = (frame_y * WIDTH as usize + frame_x) * 4;
                        if frame_idx + 3 < frame.len() {
                            frame[frame_idx..frame_idx+4].copy_from_slice(&color);
                        }
                    }
                }
            }
        }
        //println!("Warning: draw is a basic placeholder implementation.");
    }

    // Replace your existing update_sand method with this one
    fn update_sand(&mut self, x: usize, y: usize, rng: &mut FastRand) {
        let below_y = y + 1;
        
        // Always mark this cell as active for the next frame, regardless of movement
        self.mark_active_next(x, y);
        
        if below_y >= GRID_HEIGHT { return; } // Reached bottom, already marked active
        
        // Always mark the cell below as active too
        self.mark_active_next(x, below_y);

        let props_below = self.get(x, below_y).get_properties();
        let props_self = MaterialType::Sand.get_properties();

        // Fall down into less dense materials (like empty or water)
        if props_below.density < props_self.density {
            if props_below.viscosity == 0.0 { // Fall directly into non-viscous
                self.swap_and_mark(x, y, x, below_y);
                return;
            } else { // Slower fall through viscous liquids
                if rng.rand_bool(1.0 / (props_below.viscosity + 1.0)) {
                    self.swap_and_mark(x, y, x, below_y);
                    return;
                }
            }
        }

        // Slide diagonally into less dense materials
        let dir = if rng.rand() & 1 == 0 { -1 } else { 1 }; // Randomly check left or right first

        // Check first diagonal direction
        let diag1_x = x as isize + dir;
        if diag1_x >= 0 && diag1_x < GRID_WIDTH as isize {
            let diag1_x = diag1_x as usize;
            
            // Mark diagonal cell as active regardless
            self.mark_active_next(diag1_x, below_y);
            
            let props_diag1 = self.get(diag1_x, below_y).get_properties();
            if props_diag1.density < props_self.density {
                if props_diag1.viscosity == 0.0 {
                    self.swap_and_mark(x, y, diag1_x, below_y);
                    return;
                } else if rng.rand_bool(1.0 / (props_diag1.viscosity + 1.0)) {
                    self.swap_and_mark(x, y, diag1_x, below_y);
                    return;
                }
            }
        }

        // Check second diagonal direction
        let diag2_x = x as isize - dir;
        if diag2_x >= 0 && diag2_x < GRID_WIDTH as isize {
            let diag2_x = diag2_x as usize;
            
            // Mark diagonal cell as active regardless
            self.mark_active_next(diag2_x, below_y);
            
            let props_diag2 = self.get(diag2_x, below_y).get_properties();
            if props_diag2.density < props_self.density {
                if props_diag2.viscosity == 0.0 {
                    self.swap_and_mark(x, y, diag2_x, below_y);
                    return;
                } else if rng.rand_bool(1.0 / (props_diag2.viscosity + 1.0)) {
                    self.swap_and_mark(x, y, diag2_x, below_y);
                    return;
                }
            }
        }
        
        // No movement occurred, we already marked this cell as active at the beginning
    }

    // Replace your existing add_material method with this one
    pub fn add_material(&mut self, x: usize, y: usize, brush_size: usize, material: MaterialType) {
        if x >= GRID_WIDTH || y >= GRID_HEIGHT { return; } // Click outside grid

        let radius = brush_size as isize;
        let radius_squared = radius * radius;

        let start_x = (x as isize - radius).max(0) as usize;
        let end_x = (x as isize + radius).min(GRID_WIDTH as isize - 1) as usize;
        let start_y = (y as isize - radius).max(0) as usize;
        let end_y = (y as isize + radius).min(GRID_HEIGHT as isize - 1) as usize;

        // Get mutable access to the active cells set for this operation
        let mut next_active = self.next_active_cells.lock().unwrap();

        for cy in start_y..=end_y {
            for cx in start_x..=end_x {
                let dx = cx as isize - x as isize;
                let dy = cy as isize - y as isize;
                if dx * dx + dy * dy <= radius_squared {
                    let idx = Self::get_index(cx, cy);

                    // Directly modify the *current* grid and temp for immediate feedback
                    if material == MaterialType::Eraser {
                        self.grid[idx] = MaterialType::Empty.to_u8();
                        self.temp[idx] = AMBIENT_TEMP;
                    } else {
                        self.grid[idx] = material.to_u8();
                        
                        // Set appropriate initial temperatures
                        match material {
                            MaterialType::Fire => self.temp[idx] = FIRE_START_TEMP,
                            MaterialType::Lava => self.temp[idx] = LAVA_START_TEMP,
                            _ => self.temp[idx] = AMBIENT_TEMP,
                        }
                    }

                    // Mark this cell and its neighbors as active *for the next frame's update*
                    // Also add to current active cells
                    self.active_cells.insert((cx, cy));
                    next_active.insert((cx, cy));
                    
                    // Mark neighbors
                    for dy_n in -1..=1 {
                        for dx_n in -1..=1 {
                            let nx = cx as isize + dx_n;
                            let ny = cy as isize + dy_n;
                            if nx >= 0 && nx < GRID_WIDTH as isize && ny >= 0 && ny < GRID_HEIGHT as isize {
                                self.active_cells.insert((nx as usize, ny as usize));
                                next_active.insert((nx as usize, ny as usize));
                            }
                        }
                    }
                }
            }
        }
    }

    // Replace your existing update method with this one
    pub fn update(&mut self) {
        // 1. Prepare Buffers and Active Cells for the new state
        self.next_grid.copy_from_slice(&self.grid); // Start next state from current state
        self.next_temp.copy_from_slice(&self.temp);
        self.next_active_cells.lock().unwrap().clear(); // Clear the set for collecting new active cells

        // 2. Update Temperatures (Parallel, writes to next_temp)
        self.update_temperatures_active(); // This now reads from self.temp and writes to self.next_temp

        // Create a temporary list of active cells to iterate over
        let current_active: Vec<(usize, usize)> = self.active_cells.iter().cloned().collect();
        let active_count = current_active.len();
        
        // Add a check to make sure we have active cells to process
        if active_count == 0 {
            // If we have no active cells, mark cells with sand or other gravity-affected materials as active
            for y in 0..GRID_HEIGHT {
                for x in 0..GRID_WIDTH {
                    let material = self.get(x, y);
                    match material {
                        MaterialType::Sand | MaterialType::Water | MaterialType::Lava => {
                            self.mark_active_next(x, y);
                        },
                        _ => {}
                    }
                }
            }
            
            // If still no active cells, mark all cells as active to reset the system
            if self.next_active_cells.lock().unwrap().is_empty() {
                for y in 0..GRID_HEIGHT {
                    for x in 0..GRID_WIDTH {
                        self.mark_active_next(x, y);
                    }
                }
                
                // Swap buffers and return early to start fresh in the next frame
                std::mem::swap(&mut self.grid, &mut self.next_grid);
                std::mem::swap(&mut self.temp, &mut self.next_temp);
                self.active_cells = self.next_active_cells.lock().unwrap().clone();
                return;
            }
        }

        // 3. Update Material Physics (Currently Sequential)
        let mut local_rng = FastRand::new(self.rng.rand());

        // Iterate over the *cloned* list of active cells
        for &(x, y) in &current_active {
            let material = self.get(x, y);

            match material {
                MaterialType::Sand => self.update_sand(x, y, &mut local_rng),
                MaterialType::Water => self.update_water(x, y, &mut local_rng),
                MaterialType::Fire => self.update_fire(x, y, &mut local_rng),
                MaterialType::Lava => self.update_lava(x, y, &mut local_rng),
                MaterialType::Plant => self.update_plant(x, y, &mut local_rng),
                MaterialType::Stone => { self.mark_active_next(x,y); }
                MaterialType::Empty => { /* Empty cells processed by neighbors */ }
                MaterialType::Eraser => { /* Eraser is only for adding material */ }
            }
        }
        self.rng = local_rng;

        // 4. Swap Buffers and Active Sets
        std::mem::swap(&mut self.grid, &mut self.next_grid);
        std::mem::swap(&mut self.temp, &mut self.next_temp);

        // Replace the old active set with the newly collected one
        self.active_cells = self.next_active_cells.lock().unwrap().clone();
        
        // Safety check - if no active cells, mark all cells with gravity materials as active
        if self.active_cells.is_empty() {
            let mut new_active = HashSet::new();
            for y in 0..GRID_HEIGHT {
                for x in 0..GRID_WIDTH {
                    let material = self.get(x, y);
                    match material {
                        MaterialType::Sand | MaterialType::Water | MaterialType::Lava => {
                            new_active.insert((x, y));
                        },
                        _ => {}
                    }
                }
            }
            
            // If still empty (no gravity materials), mark all cells as active
            if new_active.is_empty() {
                self.active_cells = (0..GRID_WIDTH).flat_map(|x| (0..GRID_HEIGHT).map(move |y| (x, y))).collect();
            } else {
                self.active_cells = new_active;
            }
        }
    }
    
    // ... remaining code ...
}