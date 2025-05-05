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
    
    // Active cell tracking (critical for performance for non-liquids)
    active_cells: Vec<(usize, usize)>,
    next_active_cells: Vec<(usize, usize)>,
    
    // Optimization: Bounds tracking
    min_active_x: usize,
    max_active_x: usize,
    min_active_y: usize,
    max_active_y: usize,
    
    // Settings that control simulation complexity
    do_temperature_diffusion: bool,
    max_active_cells: usize,
}

impl SandSimulation {
    pub fn new() -> Self {
        let grid_size = GRID_WIDTH * GRID_HEIGHT;
        let temp = vec![AMBIENT_TEMP; grid_size];
        
        // Allocate approximately 25% of grid size for active cells
        // This is a balance between memory usage and avoiding reallocations
        let active_capacity = grid_size / 4;
        
        Self {
            grid: vec![0; grid_size],
            temp,
            vel_x: vec![0.0; grid_size],
            vel_y: vec![0.0; grid_size],
            updated: vec![false; grid_size],
            brush_size: 3,
            current_material: MaterialType::Sand,
            cursor_pos: (0, 0),
            active_cells: Vec::with_capacity(active_capacity),
            next_active_cells: Vec::with_capacity(active_capacity),
            min_active_x: GRID_WIDTH,
            max_active_x: 0,
            min_active_y: GRID_HEIGHT,
            max_active_y: 0,
            do_temperature_diffusion: false,  // Turn off by default for performance
            max_active_cells: 50000,          // Limit active cells for performance
        }
    }

    #[inline(always)]
    pub fn get_index(x: usize, y: usize) -> usize {
        y * GRID_WIDTH + x
    }

    #[inline(always)]
    pub fn get(&self, x: usize, y: usize) -> MaterialType {
        if x < GRID_WIDTH && y < GRID_HEIGHT {
            unsafe {
                MaterialType::from_u8(*self.grid.get_unchecked(Self::get_index(x, y)))
            }
        } else {
            MaterialType::Empty
        }
    }

    #[inline(always)]
    pub fn set(&mut self, x: usize, y: usize, value: MaterialType) {
        if x < GRID_WIDTH && y < GRID_HEIGHT {
            let idx = Self::get_index(x, y);
            let old_value = unsafe { *self.grid.get_unchecked(idx) };
            
            unsafe {
                *self.grid.get_unchecked_mut(idx) = value.to_u8();
            }
            
            // Only update active bounds and add to next_active_cells if the value changed
            if old_value != value.to_u8() {
                // Update active bounds
                if value != MaterialType::Empty {
                    self.min_active_x = self.min_active_x.min(x);
                    self.max_active_x = self.max_active_x.max(x);
                    self.min_active_y = self.min_active_y.min(y);
                    self.max_active_y = self.max_active_y.max(y);
                    
                    // Add to next active cells if not empty
                    if self.next_active_cells.len() < self.max_active_cells {
                        self.next_active_cells.push((x, y));
                    }
                    
                    // Also add neighbors to next active cells
                    self.add_neighbors_to_active(x, y);
                }
            }
        }
    }
    
    #[inline(always)]
    fn add_neighbors_to_active(&mut self, x: usize, y: usize) {
        if self.next_active_cells.len() >= self.max_active_cells {
            return;
        }

        // Track the material type to better understand how to propagate activity
        let center_material = self.get(x, y);
        let is_fluid = center_material == MaterialType::Water || center_material == MaterialType::Lava;
        
        // CRITICAL FIX: Extended neighbor pattern for water to prevent freezing
        // Use a much larger pattern for fluids especially water
        if is_fluid {
            // Apply a wider spread pattern for water to prevent freezing
            let water_radius = if center_material == MaterialType::Water { 8 } else { 4 };
            
            // Always include the fluid cell itself
            self.next_active_cells.push((x, y));
            
            // Expand search radius specifically for water
            for dy in -water_radius..=water_radius {
                for dx in -water_radius..=water_radius {
                    // Skip the center cell (already added)
                    if dx == 0 && dy == 0 {
                        continue;
                    }
                    
                    // Use manhattan distance to prioritize closer cells
                    let manhattan_dist = (dx as isize).abs() + (dy as isize).abs();
                    if manhattan_dist > water_radius as isize {
                        continue;
                    }
                    
                    let nx = (x as isize + dx) as usize;
                    let ny = (y as isize + dy) as usize;
                    
                    // Check bounds
                    if nx < GRID_WIDTH && ny < GRID_HEIGHT {
                        // Check if we've reached the cell limit
                        if self.next_active_cells.len() >= self.max_active_cells {
                            return;
                        }
                        
                        let neighbor_material = self.get(nx, ny);
                        
                        // Always activate water cells
                        if neighbor_material == MaterialType::Water || 
                           (neighbor_material != MaterialType::Empty && manhattan_dist <= 3) ||
                           (manhattan_dist <= 1) {  // Always activate immediate neighbors
                            self.next_active_cells.push((nx, ny));
                        }
                    }
                }
            }
        } else {
            // Original pattern for non-fluid materials
            // Center + direct neighbors + extended range
            let neighbors = [
                (x, y),                          // Center (ensure it stays active)
                (x, y.saturating_sub(1)),        // Above
                (x.saturating_sub(1), y),        // Left
                (x + 1, y.min(GRID_WIDTH-1)),    // Right 
                (x, y + 1.min(GRID_HEIGHT-1)),   // Below
                
                // Diagonals for better flow
                (x.saturating_sub(1), y.saturating_sub(1)),           // Top-left
                (x + 1.min(GRID_WIDTH-1), y.saturating_sub(1)),       // Top-right
                (x.saturating_sub(1), y + 1.min(GRID_HEIGHT-1)),      // Bottom-left
                (x + 1.min(GRID_WIDTH-1), y + 1.min(GRID_HEIGHT-1)),  // Bottom-right
                
                // Extended horizontal flow (critical for liquids)
                (x.saturating_sub(2), y),                             // Further left
                (x + 2.min(GRID_WIDTH-1), y),                         // Further right
                
                // Extended vertical for better drips and falls
                (x, y.saturating_sub(2)),                             // 2 Above 
                (x, y + 2.min(GRID_HEIGHT-1)),                        // 2 Below
                
                // Extended diagonals for better liquid spread
                (x.saturating_sub(2), y + 1.min(GRID_HEIGHT-1)),      // 2Left-1Down
                (x + 2.min(GRID_WIDTH-1), y + 1.min(GRID_HEIGHT-1)),  // 2Right-1Down
                (x.saturating_sub(2), y + 2.min(GRID_HEIGHT-1)),      // 2Left-2Down
                (x + 2.min(GRID_WIDTH-1), y + 2.min(GRID_HEIGHT-1)),  // 2Right-2Down
            ];
            
            for &(nx, ny) in neighbors.iter() {
                if nx < GRID_WIDTH && ny < GRID_HEIGHT {
                    // Check if we've reached the cell limit
                    if self.next_active_cells.len() >= self.max_active_cells {
                        break;
                    }
                    
                    // Get the material at this cell
                    let material = self.get(nx, ny);
                    
                    // More aggressive activation for fluids and their surroundings
                    let should_activate = material != MaterialType::Empty || 
                                         (is_fluid && (nx == x || ny == y || 
                                          (nx == x.saturating_sub(1) && ny == y + 1) || 
                                          (nx == x + 1 && ny == y + 1)));
                                         
                    if should_activate {
                        self.next_active_cells.push((nx, ny));
                    }
                }
            }
        }
    }

