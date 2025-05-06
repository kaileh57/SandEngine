// simulation.rs - Core simulation management and integration

use rand::prelude::*;
use crate::constants::*;
use crate::material_properties::MaterialType;
use crate::particle::Particle;
use crate::physics::PhysicsEngine;
use crate::reactions::ReactionEngine;
use crate::temperature::TemperatureSystem;

pub struct SandSimulation {
    // Particle grid storage
    grid: Vec<Option<Particle>>,
    
    // Simulation subsystems
    physics: PhysicsEngine,
    reactions: ReactionEngine,
    temperature: TemperatureSystem,
    
    // Active particle tracking
    active_cells: Vec<(usize, usize)>,
    next_active_cells: Vec<(usize, usize)>,
    
    // Activity bounds tracking
    min_active_x: usize,
    max_active_x: usize,
    min_active_y: usize,
    max_active_y: usize,
    
    // Simulation state
    pub brush_size: usize,
    pub current_material: MaterialType,
    pub cursor_pos: (usize, usize),
    
    // Settings
    do_temperature_diffusion: bool,
    max_active_cells: usize,
    rng: ThreadRng,
    
    // Temporary working vectors to avoid reallocation
    temp_updates: Vec<(usize, usize, f32)>,
}

impl SandSimulation {
    pub fn new() -> Self {
        let grid_size = GRID_WIDTH * GRID_HEIGHT;
        
        // Allocate approximately 25% of grid size for active cells
        let active_capacity = grid_size / 4;
        
        // Initialize grid with all empty cells
        let mut grid = Vec::with_capacity(grid_size);
        for _ in 0..grid_size {
            grid.push(None);
        }
        
        Self {
            grid,
            physics: PhysicsEngine::new(),
            reactions: ReactionEngine::new(),
            temperature: TemperatureSystem::new(),
            active_cells: Vec::with_capacity(active_capacity),
            next_active_cells: Vec::with_capacity(active_capacity),
            min_active_x: GRID_WIDTH,
            max_active_x: 0,
            min_active_y: GRID_HEIGHT,
            max_active_y: 0,
            brush_size: 3,
            current_material: MaterialType::Sand,
            cursor_pos: (0, 0),
            do_temperature_diffusion: true,
            max_active_cells: 50000,
            rng: rand::thread_rng(),
            temp_updates: Vec::with_capacity(1000),
        }
    }
    
    #[inline(always)]
    fn get_index(x: usize, y: usize) -> usize {
        y * GRID_WIDTH + x
    }
    
    #[inline(always)]
    pub fn get(&self, x: usize, y: usize) -> MaterialType {
        if x < GRID_WIDTH && y < GRID_HEIGHT {
            let idx = Self::get_index(x, y);
            if let Some(particle) = &self.grid[idx] {
                return particle.material;
            }
        }
        MaterialType::Empty
    }
    
    #[inline(always)]
    pub fn get_temp(&self, x: usize, y: usize) -> f32 {
        if x < GRID_WIDTH && y < GRID_HEIGHT {
            let idx = Self::get_index(x, y);
            if let Some(particle) = &self.grid[idx] {
                return particle.temperature.get();
            }
        }
        AMBIENT_TEMP
    }
    
    #[inline(always)]
    fn get_particle(&self, x: usize, y: usize) -> Option<Particle> {
        if x < GRID_WIDTH && y < GRID_HEIGHT {
            let idx = Self::get_index(x, y);
            self.grid[idx].clone()
        } else {
            None
        }
    }
    
    #[inline(always)]
    fn get_particle_mut(&mut self, x: usize, y: usize) -> Option<&mut Particle> {
        if x < GRID_WIDTH && y < GRID_HEIGHT {
            let idx = Self::get_index(x, y);
            self.grid[idx].as_mut()
        } else {
            None
        }
    }
    
    #[inline(always)]
    fn set_particle(&mut self, x: usize, y: usize, particle: Particle) -> bool {
        if x < GRID_WIDTH && y < GRID_HEIGHT {
            let idx = Self::get_index(x, y);
            
            // Store the new particle
            self.grid[idx] = Some(particle);
            
            // Update active bounds
            self.update_active_bounds(x, y);
            
            // Indicate success
            true
        } else {
            false
        }
    }
    
