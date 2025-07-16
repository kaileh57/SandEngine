use crate::particle::Particle;
use crate::materials::MaterialType;
use ahash::AHashMap;

// Chunk size - smaller chunks for better performance
pub const CHUNK_SIZE: usize = 64;
pub const CHUNK_AREA: usize = CHUNK_SIZE * CHUNK_SIZE;

pub type ChunkKey = (i32, i32);

#[derive(Debug, Clone)]
pub struct Chunk {
    pub x: i32,
    pub y: i32,
    // Use flat array for better cache performance
    pub particles: Box<[Option<Particle>; CHUNK_AREA]>,
    pub dirty: bool,
    pub active_particles: Vec<(usize, usize)>, // Local coordinates within chunk
    pub settled_particles: usize, // Count of particles that haven't moved
}

impl Chunk {
    pub fn new(x: i32, y: i32) -> Self {
        Self {
            x,
            y,
            particles: Box::new([const { None }; CHUNK_AREA]),
            dirty: false,
            active_particles: Vec::new(),
            settled_particles: 0,
        }
    }

    #[inline(always)]
    pub fn get_index(x: usize, y: usize) -> usize {
        y * CHUNK_SIZE + x
    }

    #[inline(always)]
    pub fn get_particle(&self, x: usize, y: usize) -> Option<&Particle> {
        if x < CHUNK_SIZE && y < CHUNK_SIZE {
            self.particles[Self::get_index(x, y)].as_ref()
        } else {
            None
        }
    }

    #[inline(always)]
    pub fn get_particle_mut(&mut self, x: usize, y: usize) -> Option<&mut Particle> {
        if x < CHUNK_SIZE && y < CHUNK_SIZE {
            self.particles[Self::get_index(x, y)].as_mut()
        } else {
            None
        }
    }

    pub fn set_particle(&mut self, x: usize, y: usize, particle: Particle) -> Option<Particle> {
        if x < CHUNK_SIZE && y < CHUNK_SIZE {
            let index = Self::get_index(x, y);
            let old = self.particles[index].replace(particle);
            
            if old.is_none() {
                // New particle added
                if self.particles[index].as_ref().unwrap().dynamic {
                    self.active_particles.push((x, y));
                }
            }
            
            self.dirty = true;
            old
        } else {
            None
        }
    }

    pub fn remove_particle(&mut self, x: usize, y: usize) -> Option<Particle> {
        if x < CHUNK_SIZE && y < CHUNK_SIZE {
            let index = Self::get_index(x, y);
            let removed = self.particles[index].take();
            
            if removed.is_some() {
                self.dirty = true;
                // Remove from active particles list
                self.active_particles.retain(|(px, py)| *px != x || *py != y);
            }
            
            removed
        } else {
            None
        }
    }

    pub fn is_empty(&self) -> bool {
        self.particles.iter().all(|p| p.is_none())
    }

    pub fn particle_count(&self) -> usize {
        self.particles.iter().filter(|p| p.is_some()).count()
    }

    pub fn clear(&mut self) {
        self.particles.fill(None);
        self.active_particles.clear();
        self.settled_particles = 0;
        self.dirty = true;
    }

    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    pub fn clear_dirty(&mut self) {
        self.dirty = false;
    }

    /// Get world position from chunk position and local coordinates
    pub fn world_pos(&self, local_x: usize, local_y: usize) -> (i64, i64) {
        (
            self.x as i64 * CHUNK_SIZE as i64 + local_x as i64,
            self.y as i64 * CHUNK_SIZE as i64 + local_y as i64,
        )
    }

    /// Compact active particles list by removing settled ones
    pub fn compact_active_particles(&mut self) {
        let mut to_remove = Vec::new();
        
        for (i, (x, y)) in self.active_particles.iter().enumerate() {
            if let Some(particle) = self.get_particle(*x, *y) {
                if !particle.dynamic || particle.settled_frames >= 10 {
                    to_remove.push(i);
                }
            } else {
                to_remove.push(i);
            }
        }
        
        // Remove in reverse order to maintain indices
        for i in to_remove.into_iter().rev() {
            self.active_particles.remove(i);
        }
    }
}

#[derive(Debug)]
pub struct ChunkManager {
    chunks: AHashMap<ChunkKey, Chunk>,
    active_chunks: Vec<ChunkKey>,
    pub chunk_size: usize,
}

