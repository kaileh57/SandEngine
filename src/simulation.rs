use crate::particle::Particle;
use crate::materials::MaterialType;
use crate::physics::PhysicsState;
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationState {
    pub width: usize,
    pub height: usize,
    pub particles: HashMap<(usize, usize), Particle>,
}

// Pre-computed direction lookup tables for performance
const NEIGHBOR_OFFSETS: [(i32, i32); 8] = [
    (-1, -1), (0, -1), (1, -1),
    (-1,  0),          (1,  0),
    (-1,  1), (0,  1), (1,  1),
];

#[derive(Debug)]
pub struct DirtyRect {
    pub min_x: usize,
    pub min_y: usize,
    pub max_x: usize,
    pub max_y: usize,
}

impl DirtyRect {
    pub fn new() -> Self {
        Self {
            min_x: usize::MAX,
            min_y: usize::MAX,
            max_x: 0,
            max_y: 0,
        }
    }

    pub fn expand(&mut self, x: usize, y: usize) {
        self.min_x = self.min_x.min(x);
        self.min_y = self.min_y.min(y);
        self.max_x = self.max_x.max(x);
        self.max_y = self.max_y.max(y);
    }

    pub fn is_valid(&self) -> bool {
        self.min_x != usize::MAX
    }

    pub fn clear(&mut self) {
        self.min_x = usize::MAX;
        self.min_y = usize::MAX;
        self.max_x = 0;
        self.max_y = 0;
    }
}

#[derive(Debug)]
pub struct Simulation {
    pub width: usize,
    pub height: usize,
    // Optimized flat array for better cache performance
    grid: Vec<Option<Particle>>,
    // Track dirty rectangles for efficient updates
    dirty_rect: DirtyRect,
    col_order: Vec<usize>,
    physics: PhysicsState,
    particle_count: usize,
    // Active particles that need processing (performance optimization)
    active_particles: Vec<(usize, usize)>,
}

impl Simulation {
    pub fn new(width: usize, height: usize) -> Self {
        // Use flat array for better cache performance and memory layout
        let grid = vec![None; width * height];
        let col_order: Vec<usize> = (0..width).collect();
        let physics = PhysicsState::new(width, height);

        Self {
            width,
            height,
            grid,
            dirty_rect: DirtyRect::new(),
            col_order,
            physics,
            particle_count: 0,
            active_particles: Vec::new(),
        }
    }
    
    // Helper for flat array indexing - inline for performance
    #[inline(always)]
    fn get_index(&self, x: usize, y: usize) -> usize {
        y * self.width + x
    }

    pub fn clear(&mut self) {
        self.grid.fill(None);
        self.dirty_rect.clear();
        self.particle_count = 0;
        self.active_particles.clear();
    }

    pub fn is_valid(&self, x: i32, y: i32) -> bool {
        x >= 0 && (x as usize) < self.width && y >= 0 && (y as usize) < self.height
    }

    #[inline(always)]
    pub fn get_particle(&self, x: usize, y: usize) -> Option<&Particle> {
        if x < self.width && y < self.height {
            let index = self.get_index(x, y);
            self.grid[index].as_ref()
        } else {
            None
        }
    }

    pub fn get_particle_mut(&mut self, x: usize, y: usize) -> Option<&mut Particle> {
        if x < self.width && y < self.height {
            let index = self.get_index(x, y);
            self.grid[index].as_mut()
        } else {
            None
        }
    }

    pub fn set_particle(&mut self, x: usize, y: usize, particle: Particle) -> Option<Particle> {
        if x < self.width && y < self.height {
            let mut new_particle = particle;
            new_particle.x = x;
            new_particle.y = y;
            new_particle.invalidate_color_cache();
            
            let index = self.get_index(x, y);
            let was_empty = self.grid[index].is_none();
            let is_dynamic = new_particle.dynamic;
            let old_particle = self.grid[index].replace(new_particle);
            
            if was_empty {
                self.particle_count += 1;
            }
            
            // Mark dirty region
            self.dirty_rect.expand(x, y);
            
            // Track active particles if they're dynamic
            if is_dynamic {
                self.active_particles.push((x, y));
            }
            
            old_particle
        } else {
            None
        }
    }

