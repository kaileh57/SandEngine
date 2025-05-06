// physics.rs - Movement and physical interactions

use rand::prelude::*;
use crate::constants::*;
use crate::material_properties::MaterialType;
use crate::particle::Particle;

pub struct PhysicsEngine {
    // Store random number generator for physics calculations
    rng: ThreadRng,
}

impl PhysicsEngine {
    pub fn new() -> Self {
        Self {
            rng: rand::thread_rng(),
        }
    }
    
    // Main movement handling for particles
    pub fn handle_movement(
        &mut self, 
        particle: &mut Particle, 
        x: usize, 
        y: usize, 
        get_particle: impl Fn(usize, usize) -> Option<Particle>,
        set_particle: impl FnMut(usize, usize, Particle) -> bool,
        swap_particles: impl FnMut(usize, usize, usize, usize) -> bool,
    ) -> bool {
        let mut set_particle = set_particle;
        let mut swap_particles = swap_particles;
        
        // Skip movement for immovable materials
        if particle.material == MaterialType::Generator {
            particle.processed = true;
            return false;
        }
        
        // Determine movement direction (up for gases, down for everything else)
        let is_gas = particle.material.is_gas();
        let vertical_dir: isize = if is_gas { -1 } else { 1 };
        
        // Update velocity based on material type
        self.update_velocity(particle, vertical_dir);
        
        // Check for splash effect (solids falling onto liquids)
        if !is_gas && vertical_dir == 1 && 
           self.try_splash(particle, x, y, vertical_dir, &get_particle, &mut set_particle) {
            return true;
        }
        
        // Try vertical movement (falling or rising)
        let new_y = if vertical_dir == -1 {
            y.saturating_sub(1)
        } else {
            (y + 1).min(GRID_HEIGHT - 1)
        };
        
        // Check if we can move vertically
        if new_y != y {
            // Try moving based on velocity first
            let new_x = self.calculate_new_x(x, particle.vel_x);
            
            if (new_x != x || new_y != y) && 
               self.try_move_to(particle, x, y, new_x, new_y, &get_particle, &mut set_particle) {
                return true;
            }
            
            // If velocity-based movement failed, try direct vertical movement
            if self.try_move_to(particle, x, y, x, new_y, &get_particle, &mut set_particle) {
                return true;
            }
        }
        
        // For fluids on surfaces or blocked materials, try horizontal spreading
        let is_blocked_below = y < GRID_HEIGHT - 1 && 
                               get_particle(x, y + 1).map_or(false, |p| p.material != MaterialType::Empty);
        
        if (particle.material.is_liquid() || particle.material.is_gas()) && is_blocked_below {
            if self.try_horizontal_spread(particle, x, y, &get_particle, &mut set_particle) {
                return true;
            }
        }
        
        // Try diagonal movement if vertical failed
        if !particle.material.is_rigid_solid() && 
           self.try_diagonal_movement(particle, x, y, vertical_dir, &get_particle, &mut swap_particles) {
            return true;
        }
        
        // For powders, try specific powder piling behavior
        if particle.material.is_powder() && vertical_dir == 1 {
            if self.try_powder_piling(particle, x, y, &get_particle, &mut set_particle) {
                return true;
            }
        }
        
        // Dampen velocity if we couldn't move
        particle.vel_x *= 0.5;
        particle.vel_y *= 0.5;
        
        // We didn't move
        false
    }
    
    // Update velocity based on material type and apply gravity
    fn update_velocity(&mut self, particle: &mut Particle, vertical_dir: isize) {
        // Apply appropriate gravity or updraft
        match particle.material {
            MaterialType::Sand | MaterialType::Coal | MaterialType::Gunpowder | MaterialType::Ash => {
                particle.vel_y += SAND_GRAVITY;
                if particle.vel_y > SAND_MAX_VELOCITY {
                    particle.vel_y = SAND_MAX_VELOCITY;
                }
                
                // Add random jitter to prevent grid alignment
                if self.rng.gen_bool(0.05) {
                    particle.vel_x += (self.rng.gen::<f32>() - 0.5) * 0.15;
                }
                
                // Apply horizontal dampening
                particle.vel_x *= 0.8;
            },
            MaterialType::Water | MaterialType::Acid | MaterialType::Oil => {
                particle.vel_y += WATER_GRAVITY;
                if particle.vel_y > WATER_MAX_VELOCITY {
                    particle.vel_y = WATER_MAX_VELOCITY;
                }
                
                // Water keeps more horizontal momentum
                particle.vel_x *= 0.95;
                
                // Add small random movement for better flow
                if self.rng.gen_bool(0.15) {
                    particle.vel_x += (self.rng.gen::<f32>() - 0.5) * 0.25;
                }
            },
            MaterialType::Lava => {
                particle.vel_y += LAVA_GRAVITY;
                if particle.vel_y > LAVA_MAX_VELOCITY {
                    particle.vel_y = LAVA_MAX_VELOCITY;
                }
                
                // Lava has very high viscosity
                particle.vel_x *= 0.98;
                
                if self.rng.gen_bool(0.05) {
                    particle.vel_x += (self.rng.gen::<f32>() - 0.5) * 0.1;
                }
            },
            MaterialType::Fire => {
                // Fire rises upward
                particle.vel_y -= FIRE_UPDRAFT;
                
                // Apply some randomness for flicker
                particle.vel_x += (self.rng.gen::<f32>() - 0.5) * 0.3;
            },
            MaterialType::Steam | MaterialType::Smoke | MaterialType::ToxicGas => {
                // Gases rise upward
                particle.vel_y -= GAS_UPDRAFT;
                
                // Random movement for gas diffusion
                particle.vel_x += (self.rng.gen::<f32>() - 0.5) * 0.2;
            },
            MaterialType::Stone | MaterialType::Glass => {
                particle.vel_y += STONE_GRAVITY;
                if particle.vel_y > STONE_MAX_VELOCITY {
                    particle.vel_y = STONE_MAX_VELOCITY;
                }
                
                // Stone barely moves horizontally
                particle.vel_x *= 0.7;
            },
            _ => {
                // Default gravity for other materials
                if vertical_dir > 0 {
                    particle.vel_y += GRAVITY;
                    if particle.vel_y > MAX_VELOCITY {
                        particle.vel_y = MAX_VELOCITY;
                    }
                } else {
                    particle.vel_y -= GRAVITY;
                    if particle.vel_y < -MAX_VELOCITY {
                        particle.vel_y = -MAX_VELOCITY;
                    }
                }
            }
        }
    }
    
