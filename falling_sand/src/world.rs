use std::collections::HashMap;
use rand::Rng;
use crate::{
    material::{MaterialInstance, MaterialRegistry, PhysicsType, Color},
    chunk::{Chunk, ChunkPosition, ChunkState, CHUNK_SIZE, world_to_chunk_pos, world_to_local_pos, world_to_chunk_and_local, CHUNK_AREA},
};

/// Holds information about the entire simulation
pub struct World {
    // Dimensions in pixels
    width: u32,
    height: u32,
    
    // Material registry
    pub materials: MaterialRegistry,
    
    // Active chunks
    chunks: HashMap<(i32, i32), Chunk>,
    
    // RNG for simulation
    rng: rand::rngs::ThreadRng,
}

impl World {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            materials: MaterialRegistry::new(),
            chunks: HashMap::new(),
            rng: rand::thread_rng(),
        }
    }
    
    /// Get a reference to a chunk if it exists
    pub fn get_chunk(&self, chunk_x: i32, chunk_y: i32) -> Option<&Chunk> {
        self.chunks.get(&(chunk_x, chunk_y))
    }
    
    /// Get a mutable reference to a chunk if it exists
    pub fn get_chunk_mut(&mut self, chunk_x: i32, chunk_y: i32) -> Option<&mut Chunk> {
        self.chunks.get_mut(&(chunk_x, chunk_y))
    }
    
    /// Get or create a chunk at the given position
    pub fn get_or_create_chunk(&mut self, chunk_x: i32, chunk_y: i32) -> &mut Chunk {
        if !self.chunks.contains_key(&(chunk_x, chunk_y)) {
            self.chunks.insert((chunk_x, chunk_y), Chunk::new_empty(chunk_x, chunk_y));
        }
        self.chunks.get_mut(&(chunk_x, chunk_y)).unwrap()
    }
    
    /// Get material at a world position
    pub fn get_pixel(&self, x: i32, y: i32) -> Option<&MaterialInstance> {
        let (chunk_pos, local_pos) = world_to_chunk_and_local(x, y);
        self.get_chunk(chunk_pos.0, chunk_pos.1)
            .map(|chunk| chunk.get_pixel(local_pos))
    }
    
    /// Set material at a world position
    pub fn set_pixel(&mut self, x: i32, y: i32, material: MaterialInstance) {
        let (chunk_pos, local_pos) = world_to_chunk_and_local(x, y);
        let chunk = self.get_or_create_chunk(chunk_pos.0, chunk_pos.1);
        chunk.set_pixel(local_pos, material);
        
        // Activate the chunk for simulation
        chunk.state = ChunkState::Active;
    }
    
    /// Create a circle of sand
    pub fn create_sand_circle(&mut self, center_x: i32, center_y: i32, radius: i32) {
        let sand_id = 1; // Assuming sand is at index 1
        
        for dy in -radius..=radius {
            for dx in -radius..=radius {
                let dist_sq = dx*dx + dy*dy;
                if dist_sq <= radius*radius {
                    if let Some(sand) = self.materials.create_instance(sand_id) {
                        self.set_pixel(center_x + dx, center_y + dy, sand);
                    }
                }
            }
        }
    }
    
    /// Create a circle of any material
    pub fn create_material_circle(&mut self, center_x: i32, center_y: i32, radius: i32, material: MaterialInstance) {
        for dy in -radius..=radius {
            for dx in -radius..=radius {
                let dist_sq = dx*dx + dy*dy;
                if dist_sq <= radius*radius {
                    self.set_pixel(center_x + dx, center_y + dy, material.clone());
                }
            }
        }
    }
    
    /// Update the simulation by one step
    pub fn update(&mut self) {
        self.simulate_chunks();
    }
    
    /// Simulate all active chunks
    fn simulate_chunks(&mut self) {
        // Collect all active chunks
        let chunk_keys: Vec<(i32, i32)> = self.chunks
            .iter()
            .filter(|(_, chunk)| chunk.state == ChunkState::Active)
            .map(|(pos, _)| *pos)
            .collect();
            
        // We need to clone the RNG for each chunk
        let mut rng = self.rng.clone();
        
        // Process each chunk
        for key in chunk_keys {
            if let Some(chunk) = self.chunks.get_mut(&key) {
                // Skip if nothing to simulate
                if !chunk.needs_simulation() {
                    continue;
                }
                
                self.simulate_chunk(chunk, &mut rng);
            }
        }
        
        // Update our main RNG
        self.rng = rng;
    }
    
    /// Render the world to a frame buffer
    pub fn render(&self, frame: &mut [u8]) {
        // Clear the frame
        for pixel in frame.chunks_exact_mut(4) {
            pixel.copy_from_slice(&[0, 0, 0, 255]); // Black background
        }
        
        // Render each chunk
        for (_, chunk) in &self.chunks {
            self.render_chunk(chunk, frame);
        }
    }
    
    /// Render a single chunk
    fn render_chunk(&self, chunk: &Chunk, frame: &mut [u8]) {
        let frame_width = self.width as usize;
        
        // Calculate world position of chunk's top-left corner
        let chunk_start_x = chunk.chunk_x * CHUNK_SIZE as i32;
        let chunk_start_y = chunk.chunk_y * CHUNK_SIZE as i32;
        
        // Iterate through chunk pixels
        for y in 0..CHUNK_SIZE {
            for x in 0..CHUNK_SIZE {
                let pixel_idx = (y as usize) * (CHUNK_SIZE as usize) + (x as usize);
                let material = &chunk.pixels[pixel_idx];
                
                // Skip transparent pixels
                if material.color.a == 0 {
                    continue;
                }
                
                // Calculate world coordinates
                let world_x = chunk_start_x + x as i32;
                let world_y = chunk_start_y + y as i32;
                
                // Skip if outside screen
                if world_x < 0 || world_y < 0 || 
                   world_x >= self.width as i32 || world_y >= self.height as i32 {
                    continue;
                }
                
                // Calculate frame buffer position
                let frame_idx = (world_y as usize * frame_width + world_x as usize) * 4;
                
                // Ensure we're within bounds
                if frame_idx + 3 < frame.len() {
                    frame[frame_idx] = material.color.r;
                    frame[frame_idx + 1] = material.color.g;
                    frame[frame_idx + 2] = material.color.b;
                    frame[frame_idx + 3] = material.color.a;
                }
            }
        }
    }
} 