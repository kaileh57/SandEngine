// src/grid.rs

use crate::particle::Particle;
use crate::material::MaterialType;

// Assuming AMBIENT_TEMP is a sensible default for initial empty particles.
// This might need to be configurable later.
const AMBIENT_TEMP: f32 = 20.0;

#[derive(Debug, Clone)] // Added Clone for now, might be removed if Grid becomes very large and cloning is expensive.
pub struct Grid {
    pub width: usize,
    pub height: usize,
    pub cells: Vec<Particle>,
}

impl Grid {
    pub fn new(width: usize, height: usize) -> Self {
        let mut cells = Vec::with_capacity(width * height);
        for y_coord in 0..height {
            for x_coord in 0..width {
                cells.push(Particle::new(x_coord as i32, y_coord as i32, MaterialType::EMPTY, AMBIENT_TEMP));
            }
        }
        Grid { width, height, cells }
    }

    pub fn is_valid(&self, x: i32, y: i32) -> bool {
        if x < 0 || y < 0 {
            return false;
        }
        // Cast self.width and self.height to i32 for comparison,
        // or cast x and y to usize after checking they are non-negative.
        // Choosing the latter for direct comparison with usize dimensions.
        let x_usize = x as usize;
        let y_usize = y as usize;
        x_usize < self.width && y_usize < self.height
    }

    // Private helper
    fn get_index(&self, x: i32, y: i32) -> Option<usize> {
        if self.is_valid(x, y) {
            Some(y as usize * self.width + x as usize)
        } else {
            None
        }
    }

    pub fn get_particle(&self, x: i32, y: i32) -> Option<&Particle> {
        match self.get_index(x, y) {
            Some(index) => self.cells.get(index),
            None => None,
        }
    }

    pub fn get_particle_mut(&mut self, x: i32, y: i32) -> Option<&mut Particle> {
        match self.get_index(x, y) {
            Some(index) => self.cells.get_mut(index),
            None => None,
        }
    }

    pub fn set_particle(&mut self, x: i32, y: i32, mut particle: Particle) {
        if let Some(index) = self.get_index(x, y) {
            particle.x = x;
            particle.y = y;
            particle.invalidate_color_cache();
            self.cells[index] = particle;
        }
        // If x, y are invalid, do nothing as per instruction.
    }

    pub fn swap_particles(&mut self, x1: i32, y1: i32, x2: i32, y2: i32) {
        if x1 == x2 && y1 == y2 {
            return; // Same coordinates, nothing to swap
        }

        let index1_opt = self.get_index(x1, y1);
        let index2_opt = self.get_index(x2, y2);

        if let (Some(index1), Some(index2)) = (index1_opt, index2_opt) {
            self.cells.swap(index1, index2);

            // After swapping, the particle that was at (x1,y1) is now at (x2,y2) physically in the Vec,
            // and the particle that was at (x2,y2) is now at (x1,y1) physically in the Vec.
            // We need to update their internal x,y coordinates to reflect their new grid positions.

            // The particle now at index1 is intended for grid cell (x1, y1)
            // The particle now at index2 is intended for grid cell (x2, y2)

            // Update particle now at index1 (which came from index2)
            if let Some(p1) = self.cells.get_mut(index1) {
                p1.x = x1;
                p1.y = y1;
                p1.invalidate_color_cache();
            }
            
            // Update particle now at index2 (which came from index1)
            if let Some(p2) = self.cells.get_mut(index2) {
                p2.x = x2;
                p2.y = y2;
                p2.invalidate_color_cache();
            }
        }
    }
}