    // Calculate new x position based on velocity
    fn calculate_new_x(&self, x: usize, vel_x: f32) -> usize {
        let new_x_f = x as f32 + vel_x;
        let new_x = new_x_f.round() as usize;
        new_x.min(GRID_WIDTH - 1)
    }
    
    // Try to splash liquid when solid falls onto it
    fn try_splash(
        &mut self,
        particle: &Particle,
        x: usize,
        y: usize,
        vert_dir: isize,
        get_particle: impl Fn(usize, usize) -> Option<Particle>,
        set_particle: &mut impl FnMut(usize, usize, Particle) -> bool,
    ) -> bool {
        // Only solid particles can cause splashes
        if !particle.material.is_powder() && !particle.material.is_rigid_solid() {
            return false;
        }
        
        // Check if there's liquid below
        if y >= GRID_HEIGHT - 1 {
            return false;
        }
        
        let ny = y + 1; // Always move down for splash check
        
        // Check for liquid below
        if let Some(below) = get_particle(x, ny) {
            if !below.material.is_liquid() {
                return false;
            }
            
            // Choose splash direction randomly
            let splash_dir: isize = if self.rng.gen_bool(0.5) { -1 } else { 1 };
            
            // Try splash left/right
            for &dir in &[splash_dir, -splash_dir] {
                let splash_x = match (x as isize + dir).try_into() {
                    Ok(val) if val < GRID_WIDTH => val,
                    _ => continue,
                };
                
                // Check if splash target is empty
                if let Some(target) = get_particle(splash_x, y) {
                    if target.material != MaterialType::Empty {
                        continue;
                    }
                    
                    // Clone the liquid and move it
                    let mut liquid = below.clone();
                    liquid.moved_this_step = true;
                    liquid.vel_x = dir as f32 * 1.0;
                    liquid.vel_y = -0.5;
                    
                    // Move solid down and liquid sideways
                    let mut solid = particle.clone();
                    solid.moved_this_step = true;
                    
                    if set_particle(splash_x, y, liquid) && set_particle(x, ny, solid) {
                        // Successfully splashed
                        return true;
                    }
                }
            }
        }
        
        false
    }
    
    // Try to move a particle to a new position
    fn try_move_to(
        &mut self,
        particle: &Particle,
        x: usize,
        y: usize,
        new_x: usize,
        new_y: usize,
        get_particle: impl Fn(usize, usize) -> Option<Particle>,
        set_particle: &mut impl FnMut(usize, usize, Particle) -> bool,
    ) -> bool {
        // Check if the target position is valid and empty
        if new_x >= GRID_WIDTH || new_y >= GRID_HEIGHT {
            return false;
        }
        
        if let Some(target) = get_particle(new_x, new_y) {
            if target.material != MaterialType::Empty {
                return false;
            }
            
            // Create a new particle with updated properties
            let mut new_particle = particle.clone();
            new_particle.moved_this_step = true;
            
            // Move the particle
            if set_particle(new_x, new_y, new_particle) {
                return true;
            }
        }
        
        false
    }
    
