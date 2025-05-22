//! Manages the 2D grid of particles for the simulation.
//!
//! The `Grid` struct stores particles in a flat vector and provides methods
//! to access, modify, and interact with them based on 2D coordinates.
//! It's a fundamental component of the `SimulationEngine`.

use crate::particle::Particle;
use crate::material::MaterialType;

/// Default ambient temperature for new empty particles within the grid, in Celsius.
const AMBIENT_TEMP: f32 = 20.0;

/// Represents the 2D simulation area, composed of a flat vector of `Particle` cells.
///
/// The grid is the primary data structure holding the state of all particles in the simulation.
/// It allows for efficient storage and access to particles.
#[derive(Debug, Clone)] 
pub struct Grid {
    /// Width of the grid in cells. Publicly readable for dimension queries.
    pub width: usize,
    /// Height of the grid in cells. Publicly readable for dimension queries.
    pub height: usize,
    /// A flat vector storing all particles in the grid.
    /// Cells are stored row by row, i.e., the particle at `(x, y)` is at index `y * width + x`.
    /// Publicly accessible for direct iteration, typically for rendering purposes.
    ///
    /// # Example: Iterating for rendering
    /// ```no_run
    /// # use sand_simulation_engine::grid::Grid;
    /// # let grid = Grid::new(10, 10);
    /// for particle_cell in grid.cells.iter() {
    ///     // Access particle_cell.x, particle_cell.y, particle_cell.material_type, etc.
    ///     // If using a mutable particle for get_color:
    ///     // for particle_cell_mut in grid.cells.iter_mut() { ... particle_cell_mut.get_color() ... }
    /// }
    /// ```
    pub cells: Vec<Particle>,
}

impl Grid {
    /// Creates a new grid of the specified width and height.
    ///
    /// The grid is initialized with `EMPTY` particles at `AMBIENT_TEMP`.
    /// Each particle's internal `x` and `y` coordinates are also set.
    ///
    /// # Arguments
    /// * `width` - The desired width of the grid.
    /// * `height` - The desired height of the grid.
    ///
    /// # Returns
    /// A new `Grid` instance.
    pub fn new(width: usize, height: usize) -> Self {
        let mut cells = Vec::with_capacity(width * height);
        for y_coord in 0..height {
            for x_coord in 0..width {
                cells.push(Particle::new(x_coord as i32, y_coord as i32, MaterialType::EMPTY, AMBIENT_TEMP));
            }
        }
        Grid { width, height, cells }
    }

    /// Checks if the given (x, y) coordinates are within the bounds of the grid.
    ///
    /// Coordinates must be non-negative and less than the grid's width and height respectively.
    ///
    /// # Arguments
    /// * `x` - The x-coordinate to check.
    /// * `y` - The y-coordinate to check.
    ///
    /// # Returns
    /// `true` if the coordinates are valid, `false` otherwise.
    pub fn is_valid(&self, x: i32, y: i32) -> bool {
        if x < 0 || y < 0 {
            return false;
        }
        let x_usize = x as usize;
        let y_usize = y as usize;
        x_usize < self.width && y_usize < self.height
    }

    /// Converts 2D (x, y) coordinates to a 1D index for the `cells` vector.
    /// This is a private helper method used internally by other grid operations.
    ///
    /// # Arguments
    /// * `x` - The x-coordinate.
    /// * `y` - The y-coordinate.
    ///
    /// # Returns
    /// `Some(index)` if coordinates are valid, `None` otherwise.
    fn get_index(&self, x: i32, y: i32) -> Option<usize> {
        if self.is_valid(x, y) {
            Some(y as usize * self.width + x as usize)
        } else {
            None
        }
    }

    /// Returns an immutable reference to the particle at the given (x, y) coordinates.
    ///
    /// # Arguments
    /// * `x` - The x-coordinate of the particle.
    /// * `y` - The y-coordinate of the particle.
    ///
    /// # Returns
    /// `Some(&Particle)` if the coordinates are valid and a particle exists, `None` otherwise.
    pub fn get_particle(&self, x: i32, y: i32) -> Option<&Particle> {
        match self.get_index(x, y) {
            Some(index) => self.cells.get(index),
            None => None,
        }
    }

    /// Returns a mutable reference to the particle at the given (x, y) coordinates.
    ///
    /// # Arguments
    /// * `x` - The x-coordinate of the particle.
    /// * `y` - The y-coordinate of the particle.
    ///
    /// # Returns
    /// `Some(&mut Particle)` if the coordinates are valid and a particle exists, `None` otherwise.
    pub fn get_particle_mut(&mut self, x: i32, y: i32) -> Option<&mut Particle> {
        match self.get_index(x, y) {
            Some(index) => self.cells.get_mut(index),
            None => None,
        }
    }