impl ChunkManager {
    pub fn new() -> Self {
        Self {
            chunks: AHashMap::new(),
            active_chunks: Vec::new(),
            chunk_size: CHUNK_SIZE,
        }
    }

    pub fn world_to_chunk_pos(world_x: i64, world_y: i64) -> ChunkKey {
        (
            world_x.div_euclid(CHUNK_SIZE as i64) as i32,
            world_y.div_euclid(CHUNK_SIZE as i64) as i32,
        )
    }

    pub fn world_to_local_pos(world_x: i64, world_y: i64) -> (usize, usize) {
        (
            world_x.rem_euclid(CHUNK_SIZE as i64) as usize,
            world_y.rem_euclid(CHUNK_SIZE as i64) as usize,
        )
    }

    pub fn get_chunk(&self, chunk_key: ChunkKey) -> Option<&Chunk> {
        self.chunks.get(&chunk_key)
    }

    pub fn get_chunk_mut(&mut self, chunk_key: ChunkKey) -> Option<&mut Chunk> {
        self.chunks.get_mut(&chunk_key)
    }

    pub fn get_or_create_chunk(&mut self, chunk_key: ChunkKey) -> &mut Chunk {
        self.chunks.entry(chunk_key).or_insert_with(|| {
            let chunk = Chunk::new(chunk_key.0, chunk_key.1);
            self.active_chunks.push(chunk_key);
            chunk
        })
    }

    pub fn get_particle(&self, world_x: i64, world_y: i64) -> Option<&Particle> {
        let chunk_key = Self::world_to_chunk_pos(world_x, world_y);
        let (local_x, local_y) = Self::world_to_local_pos(world_x, world_y);
        
        self.get_chunk(chunk_key)?.get_particle(local_x, local_y)
    }

    pub fn get_particle_mut(&mut self, world_x: i64, world_y: i64) -> Option<&mut Particle> {
        let chunk_key = Self::world_to_chunk_pos(world_x, world_y);
        let (local_x, local_y) = Self::world_to_local_pos(world_x, world_y);
        
        self.get_chunk_mut(chunk_key)?.get_particle_mut(local_x, local_y)
    }

    pub fn set_particle(&mut self, world_x: i64, world_y: i64, particle: Particle) -> Option<Particle> {
        let chunk_key = Self::world_to_chunk_pos(world_x, world_y);
        let (local_x, local_y) = Self::world_to_local_pos(world_x, world_y);
        
        self.get_or_create_chunk(chunk_key).set_particle(local_x, local_y, particle)
    }

    pub fn remove_particle(&mut self, world_x: i64, world_y: i64) -> Option<Particle> {
        let chunk_key = Self::world_to_chunk_pos(world_x, world_y);
        let (local_x, local_y) = Self::world_to_local_pos(world_x, world_y);
        
        self.get_chunk_mut(chunk_key)?.remove_particle(local_x, local_y)
    }

    pub fn add_particle(&mut self, world_x: i64, world_y: i64, material_type: MaterialType, temp: Option<f32>) -> bool {
        let chunk_key = Self::world_to_chunk_pos(world_x, world_y);
        let (local_x, local_y) = Self::world_to_local_pos(world_x, world_y);
        
        // Check if we can place here
        if let Some(chunk) = self.get_chunk(chunk_key) {
            if let Some(existing) = chunk.get_particle(local_x, local_y) {
                if existing.material_type == MaterialType::Generator && material_type != MaterialType::Eraser {
                    return false; // Can't overwrite generators unless erasing
                }
            }
        }
        
        if material_type == MaterialType::Eraser {
            self.remove_particle(world_x, world_y);
        } else {
            let initial_temp = match material_type {
                MaterialType::Lava => Some(2500.0),
                _ => temp,
            };
            let particle = Particle::new(world_x as usize, world_y as usize, material_type, initial_temp);
            self.set_particle(world_x, world_y, particle);
        }
        
        true
    }