    #[inline(always)]
    fn set_empty(&mut self, x: usize, y: usize) -> bool {
        if x < GRID_WIDTH && y < GRID_HEIGHT {
            let idx = Self::get_index(x, y);
            self.grid[idx] = None;
            true
        } else {
            false
        }
    }
    
    #[inline(always)]
    fn swap_particles(&mut self, x1: usize, y1: usize, x2: usize, y2: usize) -> bool {
        if x1 < GRID_WIDTH && y1 < GRID_HEIGHT && x2 < GRID_WIDTH && y2 < GRID_HEIGHT {
            let idx1 = Self::get_index(x1, y1);
            let idx2 = Self::get_index(x2, y2);
            
            self.grid.swap(idx1, idx2);
            
            // Update active bounds
            if self.grid[idx1].is_some() {
                self.update_active_bounds(x1, y1);
            }
            if self.grid[idx2].is_some() {
                self.update_active_bounds(x2, y2);
            }
            
            true
        } else {
            false
        }
    }
    
    #[inline(always)]
    fn update_active_bounds(&mut self, x: usize, y: usize) {
        self.min_active_x = self.min_active_x.min(x);
        self.max_active_x = self.max_active_x.max(x);
        self.min_active_y = self.min_active_y.min(y);
        self.max_active_y = self.max_active_y.max(y);
    }
    
    pub fn clear(&mut self) {
        // Clear the grid
        for cell in &mut self.grid {
            *cell = None;
        }
        
        // Clear active tracking
        self.active_cells.clear();
        self.next_active_cells.clear();
        
        // Reset bounds
        self.min_active_x = GRID_WIDTH;
        self.max_active_x = 0;
        self.min_active_y = GRID_HEIGHT;
        self.max_active_y = 0;
    }
    
    pub fn update(&mut self) {
        // Reset simulation step
        self.reset_simulation_step();
        
        // Process temperature diffusion if enabled
        if self.do_temperature_diffusion {
            self.temp_updates.clear();
            self.process_temperature_diffusion();
        }
        
        // Shuffle active cells for varied behavior
        self.shuffle_active_cells();
        
        // Process particles
        self.process_all_particles();
        
        // Check if we need to scan for new activity
        self.check_active_state();
    }
    
