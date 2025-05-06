// temperature.rs - Temperature system

use rand::prelude::*;
use crate::constants::*;
use crate::material_properties::MaterialType;

#[derive(Clone, Copy)]
pub struct Temperature {
    value: f32,
    color_cache: Option<[u8; 3]>,
}

impl Temperature {
    pub fn new(initial_temp: f32) -> Self {
        Self {
            value: initial_temp.max(-273.15).min(MAX_TEMP),
            color_cache: None,
        }
    }
    
    pub fn get(&self) -> f32 {
        self.value
    }
    
    pub fn set(&mut self, new_temp: f32) {
        self.value = new_temp.max(-273.15).min(MAX_TEMP);
        self.color_cache = None;
    }
    
    pub fn add(&mut self, delta: f32) {
        self.value = (self.value + delta).max(-273.15).min(MAX_TEMP);
        self.color_cache = None;
    }
    
    // Get color modification based on temperature
    pub fn get_color_modifier(&self, material: MaterialType) -> [i16; 3] {
        // No temperature visualization for empty or certain materials
        if material == MaterialType::Empty || 
           material == MaterialType::Fire || 
           material == MaterialType::Lava || 
           material == MaterialType::Steam || 
           material == MaterialType::Smoke {
            return [0, 0, 0];
        }
        
        // Calculate temperature factor (-1.0 to 1.0)
        let temp_factor = ((self.value - AMBIENT_TEMP) / 150.0).max(-1.0).min(1.0);
        
        // Red increases with temperature, blue decreases
        let r_mod = (temp_factor * 50.0) as i16;
        let g_mod = (temp_factor * 20.0) as i16;
        let b_mod = (-temp_factor * 30.0) as i16;
        
        [r_mod, g_mod, b_mod]
    }
    
    // Get color for visualization, potentially using cached value
    pub fn get_color(&self, material: MaterialType, base_color: [u8; 4]) -> [u8; 4] {
        let mut color = [0, 0, 0, 255];
        
        match material {
            MaterialType::Fire => {
                // Fire color varies with temperature and has flicker
                let temp_factor = ((self.value - 500.0) / 600.0).max(0.0).min(1.0);
                let flicker = thread_rng().gen::<f32>() * 0.3 + 0.85;
                
                color[0] = (base_color[0] as f32 * flicker + temp_factor * 60.0).min(255.0) as u8;
                color[1] = (base_color[1] as f32 * flicker * (1.0 - temp_factor * 0.6)).min(255.0) as u8;
                color[2] = (base_color[2] as f32 * flicker * (1.0 - temp_factor)).max(0.0) as u8;
            },
            MaterialType::Lava => {
                // Lava color varies with temperature
                let temp_factor = ((self.value - 1000.0) / 800.0).max(0.0).min(1.0);
                
                color[0] = (base_color[0] as f32 + temp_factor * 50.0).min(255.0) as u8;
                color[1] = (base_color[1] as f32 + temp_factor * 70.0).min(255.0) as u8;
                color[2] = (base_color[2] as f32 * (1.0 - temp_factor * 0.5)).max(0.0) as u8;
            },
            _ => {
                // Apply temperature coloring to other materials
                let temp_mod = self.get_color_modifier(material);
                
                color[0] = (base_color[0] as i16 + temp_mod[0]).max(0).min(255) as u8;
                color[1] = (base_color[1] as i16 + temp_mod[1]).max(0).min(255) as u8;
                color[2] = (base_color[2] as i16 + temp_mod[2]).max(0).min(255) as u8;
            }
        }
        
        // Copy alpha
        color[3] = base_color[3];
        
        color
    }
}

pub struct TemperatureSystem {
    rng: ThreadRng,
}

impl TemperatureSystem {
    pub fn new() -> Self {
        Self {
            rng: rand::thread_rng(),
        }
    }
    
