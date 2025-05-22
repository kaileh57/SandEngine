// In sand_simulation_engine/src/simulation_engine.rs
use crate::grid::Grid;
use crate::particle::Particle;
use crate::material::{MaterialType, is_liquid, is_powder, is_rigid_solid, get_material_properties};
use rand::seq::SliceRandom; // For shuffling
use rand::thread_rng; // For shuffling
use rand::Rng; // For random numbers in handle_movement

// Constants
const AMBIENT_TEMP: f32 = 20.0;
const FUSE_BURN_LIFESPAN_SEC: f32 = 4.0;
const TARGET_DT_SCALING: f32 = 60.0; // For rate scaling based on 60fps baseline
const MAX_TEMP: f32 = 3000.0; // Max simulation temperature

#[derive(Debug, Clone)]
pub struct SimulationEngine {
    pub grid: Grid,
    pub shuffled_column_order: Vec<usize>,
}

impl SimulationEngine {
    pub fn new(width: usize, height: usize) -> Self {
        let grid = Grid::new(width, height);
        let mut shuffled_column_order: Vec<usize> = (0..width).collect();
        shuffled_column_order.shuffle(&mut thread_rng()); // Initial shuffle

        SimulationEngine {
            grid,
            shuffled_column_order,
        }
    }

    pub fn update(&mut self, delta_time_seconds: f32) {
        // 1. Reset processed_this_step and moved_this_step for all particles
        for y_idx in 0..self.grid.height {
            for x_idx in 0..self.grid.width {
                if let Some(particle) = self.grid.get_particle_mut(x_idx as i32, y_idx as i32) {
                    particle.processed_this_step = false;
                    particle.moved_this_step = false;
                }
            }
        }

        // 2. Shuffle column processing order
        self.shuffled_column_order.shuffle(&mut thread_rng());

        // 3. Process particles bottom-up, using shuffled column order
        for y_coord in (0..self.grid.height).rev() { // y from height-1 down to 0
            for &x_coord_usize in &self.shuffled_column_order {
                let x_coord = x_coord_usize as i32;
                
                let particle_exists_and_needs_processing = {
                    if let Some(p) = self.grid.get_particle(x_coord, y_coord as i32) {
                        p.material_type != MaterialType::EMPTY && !p.processed_this_step
                    } else {
                        false
                    }
                };

                if particle_exists_and_needs_processing {
                    self.update_particle(x_coord, y_coord as i32, delta_time_seconds);
                }
            }
        }
    }
    