    // Process temperature diffusion with minimal copying
    fn process_temperature_diffusion(&mut self) {
        let x_start = self.min_active_x.saturating_sub(5);
        let x_end = (self.max_active_x + 5).min(GRID_WIDTH);
        let y_start = self.min_active_y.saturating_sub(5);
        let y_end = (self.max_active_y + 5).min(GRID_HEIGHT);
        
        // First pass: calculate temperatures without modifying particles
        for y in y_start..y_end {
            for x in x_start..x_end {
                if let Some(particle) = self.get_particle(x, y) {
                    if particle.material == MaterialType::Empty {
                        continue;
                    }
                    
                    let current_temp = particle.temperature.get();
                    let material = particle.material;
                    
                    // Skip rigid/immovable materials if not burning/hot
                    if material == MaterialType::Stone && 
                       (current_temp - AMBIENT_TEMP).abs() < 10.0 {
                        continue;
                    }
                    
                    // Natural cooling and conduction
                    let mut new_temp = current_temp;
                    let props = material.get_properties();
                    let conductivity = props.conductivity;
                    
                    // Collect neighbor temperatures - only query direct neighbors
                    let mut neighbor_temp_sum = 0.0;
                    let mut neighbor_conductivity_sum = 0.0;
                    let mut neighbor_count = 0;
                    
                    for &(dx, dy) in &[(0, -1), (-1, 0), (1, 0), (0, 1)] {
                        let nx = match (x as isize + dx).try_into() {
                            Ok(val) if val < GRID_WIDTH => val,
                            _ => continue,
                        };
                        
                        let ny = match (y as isize + dy).try_into() {
                            Ok(val) if val < GRID_HEIGHT => val,
                            _ => continue,
                        };
                        
                        if let Some(neighbor) = self.get_particle(nx, ny) {
                            let n_temp = neighbor.temperature.get();
                            let n_cond = neighbor.material.get_properties().conductivity;
                            
                            neighbor_temp_sum += n_temp * n_cond;
                            neighbor_conductivity_sum += n_cond;
                            neighbor_count += 1;
                        } else {
                            // Ambient for empty cells
                            neighbor_temp_sum += AMBIENT_TEMP * 0.1;
                            neighbor_conductivity_sum += 0.1;
                            neighbor_count += 1;
                        }
                    }
                    
                    // Temperature changes
                    if neighbor_count > 0 && (conductivity > 0.0 || neighbor_conductivity_sum > 0.0) {
                        let total_conductivity = conductivity + neighbor_conductivity_sum;
                        
                        if total_conductivity > 0.001 {
                            let weighted_avg_temp = (current_temp * conductivity + neighbor_temp_sum) / total_conductivity;
                            let mut delta_temp = (weighted_avg_temp - current_temp) * 0.5_f32.min(conductivity);
                            
                            delta_temp *= (1.0 / 60.0) * TARGET_DT_SCALING;
                            
                            new_temp += delta_temp;
                        }
                    }
                    
                    // Apply ambient cooling
                    new_temp += (AMBIENT_TEMP - new_temp) * COOLING_RATE * conductivity * (1.0 / 60.0) * TARGET_DT_SCALING;
                    
                    // Apply heat generation if material generates heat
                    if let Some(heat_gen) = props.heat_generation {
                        if heat_gen > 0.0 {
                            new_temp += heat_gen * (1.0 / 60.0) * TARGET_DT_SCALING;
                        }
                    }
                    
                    // Special case temperatures
                    match material {
                        MaterialType::Fire => new_temp = new_temp.max(600.0),
                        MaterialType::Lava => new_temp = new_temp.max(1200.0),
                        MaterialType::Generator => new_temp = new_temp.max(300.0),
                        MaterialType::Ice => new_temp = new_temp.min(-5.0),
                        _ => {}
                    }
                    
                    // Only update if temperature changed significantly
                    if (new_temp - current_temp).abs() > 0.01 {
                        self.temp_updates.push((x, y, new_temp));
                    }
                }
            }
        }
        
        // Second pass: Apply all temperature updates at once
        // Copy the updates to a local Vec to avoid borrowing issues
        let temp_updates_copy = self.temp_updates.clone();
        for (x, y, temp) in temp_updates_copy {
            if let Some(particle) = self.get_particle_mut(x, y) {
                if particle.material != MaterialType::Empty {
                    particle.temperature.set(temp);
                }
            }
        }
    }
    
    fn reset_simulation_step(&mut self) {
        // Reset all particles' processed state
        for cell in &mut self.grid {
            if let Some(particle) = cell {
                particle.reset_processed();
            }
        }
        
        // Deduplicate active cells
        if self.next_active_cells.len() > 1000 {
            self.next_active_cells.sort_unstable();
            self.next_active_cells.dedup();
        }
        
        // Swap active cells lists
        std::mem::swap(&mut self.active_cells, &mut self.next_active_cells);
        self.next_active_cells.clear();
    }
    
    fn shuffle_active_cells(&mut self) {
        let cell_count = self.active_cells.len();
        
        if cell_count > 100 {
            // For large active sets, only shuffle a portion for performance
            let shuffle_count = (cell_count / 10).min(1000);
            for i in 0..shuffle_count {
                let j = self.rng.gen_range(0..cell_count);
                self.active_cells.swap(i, j);
            }
        } else if cell_count > 0 {
            // For small sets, shuffle everything
            for i in 0..cell_count {
                let j = self.rng.gen_range(0..cell_count);
                self.active_cells.swap(i, j);
            }
        }
    }
    
    fn process_all_particles(&mut self) {
        // Process active cells first (phase 1)
        self.process_active_cells();
        
        // Process all liquids cells (phase 2)
        self.process_all_liquids();
    }
    