    // Update temperature of a single particle
    pub fn update_particle_temperature(
        &mut self, 
        particle: &mut crate::particle::Particle, 
        x: usize, 
        y: usize, 
        delta_time: f32,
        get_particle: impl Fn(usize, usize) -> Option<crate::particle::Particle>,
    ) {
        let material = particle.material;
        let current_temp = particle.temperature.get();
        let scaled_dt = delta_time * TARGET_DT_SCALING;
        
        // Skip empty cells
        if material == MaterialType::Empty {
            return;
        }
        
        // Get material conductivity
        let mut conductivity = material.get_properties().conductivity;
        
        // Adjust conductivity for specific materials
        if material == MaterialType::Generator {
            conductivity *= 0.1;
        } else if material == MaterialType::Stone || material == MaterialType::Glass {
            conductivity *= 0.3;
        }
        
        // Collect temperatures from neighbors
        let mut neighbor_temp_sum = 0.0;
        let mut neighbor_conductivity_sum = 0.0;
        let mut neighbor_count = 0;
        
        // Check all neighbors
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
                
                // Get neighbor temperature and conductivity
                let (n_temp, n_cond) = if let Some(neighbor) = get_particle(nx, ny) {
                    (
                        neighbor.temperature.get(),
                        neighbor.material.get_properties().conductivity
                    )
                } else {
                    (AMBIENT_TEMP, 0.1) // Default for out of bounds or empty
                };
                
                // Cardinal directions conduct more heat than diagonals
                let dist_factor = if dx == 0 || dy == 0 { 1.0 } else { 0.707 };
                
                neighbor_temp_sum += n_temp * n_cond * dist_factor;
                neighbor_conductivity_sum += n_cond * dist_factor;
                neighbor_count += 1;
            }
        }
        
        // Calculate new temperature
        let mut new_temp = current_temp;
        
        if neighbor_count > 0 && (conductivity > 0.0 || neighbor_conductivity_sum > 0.0) {
            let total_conductivity = conductivity + neighbor_conductivity_sum;
            
            if total_conductivity > 0.001 {
                let weighted_avg_temp = (current_temp * conductivity + neighbor_temp_sum) / total_conductivity;
                let mut delta_temp = (weighted_avg_temp - current_temp) * conductivity.min(0.5) * 0.8;
                
                // Apply inertia damping for specific materials
                if material == MaterialType::Lava || 
                   material == MaterialType::Stone || 
                   material == MaterialType::Glass || 
                   material == MaterialType::Ice {
                    delta_temp *= HIGH_INERTIA_DAMPING;
                }
                
                // Scale delta by time and clamp magnitude
                delta_temp = delta_temp.max(-50.0).min(50.0) * scaled_dt;
                
                new_temp = current_temp + delta_temp;
            }
        }
        
        // Apply ambient cooling and heat generation, scaled by time
        let properties = material.get_properties();
        new_temp += (AMBIENT_TEMP - new_temp) * COOLING_RATE * conductivity * scaled_dt;
        
        if let Some(heat_gen) = properties.heat_generation {
            if heat_gen > 0.0 {
                new_temp += heat_gen * scaled_dt;
            }
        }
        
        // Additional heat sources for specific materials
        if material == MaterialType::Fire {
            new_temp = new_temp.max(600.0); // Fire maintains high temp
        } else if material == MaterialType::Lava {
            new_temp = new_temp.max(1200.0); // Lava maintains high temp
        } else if material == MaterialType::Generator {
            new_temp = new_temp.max(300.0); // Generators produce heat
        } else if material == MaterialType::Ice {
            new_temp = new_temp.min(-5.0); // Ice stays cold
        }
        
        // Update temperature if changed significantly
        if (new_temp - current_temp).abs() > 0.01 {
            particle.temperature.set(new_temp);
        }
    }
    
    // Optimize temperature update for active area
    pub fn update_temperatures_optimized(
        &mut self,
        min_x: usize,
        max_x: usize,
        min_y: usize,
        max_y: usize,
        delta_time: f32,
        get_particle: impl Fn(usize, usize) -> Option<crate::particle::Particle>,
        get_particle_mut: impl FnMut(usize, usize) -> Option<&mut crate::particle::Particle>,
    ) {
        // Expand bounds slightly to include heat conduction beyond active area
        let x_start = min_x.saturating_sub(5);
        let x_end = (max_x + 5).min(GRID_WIDTH);
        let y_start = min_y.saturating_sub(5);
        let y_end = (max_y + 5).min(GRID_HEIGHT);
        
        // Store temperatures in a buffer to avoid issues with updating while processing
        let mut temp_buffer = vec![AMBIENT_TEMP; (x_end - x_start) * (y_end - y_start)];
        
        // First pass: calculate new temperatures
        for y in y_start..y_end {
            for x in x_start..x_end {
                if let Some(particle) = get_particle(x, y) {
                    // Skip empty space for performance
                    if particle.material == MaterialType::Empty {
                        continue;
                    }
                    
                    // Calculate buffer index
                    let idx = (y - y_start) * (x_end - x_start) + (x - x_start);
                    
                    // Get current temperature
                    let current_temp = particle.temperature.get();
                    
                    // Skip rigid/immovable materials if not burning/hot
                    if particle.material == MaterialType::Stone && 
                       (current_temp - AMBIENT_TEMP).abs() < 10.0 {
                        temp_buffer[idx] = current_temp;
                        continue;
                    }
                    
                    // Apply natural cooling
                    let mut new_temp = current_temp;
                    let properties = particle.material.get_properties();
                    let conductivity = properties.conductivity;
                    
                    // Collect neighbor temperatures
                    let mut neighbor_temp_sum = 0.0;
                    let mut neighbor_conductivity_sum = 0.0;
                    let mut neighbor_count = 0;
                    
                    // Check cardinal neighbors (simplified for performance)
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
                            let n_temp = neighbor.temperature.get();
                            let n_cond = neighbor.material.get_properties().conductivity;
                            
                            neighbor_temp_sum += n_temp * n_cond;
                            neighbor_conductivity_sum += n_cond;
                            neighbor_count += 1;
                        } else {
                            // Ambient temperature for out of bounds
                            neighbor_temp_sum += AMBIENT_TEMP * 0.1;
                            neighbor_conductivity_sum += 0.1;
                            neighbor_count += 1;
                        }
                    }
                    
                    // Calculate temperature change
                    if neighbor_count > 0 && (conductivity > 0.0 || neighbor_conductivity_sum > 0.0) {
                        let total_conductivity = conductivity + neighbor_conductivity_sum;
                        
                        if total_conductivity > 0.001 {
                            let weighted_avg_temp = (current_temp * conductivity + neighbor_temp_sum) / total_conductivity;
                            let mut delta_temp = (weighted_avg_temp - current_temp) * 0.5_f32.min(conductivity);
                            
                            // Scale by delta time
                            delta_temp *= delta_time * TARGET_DT_SCALING;
                            
                            new_temp += delta_temp;
                        }
                    }
                    
                    // Apply ambient cooling
                    new_temp += (AMBIENT_TEMP - new_temp) * COOLING_RATE * conductivity * delta_time * TARGET_DT_SCALING;
                    
                    // Apply heat generation if material generates heat
                    if let Some(heat_gen) = properties.heat_generation {
                        if heat_gen > 0.0 {
                            new_temp += heat_gen * delta_time * TARGET_DT_SCALING;
                        }
                    }
                    
                    // Special case temperatures
                    match particle.material {
                        MaterialType::Fire => new_temp = new_temp.max(600.0),
                        MaterialType::Lava => new_temp = new_temp.max(1200.0),
                        MaterialType::Generator => new_temp = new_temp.max(300.0),
                        MaterialType::Ice => new_temp = new_temp.min(-5.0),
                        _ => {}
                    }
                    
                    // Store in buffer
                    temp_buffer[idx] = new_temp;
                } else {
                    // Set ambient temperature for empty/non-existent cells
                    let idx = (y - y_start) * (x_end - x_start) + (x - x_start);
                    temp_buffer[idx] = AMBIENT_TEMP;
                }
            }
        }
        
        // Second pass: apply new temperatures
        let mut get_particle_mut = get_particle_mut;
        for y in y_start..y_end {
            for x in x_start..x_end {
                let idx = (y - y_start) * (x_end - x_start) + (x - x_start);
                let new_temp = temp_buffer[idx];
                
                if let Some(particle) = get_particle_mut(x, y) {
                    if particle.material != MaterialType::Empty {
                        let current_temp = particle.temperature.get();
                        
                        // Only update if temperature changed significantly
                        if (new_temp - current_temp).abs() > 0.01 {
                            particle.temperature.set(new_temp);
                        }
                    }
                }
            }
        }
    }
}