    fn handle_lifespan_and_burnout(&mut self, x: i32, y: i32, delta_time_seconds: f32) {
        let (
            needs_update,
            is_fuse_to_ignite, // True if FUSE is burning and needs its lifespan timer started
            initial_type,
            initial_temp,      // To calculate replacement particle's temperature
            mut current_life_opt // Option<f32>, current lifespan of the particle
        ) = {
            // Immutable borrow phase: read properties to decide action
            if let Some(particle) = self.grid.get_particle(x, y) {
                if particle.material_type == MaterialType::EMPTY { return; } // Nothing to process

                let mut needs_check_flag = false;
                let mut fuse_ignition_flag = false;

                if particle.material_type == MaterialType::FUSE && particle.burning {
                    needs_check_flag = true;
                    if particle.life_remaining_seconds.is_none() {
                        fuse_ignition_flag = true; // Fuse needs its lifespan timer started
                    }
                } else if particle.life_remaining_seconds.is_some() {
                    // Any other particle with a defined lifespan
                    needs_check_flag = true;
                }
                
                (
                    needs_check_flag,
                    fuse_ignition_flag,
                    particle.material_type,
                    particle.temperature,
                    particle.life_remaining_seconds
                )
            } else {
                return; // No particle at (x,y)
            }
        };

        if !needs_update {
            return; // No lifespan processing needed for this particle
        }

        // If a burning FUSE needs its timer started, set its initial lifespan
        if is_fuse_to_ignite {
            current_life_opt = Some(FUSE_BURN_LIFESPAN_SEC);
        }
        
        if let Some(mut current_life) = current_life_opt {
            current_life -= delta_time_seconds;

            if current_life <= 0.0 {
                // Particle burns out, determine replacement type and temperature
                let (base_replace_type, base_replace_temp) = match initial_type {
                    MaterialType::FIRE => (MaterialType::SMOKE, (initial_temp * 0.6).min(400.0)),
                    MaterialType::FUSE => (MaterialType::ASH, (initial_temp * 0.5).max(AMBIENT_TEMP)),
                    MaterialType::STEAM | MaterialType::SMOKE | MaterialType::TOXIC_GAS => (MaterialType::EMPTY, AMBIENT_TEMP),
                    _ => (initial_type, initial_temp), // Default for other types with lifespan
                };
                
                let final_replace_type = match initial_type {
                    MaterialType::FIRE | MaterialType::FUSE | MaterialType::STEAM | MaterialType::SMOKE | MaterialType::TOXIC_GAS => base_replace_type,
                    _ => { 
                        if get_material_properties(initial_type).lifespan_seconds.is_some() {
                            MaterialType::EMPTY
                        } else {
                            // This case should ideally not be reached if needs_update was true due to lifespan_seconds.is_some()
                            initial_type 
                        }
                    }
                };
                let final_replace_temp = if final_replace_type == MaterialType::EMPTY { AMBIENT_TEMP } else { base_replace_temp };

                self.grid.set_particle(x, y, Particle::new(x, y, final_replace_type, final_replace_temp));
            } else {
                // Particle still alive, update its state (lifespan, and temp for FUSE)
                if let Some(particle_mut) = self.grid.get_particle_mut(x, y) {
                    particle_mut.life_remaining_seconds = Some(current_life);
                    
                    if particle_mut.material_type == MaterialType::FUSE && particle_mut.burning {
                        // Note: JS uses particle.getProperties()[3] for melt_temp, which is melt_temperature
                        // In our MaterialProperties, melt_temperature is Some(f32) or None.
                        // We'll use MAX_TEMP as a fallback if melt_temperature is None, though FUSE should have one.
                        let fuse_melt_temp = get_material_properties(MaterialType::FUSE).melt_temperature.unwrap_or(MAX_TEMP);
                        particle_mut.temperature = (particle_mut.temperature + 5.0 * delta_time_seconds * TARGET_DT_SCALING)
                                                   .min(fuse_melt_temp); // Clamp to its own melt temp or MAX_TEMP
                        particle_mut.invalidate_color_cache();
                    } else if get_material_properties(particle_mut.material_type).lifespan_seconds.is_some() {
                        // Other particles with lifespan might change color (e.g. smoke fading)
                        particle_mut.invalidate_color_cache();
                    }
                }
            }
        }
    }

    fn update_particle(&mut self, x: i32, y: i32, delta_time_seconds: f32) {
        // Initial guard: ensure particle exists and is marked as processed for this tick.
        if let Some(p) = self.grid.get_particle_mut(x, y) {
            if p.processed_this_step { return; }
            p.processed_this_step = true; // Mark as processed for the current update cycle
        } else {
            return; // No particle at (x,y) or failed to get mutable borrow
        }

        // 1. Handle Lifespan & Burnout
        self.handle_lifespan_and_burnout(x, y, delta_time_seconds);

        // After lifespan/burnout, the particle at (x,y) might have changed or become EMPTY.
        // Re-check before proceeding.
        let particle_material_type_after_lifespan = {
            if let Some(p_after_lifespan) = self.grid.get_particle(x, y) {
                p_after_lifespan.material_type
            } else {
                // This implies the grid coordinates became invalid, which shouldn't happen.
                // Or, the particle was so catastrophically destroyed it's None (not planned).
                // For safety, assume processing for this cell stops.
                return; 
            }
        };

        if particle_material_type_after_lifespan == MaterialType::EMPTY {
            return; // Particle burned out or was removed, no further processing needed for this cell this tick
        }
        
        // TODO: 2. Update Temperature (Phase 2)
        // self.update_temperature(x, y, delta_time_seconds);
        // Re-check particle type/existence if temperature can change it to EMPTY (e.g. extreme heat)

        // TODO: 3. Handle State Changes & Special Effects (Phase 2)
        // self.handle_state_changes_and_effects(x, y, delta_time_seconds);
        // Re-check particle type/existence

        // 4. Increment timeInState (for particles that persist)
        // This must be done *before* movement, as movement might swap this particle out.
        // But *after* state changes that might alter its type or make it EMPTY.
        if let Some(p) = self.grid.get_particle_mut(x, y) {
           if p.material_type != MaterialType::EMPTY { // Check again, as TODOs above could change it
               p.time_in_state_seconds += delta_time_seconds;
           } else { return; } // Became empty from a previous TODO
        } else { return; } // Particle somehow ceased to exist from a previous TODO

        // Re-check one last time before movement, as state changes could make it EMPTY
        if self.grid.get_particle(x,y).map_or(true, |p| p.material_type == MaterialType::EMPTY) {
            return;
        }

        // 5. Handle Movement
        self.handle_movement(x, y);
    }

