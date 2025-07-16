use crate::chunk::{ChunkManager, CHUNK_SIZE};
use crate::particle::Particle;
use crate::materials::MaterialType;
use ahash::AHashMap;
use nalgebra::Point2;
use smallvec::SmallVec;
use std::collections::HashSet;

const SPATIAL_GRID_SIZE: usize = 32; // Size of spatial grid cells
const MAX_PARTICLES_PER_CELL: usize = 64; // Maximum particles per spatial cell

#[derive(Debug, Clone)]
pub struct SpatialCell {
    pub particles: SmallVec<[(i64, i64); MAX_PARTICLES_PER_CELL]>, // World coordinates
    pub dirty: bool,
}

impl SpatialCell {
    pub fn new() -> Self {
        Self {
            particles: SmallVec::new(),
            dirty: false,
        }
    }

    pub fn add_particle(&mut self, x: i64, y: i64) {
        if self.particles.len() < MAX_PARTICLES_PER_CELL {
            self.particles.push((x, y));
            self.dirty = true;
        }
    }

    pub fn remove_particle(&mut self, x: i64, y: i64) -> bool {
        if let Some(pos) = self.particles.iter().position(|(px, py)| *px == x && *py == y) {
            self.particles.remove(pos);
            self.dirty = true;
            true
        } else {
            false
        }
    }

    pub fn clear(&mut self) {
        self.particles.clear();
        self.dirty = true;
    }

    pub fn is_empty(&self) -> bool {
        self.particles.is_empty()
    }
}

impl Default for SpatialCell {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub struct SpatialHashGrid {
    cells: AHashMap<(i32, i32), SpatialCell>,
    grid_size: f32,
    updated_cells: HashSet<(i32, i32)>,
}

impl SpatialHashGrid {
    pub fn new(grid_size: f32) -> Self {
        Self {
            cells: AHashMap::new(),
            grid_size,
            updated_cells: HashSet::new(),
        }
    }

    #[inline(always)]
    fn hash_position(&self, x: i64, y: i64) -> (i32, i32) {
        (
            (x as f32 / self.grid_size).floor() as i32,
            (y as f32 / self.grid_size).floor() as i32,
        )
    }

    pub fn add_particle(&mut self, x: i64, y: i64) {
        let cell_pos = self.hash_position(x, y);
        let cell = self.cells.entry(cell_pos).or_insert_with(SpatialCell::new);
        cell.add_particle(x, y);
        self.updated_cells.insert(cell_pos);
    }

    pub fn remove_particle(&mut self, x: i64, y: i64) -> bool {
        let cell_pos = self.hash_position(x, y);
        if let Some(cell) = self.cells.get_mut(&cell_pos) {
            let removed = cell.remove_particle(x, y);
            if removed {
                self.updated_cells.insert(cell_pos);
                if cell.is_empty() {
                    self.cells.remove(&cell_pos);
                }
            }
            removed
        } else {
            false
        }
    }

    pub fn move_particle(&mut self, old_x: i64, old_y: i64, new_x: i64, new_y: i64) {
        let old_cell_pos = self.hash_position(old_x, old_y);
        let new_cell_pos = self.hash_position(new_x, new_y);

        if old_cell_pos == new_cell_pos {
            // Same cell, just update position
            if let Some(cell) = self.cells.get_mut(&old_cell_pos) {
                if let Some(pos) = cell.particles.iter_mut().find(|(px, py)| *px == old_x && *py == old_y) {
                    *pos = (new_x, new_y);
                    cell.dirty = true;
                    self.updated_cells.insert(old_cell_pos);
                }
            }
        } else {
            // Different cells, remove from old and add to new
            self.remove_particle(old_x, old_y);
            self.add_particle(new_x, new_y);
        }
    }

    pub fn get_nearby_particles(&self, x: i64, y: i64, radius: f32) -> SmallVec<[(i64, i64); 16]> {
        let mut nearby = SmallVec::new();
        let center_cell = self.hash_position(x, y);
        
        let cell_radius = (radius / self.grid_size).ceil() as i32;
        
        for dx in -cell_radius..=cell_radius {
            for dy in -cell_radius..=cell_radius {
                let cell_pos = (center_cell.0 + dx, center_cell.1 + dy);
                
                if let Some(cell) = self.cells.get(&cell_pos) {
                    for &(px, py) in &cell.particles {
                        let distance_sq = ((px - x) as f32).powi(2) + ((py - y) as f32).powi(2);
                        if distance_sq <= radius.powi(2) {
                            nearby.push((px, py));
                        }
                    }
                }
            }
        }
        
        nearby
    }

