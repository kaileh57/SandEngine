use crate::{Simulation, MaterialType, Particle};
use std::time::{Duration, Instant};

/// A game engine-style physics server for particle simulation
pub struct PhysicsEngine {
    pub simulation: Simulation,
    last_update: Instant,
    frame_count: u64,
    target_fps: f32,
}

impl PhysicsEngine {
    /// Create a new physics engine with specified grid dimensions
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            simulation: Simulation::new(width, height),
            last_update: Instant::now(),
            frame_count: 0,
            target_fps: 60.0,
        }
    }

    /// Update the physics simulation
    pub fn update(&mut self) {
        let now = Instant::now();
        let delta_time = now.duration_since(self.last_update).as_secs_f32();
        self.last_update = now;
        
        // Clamp delta time to avoid large jumps
        let delta_time = delta_time.min(1.0 / 30.0); // Max 30 FPS minimum
        
        self.simulation.update(delta_time);
        self.frame_count += 1;
    }

    /// Update with a specific delta time (useful for fixed timestep)
    pub fn update_with_delta(&mut self, delta_time: f32) {
        self.simulation.update(delta_time);
        self.frame_count += 1;
    }

    /// Add a particle at the specified position
    pub fn add_particle(&mut self, x: usize, y: usize, material: MaterialType, temp: Option<f32>) -> bool {
        self.simulation.add_particle(x, y, material, temp)
    }

    /// Add particles in a brush pattern
    pub fn paint_material(&mut self, x: usize, y: usize, material: MaterialType, brush_size: usize) -> usize {
        let start_x = x.saturating_sub(brush_size);
        let end_x = (x + brush_size).min(self.simulation.width.saturating_sub(1));
        let start_y = y.saturating_sub(brush_size);
        let end_y = (y + brush_size).min(self.simulation.height.saturating_sub(1));
        let brush_size_sq = brush_size * brush_size;
        
        let mut placed = 0;
        for px in start_x..=end_x {
            for py in start_y..=end_y {
                let dx = px as i32 - x as i32;
                let dy = py as i32 - y as i32;
                let dist_sq = (dx * dx + dy * dy) as usize;
                
                if dist_sq <= brush_size_sq {
                    if self.simulation.add_particle(px, py, material, None) {
                        placed += 1;
                    }
                }
            }
        }
        placed
    }

    /// Get particle information at position
    pub fn get_particle(&self, x: usize, y: usize) -> Option<&Particle> {
        self.simulation.get_particle(x, y)
    }

    /// Get particle data (type, temp, life, burning) at position
    pub fn get_particle_data(&self, x: usize, y: usize) -> Option<(MaterialType, f32, Option<f32>, bool)> {
        self.simulation.get_particle_data(x, y)
    }

    /// Remove particle at position
    pub fn remove_particle(&mut self, x: usize, y: usize) -> Option<Particle> {
        self.simulation.remove_particle(x, y)
    }

    /// Clear all particles
    pub fn clear(&mut self) {
        self.simulation.clear();
    }

    /// Get the current simulation state as a 2D array of particle data
    pub fn get_state(&self) -> Vec<Vec<Option<(MaterialType, f32, [u8; 3])>>> {
        let mut state = Vec::with_capacity(self.simulation.height);
        
        for y in 0..self.simulation.height {
            let mut row = Vec::with_capacity(self.simulation.width);
            for x in 0..self.simulation.width {
                if let Some(particle) = self.simulation.get_particle(x, y) {
                    let mut temp_particle = particle.clone();
                    let color = temp_particle.get_color();
                    row.push(Some((particle.material_type, particle.temp, color)));
                } else {
                    row.push(None);
                }
            }
            state.push(row);
        }
        
        state
    }

    /// Get just the material types as a 2D array (useful for rendering optimization)
    pub fn get_material_grid(&self) -> Vec<Vec<Option<MaterialType>>> {
        let mut grid = Vec::with_capacity(self.simulation.height);
        
        for y in 0..self.simulation.height {
            let mut row = Vec::with_capacity(self.simulation.width);
            for x in 0..self.simulation.width {
                if let Some(particle) = self.simulation.get_particle(x, y) {
                    row.push(Some(particle.material_type));
                } else {
                    row.push(None);
                }
            }
            grid.push(row);
        }
        
        grid
    }

    /// Count active particles
    pub fn particle_count(&self) -> usize {
        let mut count = 0;
        for y in 0..self.simulation.height {
            for x in 0..self.simulation.width {
                if self.simulation.get_particle(x, y).is_some() {
                    count += 1;
                }
            }
        }
        count
    }

    /// Get grid dimensions
    pub fn dimensions(&self) -> (usize, usize) {
        (self.simulation.width, self.simulation.height)
    }

    /// Get performance statistics
    pub fn stats(&self) -> PhysicsStats {
        PhysicsStats {
            frame_count: self.frame_count,
            particle_count: self.particle_count(),
            grid_size: (self.simulation.width, self.simulation.height),
        }
    }

    /// Set target FPS for delta time clamping
    pub fn set_target_fps(&mut self, fps: f32) {
        self.target_fps = fps;
    }
}

/// Performance and debugging statistics
#[derive(Debug, Clone)]
pub struct PhysicsStats {
    pub frame_count: u64,
    pub particle_count: usize,
    pub grid_size: (usize, usize),
}

/// Example usage patterns for game integration
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_physics_engine_basic() {
        let mut engine = PhysicsEngine::new(100, 100);
        
        // Add some sand
        assert!(engine.add_particle(50, 10, MaterialType::Sand, None));
        assert_eq!(engine.particle_count(), 1);
        
        // Update physics
        engine.update();
        
        // Sand should have moved down
        assert!(engine.get_particle(50, 10).is_none());
        assert!(engine.get_particle(50, 11).is_some());
    }

    #[test]
    fn test_brush_painting() {
        let mut engine = PhysicsEngine::new(100, 100);
        
        // Paint with brush
        let placed = engine.paint_material(50, 50, MaterialType::Water, 3);
        assert!(placed > 1); // Should place multiple particles
        
        let stats = engine.stats();
        assert_eq!(stats.particle_count, placed);
    }
}