    fn process_active_cells(&mut self) {
        // Create a local copy so we can modify active_cells safely
        let active_cells = std::mem::take(&mut self.active_cells);
        
        for (x, y) in active_cells {
            if x >= GRID_WIDTH || y >= GRID_HEIGHT {
                continue;
            }
            
            // Skip if already processed
            if let Some(particle) = self.get_particle(x, y) {
                if particle.processed {
                    continue;
                }
                
                // Skip empty cells and liquids (processed separately)
                if particle.material == MaterialType::Empty || 
                   particle.material == MaterialType::Water || 
                   particle.material == MaterialType::Lava {
                    continue;
                }
                
                // Process the particle
                self.process_particle(x, y);
            }
        }
        
        // Restore active_cells
        self.active_cells = Vec::with_capacity(self.max_active_cells);
    }
    
    fn process_all_liquids(&mut self) {
        // Instead of scanning the entire grid, only scan the active area
        let x_start = self.min_active_x.saturating_sub(5);
        let x_end = (self.max_active_x + 5).min(GRID_WIDTH);
        let y_start = self.min_active_y.saturating_sub(5);
        let y_end = (self.max_active_y + 5).min(GRID_HEIGHT);
        
        for y in y_start..y_end {
            for x in x_start..x_end {
                if let Some(particle) = self.get_particle(x, y) {
                    if particle.processed {
                        continue;
                    }
                    
                    // Process liquids
                    if particle.material == MaterialType::Water || 
                       particle.material == MaterialType::Lava {
                        self.process_particle(x, y);
                    }
                }
            }
        }
    }
    
    // Process a single particle
    fn process_particle(&mut self, x: usize, y: usize) {
        // First, get the particle and check if it exists
        let particle_opt = self.get_particle(x, y);
        if particle_opt.is_none() {
            return;
        }
        
        let mut particle = particle_opt.unwrap();
        
        // Mark as processed to avoid double processing
        particle.processed = true;
        self.set_particle(x, y, particle.clone());
        
        // Delta time (assuming 60 FPS)
        let delta_time = 1.0 / 60.0;
        
        // --------------------
        // STEP 1: Handle lifespan and burnout
        // --------------------
        
        // Check if particle has reached the end of its life
        if particle.update_life(delta_time) {
            // Get the successor material and temperature
            let (new_material, new_temp) = particle.get_successor_material();
            
            // Create new particle if needed
            if new_material != MaterialType::Empty {
                let mut new_particle = Particle::new(new_material, new_temp);
                new_particle.processed = true;
                
                if self.set_particle(x, y, new_particle) {
                    return; // Particle was replaced
                }
            } else {
                // Just remove the particle (empty)
                let empty = Particle::new(MaterialType::Empty, AMBIENT_TEMP);
                if self.set_particle(x, y, empty) {
                    return; // Particle was removed
                }
            }
        }
        
        // --------------------
        // STEP 2: Handle state changes and other reactions
        // --------------------
        
        // Increment time in state
        particle.increment_time_in_state(delta_time);
        
        // Handle state changes - first collect local neighbors
        let neighbors = self.collect_neighbors(x, y);
        
        // Create a buffer to track changes
        let mut changes = Vec::new();
        let mut active_cells_to_add = Vec::new();
        
        // Helper function to get particle from neighbors
        let get_particle_fn = |nx: usize, ny: usize| {
            for &((x1, y1), ref p) in &neighbors {
                if x1 == nx && y1 == ny {
                    return p.clone();
                }
            }
            None
        };
        
        // Helper to track changes instead of applying them immediately
        let set_particle_fn = |nx: usize, ny: usize, p: Particle| {
            changes.push((nx, ny, p));
            true
        };
        
        let add_active_cell_fn = |nx: usize, ny: usize| {
            active_cells_to_add.push((nx, ny));
        };
        
        // Handle state changes
        let state_changed = self.reactions.handle_state_changes_and_effects(
            &mut particle,
            x,
            y,
            delta_time,
            get_particle_fn,
            set_particle_fn,
            add_active_cell_fn,
        );
        
        // If we need to make changes, do them now
        for (nx, ny, p) in changes {
            self.set_particle(nx, ny, p);
        }
        
        // Add active cells
        for (nx, ny) in active_cells_to_add {
            self.add_active_cell(nx, ny);
        }
        
        // If state changed, the particle was replaced - stop processing
        if state_changed {
            return;
        }
        
        // --------------------
        // STEP 3: Handle movement
        // --------------------
        
        // Handle movement avoiding borrow checker issues
        self.handle_particle_movement(x, y, &mut particle);
        
        // If we reach here, the particle was updated but not moved
        // Update it in place and mark it as active for the next cycle
        self.set_particle(x, y, particle);
        self.add_active_cell(x, y);
    }
    