    pub fn get_neighbors(&self, world_x: i64, world_y: i64) -> Vec<Option<&Particle>> {
        let mut neighbors = Vec::with_capacity(8);
        
        for dy in -1..=1 {
            for dx in -1..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }
                
                let neighbor_x = world_x + dx;
                let neighbor_y = world_y + dy;
                neighbors.push(self.get_particle(neighbor_x, neighbor_y));
            }
        }
        
        neighbors
    }

    pub fn get_active_chunks(&self) -> &[ChunkKey] {
        &self.active_chunks
    }

    pub fn get_active_chunks_mut(&mut self) -> &mut Vec<ChunkKey> {
        &mut self.active_chunks
    }

    pub fn cleanup_empty_chunks(&mut self) {
        let mut to_remove = Vec::new();
        
        for (key, chunk) in &self.chunks {
            if chunk.is_empty() {
                to_remove.push(*key);
            }
        }
        
        for key in to_remove {
            self.chunks.remove(&key);
            self.active_chunks.retain(|k| *k != key);
        }
    }

    pub fn compact_active_chunks(&mut self) {
        // Remove chunks that are no longer active
        self.active_chunks.retain(|key| {
            if let Some(chunk) = self.chunks.get_mut(key) {
                chunk.compact_active_particles();
                !chunk.is_empty() && (chunk.is_dirty() || chunk.active_particles.len() > 0)
            } else {
                false
            }
        });
    }

    pub fn total_particles(&self) -> usize {
        self.chunks.values().map(|c| c.particle_count()).sum()
    }

    pub fn chunk_count(&self) -> usize {
        self.chunks.len()
    }

    pub fn clear_chunk(&mut self, chunk_key: ChunkKey) {
        if let Some(chunk) = self.get_chunk_mut(chunk_key) {
            chunk.clear();
        }
    }

    pub fn clear(&mut self) {
        self.chunks.clear();
        self.active_chunks.clear();
    }

    pub fn chunks_iter(&self) -> impl Iterator<Item = (&ChunkKey, &Chunk)> {
        self.chunks.iter()
    }

    pub fn chunks_iter_mut(&mut self) -> impl Iterator<Item = (&ChunkKey, &mut Chunk)> {
        self.chunks.iter_mut()
    }
}

impl Default for ChunkManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::materials::MaterialType;

    #[test]
    fn test_chunk_coordinate_conversion() {
        // Test world to chunk position conversion
        assert_eq!(ChunkManager::world_to_chunk_pos(0, 0), (0, 0));
        assert_eq!(ChunkManager::world_to_chunk_pos(63, 63), (0, 0));
        assert_eq!(ChunkManager::world_to_chunk_pos(64, 64), (1, 1));
        assert_eq!(ChunkManager::world_to_chunk_pos(-1, -1), (-1, -1));
        assert_eq!(ChunkManager::world_to_chunk_pos(-64, -64), (-1, -1));
        assert_eq!(ChunkManager::world_to_chunk_pos(-65, -65), (-2, -2));
        
        // Test world to local position conversion
        assert_eq!(ChunkManager::world_to_local_pos(0, 0), (0, 0));
        assert_eq!(ChunkManager::world_to_local_pos(63, 63), (63, 63));
        assert_eq!(ChunkManager::world_to_local_pos(64, 64), (0, 0));
        assert_eq!(ChunkManager::world_to_local_pos(-1, -1), (63, 63));
    }

    #[test]
    fn test_chunk_particle_operations() {
        let mut manager = ChunkManager::new();
        
        // Add particle
        assert!(manager.add_particle(10, 10, MaterialType::Sand, None));
        assert!(manager.get_particle(10, 10).is_some());
        
        // Remove particle
        assert!(manager.remove_particle(10, 10).is_some());
        assert!(manager.get_particle(10, 10).is_none());
        
        // Test chunk boundaries
        assert!(manager.add_particle(63, 63, MaterialType::Water, None));
        assert!(manager.add_particle(64, 64, MaterialType::Water, None));
        
        // Should be in different chunks
        let chunk1 = manager.get_chunk((0, 0)).unwrap();
        let chunk2 = manager.get_chunk((1, 1)).unwrap();
        
        assert!(chunk1.get_particle(63, 63).is_some());
        assert!(chunk2.get_particle(0, 0).is_some());
    }

    #[test]
    fn test_chunk_manager_performance() {
        let mut manager = ChunkManager::new();
        
        // Add many particles across multiple chunks
        for x in 0..200 {
            for y in 0..200 {
                if (x + y) % 3 == 0 {
                    manager.add_particle(x, y, MaterialType::Sand, None);
                }
            }
        }
        
        assert!(manager.total_particles() > 0);
        assert!(manager.chunk_count() > 1);
        
        // Test cleanup
        manager.cleanup_empty_chunks();
        assert!(manager.chunk_count() > 0); // Should still have chunks with particles
    }
}