// reactions.rs - Chemical reactions and state changes

use rand::prelude::*;
use crate::constants::*;
use crate::material_properties::MaterialType;
use crate::particle::Particle;

pub struct ReactionEngine {
    rng: ThreadRng,
}

impl ReactionEngine {
    pub fn new() -> Self {
        Self {
            rng: rand::thread_rng(),
        }
    }
    
    // Process lifespan and burnout
    pub fn handle_lifespan_and_burnout(
        &mut self, 
        particle: &mut Particle, 
        x: usize, 
        y: usize, 
        delta_time: f32,
        get_particle: impl Fn(usize, usize) -> Option<Particle>,
        set_particle: impl FnMut(usize, usize, Particle) -> bool,
    ) -> bool {
        // Check if particle has reached the end of its life
        if particle.update_life(delta_time) {
            // Get the successor material and temperature
            let (new_material, new_temp) = particle.get_successor_material();
            
            // Create new particle if needed
            if new_material != MaterialType::Empty {
                let mut new_particle = Particle::new(new_material, new_temp);
                new_particle.processed = true;
                
                let mut set_particle = set_particle;
                if set_particle(x, y, new_particle) {
                    return true; // Particle was replaced
                }
            } else {
                // Just remove the particle (empty)
                let mut set_particle = set_particle;
                let empty = Particle::new(MaterialType::Empty, AMBIENT_TEMP);
                if set_particle(x, y, empty) {
                    return true; // Particle was removed
                }
            }
        }
        
        false
    }
    