    // Helper to collect neighbors of a cell without borrow checker issues
    fn collect_neighbors(&self, x: usize, y: usize) -> Vec<((usize, usize), Option<Particle>)> {
        let mut neighbors = Vec::with_capacity(9);
        
        for dy in -1..=1 {
            for dx in -1..=1 {
                let nx = x as isize + dx;
                let ny = y as isize + dy;
                if nx >= 0 && nx < GRID_WIDTH as isize && 
                   ny >= 0 && ny < GRID_HEIGHT as isize {
                    let p = self.get_particle(nx as usize, ny as usize);
                    neighbors.push(((nx as usize, ny as usize), p));
                }
            }
        }
        
        neighbors
    }
    
    // Handle particle movement in a separate function to avoid borrow errors
    fn handle_particle_movement(&mut self, x: usize, y: usize, particle: &mut Particle) -> bool {
        // First collect neighbors to avoid borrowing issues
        let neighbors = self.collect_neighbors(x, y);
        
        // Helper function to get particle from neighbors
        let get_particle_fn = |nx: usize, ny: usize| {
            for &((x1, y1), ref p) in &neighbors {
                if x1 == nx && y1 == ny {
                    return p.clone();
                }
            }
            None
        };
        
        // Create a single movement command to apply after physics calculation
        let mut move_command: Option<(usize, usize, Particle)> = None;
        let mut swap_command: Option<(usize, usize, usize, usize)> = None;
        
        // Custom set_particle that just records the command
        let set_particle_fn = |nx: usize, ny: usize, p: Particle| {
            if nx == x && ny == y {
                // Simple update in place
                move_command = Some((nx, ny, p));
                true
            } else {
                // Movement to a new location
                move_command = Some((nx, ny, p));
                true
            }
        };
        
        // Custom swap_particles that just records the command
        let swap_particles_fn = |x1: usize, y1: usize, x2: usize, y2: usize| {
            swap_command = Some((x1, y1, x2, y2));
            true
        };
        
        // Now perform the physics calculation (without modifying the grid)
        let movement_result = self.physics.handle_movement(
            particle,
            x, 
            y,
            get_particle_fn,
            set_particle_fn,
            swap_particles_fn
        );
        
        // Apply the movement based on the commands recorded
        if let Some((nx, ny, p)) = move_command {
            if nx != x || ny != y {
                // Only clear original if new position is successfully set
                if self.set_particle(nx, ny, p) {
                    self.set_empty(x, y);
                    return true;
                }
            } else {
                // Simple update in place
                self.set_particle(nx, ny, p);
            }
        }
        
        if let Some((x1, y1, x2, y2)) = swap_command {
            self.swap_particles(x1, y1, x2, y2);
            return true;
        }
        
        movement_result
    }
    
    fn add_active_cell(&mut self, x: usize, y: usize) {
        if self.next_active_cells.len() < self.max_active_cells {
            self.next_active_cells.push((x, y));
            self.update_active_bounds(x, y);
            
            // Add neighbors for better propagation
            self.add_neighbors_to_active(x, y);
        }
    }
    