    #[inline(always)]
    pub fn get_temp(&self, x: usize, y: usize) -> f32 {
        if x < GRID_WIDTH && y < GRID_HEIGHT {
            unsafe {
                *self.temp.get_unchecked(Self::get_index(x, y))
            }
        } else {
            AMBIENT_TEMP
        }
    }

    #[inline(always)]
    pub fn set_temp(&mut self, x: usize, y: usize, value: f32) {
        if x < GRID_WIDTH && y < GRID_HEIGHT {
            let idx = Self::get_index(x, y);
            unsafe {
                *self.temp.get_unchecked_mut(idx) = value;
            }
        }
    }

    #[inline(always)]
    pub fn get_vel_x(&self, x: usize, y: usize) -> f32 {
        if x < GRID_WIDTH && y < GRID_HEIGHT {
            unsafe {
                *self.vel_x.get_unchecked(Self::get_index(x, y))
            }
        } else {
            0.0
        }
    }

    #[inline(always)]
    pub fn set_vel_x(&mut self, x: usize, y: usize, value: f32) {
        if x < GRID_WIDTH && y < GRID_HEIGHT {
            let idx = Self::get_index(x, y);
            unsafe {
                *self.vel_x.get_unchecked_mut(idx) = value;
            }
        }
    }

    #[inline(always)]
    pub fn get_vel_y(&self, x: usize, y: usize) -> f32 {
        if x < GRID_WIDTH && y < GRID_HEIGHT {
            unsafe {
                *self.vel_y.get_unchecked(Self::get_index(x, y))
            }
        } else {
            0.0
        }
    }

    #[inline(always)]
    pub fn set_vel_y(&mut self, x: usize, y: usize, value: f32) {
        if x < GRID_WIDTH && y < GRID_HEIGHT {
            let idx = Self::get_index(x, y);
            unsafe {
                *self.vel_y.get_unchecked_mut(idx) = value;
            }
        }
    }

    #[inline(always)]
    fn is_updated(&self, x: usize, y: usize) -> bool {
        if x < GRID_WIDTH && y < GRID_HEIGHT {
            unsafe {
                *self.updated.get_unchecked(Self::get_index(x, y))
            }
        } else {
            true
        }
    }

    #[inline(always)]
    fn set_updated(&mut self, x: usize, y: usize, value: bool) {
        if x < GRID_WIDTH && y < GRID_HEIGHT {
            let idx = Self::get_index(x, y);
            unsafe {
                *self.updated.get_unchecked_mut(idx) = value;
            }
        }
    }

    pub fn clear(&mut self) {
        // Use memset-like operation for better performance
        self.grid.fill(0);
        self.temp.fill(AMBIENT_TEMP);
        self.vel_x.fill(0.0);
        self.vel_y.fill(0.0);
        
        // Clear active cells tracking
        self.active_cells.clear();
        self.next_active_cells.clear();
        
        // Reset active bounds
        self.min_active_x = GRID_WIDTH;
        self.max_active_x = 0;
        self.min_active_y = GRID_HEIGHT;
        self.max_active_y = 0;
    }

    pub fn update(&mut self) {
        // Reset update flags efficiently
        self.updated.fill(false);
        
        // Deduplicate and swap active cell lists
        if self.next_active_cells.len() > 1000 {
            self.next_active_cells.sort_unstable();
            self.next_active_cells.dedup();
        }
        std::mem::swap(&mut self.active_cells, &mut self.next_active_cells);
        self.next_active_cells.clear();
        
        // Optional: Update temperature selectively (only on active cells)
        if self.do_temperature_diffusion {
            self.update_temperatures_optimized();
        }
        
        // Shuffle active cells for better visual effect and behavior (for non-liquids)
        let mut rng = rand::thread_rng();
        if self.active_cells.len() > 100 {
            let shuffle_count = (self.active_cells.len() / 10).min(1000);
            for i in 0..shuffle_count {
                let j = rng.gen_range(0..self.active_cells.len());
                self.active_cells.swap(i, j);
            }
        } else {
            for i in 0..self.active_cells.len() {
                let j = rng.gen_range(0..self.active_cells.len());
                self.active_cells.swap(i, j);
            }
        }
        
        // --- Phase 1: Process non-liquid materials from the active list --- 
        let cell_count = self.active_cells.len();
        for i in (0..cell_count).rev() {
            let (x, y) = self.active_cells[i];
            
            // Skip if out of bounds or already updated (e.g., by liquid scan)
            if x >= GRID_WIDTH || y >= GRID_HEIGHT || self.is_updated(x, y) {
                continue;
            }
            
            let idx = Self::get_index(x, y);
            
            unsafe {
                let material_u8 = *self.grid.get_unchecked(idx);
                if material_u8 == 0 { // Skip empty cells
                    continue;
                }
                
                let material = MaterialType::from_u8(material_u8);
                
                // Only process non-liquids here
                match material {
                    MaterialType::Sand => self.update_sand(x, y),
                    MaterialType::Fire => self.update_fire(x, y),
                    MaterialType::Stone => self.update_stone(x, y),
                    MaterialType::Plant => self.update_plant(x, y),
                    MaterialType::Water | MaterialType::Lava => {}, // Liquids handled in Phase 2
                    _ => {},
                }
            }
        }
        
        // --- Phase 2: Guaranteed processing for ALL liquid cells --- 
        for y in 0..GRID_HEIGHT {
            for x in 0..GRID_WIDTH {
                let idx = Self::get_index(x, y);
                let material_u8 = unsafe { *self.grid.get_unchecked(idx) };
                
                if material_u8 == MaterialType::Water as u8 {
                    // Only update if not already updated in Phase 1 (unlikely but possible)
                    if !self.is_updated(x, y) {
                        self.update_water(x, y);
                    }
                } else if material_u8 == MaterialType::Lava as u8 {
                    // Only update if not already updated in Phase 1
                    if !self.is_updated(x, y) {
                        self.update_lava(x, y);
                    }
                }
            }
        }
        
        // --- Finalization --- 
        // If the active list becomes empty (for non-liquids), run a scan
        // This can happen if only sand/etc. is present and it settles
        if self.active_cells.is_empty() && self.next_active_cells.is_empty() {
            // Check if there are actually any non-empty, non-liquid cells left
            let mut non_liquid_present = false;
            for y in 0..GRID_HEIGHT {
                for x in 0..GRID_WIDTH {
                    let mat = self.get(x, y);
                    if mat != MaterialType::Empty && mat != MaterialType::Water && mat != MaterialType::Lava {
                        non_liquid_present = true;
                        break;
                    }
                }
                if non_liquid_present {
                    break;
                }
            }
            
            // If non-liquids are present but inactive, scan for them
            if non_liquid_present {
                self.reset_active_bounds(); // Reset bounds before scanning
                self.scan_for_active_non_liquids();
            }
        }
    }

