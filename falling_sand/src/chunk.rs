use crate::material::{MaterialInstance, PhysicsType, Color};
use std::ops::{Index, IndexMut};

/// Size of a chunk in pixels
pub const CHUNK_SIZE: u16 = 32;

/// Number of cells in a chunk
pub const CHUNK_AREA: usize = (CHUNK_SIZE as usize) * (CHUNK_SIZE as usize);

/// State of a chunk in the world
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChunkState {
    Inactive, // Not being simulated
    Active,   // Currently being simulated
}

/// Position within a chunk
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChunkPosition {
    x: u16,
    y: u16,
}

impl ChunkPosition {
    /// Create a new position if within chunk bounds
    pub fn new(x: u16, y: u16) -> Option<Self> {
        if x < CHUNK_SIZE && y < CHUNK_SIZE {
            Some(Self { x, y })
        } else {
            None
        }
    }
    
    /// Create a new position without checking bounds
    /// 
    /// # Safety
    /// x and y must be less than CHUNK_SIZE
    pub unsafe fn new_unchecked(x: u16, y: u16) -> Self {
        debug_assert!(x < CHUNK_SIZE && y < CHUNK_SIZE);
        Self { x, y }
    }
    
    pub fn x(&self) -> u16 {
        self.x
    }
    
    pub fn y(&self) -> u16 {
        self.y
    }
    
    /// Convert to linear index in the chunk
    pub fn to_index(&self) -> usize {
        (self.y as usize) * (CHUNK_SIZE as usize) + (self.x as usize)
    }
}

/// Linear index within a chunk
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChunkIndex(usize);

impl ChunkIndex {
    /// Create a new index if within bounds
    pub fn new(index: usize) -> Option<Self> {
        if index < CHUNK_AREA {
            Some(Self(index))
        } else {
            None
        }
    }
    
    /// Get the underlying index
    pub fn value(&self) -> usize {
        self.0
    }
    
    /// Convert to x,y position
    pub fn to_position(&self) -> ChunkPosition {
        let x = (self.0 % (CHUNK_SIZE as usize)) as u16;
        let y = (self.0 / (CHUNK_SIZE as usize)) as u16;
        unsafe { ChunkPosition::new_unchecked(x, y) }
    }
}

impl From<ChunkPosition> for ChunkIndex {
    fn from(pos: ChunkPosition) -> Self {
        Self(pos.to_index())
    }
}

impl<T> Index<ChunkPosition> for [T; CHUNK_AREA] {
    type Output = T;
    
    fn index(&self, pos: ChunkPosition) -> &Self::Output {
        &self[pos.to_index()]
    }
}

impl<T> IndexMut<ChunkPosition> for [T; CHUNK_AREA] {
    fn index_mut(&mut self, pos: ChunkPosition) -> &mut Self::Output {
        let idx = pos.to_index();
        &mut self[idx]
    }
}

/// Represents a section of the world
pub struct Chunk {
    // Position of this chunk in the world
    pub chunk_x: i32,
    pub chunk_y: i32,
    
    // Current state of the chunk
    pub state: ChunkState,
    
    // Materials in the chunk
    pub pixels: Box<[MaterialInstance; CHUNK_AREA]>,
    
    // Flag if the chunk has been modified
    pub dirty: bool,
}

impl Chunk {
    /// Create a new empty chunk
    pub fn new_empty(chunk_x: i32, chunk_y: i32) -> Self {
        // Initialize with air
        let air = MaterialInstance::air();
        let pixels = vec![air; CHUNK_AREA].try_into()
            .expect("Chunk area size incorrect");
            
        Self {
            chunk_x,
            chunk_y,
            state: ChunkState::Inactive,
            pixels,
            dirty: true,
        }
    }
    
    /// Set a pixel at the given position
    pub fn set_pixel(&mut self, pos: ChunkPosition, material: MaterialInstance) {
        self.pixels[pos] = material;
        self.dirty = true;
    }
    
    /// Get a pixel at the given position
    pub fn get_pixel(&self, pos: ChunkPosition) -> &MaterialInstance {
        &self.pixels[pos]
    }
    
    /// Check if any pixels in this chunk need simulating
    pub fn needs_simulation(&self) -> bool {
        self.pixels.iter().any(|pixel| pixel.is_active())
    }
}

/// Helper functions for world coordinates
pub fn world_to_chunk_pos(x: i32, y: i32) -> (i32, i32) {
    // Integer division that rounds down for negative numbers
    let chunk_x = x.div_euclid(CHUNK_SIZE as i32);
    let chunk_y = y.div_euclid(CHUNK_SIZE as i32);
    (chunk_x, chunk_y)
}

pub fn world_to_local_pos(x: i32, y: i32) -> ChunkPosition {
    let local_x = (x.rem_euclid(CHUNK_SIZE as i32)) as u16;
    let local_y = (y.rem_euclid(CHUNK_SIZE as i32)) as u16;
    unsafe { ChunkPosition::new_unchecked(local_x, local_y) }
}

/// Fully resolve a world position into chunk and local position
pub fn world_to_chunk_and_local(x: i32, y: i32) -> ((i32, i32), ChunkPosition) {
    let chunk_pos = world_to_chunk_pos(x, y);
    let local_pos = world_to_local_pos(x, y);
    (chunk_pos, local_pos)
} 