    fn add_neighbors_to_active(&mut self, x: usize, y: usize) {
        if self.next_active_cells.len() >= self.max_active_cells {
            return;
        }
        
        // Get current material for context-aware neighbor activation
        let material = self.get(x, y);
        let is_fluid = material == MaterialType::Water || material == MaterialType::Lava;
        
        // Add immediate neighbors (cardinal directions)
        for &(dy, dx) in &[(-1, 0), (0, -1), (0, 1), (1, 0)] {
            let nx = match (x as isize + dx).try_into() {
                Ok(val) if val < GRID_WIDTH => val,
                _ => continue,
            };
            
            let ny = match (y as isize + dy).try_into() {
                Ok(val) if val < GRID_HEIGHT => val,
                _ => continue,
            };
            
            if self.next_active_cells.len() < self.max_active_cells {
                self.next_active_cells.push((nx, ny));
            } else {
                break;
            }
        }
        
        // For liquids, use extended neighbor pattern
        if is_fluid {
            let radius = if material == MaterialType::Water { 3 } else { 2 };
            
            for dy in (-radius as isize)..=(radius as isize) {
                for dx in (-radius as isize)..=(radius as isize) {
                    // Skip already added cardinal neighbors and center
                    if (dx == 0 && dy == 0) || 
                       (dx == 0 && dy == -1) || 
                       (dx == -1 && dy == 0) || 
                       (dx == 1 && dy == 0) || 
                       (dx == 0 && dy == 1) {
                        continue;
                    }
                    
                    // Use manhattan distance to prioritize closer cells
                    let manhattan_dist = dx.abs() + dy.abs();
                    if manhattan_dist > radius as isize {
                        continue;
                    }
                    
                    let nx = match (x as isize + dx).try_into() {
                        Ok(val) if val < GRID_WIDTH => val,
                        _ => continue,
                    };
                    
                    let ny = match (y as isize + dy).try_into() {
                        Ok(val) if val < GRID_HEIGHT => val,
                        _ => continue,
                    };
                    
                    if self.next_active_cells.len() < self.max_active_cells {
                        self.next_active_cells.push((nx, ny));
                    } else {
                        return;
                    }
                }
            }
        }
    }
    
    fn check_active_state(&mut self) {
        // If no active cells, scan for activity
        if self.active_cells.is_empty() && self.next_active_cells.is_empty() {
            self.scan_for_active_particles();
        }
    }
    
    fn scan_for_active_particles(&mut self) {
        // Reset bounds to encompass the entire grid
        self.min_active_x = 0;
        self.max_active_x = GRID_WIDTH - 1;
        self.min_active_y = 0;
        self.max_active_y = GRID_HEIGHT - 1;
        
        // Scan a subset of the grid to find potentially active particles
        let scan_step = 4; // Skip cells for performance
        
        for y in (0..GRID_HEIGHT).step_by(scan_step) {
            for x in (0..GRID_WIDTH).step_by(scan_step) {
                if let Some(particle) = self.get_particle(x, y) {
                    if particle.material == MaterialType::Empty {
                        continue;
                    }
                    
                    // Check if particle might be active
                    let should_activate = match particle.material {
                        MaterialType::Sand | MaterialType::Ash | MaterialType::Coal | MaterialType::Gunpowder => {
                            y >= GRID_HEIGHT - 1 || 
                            self.get(x, y + 1) == MaterialType::Empty ||
                            (x > 0 && self.get(x - 1, y + 1) == MaterialType::Empty) ||
                            (x < GRID_WIDTH - 1 && self.get(x + 1, y + 1) == MaterialType::Empty) ||
                            particle.vel_x.abs() > 0.05 ||
                            particle.vel_y.abs() > 0.05
                        },
                        MaterialType::Water | MaterialType::Lava | MaterialType::Acid | MaterialType::Oil => true,
                        MaterialType::Fire | MaterialType::Smoke | MaterialType::Steam | MaterialType::ToxicGas => true,
                        MaterialType::Plant => {
                            // Check for adjacent water
                            (x > 0 && self.get(x - 1, y) == MaterialType::Water) ||
                            (x < GRID_WIDTH - 1 && self.get(x + 1, y) == MaterialType::Water) ||
                            (y > 0 && self.get(x, y - 1) == MaterialType::Water) ||
                            (y < GRID_HEIGHT - 1 && self.get(x, y + 1) == MaterialType::Water)
                        },
                        _ => false
                    };
                    
                    if should_activate && self.next_active_cells.len() < self.max_active_cells {
                        self.next_active_cells.push((x, y));
                        
                        // Add neighbors
                        self.add_neighbors_to_active(x, y);
                    }
                }
            }
        }
        
        // Add some random jitter if we found active cells
        if !self.next_active_cells.is_empty() {
            let jitter_count = ((GRID_WIDTH * GRID_HEIGHT) / 2000).min(50);
            
            for _ in 0..jitter_count {
                let x = self.rng.gen_range(0..GRID_WIDTH);
                let y = self.rng.gen_range(0..GRID_HEIGHT);
                
                // Extract RNG values first to avoid borrowing self multiple times
                let rx = self.rng.gen::<f32>() - 0.5;
                let ry = self.rng.gen::<f32>() - 0.5;
                
                if let Some(particle) = self.get_particle_mut(x, y) {
                    if particle.material != MaterialType::Empty && 
                       particle.material != MaterialType::Generator {
                        // Add random velocity
                        particle.vel_x += rx * 0.2;
                        particle.vel_y += ry * 0.2;
                        
                        // Add to active cells
                        if self.next_active_cells.len() < self.max_active_cells {
                            self.next_active_cells.push((x, y));
                        }
                    }
                }
            }
        }
    }
    