    // Simplified scan function - only looks for non-liquids that might need activation
    fn scan_for_active_non_liquids(&mut self) {
        let x_start = self.min_active_x;
        let x_end = self.max_active_x.min(GRID_WIDTH);
        let y_start = self.min_active_y;
        let y_end = self.max_active_y.min(GRID_HEIGHT);
        
        for y in y_start..y_end {
            for x in x_start..x_end {
                let idx = Self::get_index(x, y);
                unsafe {
                    let material_u8 = *self.grid.get_unchecked(idx);
                    if material_u8 != 0 {
                        let material = MaterialType::from_u8(material_u8);
                        
                        // Skip liquids - they are always processed
                        if material == MaterialType::Water || material == MaterialType::Lava {
                            continue;
                        }
                        
                        // Check conditions where non-liquids should be active
                        let should_activate = match material {
                            MaterialType::Sand | MaterialType::Plant => {
                                y >= GRID_HEIGHT - 1 || 
                                self.get(x, y + 1) == MaterialType::Empty || 
                                (x > 0 && self.get(x - 1, y + 1) == MaterialType::Empty) ||
                                (x < GRID_WIDTH - 1 && self.get(x + 1, y + 1) == MaterialType::Empty) ||
                                self.get_vel_x(x, y).abs() > 0.05 ||
                                self.get_vel_y(x, y).abs() > 0.05
                            },
                            MaterialType::Fire => true,
                            MaterialType::Stone => {
                                y >= GRID_HEIGHT - 1 || 
                                self.get(x, y + 1) == MaterialType::Empty ||
                                self.get_vel_x(x, y).abs() > 0.05 ||
                                self.get_vel_y(x, y).abs() > 0.05
                            },
                            _ => false
                        };
                        
                        if should_activate && self.next_active_cells.len() < self.max_active_cells {
                            self.next_active_cells.push((x, y));
                            
                            // Update bounds based on these potentially active non-liquids
                            self.min_active_x = self.min_active_x.min(x);
                            self.max_active_x = self.max_active_x.max(x);
                            self.min_active_y = self.min_active_y.min(y);
                            self.max_active_y = self.max_active_y.max(y);
                        }
                    }
                }
            }
        }
        
        // Add some jitter specifically to non-liquids if the list was empty
        if self.next_active_cells.is_empty() { return; }
        
        let mut rng = rand::thread_rng();
        let jitter_count = ((x_end - x_start) * (y_end - y_start) / 200).max(5).min(50);
        
        for _ in 0..jitter_count {
            let x = rng.gen_range(x_start..x_end);
            let y = rng.gen_range(y_start..y_end);
            
            let material = self.get(x, y);
            if material != MaterialType::Empty && material != MaterialType::Water && material != MaterialType::Lava {
                let vx = self.get_vel_x(x, y);
                let vy = self.get_vel_y(x, y);
                self.set_vel_x(x, y, vx + (rng.gen::<f32>() - 0.5) * 0.2);
                self.set_vel_y(x, y, vy + (rng.gen::<f32>() - 0.5) * 0.2);
                
                if self.next_active_cells.len() < self.max_active_cells {
                    self.next_active_cells.push((x, y));
                }
            }
        }
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
        
        // Add random jitter occasionally to break patterns
        let mut rng = rand::thread_rng();
        if rng.gen_bool(0.05) {
            vx += (rng.gen::<f32>() - 0.5) * 0.15;
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
        
        if free_fall {
            // If calculated position is different and free
            if (new_x != x || new_y != y) && new_y < GRID_HEIGHT && 
               self.can_move_to(new_x, new_y) {
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
            if y < GRID_HEIGHT - 1 && self.can_move_to(x, y + 1) {
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
        
        // Reduce the chance to stay put, which ensures more movement
        if is_supported && (left_support || right_support) && rng.gen_bool(0.8) {
            // Form a stable pyramid by staying in place
            // Retain a tiny bit of momentum for future movement possibility
            self.set_vel_x(x, y, vx * 0.05);
            self.set_vel_y(x, y, vy * 0.05);
            
            // Make sure to add this cell to next active cells
            if self.next_active_cells.len() < self.max_active_cells {
                self.next_active_cells.push((x, y));
            }
            
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
                rng.gen_bool(0.5)
            };
            
            if go_left {
                // Try left diagonal
                if x > 0 && self.can_move_to(x - 1, y + 1) {
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
                if x < GRID_WIDTH - 1 && self.can_move_to(x + 1, y + 1) {
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
                if x < GRID_WIDTH - 1 && self.can_move_to(x + 1, y + 1) {
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
                if x > 0 && self.can_move_to(x - 1, y + 1) {
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
        // Keep more momentum to avoid permanent stalling
        self.set_vel_x(x, y, vx * 0.2); // Increased from 0.1
        self.set_vel_y(x, y, vy * 0.2); // Increased from 0.1
        
        // Always add to next active cells to prevent freezing
        if self.next_active_cells.len() < self.max_active_cells {
            self.next_active_cells.push((x, y));
            
            // Check for potentially unstable neighbors and activate them too
            if !is_supported || !left_support || !right_support {
                self.add_neighbors_to_active(x, y);
            }
        }
    }

    fn update_water(&mut self, x: usize, y: usize) {
        // Mark as updated (prevent double processing)
        self.set_updated(x, y, true);
        
        // Add self and neighbors to next active list *for non-liquid interactions*
        if self.next_active_cells.len() < self.max_active_cells {
            self.next_active_cells.push((x, y));
            // Add immediate neighbors
            if x > 0 { self.next_active_cells.push((x-1, y)); }
            if x < GRID_WIDTH - 1 { self.next_active_cells.push((x+1, y)); }
            if y > 0 { self.next_active_cells.push((x, y-1)); }
            if y < GRID_HEIGHT - 1 { self.next_active_cells.push((x, y+1)); }
        }
        
        // --- Original Water Physics Logic Start ---
        let mut rng = rand::thread_rng();
        
        // Apply gravity (acceleration) - water accelerates faster than sand
        let mut vx = self.get_vel_x(x, y);
        let mut vy = self.get_vel_y(x, y) + WATER_GRAVITY;
        
        // Apply terminal velocity limit
        if vy > WATER_MAX_VELOCITY {
            vy = WATER_MAX_VELOCITY;
        }
        
        // Water keeps more horizontal momentum for better flow
        vx *= 0.99;
        
        // Add small random movement for natural flow
        if rng.gen_bool(0.15) {
            vx += (rng.gen::<f32>() - 0.5) * 0.25;
        }
        
        // Calculate potential new position based on velocity
        let new_x = (x as f32 + vx).round() as usize;
        let new_y = (y as f32 + vy).round() as usize;
        
        // Boundary checks
        let new_x = new_x.min(GRID_WIDTH - 1);
        let new_y = new_y.min(GRID_HEIGHT - 1);
        
        // ENHANCED WATER EQUALIZATION: Check if we're on top of another water cell or solid and should spread
        let on_water = y < GRID_HEIGHT - 1 && self.get(x, y + 1) == MaterialType::Water;
        let below_is_solid = y < GRID_HEIGHT - 1 && 
                             self.get(x, y + 1) != MaterialType::Empty && 
                             self.get(x, y + 1) != MaterialType::Water;
        
        // Try moving to the calculated position based on velocity first
        if (new_x != x || new_y != y) && self.can_move_to(new_x, new_y) {
            self.set(new_x, new_y, MaterialType::Water);
            self.set(x, y, MaterialType::Empty);
            // Transfer velocity
            self.set_vel_x(new_x, new_y, vx);
            self.set_vel_y(new_x, new_y, vy);
            self.set_updated(new_x, new_y, true);
            return; // Moved, exit update for this cell
        }
        
        // Try to move down if we can't move based on velocity
        if y < GRID_HEIGHT - 1 && self.can_move_to(x, y + 1) {
            self.set(x, y + 1, MaterialType::Water);
            self.set(x, y, MaterialType::Empty);
            // Transfer velocity
            self.set_vel_x(x, y + 1, vx);
            self.set_vel_y(x, y + 1, vy);
            self.set_updated(x, y + 1, true);
            return; // Moved, exit update for this cell
        }
        
        // **FORCE SPREADING WHEN ON SURFACE**: Check and spread in both directions when possible
        if (on_water || below_is_solid) && rng.gen_bool(0.7) { // Keep probability for performance
            // Check distances in both directions
            let left_empty_dist = if x > 0 {
                let mut dist = 0;
                for i in 1..=x.min(8) { 
                    if self.get(x - i, y) != MaterialType::Empty ||
                       (y < GRID_HEIGHT - 1 && self.get(x - i, y + 1) == MaterialType::Empty) {
                        break;
                    }
                    dist = i;
                }
                dist
            } else {
                0
            };
            
            let right_empty_dist = if x < GRID_WIDTH - 1 {
                let mut dist = 0;
                for i in 1..=(GRID_WIDTH - 1 - x).min(8) { 
                    if self.get(x + i, y) != MaterialType::Empty ||
                       (y < GRID_HEIGHT - 1 && self.get(x + i, y + 1) == MaterialType::Empty) {
                        break;
                    }
                    dist = i;
                }
                dist
            } else {
                0
            };
            
            // **NEW SPLITTING LOGIC**: Force spread if space exists on both sides
            if left_empty_dist > 0 && right_empty_dist > 0 {
                // Calculate potential flow positions (simplified to 1 cell for now)
                let left_x = x - 1; 
                let right_x = x + 1;
                
                let can_flow_left = self.can_move_to(left_x, y);
                let can_flow_right = self.can_move_to(right_x, y);
                
                if can_flow_left && can_flow_right && self.next_active_cells.len() < self.max_active_cells - 1 {
                    // **DUPLICATE/SPLIT**: Move original left, create new one right
                    self.set(left_x, y, MaterialType::Water); // Move original
                    self.set_vel_x(left_x, y, -0.5 + vx * 0.3); // Bias velocity slightly
                    self.set_vel_y(left_x, y, 0.1);
                    self.set_updated(left_x, y, true);
                    
                    self.set(right_x, y, MaterialType::Water); // Create new one
                    self.set_vel_x(right_x, y, 0.5 + vx * 0.3); // Bias velocity slightly
                    self.set_vel_y(right_x, y, 0.1);
                    self.set_updated(right_x, y, true);
                    
                    // Clear the original position
                    self.set(x, y, MaterialType::Empty);
                    self.set_vel_x(x, y, 0.0);
                    self.set_vel_y(x, y, 0.0);

                    return; // Moved (split), exit update
                }
                // Fallback: If can't split (e.g., active cell limit), choose one direction randomly
                else if can_flow_left && can_flow_right { 
                     if rng.gen_bool(0.5) { 
                         if self.can_move_to(left_x, y) {
                             self.set(left_x, y, MaterialType::Water);
                             self.set(x, y, MaterialType::Empty);
                             self.set_vel_x(left_x, y, -0.8);
                             self.set_vel_y(left_x, y, 0.1);
                             self.set_updated(left_x, y, true);
                             return;
                         }
                     } else {
                         if self.can_move_to(right_x, y) {
                             self.set(right_x, y, MaterialType::Water);
                             self.set(x, y, MaterialType::Empty);
                             self.set_vel_x(right_x, y, 0.8);
                             self.set_vel_y(right_x, y, 0.1);
                             self.set_updated(right_x, y, true);
                             return;
                         }
                     }
                }
                // If only one direction is available, go that way (keep this logic)
                else if can_flow_left {
                    self.set(left_x, y, MaterialType::Water);
                    self.set(x, y, MaterialType::Empty);
                    self.set_vel_x(left_x, y, -0.8);
                    self.set_vel_y(left_x, y, 0.1);
                    self.set_updated(left_x, y, true);
                    return;
                }
                else if can_flow_right {
                    self.set(right_x, y, MaterialType::Water);
                    self.set(x, y, MaterialType::Empty);
                    self.set_vel_x(right_x, y, 0.8);
                    self.set_vel_y(right_x, y, 0.1);
                    self.set_updated(right_x, y, true);
                    return;
                }
            }
            // If only one direction has space, flow that way (keep this logic)
            else if left_empty_dist > 0 {
                let flow_dist = (left_empty_dist as f32 * rng.gen_range(0.3..0.6)) as usize;
                let nx = x - flow_dist.max(1);
                if self.can_move_to(nx, y) {
                    self.set(nx, y, MaterialType::Water);
                    self.set(x, y, MaterialType::Empty);
                    self.set_vel_x(nx, y, -0.8 - (flow_dist as f32 * 0.1));
                    self.set_vel_y(nx, y, 0.1);
                    self.set_updated(nx, y, true);
                    return;
                }
            }
            else if right_empty_dist > 0 {
                let flow_dist = (right_empty_dist as f32 * rng.gen_range(0.3..0.6)) as usize;
                let nx = x + flow_dist.max(1);
                if self.can_move_to(nx, y) {
                    self.set(nx, y, MaterialType::Water);
                    self.set(x, y, MaterialType::Empty);
                    self.set_vel_x(nx, y, 0.8 + (flow_dist as f32 * 0.1));
                    self.set_vel_y(nx, y, 0.1);
                    self.set_updated(nx, y, true);
                    return;
                }
            }
        }
        
        // Try to move diagonally down (Keep balanced logic from previous step)
        if y < GRID_HEIGHT - 1 {
            // BALANCED DIAGONAL CHECK: Simultaneously check both diagonals
            let can_go_left_diagonal = x > 0 && self.can_move_to(x - 1, y + 1);
            let can_go_right_diagonal = x < GRID_WIDTH - 1 && self.can_move_to(x + 1, y + 1);
            
            if can_go_left_diagonal && can_go_right_diagonal {
                const VX_THRESHOLD: f32 = 0.05;
                let go_left = if vx < -VX_THRESHOLD {
                    true
                } else if vx > VX_THRESHOLD {
                    false
                } else {
                    rng.gen_bool(0.5)
                };
                
                if go_left {
                    self.set(x - 1, y + 1, MaterialType::Water);
                    self.set(x, y, MaterialType::Empty);
                    self.set_vel_x(x - 1, y + 1, vx - 0.6);
                    self.set_vel_y(x - 1, y + 1, vy * 0.9);
                    self.set_updated(x - 1, y + 1, true);
                } else {
                    self.set(x + 1, y + 1, MaterialType::Water);
                    self.set(x, y, MaterialType::Empty);
                    self.set_vel_x(x + 1, y + 1, vx + 0.6);
                    self.set_vel_y(x + 1, y + 1, vy * 0.9);
                    self.set_updated(x + 1, y + 1, true);
                }
                return;
            }
            else if can_go_left_diagonal {
                self.set(x - 1, y + 1, MaterialType::Water);
                self.set(x, y, MaterialType::Empty);
                self.set_vel_x(x - 1, y + 1, vx - 0.6);
                self.set_vel_y(x - 1, y + 1, vy * 0.9);
                self.set_updated(x - 1, y + 1, true);
                return;
            }
            else if can_go_right_diagonal {
                self.set(x + 1, y + 1, MaterialType::Water);
                self.set(x, y, MaterialType::Empty);
                self.set_vel_x(x + 1, y + 1, vx + 0.6);
                self.set_vel_y(x + 1, y + 1, vy * 0.9);
                self.set_updated(x + 1, y + 1, true);
                return;
            }
        }
        
        // BALANCED HORIZONTAL SPREAD (Basic): Try both sides simultaneously if not on a surface
        if !(on_water || below_is_solid) { // Only apply basic horizontal if not doing surface spread
            let can_go_left = x > 0 && self.can_move_to(x - 1, y);
            let can_go_right = x < GRID_WIDTH - 1 && self.can_move_to(x + 1, y);
            
            if can_go_left && can_go_right {
                const VX_THRESHOLD: f32 = 0.05;
                let go_left = if vx < -VX_THRESHOLD {
                    true
                } else if vx > VX_THRESHOLD {
                    false
                } else {
                    rng.gen_bool(0.5) 
                };
                
                if go_left {
                    self.set(x - 1, y, MaterialType::Water);
                    self.set(x, y, MaterialType::Empty);
                    self.set_vel_x(x - 1, y, vx - 0.4);
                    self.set_vel_y(x - 1, y, 0.1);
                    self.set_updated(x - 1, y, true);
                } else {
                    self.set(x + 1, y, MaterialType::Water);
                    self.set(x, y, MaterialType::Empty);
                    self.set_vel_x(x + 1, y, vx + 0.4);
                    self.set_vel_y(x + 1, y, 0.1);
                    self.set_updated(x + 1, y, true);
                }
                return;
            }
            else if can_go_left {
                self.set(x - 1, y, MaterialType::Water);
                self.set(x, y, MaterialType::Empty);
                self.set_vel_x(x - 1, y, vx - 0.4);
                self.set_vel_y(x - 1, y, 0.1);
                self.set_updated(x - 1, y, true);
                return;
            }
            else if can_go_right {
                self.set(x + 1, y, MaterialType::Water);
                self.set(x, y, MaterialType::Empty);
                self.set_vel_x(x + 1, y, vx + 0.4);
                self.set_vel_y(x + 1, y, 0.1);
                self.set_updated(x + 1, y, true);
                return;
            }
        }
        
        // If we're on water, add extra jitter to prevent stalling
        if on_water {
            vx += (rng.gen::<f32>() - 0.5) * 1.2;
            vy *= 0.2;
        }
        
        // If we couldn't move, preserve more momentum for future updates
        self.set_vel_x(x, y, vx * 0.95); 
        self.set_vel_y(x, y, vy * 0.7);  
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
               self.can_move_to(new_x, new_y) {
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
            if y < GRID_HEIGHT - 1 && self.can_move_to(x, y + 1) {
                self.set(x, y + 1, MaterialType::Stone);
                self.set(x, y, MaterialType::Empty);
                
                // Transfer velocity with some dampening (stone is heavy)
                self.set_vel_x(x, y + 1, vx * 0.9);
                self.set_vel_y(x, y + 1, vy);
                
                self.set_updated(x, y + 1, true);
                return;
            }
        }
        
        // Rest of stone movement logic remains the same
        // ...
        
        // If we couldn't move, retain very little momentum
        self.set_vel_x(x, y, vx * 0.05);
        self.set_vel_y(x, y, vy * 0.05);
        
        // Only add to next active cells if we have momentum
        if (vx.abs() > 0.05 || vy.abs() > 0.05) && self.next_active_cells.len() < self.max_active_cells {
            self.next_active_cells.push((x, y));
        }
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
               self.can_move_to(new_x, new_y) {
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
            if y < GRID_HEIGHT - 1 && self.can_move_to(x, y + 1) {
                self.set(x, y + 1, MaterialType::Plant);
                self.set(x, y, MaterialType::Empty);
                
                // Transfer velocity for continued acceleration
                self.set_vel_x(x, y + 1, vx * 0.9);
                self.set_vel_y(x, y + 1, vy);
                
                self.set_updated(x, y + 1, true);
                return;
            }
        }
        
        // Rest of plant movement logic remains the same
        // ...
        
        // If we couldn't move, lose momentum gradually (plants retain more than stone)
        self.set_vel_x(x, y, vx * 0.2);
        self.set_vel_y(x, y, vy * 0.1);
        
        // Only add to next active cells if we have momentum
        if (vx.abs() > 0.1 || vy.abs() > 0.1) && self.next_active_cells.len() < self.max_active_cells {
            self.next_active_cells.push((x, y));
        }
    }

    fn update_fire(&mut self, x: usize, y: usize) {
        // Mark as updated
        self.set_updated(x, y, true);
        
        // Fire rises and spreads
        let mut vx = self.get_vel_x(x, y);
        let vy = self.get_vel_y(x, y) - FIRE_UPDRAFT; // Fire rises upward
        
        // Apply some randomness for flicker
        let mut rng = rand::thread_rng();
        vx += (rng.gen::<f32>() - 0.5) * 0.3;
        
        // Fire has a chance to extinguish
        if rng.gen_bool(0.1) {
            self.set(x, y, MaterialType::Empty);
            return;
        }
        
        // Calculate new position
        let new_x = (x as f32 + vx).round() as usize;
        let new_y = (y as f32 + vy).round() as usize;
        
        // Boundary checks
        let new_x = new_x.min(GRID_WIDTH - 1);
        let new_y = if new_y < GRID_HEIGHT { new_y } else { y };
        
        // Try to move to the new position if it's empty
        if new_x != x || new_y != y {
            if self.can_move_to(new_x, new_y) {
                self.set(new_x, new_y, MaterialType::Fire);
                self.set(x, y, MaterialType::Empty);
                
                // Transfer velocity and temperature
                self.set_vel_x(new_x, new_y, vx);
                self.set_vel_y(new_x, new_y, vy);
                
                let temp = self.get_temp(x, y);
                self.set_temp(new_x, new_y, temp);
                self.set_temp(x, y, AMBIENT_TEMP);
                
                self.set_updated(new_x, new_y, true);
                return;
            }
        }
        
        // Always add fire to next active cells
        if self.next_active_cells.len() < self.max_active_cells {
            self.next_active_cells.push((x, y));
        }
    }

    fn update_lava(&mut self, x: usize, y: usize) {
        // Mark as updated (prevent double processing)
        self.set_updated(x, y, true);
        
        // Add self and neighbors to next active list *for non-liquid interactions*
        if self.next_active_cells.len() < self.max_active_cells {
            self.next_active_cells.push((x, y));
            // Add immediate neighbors
            if x > 0 { self.next_active_cells.push((x-1, y)); }
            if x < GRID_WIDTH - 1 { self.next_active_cells.push((x+1, y)); }
            if y > 0 { self.next_active_cells.push((x, y-1)); }
            if y < GRID_HEIGHT - 1 { self.next_active_cells.push((x, y+1)); }
        }

        // --- Original Lava Physics Logic Start ---
        let mut rng = rand::thread_rng();
        
        // Check if lava is cooling to stone
        let current_temp = self.get_temp(x, y);
        if current_temp < 1000.0 && rng.gen_bool(0.05) {
            self.set(x, y, MaterialType::Stone);
            self.set_vel_x(x, y, 0.0);
            self.set_vel_y(x, y, 0.0);
            self.set_updated(x, y, true); // Mark stone as updated here
            return; // Became stone, exit update
        }

        // Apply gravity (acceleration) - lava moves slower than water
        let mut vx = self.get_vel_x(x, y);
        let mut vy = self.get_vel_y(x, y) + LAVA_GRAVITY;
        
        // Apply terminal velocity limit - lava is more viscous
        if vy > LAVA_MAX_VELOCITY {
            vy = LAVA_MAX_VELOCITY;
        }
        
        // Lava has higher friction, so horizontal momentum decays slower to promote flow
        vx *= 0.98;
        
        // Add small random movement for natural flow
        if rng.gen_bool(0.05) {
            vx += (rng.gen::<f32>() - 0.5) * 0.1;
        }
        
        // Calculate potential new position based on velocity
        let new_x = (x as f32 + vx).round() as usize;
        let new_y = (y as f32 + vy).round() as usize;
        
        // Boundary checks
        let new_x = new_x.min(GRID_WIDTH - 1);
        let new_y = new_y.min(GRID_HEIGHT - 1);

        // Check for surface condition (similar to water)
        let on_lava = y < GRID_HEIGHT - 1 && self.get(x, y + 1) == MaterialType::Lava;
        let below_is_solid = y < GRID_HEIGHT - 1 && 
                             self.get(x, y + 1) != MaterialType::Empty && 
                             self.get(x, y + 1) != MaterialType::Lava;

        // Try moving to the calculated position based on velocity
        if (new_x != x || new_y != y) && self.can_move_to(new_x, new_y) {
            self.move_lava(x, y, new_x, new_y, vx, vy);
            return; // Moved, exit update for this cell
        }
        
        // Try direct down
        if y < GRID_HEIGHT - 1 && self.can_move_to(x, y + 1) {
            self.move_lava(x, y, x, y + 1, vx * 0.95, vy);
            return; // Moved, exit update for this cell
        }
        
        // **LAVA SPREADING**: Similar to water but less aggressive
        if (on_lava || below_is_solid) && rng.gen_bool(0.5) { // Less chance than water
            let can_go_left = x > 0 && self.can_move_to(x - 1, y);
            let can_go_right = x < GRID_WIDTH - 1 && self.can_move_to(x + 1, y);
            
            // **SPLIT LOGIC for LAVA** (simplified)
            if can_go_left && can_go_right && self.next_active_cells.len() < self.max_active_cells - 1 {
                 // Less likely to split than water
                if rng.gen_bool(0.1) { 
                    self.move_lava(x, y, x - 1, y, -0.3 + vx * 0.2, 0.05); // Move original left
                    self.set(x + 1, y, MaterialType::Lava); // Create new one right
                    self.set_temp(x + 1, y, self.get_temp(x, y)); // Approx temp
                    self.set_vel_x(x + 1, y, 0.3 + vx * 0.2);
                    self.set_vel_y(x + 1, y, 0.05);
                    self.set_updated(x + 1, y, true); 
                    // Original (x,y) is cleared inside move_lava for the first move
                    return;
                } else { 
                    // If not splitting, choose one direction based on velocity/random
                    const VX_THRESHOLD: f32 = 0.05;
                    let go_left = if vx < -VX_THRESHOLD { true } 
                                  else if vx > VX_THRESHOLD { false } 
                                  else { rng.gen_bool(0.5) };
                    
                    if go_left {
                         self.move_lava(x, y, x - 1, y, vx * 0.7 - 0.1, vy * 0.5);
                    } else {
                         self.move_lava(x, y, x + 1, y, vx * 0.7 + 0.1, vy * 0.5);
                    }
                    return;
                }
            }
            // If only one direction is available, go that way
            else if can_go_left {
                self.move_lava(x, y, x - 1, y, vx * 0.7 - 0.1, vy * 0.5);
                return;
            }
            else if can_go_right {
                self.move_lava(x, y, x + 1, y, vx * 0.7 + 0.1, vy * 0.5);
                return;
            }
        }
        
        // Try diagonal movement if horizontal/surface spreading failed
        // (Keep balanced logic)
        if y < GRID_HEIGHT - 1 { 
            let can_go_left_diagonal = x > 0 && self.can_move_to(x - 1, y + 1);
            let can_go_right_diagonal = x < GRID_WIDTH - 1 && self.can_move_to(x + 1, y + 1);
            
            if can_go_left_diagonal && can_go_right_diagonal {
                const VX_THRESHOLD: f32 = 0.05;
                let go_left = if vx < -VX_THRESHOLD { true } 
                              else if vx > VX_THRESHOLD { false } 
                              else { rng.gen_bool(0.5) };
                
                if go_left {
                    self.move_lava(x, y, x - 1, y + 1, vx * 0.7 - 0.1, vy * 0.9);
                } else {
                    self.move_lava(x, y, x + 1, y + 1, vx * 0.7 + 0.1, vy * 0.9);
                }
                return;
            }
            else if can_go_left_diagonal {
                self.move_lava(x, y, x - 1, y + 1, vx * 0.7 - 0.1, vy * 0.9);
                return;
            }
            else if can_go_right_diagonal {
                self.move_lava(x, y, x + 1, y + 1, vx * 0.7 + 0.1, vy * 0.9);
                return;
            }
        }
        
        // If still here, preserve momentum and stay active
        self.set_vel_x(x, y, vx * 0.9);
        self.set_vel_y(x, y, vy * 0.5);
    }

    // Helper function to move lava and handle temperature/velocity transfer
    fn move_lava(&mut self, x_old: usize, y_old: usize, x_new: usize, y_new: usize, vx_new: f32, vy_new: f32) {
        self.set(x_new, y_new, MaterialType::Lava);
        
        // Move temperature
        let temp = self.get_temp(x_old, y_old);
        self.set_temp(x_new, y_new, temp);
        self.set_temp(x_old, y_old, AMBIENT_TEMP);
        
        // Transfer velocity
        self.set_vel_x(x_new, y_new, vx_new);
        self.set_vel_y(x_new, y_new, vy_new);
        
        // Clear old position
        self.set(x_old, y_old, MaterialType::Empty);
        self.set_vel_x(x_old, y_old, 0.0);
        self.set_vel_y(x_old, y_old, 0.0);
        
        self.set_updated(x_new, y_new, true);
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
        // Optimized drawing - draw only the active area with a small margin
        // This significantly improves performance for sparse simulations
        let x_start = self.min_active_x.saturating_sub(5);
        let x_end = (self.max_active_x + 5).min(GRID_WIDTH);
        let y_start = self.min_active_y.saturating_sub(5);
        let y_end = (self.max_active_y + 5).min(GRID_HEIGHT);
        
        // If no active area, only draw the border
        if x_start > x_end || y_start > y_end {
            // Draw a border around the simulation area
            self.draw_border(frame);
            return;
        }
        
        // Draw the active area
        for y in y_start..y_end {
            for x in x_start..x_end {
                let material = self.get(x, y);
                let props = material.get_properties();
                let base_color = props.color;
                
                // Skip drawing empty cells for performance
                if material == MaterialType::Empty {
                    continue;
                }
                
                // Apply temperature color modification
                let temp_mod = self.get_temp_color_modifier(x, y);
                
                let r = (base_color[0] as i16 + temp_mod[0]).max(0).min(255) as u8;
                let g = (base_color[1] as i16 + temp_mod[1]).max(0).min(255) as u8;
                let b = (base_color[2] as i16 + temp_mod[2]).max(0).min(255) as u8;
                let a = base_color[3];
                
                // Draw the cell (scaled by CELL_SIZE) using an optimized loop
                if CELL_SIZE == 1 {
                    // Special case for CELL_SIZE = 1 (much faster)
                    let idx = ((y * WINDOW_WIDTH as usize) + x) * 4;
                    if idx + 3 < frame.len() {
                        frame[idx] = r;
                        frame[idx + 1] = g;
                        frame[idx + 2] = b;
                        frame[idx + 3] = a;
                    }
                } else {
                    // General case for any CELL_SIZE
                    let start_px = x * CELL_SIZE;
                    let start_py = y * CELL_SIZE;
                    let end_px = start_px + CELL_SIZE;
                    let end_py = start_py + CELL_SIZE;
                    
                    for py in start_py..end_py {
                        let row_offset = py * WINDOW_WIDTH as usize * 4;
                        for px in start_px..end_px {
                            let idx = row_offset + px * 4;
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
        }
        
        // Draw a border around the simulation area
        self.draw_border(frame);
    }
    
    fn draw_border(&self, frame: &mut [u8]) {
        // Draw border (1 pixel width)
        for y in 0..HEIGHT as usize {
            for x in 0..WIDTH as usize {
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

    // New method for bulk material creation to avoid constant active cell recomputation
    pub fn add_materials_bulk(&mut self, positions: &[(usize, usize)], material: MaterialType) {
        if positions.is_empty() {
            return;
        }
        
        // Calculate initial temp and velocity based on material type
        let (init_temp, init_vx_factor, init_vy_factor) = match material {
            MaterialType::Fire => (800.0, 0.4, -0.5),
            MaterialType::Lava => (1800.0, 0.5, 0.2),
            MaterialType::Water => (AMBIENT_TEMP, 1.5, 0.3),
            MaterialType::Sand => (AMBIENT_TEMP, 0.8, 0.5),
            MaterialType::Stone => (AMBIENT_TEMP, 0.4, 0.6),
            MaterialType::Plant => (AMBIENT_TEMP, 0.6, 0.4),
            _ => (AMBIENT_TEMP, 0.0, 0.0),
        };
        
        let mut rng = rand::thread_rng();
        
        // First pass: set all materials without bounds checks
        for &(x, y) in positions {
            if x < GRID_WIDTH && y < GRID_HEIGHT {
                if material == MaterialType::Eraser {
                    let idx = Self::get_index(x, y);
                    unsafe {
                        *self.grid.get_unchecked_mut(idx) = 0;
                        *self.temp.get_unchecked_mut(idx) = AMBIENT_TEMP;
                        *self.vel_x.get_unchecked_mut(idx) = 0.0;
                        *self.vel_y.get_unchecked_mut(idx) = 0.0;
                    }
                } else {
                    let idx = Self::get_index(x, y);
                    
                    // Improved random velocity distribution for liquids to avoid left/right bias
                    let random_vx = if material == MaterialType::Water || material == MaterialType::Lava {
                        // Generate a more balanced velocity for liquids - sometimes zero, sometimes small random value
                        if rng.gen_bool(0.5) {
                            0.0 // 50% chance of no initial horizontal motion
                        } else {
                            (rng.gen::<f32>() - 0.5) * init_vx_factor * 0.8 // 50% chance of small random motion
                        }
                    } else {
                        (rng.gen::<f32>() - 0.5) * init_vx_factor  // Normal behavior for non-liquids
                    };
                    
                    let random_vy = rng.gen::<f32>() * init_vy_factor;
                    
                    unsafe {
                        *self.grid.get_unchecked_mut(idx) = material.to_u8();
                        *self.temp.get_unchecked_mut(idx) = init_temp;
                        *self.vel_x.get_unchecked_mut(idx) = random_vx;
                        *self.vel_y.get_unchecked_mut(idx) = random_vy + init_vy_factor;
                    }
                }
            }
        }
        
        // Second pass: update bounds and add to active cells
        // This is more efficient than updating for each cell
        for &(x, y) in positions {
            if x < GRID_WIDTH && y < GRID_HEIGHT && material != MaterialType::Eraser {
                // Update active bounds
                self.min_active_x = self.min_active_x.min(x);
                self.max_active_x = self.max_active_x.max(x);
                self.min_active_y = self.min_active_y.min(y);
                self.max_active_y = self.max_active_y.max(y);
                
                // Add to next active cells
                if self.next_active_cells.len() < self.max_active_cells {
                    self.next_active_cells.push((x, y));
                }
            }
        }
        
        // Third pass: add neighbors
        // Copy to avoid borrow issues
        let cells_to_add = self.next_active_cells.len();
        let mut neighbor_positions = Vec::with_capacity(cells_to_add * 4);
        
        for i in 0..cells_to_add {
            let (x, y) = self.next_active_cells[i];
            
            // Only add essential neighbors
            if x > 0 { neighbor_positions.push((x-1, y)); }
            if x < GRID_WIDTH - 1 { neighbor_positions.push((x+1, y)); }
            if y > 0 { neighbor_positions.push((x, y-1)); }
            if y < GRID_HEIGHT - 1 { neighbor_positions.push((x, y+1)); }
        }
        
        // Add neighbors to active cells (deduplicate later when needed)
        for &(nx, ny) in &neighbor_positions {
            if self.next_active_cells.len() < self.max_active_cells {
                self.next_active_cells.push((nx, ny));
            } else {
                break;
            }
        }
    }
    
    pub fn add_material(&mut self, x: usize, y: usize, brush_size: usize, material: MaterialType) {
        let start_x = x.saturating_sub(brush_size);
        let end_x = (x + brush_size).min(GRID_WIDTH - 1);
        let start_y = y.saturating_sub(brush_size);
        let end_y = (y + brush_size).min(GRID_HEIGHT - 1);
        
        let brush_size_squared = (brush_size * brush_size) as isize;
        
        // Pre-calculate all positions for bulk update
        let mut positions = Vec::new();
        
        for cy in start_y..=end_y {
            for cx in start_x..=end_x {
                let dx = cx as isize - x as isize;
                let dy = cy as isize - y as isize;
                if dx * dx + dy * dy <= brush_size_squared {
                    positions.push((cx, cy));
                }
            }
        }
        
        // Apply bulk update
        self.add_materials_bulk(&positions, material);
    }

    fn reset_active_bounds(&mut self) {
        // Reset bounds to encompass the entire grid
        self.min_active_x = 0;
        self.max_active_x = GRID_WIDTH - 1;
        self.min_active_y = 0;
        self.max_active_y = GRID_HEIGHT - 1;
    }

    #[inline(always)]
    fn can_move_to(&self, x: usize, y: usize) -> bool {
        if x < GRID_WIDTH && y < GRID_HEIGHT {
            let idx = Self::get_index(x, y);
            unsafe {
                *self.grid.get_unchecked(idx) == 0 // Check if cell is empty
            }
        } else {
            false // Out of bounds
        }
    }

    fn update_temperatures_optimized(&mut self) {
        // Only update temperatures in active area for performance
        let x_start = self.min_active_x.saturating_sub(5);
        let x_end = (self.max_active_x + 5).min(GRID_WIDTH);
        let y_start = self.min_active_y.saturating_sub(5);
        let y_end = (self.max_active_y + 5).min(GRID_HEIGHT);
        
        // Temporary buffer for calculating new temperatures
        let mut new_temps = vec![0.0; (x_end - x_start) * (y_end - y_start)];
        
        // Compute diffusion only in active area
        for y in y_start..y_end {
            for x in x_start..x_end {
                let idx = Self::get_index(x, y);
                let local_idx = (y - y_start) * (x_end - x_start) + (x - x_start);
                
                let current_temp = unsafe { *self.temp.get_unchecked(idx) };
                
                // Apply natural cooling for all cells
                let mut new_temp = current_temp * (1.0 - COOLING_RATE);
                
                // Get material type
                let material_u8 = unsafe { *self.grid.get_unchecked(idx) };
                
                // Skip diffusion for empty cells and solid materials
                if material_u8 == 0 || MaterialType::from_u8(material_u8) == MaterialType::Stone {
                    new_temps[local_idx] = new_temp;
                    continue;
                }
                
                // Simple diffusion algorithm - average with neighbors
                let mut sum_temp = current_temp;
                let mut count = 1.0;
                
                // Check each neighbor
                let neighbors = [
                    (x.saturating_sub(1), y), // Left
                    (x + 1, y),               // Right
                    (x, y.saturating_sub(1)), // Up
                    (x, y + 1),               // Down
                ];
                
                for &(nx, ny) in &neighbors {
                    if nx < GRID_WIDTH && ny < GRID_HEIGHT {
                        let n_idx = Self::get_index(nx, ny);
                        let n_temp = unsafe { *self.temp.get_unchecked(n_idx) };
                        sum_temp += n_temp;
                        count += 1.0;
                    }
                }
                
                // Average temperatures
                new_temp = sum_temp / count;
                
                // Special case: Fire and Lava maintain high temperatures
                if MaterialType::from_u8(material_u8) == MaterialType::Fire {
                    new_temp = new_temp.max(600.0);
                } else if MaterialType::from_u8(material_u8) == MaterialType::Lava {
                    new_temp = new_temp.max(1200.0);
                }
                
                new_temps[local_idx] = new_temp;
            }
        }
        
        // Apply new temperatures
        for y in y_start..y_end {
            for x in x_start..x_end {
                let idx = Self::get_index(x, y);
                let local_idx = (y - y_start) * (x_end - x_start) + (x - x_start);
                
                unsafe {
                    *self.temp.get_unchecked_mut(idx) = new_temps[local_idx];
                }
            }
        }
    }
}