    /// Sets a particle at the given (x, y) coordinates in the grid.
    ///
    /// If the coordinates are valid:
    /// - The provided `particle`'s internal `x` and `y` fields are updated to match the grid position.
    /// - The particle's color cache is invalidated, as its state or environment might change.
    /// - The particle is placed into the grid at the specified location.
    /// If the coordinates are invalid, this function does nothing.
    ///
    /// # Arguments
    /// * `x` - The x-coordinate where the particle should be placed.
    /// * `y` - The y-coordinate where the particle should be placed.
    /// * `particle` - The `Particle` to place in the grid.
    pub fn set_particle(&mut self, x: i32, y: i32, mut particle: Particle) {
        if let Some(index) = self.get_index(x, y) {
            particle.x = x;
            particle.y = y;
            particle.invalidate_color_cache(); 
            self.cells[index] = particle;
        }
    }

    /// Swaps the particles at two given coordinate pairs (`x1`, `y1`) and (`x2`, `y2`).
    ///
    /// If the coordinates are the same or if either pair is invalid, no action is taken.
    /// After a successful swap, the internal `x` and `y` coordinates of both involved particles
    /// are updated to reflect their new positions in the grid. Their color caches are also invalidated
    /// as their rendering might change due to new surroundings or state.
    ///
    /// # Arguments
    /// * `x1`, `y1` - Coordinates of the first particle.
    /// * `x2`, `y2` - Coordinates of the second particle.
    pub fn swap_particles(&mut self, x1: i32, y1: i32, x2: i32, y2: i32) {
        if x1 == x2 && y1 == y2 { // No need to swap if coordinates are identical.
            return; 
        }

        let index1_opt = self.get_index(x1, y1);
        let index2_opt = self.get_index(x2, y2);

        if let (Some(index1), Some(index2)) = (index1_opt, index2_opt) {
            self.cells.swap(index1, index2); // Perform the swap in the underlying vector.

            // Update internal coordinates and invalidate color cache for the particle now at (x1, y1) (originally at (x2,y2)).
            if let Some(p1_now_at_idx1) = self.cells.get_mut(index1) {
                p1_now_at_idx1.x = x1;
                p1_now_at_idx1.y = y1;
                p1_now_at_idx1.invalidate_color_cache();
            }
            
            // Update internal coordinates and invalidate color cache for the particle now at (x2, y2) (originally at (x1,y1)).
            if let Some(p2_now_at_idx2) = self.cells.get_mut(index2) {
                p2_now_at_idx2.x = x2;
                p2_now_at_idx2.y = y2;
                p2_now_at_idx2.invalidate_color_cache();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*; 
    use crate::particle::Particle;
    use crate::material::MaterialType;

    #[test]
    fn test_grid_creation() {
        let grid = Grid::new(10, 5);
        assert_eq!(grid.width, 10);
        assert_eq!(grid.height, 5);
        assert_eq!(grid.cells.len(), 50);
        assert_eq!(grid.get_particle(0,0).unwrap().material_type, MaterialType::EMPTY);
    }

    #[test]
    fn test_grid_set_get() {
        let mut grid = Grid::new(10, 10);
        let p = Particle::new(5, 5, MaterialType::SAND, AMBIENT_TEMP);
        grid.set_particle(5, 5, p); 
        let retrieved = grid.get_particle(5, 5).unwrap();
        assert_eq!(retrieved.material_type, MaterialType::SAND);
        assert_eq!(retrieved.x, 5); 
        assert_eq!(retrieved.y, 5);
    }

    #[test]
    fn test_grid_swap() {
        let mut grid = Grid::new(10, 10);
        let p1 = Particle::new(1, 1, MaterialType::SAND, AMBIENT_TEMP);
        let p2 = Particle::new(2, 2, MaterialType::WATER, AMBIENT_TEMP);
        
        grid.set_particle(1, 1, p1.clone()); 
        grid.set_particle(2, 2, p2.clone());

        grid.swap_particles(1, 1, 2, 2);

        let new_p1_pos = grid.get_particle(2, 2).unwrap(); 
        let new_p2_pos = grid.get_particle(1, 1).unwrap(); 

        assert_eq!(new_p1_pos.material_type, MaterialType::SAND);
        assert_eq!(new_p1_pos.x, 2); 
        assert_eq!(new_p1_pos.y, 2);

        assert_eq!(new_p2_pos.material_type, MaterialType::WATER);
        assert_eq!(new_p2_pos.x, 1);
        assert_eq!(new_p2_pos.y, 1);
    }
}