    // Process state changes and effects
    pub fn handle_state_changes_and_effects(
        &mut self, 
        particle: &mut Particle, 
        x: usize, 
        y: usize, 
        delta_time: f32,
        get_particle: impl Fn(usize, usize) -> Option<Particle>,
        set_particle: impl FnMut(usize, usize, Particle) -> bool,
        add_active_cell: impl FnMut(usize, usize),
    ) -> bool {
        let mut set_particle = set_particle;
        let mut add_active_cell = add_active_cell;
        
        // Get current state
        let material = particle.material;
        let temp = particle.temperature.get();
        let scaled_dt = delta_time * TARGET_DT_SCALING;
        
        // --- 1. Check for ignition of flammable materials ---
        let props = material.get_properties();
        if props.flammable && props.ignition_temp.is_some() {
            let ignition_temp = props.ignition_temp.unwrap();
            
            // Check if temperature is above ignition point
            if temp >= ignition_temp {
                // Check for external ignition sources
                let mut external_ignition = false;
                let mut max_neighbor_temp = temp;
                
                for dy in -1..=1 {
                    for dx in -1..=1 {
                        if dx == 0 && dy == 0 {
                            continue;
                        }
                        
                        let nx = match (x as isize + dx).try_into() {
                            Ok(val) if val < GRID_WIDTH => val,
                            _ => continue,
                        };
                        
                        let ny = match (y as isize + dy).try_into() {
                            Ok(val) if val < GRID_HEIGHT => val,
                            _ => continue,
                        };
                        
                        if let Some(neighbor) = get_particle(nx, ny) {
                            if neighbor.material == MaterialType::Fire || 
                               neighbor.material == MaterialType::Lava || 
                               (neighbor.material == MaterialType::Fuse && neighbor.burning) {
                                external_ignition = true;
                                max_neighbor_temp = max_neighbor_temp.max(neighbor.temperature.get());
                            }
                        }
                    }
                }
                
                // Determine if we should ignite
                match material {
                    MaterialType::Plant | MaterialType::Wood | MaterialType::Coal | 
                    MaterialType::Oil => {
                        // Needs external ignition or very high temp
                        if external_ignition || temp > ignition_temp + 100.0 {
                            let initial_fire_temp = 800.0_f32.max(max_neighbor_temp);
                            let mut new_particle = Particle::new(MaterialType::Fire, initial_fire_temp);
                            
                            // Adjust lifespan based on fuel type
                            match material {
                                MaterialType::Wood => new_particle.life_remaining = Some(3.0),
                                MaterialType::Coal => new_particle.life_remaining = Some(4.0),
                                _ => new_particle.life_remaining = Some(FIRE_LIFESPAN),
                            }
                            
                            if set_particle(x, y, new_particle) {
                                return true; // Material changed
                            }
                        }
                    },
                    MaterialType::Gunpowder => {
                        // Gunpowder ignites easily if temp threshold met
                        if self.explode(x, y, GUNPOWDER_YIELD, get_particle, &mut set_particle, &mut add_active_cell) {
                            return true; // Explosion handled the particle
                        }
                    },
                    MaterialType::Fuse => {
                        // Fuse needs external ignition to start burning flag
                        if !particle.burning && external_ignition {
                            particle.burning = true;
                            particle.life_remaining = Some(FUSE_BURN_LIFESPAN);
                            particle.temperature.set(ignition_temp + 50.0_f32.max(temp));
                            return false; // Particle updated but not replaced
                        }
                    },
                    _ => {}
                }
            }
        }
        
        // --- 2. Check for melting ---
        if let Some(melt_temp) = props.melt_temp {
            if temp >= melt_temp + PHASE_CHANGE_TEMP_BUFFER && !material.is_liquid() {
                let new_material = match material {
                    MaterialType::Sand => MaterialType::Glass,
                    MaterialType::Glass => MaterialType::Lava,
                    MaterialType::Ice => MaterialType::Water,
                    _ => material,
                };
                
                if new_material != material {
                    // Handle melting
                    let mut new_particle = Particle::new(new_material, temp);
                    new_particle.processed = true;
                    
                    if set_particle(x, y, new_particle) {
                        return true; // Material changed
                    }
                }
            }
        }
        
        // --- 3. Check for boiling ---
        if let Some(boil_temp) = props.boil_temp {
            if temp >= boil_temp + PHASE_CHANGE_TEMP_BUFFER {
                let new_material = match material {
                    MaterialType::Water => MaterialType::Steam,
                    MaterialType::Acid => MaterialType::ToxicGas,
                    _ => material,
                };
                
                if new_material != material {
                    // Handle boiling
                    let mut new_particle = Particle::new(new_material, temp);
                    new_particle.processed = true;
                    
                    if set_particle(x, y, new_particle) {
                        return true; // Material changed
                    }
                }
            }
        }
        
        // --- 4. Check for freezing/condensation ---
        if let Some(freeze_temp) = props.freeze_temp {
            if temp <= freeze_temp - PHASE_CHANGE_TEMP_BUFFER {
                let new_material = match material {
                    MaterialType::Water => MaterialType::Ice,
                    MaterialType::Lava => MaterialType::Stone,
                    MaterialType::Steam => {
                        // Condensation conditions
                        if particle.time_in_state >= MIN_STATE_SECONDS {
                            let condensation_chance = if y < CONDENSATION_Y_LIMIT {
                                1.0 // Always condense if cool enough and high up
                            } else {
                                CONDENSATION_CHANCE_ANYWHERE_PER_SEC * scaled_dt
                            };
                            
                            if self.rng.gen::<f32>() < condensation_chance {
                                MaterialType::Water
                            } else {
                                MaterialType::Steam
                            }
                        } else {
                            MaterialType::Steam
                        }
                    },
                    _ => material,
                };
                
                if new_material != material {
                    // Handle freezing/condensation
                    let mut new_particle = Particle::new(new_material, temp);
                    new_particle.processed = true;
                    
                    if set_particle(x, y, new_particle) {
                        return true; // Material changed
                    }
                }
            }
        }
        
        // --- 5. Handle special material interactions ---
        self.handle_special_material_effects(particle, x, y, delta_time, get_particle, &mut set_particle, &mut add_active_cell)
    }
    
    // Handle special effects based on material type
    fn handle_special_material_effects(
        &mut self, 
        particle: &mut Particle, 
        x: usize, 
        y: usize, 
        delta_time: f32,
        get_particle: impl Fn(usize, usize) -> Option<Particle>,
        set_particle: &mut impl FnMut(usize, usize, Particle) -> bool,
        add_active_cell: &mut impl FnMut(usize, usize),
    ) -> bool {
        let material = particle.material;
        let scaled_dt = delta_time * TARGET_DT_SCALING;
        
        match material {
            MaterialType::Fire => {
                self.handle_fire_effects(particle, x, y, delta_time, get_particle, set_particle, add_active_cell)
            },
            MaterialType::Lava => {
                self.handle_lava_effects(particle, x, y, delta_time, get_particle, set_particle, add_active_cell)
            },
            MaterialType::Fuse if particle.burning => {
                self.handle_burning_fuse_effects(particle, x, y, delta_time, get_particle, set_particle)
            },
            MaterialType::Acid => {
                self.handle_acid_effects(particle, x, y, delta_time, get_particle, set_particle)
            },
            MaterialType::Plant => {
                self.handle_plant_effects(particle, x, y, delta_time, get_particle, set_particle)
            },
            _ => false
        }
    }
    