    pub fn handle_movement(&mut self, x: i32, y: i32) {
        let initial_particle_type;
        let initial_density;
        let initial_props_is_gas;
        let initial_props_is_liquid;
        let initial_props_is_solid;
        let initial_particle_moved_this_step; 

        {
            let p_check_opt = self.grid.get_particle(x, y);
            if p_check_opt.is_none() { return; } 

            let p_check = p_check_opt.unwrap();
            if p_check.material_type == MaterialType::EMPTY { return; } 
            if p_check.material_type == MaterialType::GENERATOR {
                return; 
            }
            let props = p_check.get_material_properties();
            initial_particle_type = p_check.material_type;
            initial_density = props.density;
            initial_props_is_gas = initial_density < 0.0;
            initial_props_is_liquid = is_liquid(initial_particle_type);
            initial_props_is_solid = !initial_props_is_gas && !initial_props_is_liquid;
            initial_particle_moved_this_step = p_check.moved_this_step; 
        }


        let vert_dir: i32 = if initial_props_is_gas { -1 } else { 1 };
        let ny = y + vert_dir; 

        if initial_props_is_solid && vert_dir == 1 {
            let mut splashed = false;
            if self.grid.is_valid(x, ny) {
                let target_below_type_opt = self.grid.get_particle(x, ny).map(|p| p.material_type);
                if let Some(target_below_type) = target_below_type_opt {
                    if is_liquid(target_below_type) {
                        let splash_dx_rand = rand::random::<f32>();
                        let initial_splash_dx = if splash_dx_rand < 0.5 { -1 } else { 1 };
                        for i in 0..2 {
                            let current_splash_dx = if i == 0 { initial_splash_dx } else { -initial_splash_dx };
                            let splash_x = x + current_splash_dx;
                            let splash_y = y; 

                            if self.grid.is_valid(splash_x, splash_y) &&
                               self.grid.get_particle(splash_x, splash_y).map_or(false, |p| p.material_type == MaterialType::EMPTY)
                            {
                                let falling_solid_clone = self.grid.get_particle(x, y).unwrap().clone();
                                let liquid_to_splash_clone = self.grid.get_particle(x, ny).unwrap().clone();

                                self.grid.set_particle(splash_x, splash_y, liquid_to_splash_clone);
                                if let Some(p) = self.grid.get_particle_mut(splash_x, splash_y) { p.moved_this_step = true; }

                                self.grid.set_particle(x, ny, falling_solid_clone);
                                if let Some(p) = self.grid.get_particle_mut(x, ny) { p.moved_this_step = true; }
                                
                                self.grid.set_particle(x, y, Particle::new(x, y, MaterialType::EMPTY, AMBIENT_TEMP));
                                splashed = true;
                                break;
                            }
                        }
                    }
                }
            }
            if splashed { return; }
        }

        if !self.grid.is_valid(x, ny) { 
            return; 
        }

        let target_at_ny_type_opt = self.grid.get_particle(x, ny).map(|p| p.material_type);

        if let Some(target_type_at_ny) = target_at_ny_type_opt {
            if target_type_at_ny == MaterialType::GENERATOR {
            } else if target_type_at_ny == MaterialType::EMPTY {
                self.grid.swap_particles(x, y, x, ny);
                if let Some(p_now_at_ny) = self.grid.get_particle_mut(x, ny) { p_now_at_ny.moved_this_step = true; }
                return;
            } else { 
                let target_density_at_ny = self.grid.get_particle(x, ny).unwrap().get_material_properties().density;
                let should_push = if initial_props_is_gas { target_density_at_ny > initial_density } else { initial_density > target_density_at_ny };

                if should_push {
                    let push_dir_rand = rand::random::<f32>();
                    let initial_push_dx = if push_dir_rand < 0.5 { -1 } else { 1 };
                    let mut pushed = false;
                    for i in 0..2 {
                        let current_push_dx = if i == 0 { initial_push_dx } else { -initial_push_dx };
                        let push_target_x = x + current_push_dx;
                        let push_target_y = ny;

                        if self.grid.is_valid(push_target_x, push_target_y) &&
                           self.grid.get_particle(push_target_x, push_target_y).map_or(false, |p| p.material_type == MaterialType::EMPTY)
                        {
                            let particle_to_push_clone = self.grid.get_particle(x, ny).unwrap().clone();
                            let current_particle_clone = self.grid.get_particle(x, y).unwrap().clone();
                            
                            self.grid.set_particle(push_target_x, push_target_y, particle_to_push_clone);
                            if let Some(p) = self.grid.get_particle_mut(push_target_x, push_target_y) { p.moved_this_step = true; }

                            self.grid.set_particle(x, ny, current_particle_clone);
                            if let Some(p) = self.grid.get_particle_mut(x, ny) { p.moved_this_step = true; }
                            
                            self.grid.set_particle(x, y, Particle::new(x, y, MaterialType::EMPTY, AMBIENT_TEMP));
                            pushed = true;
                            break;
                        }
                    }
                    if pushed { return; }
                }

                let density_allows_swap = if initial_props_is_gas { target_density_at_ny > initial_density } else { initial_density > target_density_at_ny };
                if density_allows_swap {
                    self.grid.swap_particles(x, y, x, ny);
                    if let Some(p_now_at_ny) = self.grid.get_particle_mut(x, ny) { p_now_at_ny.moved_this_step = true; }
                    if let Some(p_now_at_xy) = self.grid.get_particle_mut(x, y) {
                        if p_now_at_xy.material_type != MaterialType::EMPTY {
                             p_now_at_xy.moved_this_step = true;
                        }
                    }
                    return;
                }
            }
        } 

        if vert_dir == 1 && is_powder(initial_particle_type) {
            if let Some(p_below) = self.grid.get_particle(x, y + 1) {
                if is_rigid_solid(p_below.material_type) {
                    return; 
                }
            }
        }

        if !is_rigid_solid(initial_particle_type) {
            let diag_rand_dir = rand::random::<f32>();
            let dx1 = if diag_rand_dir < 0.5 { -1 } else { 1 };
            let dx_order = [dx1, -dx1];

            for &check_dx in &dx_order {
                let diag_x = x + check_dx;
                let diag_y = y + vert_dir;

                if self.grid.is_valid(diag_x, diag_y) && 
                   self.grid.get_particle(diag_x, diag_y).map_or(false, |p| p.material_type == MaterialType::EMPTY) 
                {
                    let mut vertical_blocked = false;
                    if let Some(vn) = self.grid.get_particle(x, ny) { 
                        if vn.material_type != MaterialType::EMPTY {
                            if vn.material_type == MaterialType::GENERATOR {
                                vertical_blocked = true;
                            } else {
                                let vn_density = vn.get_material_properties().density;
                                if initial_props_is_gas {
                                    if vn_density <= initial_density { vertical_blocked = true; }
                                } else {
                                    if vn_density >= initial_density { vertical_blocked = true; }
                                }
                            }
                        }
                    } else { 
                        vertical_blocked = true; 
                    }

                    if vertical_blocked {
                        self.grid.swap_particles(x, y, diag_x, diag_y);
                        if let Some(p_now_at_diag) = self.grid.get_particle_mut(diag_x, diag_y) { p_now_at_diag.moved_this_step = true; }
                        return;
                    }
                }
            }
        }

        let am_powder = is_powder(initial_particle_type);
        let am_liquid_or_gas = initial_props_is_liquid || initial_props_is_gas;

        if am_liquid_or_gas {
            let viscosity_opt = self.grid.get_particle(x,y).map(|p| p.get_material_properties().viscosity);
            if viscosity_opt.is_none() { return; } 
            let viscosity = viscosity_opt.unwrap();

            let dx_rand = rand::random::<f32>();
            let dx1 = if dx_rand < 0.5 { -1 } else { 1 };
            let dx_order = [dx1, -dx1];

            for &check_dx in &dx_order {
                let side_x = x + check_dx;
                let side_y = y;

                if !self.grid.is_valid(side_x, side_y) { continue; }

                let side_target_opt = self.grid.get_particle(side_x, side_y);
                if side_target_opt.map_or(false, |p| p.material_type == MaterialType::EMPTY) {
                    let move_chance = if initial_props_is_liquid { (1.0 - viscosity * 0.1).max(0.1) } else { 1.0 };
                    if rand::random::<f32>() < move_chance {
                        self.grid.swap_particles(x, y, side_x, side_y);
                        if let Some(p_now_at_side) = self.grid.get_particle_mut(side_x, side_y) { p_now_at_side.moved_this_step = true; }
                        return;
                    }
                } else if initial_props_is_liquid &&
                          side_target_opt.map_or(false, |p| is_liquid(p.material_type)) &&
                          initial_particle_moved_this_step 
                {
                    let push_beyond_x = side_x + check_dx;
                    if self.grid.is_valid(push_beyond_x, y) && self.grid.get_particle(push_beyond_x, y).map_or(false, |p| p.material_type == MaterialType::EMPTY) {
                        let push_chance = 0.5 / viscosity;
                        if rand::random::<f32>() < push_chance {
                            let particle_to_push_clone = self.grid.get_particle(side_x, y).unwrap().clone();
                            let current_particle_clone = self.grid.get_particle(x, y).unwrap().clone();

                            self.grid.set_particle(push_beyond_x, y, particle_to_push_clone);
                            if let Some(p) = self.grid.get_particle_mut(push_beyond_x, y) { p.moved_this_step = true; }
                            
                            self.grid.set_particle(side_x, y, current_particle_clone);
                            if let Some(p) = self.grid.get_particle_mut(side_x, y) { p.moved_this_step = true; }
                            
                            self.grid.set_particle(x, y, Particle::new(x, y, MaterialType::EMPTY, AMBIENT_TEMP));
                            return;
                        }
                    }
                }
            }
        } else if am_powder && vert_dir == 1 { 
            if let Some(p_below) = self.grid.get_particle(x, y + 1) {
                if p_below.material_type != MaterialType::EMPTY && p_below.material_type != MaterialType::GENERATOR {
                    let dx_rand = rand::random::<f32>();
                    let dx1 = if dx_rand < 0.5 { -1 } else { 1 };
                    let dx_order = [dx1, -dx1];

                    for &check_dx in &dx_order {
                        let pile_x = x + check_dx;
                        let pile_y = y + 1;
                        if self.grid.is_valid(pile_x, pile_y) && self.grid.get_particle(pile_x, pile_y).map_or(false, |p| p.material_type == MaterialType::EMPTY) {
                            self.grid.swap_particles(x, y, pile_x, pile_y);
                            if let Some(p_now_at_pile) = self.grid.get_particle_mut(pile_x, pile_y) { p_now_at_pile.moved_this_step = true; }
                            return;
                        }
                    }
                }
            }
        }
    }
}
