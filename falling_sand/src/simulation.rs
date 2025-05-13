use rand::Rng;
use crate::{
    material::{MaterialInstance, PhysicsType},
    chunk::{Chunk, ChunkPosition, CHUNK_SIZE},
    world::World,
};

impl World {
    /// Add simulation method to our World implementation
    pub fn simulate_chunk(&mut self, chunk: &mut Chunk, rng: &mut rand::rngs::ThreadRng) {
        // We need to process pixels in a specific order depending on movement direction
        // For falling sand, we process from bottom to top to prevent cascading updates
        
        // Create a vector of indices to process
        let mut indices: Vec<usize> = (0..CHUNK_SIZE as usize * CHUNK_SIZE as usize).collect();
        
        // For downward movement, process bottom to top
        indices.sort_by_key(|&idx| {
            let y = idx / CHUNK_SIZE as usize;
            CHUNK_SIZE as usize - y // Reverse y to go bottom to top
        });
        
        // Process each pixel
        for idx in indices {
            let x = (idx % CHUNK_SIZE as usize) as u16;
            let y = (idx / CHUNK_SIZE as usize) as u16;
            
            // Skip if this position is air or non-movable
            if !is_active_material(&chunk.pixels[idx]) {
                continue;
            }
            
            // Get a safe position that's guaranteed to be within chunk bounds
            let pos = unsafe { ChunkPosition::new_unchecked(x, y) };
            
            // Process based on material type
            match chunk.pixels[pos].physics {
                PhysicsType::Sand => self.simulate_sand(chunk, pos, rng),
                _ => {} // Other material types aren't implemented yet
            }
        }
    }
    
    /// Simulate a sand particle
    fn simulate_sand(&mut self, chunk: &mut Chunk, pos: ChunkPosition, rng: &mut rand::rngs::ThreadRng) {
        // Get world coordinates
        let world_x = chunk.chunk_x * CHUNK_SIZE as i32 + pos.x() as i32;
        let world_y = chunk.chunk_y * CHUNK_SIZE as i32 + pos.y() as i32;
        
        // Check if can move down
        if self.can_move_to(world_x, world_y + 1) {
            // Move straight down
            self.move_material(world_x, world_y, world_x, world_y + 1);
        } 
        // Check if can move diagonally down
        else {
            // Randomly choose left or right first
            let try_left_first = rng.gen_bool(0.5);
            
            if try_left_first {
                // Try left-down first, then right-down
                if self.can_move_to(world_x - 1, world_y + 1) {
                    self.move_material(world_x, world_y, world_x - 1, world_y + 1);
                } else if self.can_move_to(world_x + 1, world_y + 1) {
                    self.move_material(world_x, world_y, world_x + 1, world_y + 1);
                }
            } else {
                // Try right-down first, then left-down
                if self.can_move_to(world_x + 1, world_y + 1) {
                    self.move_material(world_x, world_y, world_x + 1, world_y + 1);
                } else if self.can_move_to(world_x - 1, world_y + 1) {
                    self.move_material(world_x, world_y, world_x - 1, world_y + 1);
                }
            }
        }
    }
    
    /// Check if a material can move to a specific position
    fn can_move_to(&self, x: i32, y: i32) -> bool {
        // Get the material at target position
        if let Some(material) = self.get_pixel(x, y) {
            material.physics == PhysicsType::Air
        } else {
            // If position is outside the known chunks, treat it as valid
            // This allows particles to "fall off" the known world
            true
        }
    }
    
    /// Move a material from one position to another
    fn move_material(&mut self, from_x: i32, from_y: i32, to_x: i32, to_y: i32) {
        // Get material at source
        let material = if let Some(mat) = self.get_pixel(from_x, from_y).cloned() {
            mat
        } else {
            return; // Source doesn't exist
        };
        
        // Set air at source
        self.set_pixel(from_x, from_y, MaterialInstance::air());
        
        // Set material at destination
        self.set_pixel(to_x, to_y, material);
    }
}

/// Helper to check if a material needs to be simulated
fn is_active_material(material: &MaterialInstance) -> bool {
    match material.physics {
        PhysicsType::Air => false,
        PhysicsType::Sand => true,
        PhysicsType::Solid => false,
    }
} 