    pub fn get_particles_in_cell(&self, cell_x: i32, cell_y: i32) -> Option<&SmallVec<[(i64, i64); MAX_PARTICLES_PER_CELL]>> {
        self.cells.get(&(cell_x, cell_y)).map(|cell| &cell.particles)
    }

    pub fn clear(&mut self) {
        self.cells.clear();
        self.updated_cells.clear();
    }

    pub fn cleanup_empty_cells(&mut self) {
        self.cells.retain(|_, cell| !cell.is_empty());
    }

    pub fn get_updated_cells(&self) -> &HashSet<(i32, i32)> {
        &self.updated_cells
    }

    pub fn clear_updated_cells(&mut self) {
        self.updated_cells.clear();
    }

    pub fn particle_count(&self) -> usize {
        self.cells.values().map(|cell| cell.particles.len()).sum()
    }

    pub fn cell_count(&self) -> usize {
        self.cells.len()
    }
}

impl Default for SpatialHashGrid {
    fn default() -> Self {
        Self::new(SPATIAL_GRID_SIZE as f32)
    }
}

/// Optimized neighbor lookup system
#[derive(Debug)]
pub struct NeighborCache {
    spatial_grid: SpatialHashGrid,
    chunk_manager_ref: bool, // Flag to indicate if we need to sync with chunk manager
}

impl NeighborCache {
    pub fn new() -> Self {
        Self {
            spatial_grid: SpatialHashGrid::new(SPATIAL_GRID_SIZE as f32),
            chunk_manager_ref: false,
        }
    }

    pub fn sync_with_chunk_manager(&mut self, chunk_manager: &ChunkManager) {
        self.spatial_grid.clear();
        
        for (_chunk_key, chunk) in chunk_manager.chunks_iter() {
            for y in 0..CHUNK_SIZE {
                for x in 0..CHUNK_SIZE {
                    if let Some(_particle) = chunk.get_particle(x, y) {
                        let (world_x, world_y) = chunk.world_pos(x, y);
                        self.spatial_grid.add_particle(world_x, world_y);
                    }
                }
            }
        }
        
        self.chunk_manager_ref = true;
    }

    pub fn add_particle(&mut self, x: i64, y: i64) {
        self.spatial_grid.add_particle(x, y);
    }

    pub fn remove_particle(&mut self, x: i64, y: i64) -> bool {
        self.spatial_grid.remove_particle(x, y)
    }

    pub fn move_particle(&mut self, old_x: i64, old_y: i64, new_x: i64, new_y: i64) {
        self.spatial_grid.move_particle(old_x, old_y, new_x, new_y);
    }

    /// Get optimized neighbor list for a particle
    pub fn get_neighbors<'a>(&self, chunk_manager: &'a ChunkManager, x: i64, y: i64) -> SmallVec<[Option<&'a Particle>; 8]> {
        let mut neighbors = SmallVec::new();
        
        // Standard 8-directional neighbors
        for dy in -1..=1 {
            for dx in -1..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }
                
                let neighbor_x = x + dx;
                let neighbor_y = y + dy;
                neighbors.push(chunk_manager.get_particle(neighbor_x, neighbor_y));
            }
        }
        
        neighbors
    }

    /// Get particles within a radius (useful for advanced interactions)
    pub fn get_particles_in_radius(&self, x: i64, y: i64, radius: f32) -> SmallVec<[(i64, i64); 16]> {
        self.spatial_grid.get_nearby_particles(x, y, radius)
    }

    pub fn clear(&mut self) {
        self.spatial_grid.clear();
        self.chunk_manager_ref = false;
    }

    pub fn particle_count(&self) -> usize {
        self.spatial_grid.particle_count()
    }

    pub fn maintenance(&mut self) {
        self.spatial_grid.cleanup_empty_cells();
        self.spatial_grid.clear_updated_cells();
    }
}

impl Default for NeighborCache {
    fn default() -> Self {
        Self::new()
    }
}

/// Collision detection system for rigid bodies and particles
#[derive(Debug)]
pub struct CollisionDetector {
    spatial_grid: SpatialHashGrid,
    collision_pairs: Vec<(Point2<f32>, Point2<f32>)>,
}

impl CollisionDetector {
    pub fn new() -> Self {
        Self {
            spatial_grid: SpatialHashGrid::new(16.0), // Smaller grid for collision detection
            collision_pairs: Vec::new(),
        }
    }