    // Try horizontal spreading for liquids and gases
    fn try_horizontal_spread(
        &mut self,
        particle: &Particle,
        x: usize,
        y: usize,
        get_particle: impl Fn(usize, usize) -> Option<Particle>,
        set_particle: &mut impl FnMut(usize, usize, Particle) -> bool,
    ) -> bool {
        // Get material properties
        let props = particle.material.get_properties();
        let viscosity = props.viscosity;
        
        // Determine spread direction based on velocity or random
        let spread_dir: isize = if particle.vel_x.abs() > 0.05 {
            if particle.vel_x > 0.0 { 1 } else { -1 }
        } else {
            if self.rng.gen_bool(0.5) { 1 } else { -1 }
        };
        
        // Check both directions, starting with the preferred one
        for &dir in &[spread_dir, -spread_dir] {
            let nx = match (x as isize + dir).try_into() {
                Ok(val) if val < GRID_WIDTH => val,
                _ => continue,
            };
            
            // Check if target is empty
            if let Some(target) = get_particle(nx, y) {
                if target.material != MaterialType::Empty {
                    continue;
                }
                
                // For liquids, consider viscosity
                if particle.material.is_liquid() {
                    let flow_chance = 1.0 - viscosity * 0.1;
                    if self.rng.gen::<f32>() >= flow_chance {
                        continue;
                    }
                }
                
                // Create a new particle with updated properties
                let mut new_particle = particle.clone();
                new_particle.moved_this_step = true;
                new_particle.vel_x = dir as f32 * 0.6;
                new_particle.vel_y = 0.1;
                
                // Move the particle
                if set_particle(nx, y, new_particle) {
                    return true;
                }
            }
        }
        
        false
    }
    
    // Try diagonal movement
    fn try_diagonal_movement(
        &mut self,
        particle: &Particle,
        x: usize,
        y: usize,
        vertical_dir: isize,
        get_particle: impl Fn(usize, usize) -> Option<Particle>,
        swap_particles: &mut impl FnMut(usize, usize, usize, usize) -> bool,
    ) -> bool {
        // Determine preferred diagonal direction based on velocity or random
        let diag_dir: isize = if particle.vel_x.abs() > 0.05 {
            if particle.vel_x > 0.0 { 1 } else { -1 }
        } else {
            if self.rng.gen_bool(0.5) { 1 } else { -1 }
        };
        
        // Check both diagonal directions, starting with the preferred one
        for &dir in &[diag_dir, -diag_dir] {
            let nx = match (x as isize + dir).try_into() {
                Ok(val) if val < GRID_WIDTH => val,
                _ => continue,
            };
            
            let ny = match (y as isize + vertical_dir).try_into() {
                Ok(val) if val < GRID_HEIGHT => val,
                _ => continue,
            };
            
            // Check if diagonal target is empty
            if let Some(diag_target) = get_particle(nx, ny) {
                if diag_target.material != MaterialType::Empty {
                    continue;
                }
                
                // Check if vertical movement is blocked by something proper
                if let Some(vertical_target) = get_particle(x, ny) {
                    if vertical_target.material == MaterialType::Empty {
                        continue; // Not blocked, should try vertical first
                    }
                    
                    // For density-based movement, check if the vertical target is appropriate
                    if particle.material.is_gas() && 
                       !particle.is_lighter_than(vertical_target.material) {
                        continue;
                    }
                    
                    if !particle.material.is_gas() && 
                       !particle.is_heavier_than(vertical_target.material) && 
                       vertical_target.material != MaterialType::Generator {
                        continue;
                    }
                    
                    // Swap particles diagonally
                    if swap_particles(x, y, nx, ny) {
                        return true;
                    }
                }
            }
        }
        
        false
    }
    
    // Try powder piling behavior
    fn try_powder_piling(
        &mut self,
        particle: &Particle,
        x: usize,
        y: usize,
        get_particle: impl Fn(usize, usize) -> Option<Particle>,
        set_particle: &mut impl FnMut(usize, usize, Particle) -> bool,
    ) -> bool {
        // Check if below is occupied
        if y >= GRID_HEIGHT - 1 {
            return false;
        }
        
        let below_y = y + 1;
        
        if let Some(below) = get_particle(x, below_y) {
            if below.material == MaterialType::Empty {
                return false; // Should fall straight down instead
            }
            
            // Determine pile direction based on velocity or random
            let pile_dir: isize = if particle.vel_x.abs() > 0.05 {
                if particle.vel_x > 0.0 { 1 } else { -1 }
            } else {
                if self.rng.gen_bool(0.5) { 1 } else { -1 }
            };
            
            // Check both pile directions, starting with the preferred one
            for &dir in &[pile_dir, -pile_dir] {
                let nx = match (x as isize + dir).try_into() {
                    Ok(val) if val < GRID_WIDTH => val,
                    _ => continue,
                };
                
                // Check if diagonal-down is empty
                if let Some(diag_target) = get_particle(nx, below_y) {
                    if diag_target.material != MaterialType::Empty {
                        continue;
                    }
                    
                    // Create a new particle with updated properties
                    let mut new_particle = particle.clone();
                    new_particle.moved_this_step = true;
                    new_particle.vel_x = dir as f32 * 0.2;
                    new_particle.vel_y = 0.3;
                    
                    // Move the particle
                    if set_particle(nx, below_y, new_particle) {
                        return true;
                    }
                }
            }
        }
        
        false
    }
}