    pub fn remove_particle(&mut self, x: usize, y: usize) -> Option<Particle> {
        if x < self.width && y < self.height {
            let index = self.get_index(x, y);
            if let Some(particle) = self.grid[index].take() {
                self.particle_count = self.particle_count.saturating_sub(1);
                self.dirty_rect.expand(x, y);
                Some(particle)
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn swap_particles(&mut self, x1: usize, y1: usize, x2: usize, y2: usize) {
        if x1 < self.width && y1 < self.height && x2 < self.width && y2 < self.height {
            let index1 = self.get_index(x1, y1);
            let index2 = self.get_index(x2, y2);
            
            let p1 = self.grid[index1].take();
            let p2 = self.grid[index2].take();

            if let Some(mut p1) = p1 {
                p1.x = x2;
                p1.y = y2;
                p1.invalidate_color_cache();
                self.grid[index2] = Some(p1);
            }

            if let Some(mut p2) = p2 {
                p2.x = x1;
                p2.y = y1;
                p2.invalidate_color_cache();
                self.grid[index1] = Some(p2);
            }
            
            self.dirty_rect.expand(x1, y1);
            self.dirty_rect.expand(x2, y2);
        }
    }

    pub fn update(&mut self, delta_time: f32) {
        // Early exit if no dirty region
        if !self.dirty_rect.is_valid() {
            return;
        }

        // Reset processed and moved flags only in dirty region
        for y in self.dirty_rect.min_y..=self.dirty_rect.max_y.min(self.height - 1) {
            for x in self.dirty_rect.min_x..=self.dirty_rect.max_x.min(self.width - 1) {
                let index = self.get_index(x, y);
                if let Some(particle) = &mut self.grid[index] {
                    particle.processed = false;
                    particle.moved_this_step = false;
                }
            }
        }

        // Shuffle column processing order
        let mut rng = rand::thread_rng();
        self.col_order.shuffle(&mut rng);
        let col_order = self.col_order.clone();

        // Process particles bottom-up, only in dirty region
        let mut new_dirty_rect = DirtyRect::new();
        
        // Process with chunked approach for better performance
        const CHUNK_SIZE: usize = 16;
        let dirty_width = self.dirty_rect.max_x.min(self.width - 1) - self.dirty_rect.min_x + 1;
        let dirty_height = self.dirty_rect.max_y.min(self.height - 1) - self.dirty_rect.min_y + 1;
        
        // Process in chunks to improve cache locality
        for chunk_y in (0..((dirty_height + CHUNK_SIZE - 1) / CHUNK_SIZE)).rev() {
            for chunk_x in 0..((dirty_width + CHUNK_SIZE - 1) / CHUNK_SIZE) {
                let start_x = self.dirty_rect.min_x + chunk_x * CHUNK_SIZE;
                let end_x = (start_x + CHUNK_SIZE).min(self.dirty_rect.max_x.min(self.width - 1) + 1);
                let start_y = self.dirty_rect.min_y + chunk_y * CHUNK_SIZE;
                let end_y = (start_y + CHUNK_SIZE).min(self.dirty_rect.max_y.min(self.height - 1) + 1);
                
                for y in (start_y..end_y).rev() {
                    for &x in &col_order {
                        if x >= start_x && x < end_x {
                            let index = self.get_index(x, y);
                            if let Some(particle) = self.grid[index].take() {
                                if !particle.processed && particle.material_type != MaterialType::Empty {
                                    // Skip processing for static particles that are settled
                                    if !particle.dynamic && particle.settled_frames > 30 {
                                        self.grid[index] = Some(particle);
                                        continue;
                                    }
                                    
                                    let updated_particle = self.update_particle(particle, delta_time);
                                    if let Some(updated) = updated_particle {
                                        if updated.material_type != MaterialType::Empty {
                                            let new_x = updated.x;
                                            let new_y = updated.y;
                                            let new_index = self.get_index(new_x, new_y);
                                            self.grid[new_index] = Some(updated);
                                            
                                            // Track new dirty region
                                            new_dirty_rect.expand(new_x, new_y);
                                            if new_x != x || new_y != y {
                                                new_dirty_rect.expand(x, y); // Mark old position as dirty too
                                            }
                                        }
                                    }
                                } else {
                                    self.grid[index] = Some(particle);
                                }
                            }
                        }
                    }
                }
            }
        }

        // Update dirty rectangle for next frame
        self.dirty_rect = new_dirty_rect;
    }

    fn update_particle(&mut self, mut particle: Particle, delta_time: f32) -> Option<Particle> {
        particle.processed = true;
        let (x, y) = (particle.x, particle.y);

        // 1. Handle lifespan and burnout
        if let Some(new_particle) = self.physics.handle_lifespan_and_burnout(&mut particle, delta_time) {
            return Some(new_particle);
        }

        // Dynamic flag optimization: skip expensive physics for static particles
        let skip_physics = !particle.dynamic && particle.settled_frames > 10;

        let (state_change_result, new_particles) = if skip_physics {
            // Just increment time for static particles
            particle.time_in_state += delta_time;
            particle.settled_frames = particle.settled_frames.saturating_add(1);
            (None, Vec::new())
        } else {
            // 2. Get neighbors for temperature and state change calculations
            let neighbors = self.get_neighbors(x, y);

            // 3. Update temperature
            self.physics.update_temperature(&mut particle, &neighbors, delta_time);

            // 4. Handle state changes and effects
            self.physics.handle_state_changes_and_effects(&mut particle, &neighbors, delta_time)
        };
        
        // Place new particles from effects
        for (nx, ny, new_particle) in new_particles {
            if nx < self.width && ny < self.height {
                let index = self.get_index(nx, ny);
                self.grid[index] = Some(new_particle);
            }
        }

        if let Some(new_particle) = state_change_result {
            return Some(new_particle);
        }

        // 5. Increment time in state
        particle.time_in_state += delta_time;

        // 6. Handle movement
        let (new_x, new_y) = self.handle_movement(&mut particle);
        particle.x = new_x;
        particle.y = new_y;
        if new_x != x || new_y != y {
            particle.moved_this_step = true;
            particle.settled_frames = 0; // Reset settled counter when moving
        } else {
            particle.settled_frames = particle.settled_frames.saturating_add(1);
            // If particle becomes static, it might be removed from active tracking
            if particle.dynamic && particle.settled_frames > 20 {
                // Will be handled by the active particle cleanup
            }
        }

        Some(particle)
    }

    fn get_neighbors(&self, x: usize, y: usize) -> Vec<Option<&Particle>> {
        let mut neighbors = Vec::with_capacity(8);

        for &(dx, dy) in &NEIGHBOR_OFFSETS {
            let nx = x as i32 + dx;
            let ny = y as i32 + dy;
            if self.is_valid(nx, ny) {
                neighbors.push(self.get_particle(nx as usize, ny as usize));
            } else {
                neighbors.push(None);
            }
        }

        neighbors
    }

    #[inline(always)]
    fn handle_movement(&mut self, particle: &mut Particle) -> (usize, usize) {
        let (x, y) = (particle.x, particle.y);
        let props = particle.get_properties();
        
        if particle.material_type == MaterialType::Generator {
            return (x, y); // Generators are immovable
        }

        let density = props.density;
        let is_gas = density < 0.0;
        let is_liquid = props.is_liquid(particle.material_type);
        let is_powder = props.is_powder(particle.material_type);

        let vert_dir = if is_gas { -1 } else { 1 };
        let ny = y as i32 + vert_dir;

        // Check boundaries
        if !self.is_valid(x as i32, ny) {
            return (x, y);
        }

        let target_y = ny as usize;

        // Try vertical movement first - check if target cell is empty
        if self.get_particle(x, target_y).is_none() {
            // Check for fast falling - if multiple cells below are empty, convert to falling particle
            if !is_gas && particle.material_type == MaterialType::Sand {
                let mut empty_count = 0;
                for check_y in (target_y + 1)..(target_y + 5).min(self.height) {
                    if self.get_particle(x, check_y).is_none() {
                        empty_count += 1;
                    } else {
                        break;
                    }
                }
                
                // If 3+ cells below are empty, this would become a particle in the reference
                // For now, just move down 2 cells for speed
                if empty_count >= 3 && target_y + 1 < self.height {
                    return (x, target_y + 1);
                }
            }
            
            // Target cell is truly empty (None), move there
            return (x, target_y);
        } else if let Some(target_particle) = self.get_particle(x, target_y) {
            // If target contains an Empty particle, move there
            if target_particle.material_type == MaterialType::Empty {
                return (x, target_y);
            }

            // Check for density-based swapping
            let target_props = target_particle.get_properties();
            let target_density = target_props.density;
            let should_swap = if is_gas {
                target_density > density
            } else {
                density > target_density
            };

            if should_swap && target_particle.material_type != MaterialType::Generator {
                // Need to handle swapping differently - for now just don't move
                // TODO: Implement proper swapping in the main update loop
                return (x, y);
            }
        }

        // Try diagonal movement for non-rigid materials
        if !matches!(particle.material_type, MaterialType::Stone | MaterialType::Glass | MaterialType::Wood | MaterialType::Ice) {
            let directions = if rand::random::<bool>() { [-1, 1] } else { [1, -1] };
            
            for &dx in &directions {
                let diag_x = x as i32 + dx;
                let diag_y = ny;
                
                if self.is_valid(diag_x, diag_y) {
                    let diag_x = diag_x as usize;
                    let diag_y = diag_y as usize;
                    
                    if self.get_particle(diag_x, diag_y).is_none() {
                        // Empty diagonal spot
                        return (diag_x, diag_y);
                    } else if let Some(diag_target) = self.get_particle(diag_x, diag_y) {
                        if diag_target.material_type == MaterialType::Empty {
                            return (diag_x, diag_y);
                        }
                    }
                }
            }
        }

        // Horizontal movement for liquids and gases
        if is_liquid || is_gas {
            let directions = if rand::random::<bool>() { [-1, 1] } else { [1, -1] };
            
            for &dx in &directions {
                let side_x = x as i32 + dx;
                
                if self.is_valid(side_x, y as i32) {
                    let side_x = side_x as usize;
                    
                    if self.get_particle(side_x, y).is_none() {
                        // Empty side spot
                        let move_chance = if is_liquid {
                            (1.0 - props.viscosity * 0.1).max(0.1)
                        } else {
                            1.0
                        };
                        
                        if rand::random::<f32>() < move_chance {
                            return (side_x, y);
                        }
                    } else if let Some(side_target) = self.get_particle(side_x, y) {
                        if side_target.material_type == MaterialType::Empty {
                            let move_chance = if is_liquid {
                                (1.0 - props.viscosity * 0.1).max(0.1)
                            } else {
                                1.0
                            };
                            
                            if rand::random::<f32>() < move_chance {
                                return (side_x, y);
                            }
                        }
                    }
                }
            }
        }

        // Powder piling for falling powders
        if is_powder && vert_dir == 1 {
            if y + 1 < self.height {
                if let Some(below) = self.get_particle(x, y + 1) {
                    if below.material_type != MaterialType::Empty && below.material_type != MaterialType::Generator {
                        let directions = if rand::random::<bool>() { [-1, 1] } else { [1, -1] };
                        
                        for &dx in &directions {
                            let pile_x = x as i32 + dx;
                            let pile_y = y + 1;
                            
                            if self.is_valid(pile_x, pile_y as i32) {
                                let pile_x = pile_x as usize;
                                
                                if self.get_particle(pile_x, pile_y).is_none() {
                                    // Empty pile spot
                                    return (pile_x, pile_y);
                                } else if let Some(pile_target) = self.get_particle(pile_x, pile_y) {
                                    if pile_target.material_type == MaterialType::Empty {
                                        return (pile_x, pile_y);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // No movement possible
        (x, y)
    }

    pub fn add_particle(&mut self, x: usize, y: usize, material_type: MaterialType, temp: Option<f32>) -> bool {
        if x < self.width && y < self.height {
            // Check if we can place here - only protect generators from non-eraser materials
            if let Some(existing) = self.get_particle(x, y) {
                if existing.material_type == MaterialType::Generator && material_type != MaterialType::Eraser {
                    return false; // Can't overwrite generators unless erasing
                }
            }
            
            if material_type == MaterialType::Eraser {
                self.remove_particle(x, y);
            } else {
                let initial_temp = match material_type {
                    MaterialType::Lava => Some(2500.0),
                    _ => temp,
                };
                let particle = Particle::new(x, y, material_type, initial_temp);
                self.set_particle(x, y, particle);
            }
            true
        } else {
            false
        }
    }

    pub fn get_state(&self) -> SimulationState {
        let mut particles = HashMap::new();
        
        for y in 0..self.height {
            for x in 0..self.width {
                let index = self.get_index(x, y);
                if let Some(particle) = &self.grid[index] {
                    if particle.material_type != MaterialType::Empty {
                        particles.insert((x, y), particle.clone());
                    }
                }
            }
        }

        SimulationState {
            width: self.width,
            height: self.height,
            particles,
        }
    }

    pub fn get_particle_data(&self, x: usize, y: usize) -> Option<(MaterialType, f32, Option<f32>, bool)> {
        if let Some(particle) = self.get_particle(x, y) {
            Some((
                particle.material_type,
                particle.temp,
                particle.life,
                particle.burning,
            ))
        } else {
            None
        }
    }
}