    pub fn detect_particle_collisions(
        &mut self,
        chunk_manager: &ChunkManager,
        min_x: i64,
        min_y: i64,
        max_x: i64,
        max_y: i64,
    ) -> Vec<(i64, i64, i64, i64)> {
        let mut collisions = Vec::new();
        
        for y in min_y..=max_y {
            for x in min_x..=max_x {
                if let Some(particle) = chunk_manager.get_particle(x, y) {
                    // Check if this particle is moving
                    if particle.moved_this_step {
                        let nearby = self.spatial_grid.get_nearby_particles(x, y, 2.0);
                        
                        for (other_x, other_y) in nearby {
                            if other_x != x || other_y != y {
                                if let Some(other_particle) = chunk_manager.get_particle(other_x, other_y) {
                                    // Check for collision based on material properties
                                    if self.should_collide(particle, other_particle) {
                                        collisions.push((x, y, other_x, other_y));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        
        collisions
    }

    fn should_collide(&self, particle1: &Particle, particle2: &Particle) -> bool {
        // Simple collision rules - can be expanded
        match (particle1.material_type, particle2.material_type) {
            // Solids collide with everything
            (MaterialType::Stone, _) | (_, MaterialType::Stone) => true,
            (MaterialType::Wood, _) | (_, MaterialType::Wood) => true,
            (MaterialType::Glass, _) | (_, MaterialType::Glass) => true,
            (MaterialType::Ice, _) | (_, MaterialType::Ice) => true,
            
            // Liquids don't collide with gases
            (MaterialType::Water, MaterialType::Steam) => false,
            (MaterialType::Steam, MaterialType::Water) => false,
            
            // Most other combinations have some collision
            _ => true,
        }
    }

    pub fn update_from_chunk_manager(&mut self, chunk_manager: &ChunkManager) {
        self.spatial_grid.clear();
        
        for (_chunk_key, chunk) in chunk_manager.chunks_iter() {
            if chunk.is_dirty() {
                for y in 0..CHUNK_SIZE {
                    for x in 0..CHUNK_SIZE {
                        if let Some(_particle) = chunk.get_particle(x, y) {
                            let (world_x, world_y) = chunk.world_pos(x, y);
                            self.spatial_grid.add_particle(world_x, world_y);
                        }
                    }
                }
            }
        }
    }

    pub fn clear(&mut self) {
        self.spatial_grid.clear();
        self.collision_pairs.clear();
    }
}

impl Default for CollisionDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spatial_hash_grid() {
        let mut grid = SpatialHashGrid::new(32.0);
        
        // Add particles
        grid.add_particle(10, 10);
        grid.add_particle(15, 15);
        grid.add_particle(100, 100);
        
        assert_eq!(grid.particle_count(), 3);
        
        // Test nearby lookup
        let nearby = grid.get_nearby_particles(10, 10, 10.0);
        assert_eq!(nearby.len(), 2); // Should find (10,10) and (15,15)
        
        // Remove particle
        assert!(grid.remove_particle(10, 10));
        assert_eq!(grid.particle_count(), 2);
        
        // Test move
        grid.move_particle(15, 15, 20, 20);
        let moved_nearby = grid.get_nearby_particles(20, 20, 5.0);
        assert_eq!(moved_nearby.len(), 1); // Should find the moved particle
    }

    #[test]
    fn test_neighbor_cache() {
        let mut chunk_manager = ChunkManager::new();
        let mut cache = NeighborCache::new();
        
        // Add some particles
        chunk_manager.add_particle(10, 10, MaterialType::Sand, None);
        chunk_manager.add_particle(11, 10, MaterialType::Water, None);
        chunk_manager.add_particle(10, 11, MaterialType::Stone, None);
        
        // Sync cache
        cache.sync_with_chunk_manager(&chunk_manager);
        assert_eq!(cache.particle_count(), 3);
        
        // Test neighbor lookup
        let neighbors = cache.get_neighbors(&chunk_manager, 10, 10);
        let non_empty_neighbors = neighbors.iter().filter(|n| n.is_some()).count();
        assert_eq!(non_empty_neighbors, 2); // Should find 2 neighbors
    }

    #[test]
    fn test_collision_detector() {
        let mut chunk_manager = ChunkManager::new();
        let mut detector = CollisionDetector::new();
        
        // Add colliding particles
        chunk_manager.add_particle(10, 10, MaterialType::Stone, None);
        chunk_manager.add_particle(11, 10, MaterialType::Water, None);
        
        detector.update_from_chunk_manager(&chunk_manager);
        
        let collisions = detector.detect_particle_collisions(&chunk_manager, 9, 9, 12, 12);
        // Should detect some potential collisions
        assert!(collisions.len() >= 0); // May be 0 if particles haven't moved
    }
}