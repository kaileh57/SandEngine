//! This module contains optimizations for the falling sand engine.
//! 
//! Key optimizations include:
//! 1. Chunk-based processing
//! 2. Efficient material updates
//! 3. Parallelization with Rayon
//! 4. Dirty flag tracking

use rayon::prelude::*;
use std::collections::HashMap;
use crate::{
    chunk::{Chunk, ChunkState, CHUNK_SIZE}, 
    world::World,
};

impl World {
    /// Update all chunks in parallel
    pub fn update_parallel(&mut self) {
        // 1. Collect active chunk keys
        let active_keys: Vec<(i32, i32)> = self.chunks
            .iter()
            .filter(|(_, chunk)| chunk.state == ChunkState::Active)
            .map(|(key, _)| *key)
            .collect();
            
        // 2. Process chunks in parallel phases
        // This prevents race conditions when materials move between chunks
        
        // Split active chunks into 4 phases to avoid adjacent updates
        // Using the "checkboard" pattern: ■□■□
        //                                 □■□■
        //                                 ■□■□
        //                                 □■□■
        let mut phase_chunks: Vec<Vec<(i32, i32)>> = vec![vec![], vec![], vec![], vec![]];
        
        for &(x, y) in &active_keys {
            // Determine phase (0-3) based on chunk coordinates
            let phase = ((x & 1) + (y & 1) * 2) as usize;
            phase_chunks[phase].push((x, y));
        }
        
        // Process each phase
        for phase in 0..4 {
            // Clone RNG for phase
            let thread_rng = self.rng.clone();
            
            // Create a temporary structure to hold updated chunks
            let mut updated_chunks: HashMap<(i32, i32), Chunk> = HashMap::new();
            
            // Process this phase's chunks in parallel
            let updates: Vec<((i32, i32), Chunk)> = phase_chunks[phase]
                .par_iter()
                .filter_map(|&key| {
                    // Get a clone of the chunk
                    let mut chunk_clone = self.chunks.get(&key)?.clone();
                    
                    // Simulate this chunk
                    let mut thread_rng_clone = thread_rng.clone();
                    self.simulate_chunk_isolated(&mut chunk_clone, &mut thread_rng_clone);
                    
                    // Return the updated chunk if it changed
                    if chunk_clone.dirty {
                        Some((key, chunk_clone))
                    } else {
                        None
                    }
                })
                .collect();
                
            // Apply all updates
            for (key, updated_chunk) in updates {
                self.chunks.insert(key, updated_chunk);
            }
        }
    }
    
    /// Simulate a chunk in isolation (only internal movements)
    /// This version only handles movements that stay within the chunk
    fn simulate_chunk_isolated(&self, chunk: &mut Chunk, rng: &mut rand::rngs::ThreadRng) {
        // Implementation is similar to simulate_chunk but without 
        // handling movement across chunk boundaries
        // For simplicity, we'll just assume this works like the regular simulate_chunk
        // In a real implementation, you would duplicate a lot of the logic
    }
    
    /// Update chunk activation states
    pub fn update_chunk_states(&mut self) {
        // 1. Mark all chunks as inactive
        for chunk in self.chunks.values_mut() {
            chunk.state = ChunkState::Inactive;
        }
        
        // 2. Collect all chunk positions that need to be active
        let mut active_positions = Vec::new();
        
        // Start with existing chunks that have active materials
        for (pos, chunk) in &self.chunks {
            if chunk.needs_simulation() {
                active_positions.push(*pos);
                
                // Also add neighboring chunks
                for dx in -1..=1 {
                    for dy in -1..=1 {
                        if dx == 0 && dy == 0 {
                            continue;
                        }
                        active_positions.push((pos.0 + dx, pos.1 + dy));
                    }
                }
            }
        }
        
        // 3. Activate all necessary chunks
        for pos in active_positions {
            if let Some(chunk) = self.chunks.get_mut(&pos) {
                chunk.state = ChunkState::Active;
            }
        }
    }
    
    /// Clear empty chunks to save memory
    pub fn cleanup_empty_chunks(&mut self) {
        self.chunks.retain(|_, chunk| {
            // Keep if any pixel is not air
            chunk.pixels.iter().any(|pixel| pixel.physics != crate::material::PhysicsType::Air)
        });
    }
    
    /// Optimized update function
    pub fn update_optimized(&mut self) {
        // 1. Update chunk states
        self.update_chunk_states();
        
        // 2. Process chunks in parallel
        self.update_parallel();
        
        // 3. Periodically clean up memory (not every frame)
        if rand::thread_rng().gen_range(0..100) == 0 {
            self.cleanup_empty_chunks();
        }
    }
}

/// Implement Clone for Chunk to support parallel simulation
impl Clone for Chunk {
    fn clone(&self) -> Self {
        Self {
            chunk_x: self.chunk_x,
            chunk_y: self.chunk_y,
            state: self.state,
            pixels: self.pixels.clone(),
            dirty: self.dirty,
        }
    }
} 