    // Handle fire effects
    fn handle_fire_effects(
        &mut self, 
        particle: &mut Particle, 
        x: usize, 
        y: usize, 
        delta_time: f32,
        get_particle: impl Fn(usize, usize) -> Option<Particle>,
        set_particle: &mut impl FnMut(usize, usize, Particle) -> bool,
        add_active_cell: &mut impl FnMut(usize, usize),
    ) -> bool {
        let scaled_dt = delta_time * TARGET_DT_SCALING;
        let mut fuel_found = false;
        let mut extinguish = false;
        
        // Check neighbors for fuel and water
        for dy in -1..=1 {
            for dx in -1..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }
                
                let nx = match (x as isize + dx).try_into() {
                    Ok(val) if val < GRID_WIDTH => val,
                    _ => continue,
                };
                
                let ny = match (y as isize + dy).try_into() {
                    Ok(val) if val < GRID_HEIGHT => val,
                    _ => continue,
                };
                
                if let Some(neighbor) = get_particle(nx, ny) {
                    let n_mat = neighbor.material;
                    
                    // Check for extinguishing materials
                    if n_mat == MaterialType::Water || n_mat == MaterialType::Ice {
                        particle.temperature.add(-WATER_COOLING_FACTOR * scaled_dt);
                        
                        if let Some(life) = &mut particle.life_remaining {
                            *life = (*life - 10.0 * delta_time).max(0.01);
                        }
                        
                        if self.rng.gen::<f32>() < 0.5 * scaled_dt {
                            // Convert water to steam or ice to water
                            let mut new_neighbor = if n_mat == MaterialType::Water {
                                Particle::new(MaterialType::Steam, neighbor.temperature.get())
                            } else {
                                Particle::new(MaterialType::Water, neighbor.temperature.get())
                            };
                            new_neighbor.processed = true;
                            
                            if set_particle(nx, ny, new_neighbor) {
                                add_active_cell(nx, ny);
                            }
                        }
                        
                        if particle.temperature.get() < 300.0 {
                            extinguish = true;
                        }
                    }
                    
                    // Check for fuel
                    let n_props = n_mat.get_properties();
                    if n_props.flammable {
                        fuel_found = true;
                        
                        // Check if fuel should ignite
                        if let Some(ignition_temp) = n_props.ignition_temp {
                            if neighbor.temperature.get() >= ignition_temp {
                                // Handle different materials
                                match n_mat {
                                    MaterialType::Gunpowder => {
                                        if self.explode(nx, ny, GUNPOWDER_YIELD, get_particle, set_particle, add_active_cell) {
                                            continue;
                                        }
                                    },
                                    MaterialType::Fuse if !neighbor.burning => {
                                        let mut burning_fuse = neighbor.clone();
                                        burning_fuse.burning = true;
                                        burning_fuse.life_remaining = Some(FUSE_BURN_LIFESPAN);
                                        burning_fuse.temperature.set(ignition_temp + 50.0_f32.max(neighbor.temperature.get()));
                                        burning_fuse.processed = true;
                                        
                                        if set_particle(nx, ny, burning_fuse) {
                                            add_active_cell(nx, ny);
                                        }
                                    },
                                    _ if n_mat != MaterialType::Fire => {
                                        // Convert to fire with appropriate lifespan
                                        let initial_fire_temp = 800.0_f32.max(neighbor.temperature.get());
                                        let initial_fire_life = match n_mat {
                                            MaterialType::Wood => 3.0,
                                            MaterialType::Coal => 4.0,
                                            _ => FIRE_LIFESPAN,
                                        };
                                        
                                        let mut new_fire = Particle::new(MaterialType::Fire, initial_fire_temp);
                                        new_fire.life_remaining = Some(initial_fire_life);
                                        new_fire.processed = true;
                                        
                                        if set_particle(nx, ny, new_fire) {
                                            add_active_cell(nx, ny);
                                        }
                                    },
                                    _ => {}
                                }
                            }
                        }
                    }
                }
            }
        }
        
        // Extend fire life if fuel nearby
        if fuel_found {
            if let Some(life) = &mut particle.life_remaining {
                *life = life.max(FIRE_LIFESPAN);
            }
        }
        
        // Spawn smoke randomly
        if !extinguish && self.rng.gen::<f32>() < 0.1 * scaled_dt {
            let smoke_x = x + if self.rng.gen_bool(0.5) { 1 } else { 0 };
            let smoke_y = y.saturating_sub(1);
            
            if smoke_x < GRID_WIDTH {
                if let Some(target) = get_particle(smoke_x, smoke_y) {
                    if target.material == MaterialType::Empty {
                        let smoke = Particle::new(MaterialType::Smoke, particle.temperature.get() * 0.5);
                        if set_particle(smoke_x, smoke_y, smoke) {
                            add_active_cell(smoke_x, smoke_y);
                        }
                    }
                }
            }
        }
        
        // Extinguish fire
        if extinguish {
            let smoke = Particle::new(MaterialType::Smoke, particle.temperature.get());
            if set_particle(x, y, smoke) {
                return true; // Fire was extinguished
            }
        }
        
        false
    }
    
    // Handle lava effects
    fn handle_lava_effects(
        &mut self, 
        particle: &mut Particle, 
        x: usize, 
        y: usize, 
        delta_time: f32,
        get_particle: impl Fn(usize, usize) -> Option<Particle>,
        set_particle: &mut impl FnMut(usize, usize, Particle) -> bool,
        add_active_cell: &mut impl FnMut(usize, usize),
    ) -> bool {
        let scaled_dt = delta_time * TARGET_DT_SCALING;
        
        // Check for cooling to stone
        let current_temp = particle.temperature.get();
        if current_temp < 1000.0 && self.rng.gen::<f32>() < 0.05 * scaled_dt {
            let stone = Particle::new(MaterialType::Stone, current_temp);
            if set_particle(x, y, stone) {
                return true; // Lava cooled to stone
            }
        }
        
        // Check neighbors for ignition
        for dy in -1..=1 {
            for dx in -1..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }
                
                let nx = match (x as isize + dx).try_into() {
                    Ok(val) if val < GRID_WIDTH => val,
                    _ => continue,
                };
                
                let ny = match (y as isize + dy).try_into() {
                    Ok(val) if val < GRID_HEIGHT => val,
                    _ => continue,
                };
                
                if let Some(neighbor) = get_particle(nx, ny) {
                    let n_mat = neighbor.material;
                    let n_props = n_mat.get_properties();
                    
                    // Check if material should ignite
                    if n_props.flammable {
                        if let Some(ignition_temp) = n_props.ignition_temp {
                            if neighbor.temperature.get() >= ignition_temp {
                                // Handle different materials
                                match n_mat {
                                    MaterialType::Gunpowder => {
                                        if self.explode(nx, ny, GUNPOWDER_YIELD, get_particle, set_particle, add_active_cell) {
                                            continue;
                                        }
                                    },
                                    MaterialType::Fuse if !neighbor.burning => {
                                        let mut burning_fuse = neighbor.clone();
                                        burning_fuse.burning = true;
                                        burning_fuse.life_remaining = Some(FUSE_BURN_LIFESPAN);
                                        burning_fuse.temperature.set(ignition_temp + 50.0_f32.max(neighbor.temperature.get()));
                                        burning_fuse.processed = true;
                                        
                                        if set_particle(nx, ny, burning_fuse) {
                                            add_active_cell(nx, ny);
                                        }
                                    },
                                    _ if n_mat != MaterialType::Fire => {
                                        // Convert to fire with appropriate lifespan
                                        let initial_fire_temp = 1000.0_f32.max(neighbor.temperature.get()); // Lava makes hotter fire
                                        let initial_fire_life = match n_mat {
                                            MaterialType::Wood => 3.0,
                                            MaterialType::Coal => 4.0,
                                            _ => FIRE_LIFESPAN,
                                        };
                                        
                                        let mut new_fire = Particle::new(MaterialType::Fire, initial_fire_temp);
                                        new_fire.life_remaining = Some(initial_fire_life);
                                        new_fire.processed = true;
                                        
                                        if set_particle(nx, ny, new_fire) {
                                            add_active_cell(nx, ny);
                                        }
                                    },
                                    _ => {}
                                }
                            }
                        }
                    }
                }
            }
        }
        
        false
    }
    
    // Handle burning fuse effects
    fn handle_burning_fuse_effects(
        &mut self, 
        particle: &mut Particle, 
        x: usize, 
        y: usize, 
        delta_time: f32,
        get_particle: impl Fn(usize, usize) -> Option<Particle>,
        set_particle: &mut impl FnMut(usize, usize, Particle) -> bool,
    ) -> bool {
        let scaled_dt = delta_time * TARGET_DT_SCALING;
        
        // Heat up neighbors and spread burning
        for dy in -1..=1 {
            for dx in -1..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }
                
                let nx = match (x as isize + dx).try_into() {
                    Ok(val) if val < GRID_WIDTH => val,
                    _ => continue,
                };
                
                let ny = match (y as isize + dy).try_into() {
                    Ok(val) if val < GRID_HEIGHT => val,
                    _ => continue,
                };
                
                if let Some(neighbor) = get_particle(nx, ny) {
                    // Heat neighbor
                    let mut new_neighbor = neighbor.clone();
                    new_neighbor.temperature.add(20.0 * scaled_dt);
                    
                    // Check for ignition
                    let n_mat = neighbor.material;
                    let n_props = n_mat.get_properties();
                    
                    if n_props.flammable {
                        if let Some(ignition_temp) = n_props.ignition_temp {
                            if new_neighbor.temperature.get() >= ignition_temp {
                                // Handle different materials
                                match n_mat {
                                    MaterialType::Fuse if !neighbor.burning => {
                                        new_neighbor.burning = true;
                                        new_neighbor.life_remaining = Some(FUSE_BURN_LIFESPAN);
                                        new_neighbor.temperature.set(ignition_temp + 50.0);
                                    },
                                    MaterialType::Gunpowder => {
                                        // Just set temperature, explosion will be handled later
                                        new_neighbor.temperature.set(ignition_temp + 50.0);
                                    },
                                    _ if n_mat != MaterialType::Fire && n_mat != MaterialType::Fuse => {
                                        // Convert to fire
                                        let mut new_fire = Particle::new(MaterialType::Fire, ignition_temp + 50.0);
                                        new_fire.life_remaining = Some(FIRE_LIFESPAN);
                                        
                                        if set_particle(nx, ny, new_fire) {
                                            continue;
                                        }
                                    },
                                    _ => {}
                                }
                            }
                        }
                    }
                    
                    // Update neighbor
                    set_particle(nx, ny, new_neighbor);
                }
            }
        }
        
        false
    }
    
    // Handle acid effects
    fn handle_acid_effects(
        &mut self, 
        particle: &mut Particle, 
        x: usize, 
        y: usize, 
        delta_time: f32,
        get_particle: impl Fn(usize, usize) -> Option<Particle>,
        set_particle: &mut impl FnMut(usize, usize, Particle) -> bool,
    ) -> bool {
        let props = particle.material.get_properties();
        let corrosive_power = props.corrosive_power.unwrap_or(0.0);
        let scaled_dt = delta_time * TARGET_DT_SCALING;
        
        if corrosive_power > 0.0 {
            let mut consumed = false;
            
            // Check cardinal neighbors
            for &(dx, dy) in &[(0, -1), (-1, 0), (1, 0), (0, 1)] {
                let nx = match (x as isize + dx).try_into() {
                    Ok(val) if val < GRID_WIDTH => val,
                    _ => continue,
                };
                
                let ny = match (y as isize + dy).try_into() {
                    Ok(val) if val < GRID_HEIGHT => val,
                    _ => continue,
                };
                
                if let Some(neighbor) = get_particle(nx, ny) {
                    let n_mat = neighbor.material;
                    
                    // Skip immune materials
                    let immune = [
                        MaterialType::Empty,
                        MaterialType::Acid,
                        MaterialType::Glass,
                        MaterialType::Generator,
                    ];
                    
                    if !immune.contains(&n_mat) {
                        // Check if acid dissolves material
                        if self.rng.gen::<f32>() < corrosive_power * scaled_dt {
                            // Special case for stone: chance to convert to sand
                            if n_mat == MaterialType::Stone && self.rng.gen::<f32>() < 0.3 {
                                let sand = Particle::new(MaterialType::Sand, neighbor.temperature.get());
                                if set_particle(nx, ny, sand) {
                                    // Continue with other neighbors
                                    continue;
                                }
                            }
                            
                            // Dissolve the material
                            let empty = Particle::new(MaterialType::Empty, AMBIENT_TEMP);
                            if set_particle(nx, ny, empty) {
                                // Generate toxic gas above dissolved material
                                let gas_spawn_temp = particle.temperature.get() * ACID_GAS_TEMP_FACTOR;
                                let gas_y = ny.saturating_sub(1);
                                
                                if let Some(target) = get_particle(nx, gas_y) {
                                    if target.material == MaterialType::Empty {
                                        let toxic_gas = Particle::new(MaterialType::ToxicGas, gas_spawn_temp);
                                        set_particle(nx, gas_y, toxic_gas);
                                    }
                                }
                                
                                // Chance for acid to be consumed
                                if self.rng.gen::<f32>() < 0.05 * scaled_dt {
                                    consumed = true;
                                    break;
                                }
                            }
                        }
                    }
                }
            }
            
            // If acid was consumed, remove it
            if consumed {
                let empty = Particle::new(MaterialType::Empty, AMBIENT_TEMP);
                if set_particle(x, y, empty) {
                    return true; // Acid was consumed
                }
            }
        }
        
        false
    }
    
    // Handle plant effects
    fn handle_plant_effects(
        &mut self, 
        particle: &mut Particle, 
        x: usize, 
        y: usize, 
        delta_time: f32,
        get_particle: impl Fn(usize, usize) -> Option<Particle>,
        set_particle: &mut impl FnMut(usize, usize, Particle) -> bool,
    ) -> bool {
        let scaled_dt = delta_time * TARGET_DT_SCALING;
        let current_temp = particle.temperature.get();
        
        // Check for adjacent water and empty spots
        let mut has_adjacent_water = false;
        let mut empty_neighbors = Vec::new();
        
        // Scan cardinal neighbors
        for &(dx, dy) in &[(0, -1), (-1, 0), (1, 0), (0, 1)] {
            let nx = match (x as isize + dx).try_into() {
                Ok(val) if val < GRID_WIDTH => val,
                _ => continue,
            };
            
            let ny = match (y as isize + dy).try_into() {
                Ok(val) if val < GRID_HEIGHT => val,
                _ => continue,
            };
            
            if let Some(neighbor) = get_particle(nx, ny) {
                if neighbor.material == MaterialType::Water {
                    has_adjacent_water = true;
                } else if neighbor.material == MaterialType::Empty {
                    empty_neighbors.push((nx, ny));
                } else if neighbor.material == MaterialType::ToxicGas {
                    // Plants die from toxic gas
                    let toxic_power = neighbor.material.get_properties()
                        .corrosive_power.unwrap_or(0.01);
                    
                    if self.rng.gen::<f32>() < toxic_power * 5.0 * scaled_dt {
                        let empty = Particle::new(MaterialType::Empty, AMBIENT_TEMP);
                        if set_particle(x, y, empty) {
                            return true; // Plant died
                        }
                    }
                }
            }
        }
        
        // Try to grow into an empty neighbor if water is adjacent and temp is right
        if has_adjacent_water && !empty_neighbors.is_empty() && 
           AMBIENT_TEMP < current_temp && current_temp < 50.0 {
            if self.rng.gen::<f32>() < PLANT_GROWTH_CHANCE_PER_SEC * scaled_dt {
                // Pick a random empty neighbor
                let idx = self.rng.gen_range(0..empty_neighbors.len());
                let (nx, ny) = empty_neighbors[idx];
                
                // Grow a new plant
                let new_plant = Particle::new(MaterialType::Plant, current_temp);
                if set_particle(nx, ny, new_plant) {
                    // Growth successful, continue with potential water conversion
                }
            }
        }
        
        // Try to convert adjacent water directly into plant
        if has_adjacent_water && AMBIENT_TEMP < current_temp && current_temp < 50.0 {
            if self.rng.gen::<f32>() < PLANT_GROWTH_CHANCE_PER_SEC * scaled_dt * 0.5 {
                // Check cardinal neighbors to find water
                for &(dx, dy) in &[(0, -1), (-1, 0), (1, 0), (0, 1)] {
                    let nx = match (x as isize + dx).try_into() {
                        Ok(val) if val < GRID_WIDTH => val,
                        _ => continue,
                    };
                    
                    let ny = match (y as isize + dy).try_into() {
                        Ok(val) if val < GRID_HEIGHT => val,
                        _ => continue,
                    };
                    
                    if let Some(neighbor) = get_particle(nx, ny) {
                        if neighbor.material == MaterialType::Water {
                            // Convert water to plant
                            let new_plant = Particle::new(MaterialType::Plant, current_temp);
                            if set_particle(nx, ny, new_plant) {
                                break; // Successfully converted one water particle
                            }
                        }
                    }
                }
            }
        }
        
        false
    }
    
    // Create an explosion at a given point
    pub fn explode(
        &mut self, 
        cx: usize, 
        cy: usize, 
        radius: usize,
        get_particle: impl Fn(usize, usize) -> Option<Particle>,
        set_particle: &mut impl FnMut(usize, usize, Particle) -> bool,
        add_active_cell: &mut impl FnMut(usize, usize),
    ) -> bool {
        // Clear the center point
        let empty = Particle::new(MaterialType::Empty, AMBIENT_TEMP);
        let mut result = false;
        
        if set_particle(cx, cy, empty) {
            result = true;
        }
        
        // Collect particles within the radius
        let mut affected_particles = Vec::new();
        
        for dy in -(radius as isize)..=(radius as isize) {
            for dx in -(radius as isize)..=(radius as isize) {
                let dist_sq = dx * dx + dy * dy;
                
                // Check if within radius
                if dist_sq <= (radius * radius) as isize {
                    let px = match (cx as isize + dx).try_into() {
                        Ok(val) if val < GRID_WIDTH => val,
                        _ => continue,
                    };
                    
                    let py = match (cy as isize + dy).try_into() {
                        Ok(val) if val < GRID_HEIGHT => val,
                        _ => continue,
                    };
                    
                    if let Some(particle) = get_particle(px, py) {
                        if particle.material != MaterialType::Empty {
                            affected_particles.push(((px, py), particle.clone(), (dist_sq as f32).sqrt()));
                        }
                    }
                }
            }
        }
        
        // Process affected particles
        for ((px, py), mut particle, dist) in affected_particles {
            let radius_f = radius as f32;
            let effect_strength = 1.0 - (dist / radius_f).max(0.0).min(1.0);
            
            // Heat up the particle
            particle.temperature.add(1500.0 * effect_strength);
            
            // Chance to convert to fire or smoke
            let destroy_chance = effect_strength * 0.95;
            
            if self.rng.gen::<f32>() < destroy_chance {
                let material = particle.material;
                
                // Don't affect generators or empty space
                if material == MaterialType::Generator || material == MaterialType::Empty {
                    continue;
                }
                
                // Convert to fire or smoke based on material and chance
                if self.rng.gen::<f32>() < 0.6 * effect_strength && 
                   material != MaterialType::Water && material != MaterialType::Ice {
                    let mut fire = Particle::new(MaterialType::Fire, particle.temperature.get());
                    fire.life_remaining = Some(FIRE_LIFESPAN * effect_strength * 0.5);
                    fire.processed = true;
                    
                    if set_particle(px, py, fire) {
                        add_active_cell(px, py);
                    }
                } else {
                    let mut smoke = Particle::new(MaterialType::Smoke, particle.temperature.get());
                    smoke.life_remaining = Some(SMOKE_LIFESPAN * effect_strength);
                    smoke.processed = true;
                    
                    if set_particle(px, py, smoke) {
                        add_active_cell(px, py);
                    }
                }
            } else {
                // Special conversions
                let material = particle.material;
                let new_material = match material {
                    MaterialType::Stone | MaterialType::Glass if self.rng.gen::<f32>() < effect_strength * 0.3 => {
                        Some(MaterialType::Sand)
                    },
                    MaterialType::Wood if self.rng.gen::<f32>() < effect_strength * 0.5 => {
                        Some(MaterialType::Ash)
                    },
                    _ => None
                };
                
                if let Some(new_mat) = new_material {
                    particle.change_material(new_mat, None);
                }
                
                // Update the particle
                particle.processed = true;
                if set_particle(px, py, particle) {
                    add_active_cell(px, py);
                }
            }
        }
        
        result
    }
}