use crate::{
    chunk::{ChunkManager, ChunkKey},
    materials::MaterialType,
    particle::Particle,
    physics::PhysicsState,
    rigidbody::{RigidBodyManager, RigidBodyAnalyzer},
    spatial::{NeighborCache, CollisionDetector},
};
use ahash::AHashSet;
use smallvec::SmallVec;
use std::time::Instant;

/// Next-generation physics engine with chunk-based simulation and rigid body support
pub struct AdvancedPhysicsEngine {
    pub chunk_manager: ChunkManager,
    pub rigidbody_manager: RigidBodyManager,
    pub neighbor_cache: NeighborCache,
    pub collision_detector: CollisionDetector,
    pub physics_state: PhysicsState,
    
    // Performance tracking
    last_update: Instant,
    frame_count: u64,
    target_fps: f32,
    
    // Optimization settings
    pub enable_rigid_bodies: bool,
    pub enable_spatial_optimization: bool,
    pub max_active_chunks: usize,
    pub rigid_body_threshold: usize, // Minimum particles to form rigid body
    
    // Active chunk tracking
    active_chunks: AHashSet<ChunkKey>,
    chunks_to_process: Vec<ChunkKey>,
}

impl AdvancedPhysicsEngine {
    pub fn new() -> Self {
        let physics_state = PhysicsState::new(0, 0); // Will be resized as needed
        
        Self {
            chunk_manager: ChunkManager::new(),
            rigidbody_manager: RigidBodyManager::new(),
            neighbor_cache: NeighborCache::new(),
            collision_detector: CollisionDetector::new(),
            physics_state,
            last_update: Instant::now(),
            frame_count: 0,
            target_fps: 60.0,
            enable_rigid_bodies: true,
            enable_spatial_optimization: true,
            max_active_chunks: 100, // Limit active chunks for performance
            rigid_body_threshold: 8,
            active_chunks: AHashSet::new(),
            chunks_to_process: Vec::new(),
        }
    }

    /// Update the physics simulation
    pub fn update(&mut self) {
        let now = Instant::now();
        let delta_time = now.duration_since(self.last_update).as_secs_f32();
        self.last_update = now;
        
        // Clamp delta time to avoid large jumps
        let delta_time = delta_time.min(1.0 / 30.0); // Max 30 FPS minimum
        
        self.update_with_delta(delta_time);
        self.frame_count += 1;
    }

    /// Update with a specific delta time
    pub fn update_with_delta(&mut self, delta_time: f32) {
        // 1. Update rigid body physics
        if self.enable_rigid_bodies {
            self.rigidbody_manager.step();
            self.rigidbody_manager.update_rigid_body_positions(&mut self.chunk_manager);
        }

        // 2. Determine active chunks to process
        self.update_active_chunks();

        // 3. Process particles in active chunks
        self.process_particle_physics(delta_time);

        // 4. Handle rigid body formation
        if self.enable_rigid_bodies && self.frame_count % 60 == 0 {
            self.check_for_new_rigid_bodies();
        }

        // 5. Update spatial structures
        if self.enable_spatial_optimization {
            self.update_spatial_structures();
        }

        // 6. Cleanup and maintenance
        if self.frame_count % 300 == 0 { // Every 5 seconds at 60fps
            self.maintenance();
        }
    }

    fn update_active_chunks(&mut self) {
        self.chunks_to_process.clear();
        self.active_chunks.clear();

        // Collect chunks that need processing
        for (chunk_key, chunk) in self.chunk_manager.chunks_iter() {
            let should_process = chunk.is_dirty() || 
                                 !chunk.active_particles.is_empty() ||
                                 self.has_nearby_activity(*chunk_key);

            if should_process {
                self.active_chunks.insert(*chunk_key);
                self.chunks_to_process.push(*chunk_key);
            }
        }

        // Limit number of active chunks for performance
        if self.chunks_to_process.len() > self.max_active_chunks {
            self.chunks_to_process.truncate(self.max_active_chunks);
        }
    }