    // Public method for material visualization
    pub fn get_temp_color_modifier(&self, x: usize, y: usize) -> [i16; 3] {
        if x < GRID_WIDTH && y < GRID_HEIGHT {
            if let Some(particle) = self.get_particle(x, y) {
                return particle.temperature.get_color_modifier(particle.material);
            }
        }
        
        [0, 0, 0]
    }
    
    // Draw the simulation to a frame buffer
    pub fn draw(&self, frame: &mut [u8]) {
        // Optimized drawing - draw only the active area with a small margin
        let x_start = self.min_active_x.saturating_sub(5);
        let x_end = (self.max_active_x + 5).min(GRID_WIDTH);
        let y_start = self.min_active_y.saturating_sub(5);
        let y_end = (self.max_active_y + 5).min(GRID_HEIGHT);
        
        // If no active area, only draw the border
        if x_start > x_end || y_start > y_end {
            self.draw_border(frame);
            return;
        }
        
        // Draw the active area
        for y in y_start..y_end {
            for x in x_start..x_end {
                let material = self.get(x, y);
                
                // Skip drawing empty cells for performance
                if material == MaterialType::Empty {
                    continue;
                }
                
                // Get particle for proper color calculation
                if let Some(particle) = self.get_particle(x, y) {
                    let base_color = material.get_color();
                    let color = particle.temperature.get_color(material, base_color);
                    
                    // Draw the cell (scaled by CELL_SIZE)
                    let start_px = x * CELL_SIZE;
                    let start_py = y * CELL_SIZE;
                    let end_px = start_px + CELL_SIZE;
                    let end_py = start_py + CELL_SIZE;
                    
                    for py in start_py..end_py {
                        let row_offset = py * WINDOW_WIDTH as usize * 4;
                        for px in start_px..end_px {
                            let idx = row_offset + px * 4;
                            if idx + 3 < frame.len() {
                                frame[idx] = color[0];
                                frame[idx + 1] = color[1];
                                frame[idx + 2] = color[2];
                                frame[idx + 3] = color[3];
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
    
    // Add material using the brush
    pub fn add_material(&mut self, x: usize, y: usize, brush_size: usize, material: MaterialType) {
        let start_x = x.saturating_sub(brush_size);
        let end_x = (x + brush_size).min(GRID_WIDTH - 1);
        let start_y = y.saturating_sub(brush_size);
        let end_y = (y + brush_size).min(GRID_HEIGHT - 1);
        
        let brush_size_squared = (brush_size * brush_size) as isize;
        
        // Pre-calculate all positions to add material
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
        
        // Add material to all calculated positions
        for (cx, cy) in positions {
            // Skip if trying to replace a generator (unless erasing)
            if self.get(cx, cy) == MaterialType::Generator && material != MaterialType::Eraser {
                continue;
            }
            
            // Determine initial properties based on material
            let (init_temp, vel_x, vel_y) = match material {
                MaterialType::Fire => (800.0, 0.0, -0.1),
                MaterialType::Lava => (1800.0, 0.0, 0.0),
                MaterialType::Water => (AMBIENT_TEMP, (self.rng.gen::<f32>() - 0.5) * 0.2, 0.1),
                _ => (AMBIENT_TEMP, 0.0, 0.0),
            };
            
            if material == MaterialType::Eraser {
                // Simply clear the cell
                self.set_empty(cx, cy);
            } else {
                // Create a new particle
                let particle = Particle::new_with_velocity(material, init_temp, vel_x, vel_y);
                self.set_particle(cx, cy, particle);
                
                // Add to active cells
                self.add_active_cell(cx, cy);
            }
        }
    }
}