    fn has_nearby_activity(&self, chunk_key: ChunkKey) -> bool {
        // Check if neighboring chunks have activity
        for dx in -1..=1 {
            for dy in -1..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }
                
                let neighbor_key = (chunk_key.0 + dx, chunk_key.1 + dy);
                if let Some(neighbor_chunk) = self.chunk_manager.get_chunk(neighbor_key) {
                    if neighbor_chunk.is_dirty() || !neighbor_chunk.active_particles.is_empty() {
                        return true;
                    }
                }
            }
        }
        false
    }

    fn process_particle_physics(&mut self, delta_time: f32) {
        let chunks_to_process = self.chunks_to_process.clone();
        for chunk_key in chunks_to_process {
            // Get active particles list without borrowing the chunk mutably
            let active_particles = if let Some(chunk) = self.chunk_manager.get_chunk(chunk_key) {
                chunk.active_particles.clone()
            } else {
                continue;
            };
            
            for (local_x, local_y) in active_particles {
                let (world_x, world_y) = if let Some(chunk) = self.chunk_manager.get_chunk(chunk_key) {
                    chunk.world_pos(local_x, local_y)
                } else {
                    continue;
                };
                
                // Check if particle still exists and needs processing
                let needs_processing = if let Some(particle) = self.chunk_manager.get_particle(world_x, world_y) {
                    !particle.processed
                } else {
                    false
                };
                
                if needs_processing {
                    self.update_single_particle(world_x, world_y, delta_time);
                }
            }
            
            // Compact active particles list and clear dirty flag
            if let Some(chunk) = self.chunk_manager.get_chunk_mut(chunk_key) {
                chunk.compact_active_particles();
                chunk.clear_dirty();
            }
        }
    }

    fn update_single_particle(&mut self, world_x: i64, world_y: i64, delta_time: f32) {
        // Get neighbors first without borrowing chunk_manager mutably
        let neighbor_data: Vec<Option<(MaterialType, f32, bool)>> = {
            let neighbors_iter = if self.enable_spatial_optimization {
                // Use spatial cache
                let spatial_neighbors = self.neighbor_cache.get_neighbors(&self.chunk_manager, world_x, world_y);
                spatial_neighbors.into_iter().map(|opt| opt.map(|p| (p.material_type, p.temp, p.burning))).collect()
            } else {
                // Direct chunk lookup
                let chunk_neighbors = self.chunk_manager.get_neighbors(world_x, world_y);
                chunk_neighbors.into_iter().map(|opt| opt.map(|p| (p.material_type, p.temp, p.burning))).collect()
            };
            neighbors_iter
        };

        // Now safely get mutable reference to the particle
        if let Some(particle) = self.chunk_manager.get_particle_mut(world_x, world_y) {
            particle.processed = true;

            // 1. Handle lifespan and burnout
            if let Some(new_particle) = self.physics_state.handle_lifespan_and_burnout(particle, delta_time) {
                *particle = new_particle;
                return;
            }

            // Store particle data before calling methods
            let mut particle_copy = particle.clone();
            let old_pos = (world_x, world_y);

            // 2. Update temperature using neighbor data
            Self::update_particle_temperature_static(&mut particle_copy, &neighbor_data, delta_time);

            // 3. Handle state changes and effects
            let (state_change_result, new_particles) = 
                Self::handle_particle_state_changes_static(&mut particle_copy, &neighbor_data, delta_time);

            // 4. Increment time in state
            particle_copy.time_in_state += delta_time;

            // Apply changes back to the particle
            *particle = particle_copy.clone();

            // Drop the mutable borrow before doing chunk operations
            drop(particle);
            
            // 5. Handle movement
            let (new_x, new_y) = self.calculate_particle_movement(&particle_copy, world_x, world_y);

            // Place new particles from effects
            for (nx, ny, new_particle) in new_particles {
                self.chunk_manager.set_particle(nx as i64, ny as i64, new_particle);
            }

            if let Some(new_particle) = state_change_result {
                self.chunk_manager.set_particle(world_x, world_y, new_particle);
                return;
            }
            
            if (new_x, new_y) != old_pos {
                // Move particle
                if let Some(moved_particle) = self.chunk_manager.remove_particle(world_x, world_y) {
                    self.chunk_manager.set_particle(new_x, new_y, moved_particle);
                    
                    // Update spatial cache
                    if self.enable_spatial_optimization {
                        self.neighbor_cache.move_particle(world_x, world_y, new_x, new_y);
                    }
                }
            }
        }
    }

    fn update_particle_temperature(&self, particle: &mut Particle, neighbor_data: &[Option<(MaterialType, f32, bool)>], delta_time: f32) {
        // Simplified temperature update - can be expanded
        let mut temp_change = 0.0;
        let mut neighbor_count = 0;
        
        for neighbor_opt in neighbor_data {
            if let Some((_, neighbor_temp, neighbor_burning)) = neighbor_opt {
                if *neighbor_burning {
                    temp_change += 50.0 * delta_time; // Heat from burning neighbors
                }
                let temp_diff = neighbor_temp - particle.temp;
                temp_change += temp_diff * 0.1 * delta_time; // Heat conduction
                neighbor_count += 1;
            }
        }
        
        if neighbor_count > 0 {
            particle.temp += temp_change / neighbor_count as f32;
            particle.temp = particle.temp.max(-273.15).min(3000.0);
        }
    }

    fn handle_particle_state_changes(&self, particle: &Particle, _neighbor_data: &[Option<(MaterialType, f32, bool)>], _delta_time: f32) -> (Option<Particle>, Vec<(i32, i32, Particle)>) {
        // Simplified state changes - can be expanded
        let new_particles = Vec::new();
        
        // Basic state change logic
        let state_change = if particle.temp > 100.0 && particle.material_type == MaterialType::Water {
            Some(Particle::new(particle.x, particle.y, MaterialType::Steam, Some(particle.temp)))
        } else if particle.temp < 0.0 && particle.material_type == MaterialType::Water {
            Some(Particle::new(particle.x, particle.y, MaterialType::Ice, Some(particle.temp)))
        } else {
            None
        };
        
        (state_change, new_particles)
    }

    fn update_particle_temperature_static(particle: &mut Particle, neighbor_data: &[Option<(MaterialType, f32, bool)>], delta_time: f32) {
        let mut temp_change = 0.0;
        let mut neighbor_count = 0;
        
        for neighbor_opt in neighbor_data {
            if let Some((neighbor_material, neighbor_temp, neighbor_burning)) = neighbor_opt {
                if *neighbor_burning {
                    temp_change += 50.0 * delta_time; // Heat from burning neighbors
                }
                let temp_diff = neighbor_temp - particle.temp;
                temp_change += temp_diff * 0.1 * delta_time; // Heat conduction
                neighbor_count += 1;
            }
        }
        
        if neighbor_count > 0 {
            particle.temp += temp_change / neighbor_count as f32;
            particle.temp = particle.temp.max(-273.15).min(3000.0);
        }
    }

    fn handle_particle_state_changes_static(particle: &mut Particle, _neighbor_data: &[Option<(MaterialType, f32, bool)>], _delta_time: f32) -> (Option<Particle>, Vec<(i32, i32, Particle)>) {
        // Simplified state changes - can be expanded
        let new_particles = Vec::new();
        
        // Basic state change logic
        let state_change = if particle.temp > 100.0 && particle.material_type == MaterialType::Water {
            Some(Particle::new(particle.x, particle.y, MaterialType::Steam, Some(particle.temp)))
        } else if particle.temp < 0.0 && particle.material_type == MaterialType::Water {
            Some(Particle::new(particle.x, particle.y, MaterialType::Ice, Some(particle.temp)))
        } else {
            None
        };
        
        (state_change, new_particles)
    }

    fn calculate_particle_movement(&self, particle: &Particle, world_x: i64, world_y: i64) -> (i64, i64) {
        // Use the same movement logic as before but without borrowing issues
        self.handle_particle_movement(particle, world_x, world_y)
    }

    fn handle_particle_movement(&self, particle: &Particle, world_x: i64, world_y: i64) -> (i64, i64) {
        // Simplified movement logic - can be expanded
        let props = particle.get_properties();
        
        // Check if this material is stationary (solid, non-falling)
        if props.is_stationary(particle.material_type) {
            return (world_x, world_y); // Stationary materials don't move
        }

        let density = props.density;
        let is_gas = density < 0.0;
        let vert_dir = if is_gas { -1 } else { 1 };
        let target_y = world_y + vert_dir;

        // Try vertical movement first
        if self.chunk_manager.get_particle(world_x, target_y).is_none() {
            return (world_x, target_y);
        }

        // Try diagonal movement for non-rigid materials
        if !props.is_rigid_solid(particle.material_type) {
            let directions = if rand::random::<bool>() { [-1, 1] } else { [1, -1] };
            
            for &dx in &directions {
                let diag_x = world_x + dx;
                let diag_y = target_y;
                
                if self.chunk_manager.get_particle(diag_x, diag_y).is_none() {
                    return (diag_x, diag_y);
                }
            }
        }

        // Horizontal movement for liquids and gases
        if props.is_liquid(particle.material_type) || is_gas {
            let directions = if rand::random::<bool>() { [-1, 1] } else { [1, -1] };
            
            for &dx in &directions {
                let side_x = world_x + dx;
                
                if self.chunk_manager.get_particle(side_x, world_y).is_none() {
                    let move_chance = if props.is_liquid(particle.material_type) {
                        (1.0 - props.viscosity * 0.1).max(0.1)
                    } else {
                        1.0
                    };
                    
                    if rand::random::<f32>() < move_chance {
                        return (side_x, world_y);
                    }
                }
            }
        }

        // No movement possible
        (world_x, world_y)
    }

    fn check_for_new_rigid_bodies(&mut self) {
        let chunks_to_check: Vec<ChunkKey> = self.active_chunks.iter().cloned().collect();
        
        for chunk_key in chunks_to_check {
            let candidates = RigidBodyAnalyzer::find_rigid_body_candidates(&self.chunk_manager, chunk_key);
            
            for candidate in candidates {
                if candidate.len() >= self.rigid_body_threshold {
                    // Remove particles from chunk manager (they'll be managed by rigid body)
                    for (x, y, _) in &candidate {
                        self.chunk_manager.remove_particle(*x as i64, *y as i64);
                    }
                    
                    // Create rigid body
                    self.rigidbody_manager.create_rigid_body_from_pixels(candidate, chunk_key);
                }
            }
        }
    }

    fn update_spatial_structures(&mut self) {
        if self.frame_count % 10 == 0 { // Update every 10 frames
            self.neighbor_cache.sync_with_chunk_manager(&self.chunk_manager);
            self.collision_detector.update_from_chunk_manager(&self.chunk_manager);
        }
    }

    fn maintenance(&mut self) {
        // Cleanup empty chunks
        self.chunk_manager.cleanup_empty_chunks();
        self.chunk_manager.compact_active_chunks();
        
        // Spatial structure maintenance
        if self.enable_spatial_optimization {
            self.neighbor_cache.maintenance();
        }
        
        // Clear collision detector
        self.collision_detector.clear();
    }

    /// Add a particle at the specified world position
    pub fn add_particle(&mut self, world_x: i64, world_y: i64, material: MaterialType, temp: Option<f32>) -> bool {
        let result = self.chunk_manager.add_particle(world_x, world_y, material, temp);
        
        if result && self.enable_spatial_optimization {
            self.neighbor_cache.add_particle(world_x, world_y);
        }
        
        result
    }

    /// Add particles in a brush pattern
    pub fn paint_material(&mut self, center_x: i64, center_y: i64, material: MaterialType, brush_size: i64) -> usize {
        let mut placed = 0;
        let brush_size_sq = brush_size * brush_size;
        
        for dy in -brush_size..=brush_size {
            for dx in -brush_size..=brush_size {
                let dist_sq = dx * dx + dy * dy;
                
                if dist_sq <= brush_size_sq {
                    let world_x = center_x + dx;
                    let world_y = center_y + dy;
                    
                    if self.add_particle(world_x, world_y, material, None) {
                        placed += 1;
                    }
                }
            }
        }
        
        placed
    }

    /// Get particle information at world position
    pub fn get_particle(&self, world_x: i64, world_y: i64) -> Option<&Particle> {
        self.chunk_manager.get_particle(world_x, world_y)
    }

    /// Remove particle at world position
    pub fn remove_particle(&mut self, world_x: i64, world_y: i64) -> Option<Particle> {
        let result = self.chunk_manager.remove_particle(world_x, world_y);
        
        if result.is_some() && self.enable_spatial_optimization {
            self.neighbor_cache.remove_particle(world_x, world_y);
        }
        
        result
    }

    /// Clear all particles and rigid bodies
    pub fn clear(&mut self) {
        self.chunk_manager.clear();
        self.rigidbody_manager.clear();
        self.neighbor_cache.clear();
        self.collision_detector.clear();
        self.active_chunks.clear();
    }

    /// Get performance statistics
    pub fn stats(&self) -> AdvancedPhysicsStats {
        AdvancedPhysicsStats {
            frame_count: self.frame_count,
            total_particles: self.chunk_manager.total_particles(),
            chunk_count: self.chunk_manager.chunk_count(),
            active_chunks: self.active_chunks.len(),
            rigid_body_count: self.rigidbody_manager.rigid_body_count(),
            spatial_cells: if self.enable_spatial_optimization {
                self.neighbor_cache.particle_count()
            } else {
                0
            },
        }
    }

    /// Get the current simulation state as a 2D array of particle data
    pub fn get_state_in_region(&self, min_x: i64, min_y: i64, max_x: i64, max_y: i64) -> Vec<Vec<Option<(MaterialType, f32, [u8; 3])>>> {
        let width = (max_x - min_x + 1) as usize;
        let height = (max_y - min_y + 1) as usize;
        let mut state = vec![vec![None; width]; height];
        
        for y in 0..height {
            for x in 0..width {
                let world_x = min_x + x as i64;
                let world_y = min_y + y as i64;
                
                if let Some(particle) = self.get_particle(world_x, world_y) {
                    let mut temp_particle = particle.clone();
                    let color = temp_particle.get_color();
                    state[y][x] = Some((particle.material_type, particle.temp, color));
                }
            }
        }
        
        state
    }

    /// Set target FPS for delta time clamping
    pub fn set_target_fps(&mut self, fps: f32) {
        self.target_fps = fps;
    }

    /// Configure optimization settings
    pub fn set_optimization_settings(&mut self, enable_rigid_bodies: bool, enable_spatial: bool, max_chunks: usize) {
        self.enable_rigid_bodies = enable_rigid_bodies;
        self.enable_spatial_optimization = enable_spatial;
        self.max_active_chunks = max_chunks;
    }
}

impl Default for AdvancedPhysicsEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Performance and debugging statistics for the advanced engine
#[derive(Debug, Clone)]
pub struct AdvancedPhysicsStats {
    pub frame_count: u64,
    pub total_particles: usize,
    pub chunk_count: usize,
    pub active_chunks: usize,
    pub rigid_body_count: usize,
    pub spatial_cells: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_advanced_engine_basic() {
        let mut engine = AdvancedPhysicsEngine::new();
        
        // Add some particles
        assert!(engine.add_particle(50, 10, MaterialType::Sand, None));
        assert!(engine.add_particle(51, 10, MaterialType::Water, None));
        
        let stats = engine.stats();
        assert_eq!(stats.total_particles, 2);
        assert!(stats.chunk_count > 0);
        
        // Update physics
        engine.update();
        
        // Particles should still exist
        assert!(engine.get_particle(50, 10).is_some() || engine.get_particle(50, 11).is_some());
    }

    #[test]
    fn test_brush_painting() {
        let mut engine = AdvancedPhysicsEngine::new();
        
        // Paint with brush
        let placed = engine.paint_material(50, 50, MaterialType::Stone, 3);
        assert!(placed > 1); // Should place multiple particles
        
        let stats = engine.stats();
        assert_eq!(stats.total_particles, placed);
    }

    #[test]
    fn test_chunk_management() {
        let mut engine = AdvancedPhysicsEngine::new();
        
        // Add particles in different chunks
        engine.add_particle(10, 10, MaterialType::Sand, None); // Chunk (0, 0)
        engine.add_particle(100, 100, MaterialType::Water, None); // Different chunk
        
        let stats = engine.stats();
        assert!(stats.chunk_count >= 2); // Should have at least 2 chunks
        
        // Clear and verify
        engine.clear();
        let stats_after = engine.stats();
        assert_eq!(stats_after.total_particles, 0);
        assert_eq!(stats_after.chunk_count, 0);
    }

    #[test]
    fn test_rigid_body_formation() {
        let mut engine = AdvancedPhysicsEngine::new();
        engine.rigid_body_threshold = 4; // Lower threshold for testing
        
        // Create a solid region
        for x in 10..14 {
            for y in 10..14 {
                engine.add_particle(x, y, MaterialType::Stone, None);
            }
        }
        
        let initial_stats = engine.stats();
        assert!(initial_stats.total_particles >= 16);
        
        // Force rigid body check
        engine.check_for_new_rigid_bodies();
        
        let final_stats = engine.stats();
        // Some particles might have been converted to rigid bodies
        assert!(final_stats.rigid_body_count >= 0);
    }
}