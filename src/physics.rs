use crate::particle::Particle;
use crate::materials::{get_material_properties, MaterialType};

const AMBIENT_TEMP: f32 = 20.0;
const COOLING_RATE: f32 = 0.005;
const FIRE_HEAT_TRANSFER: f32 = 60.0;
const WATER_COOLING_FACTOR: f32 = 80.0;
const PLANT_GROWTH_CHANCE_PER_SEC: f32 = 0.09;
const MAX_TEMP: f32 = 3000.0;
const DEFAULT_FIRE_LIFESPAN_SEC: f32 = 1.0;
const FUSE_BURN_LIFESPAN_SEC: f32 = 4.0;
const CONDENSATION_Y_LIMIT: usize = 5;
const CONDENSATION_CHANCE_ANYWHERE_PER_SEC: f32 = 0.006;
const PHASE_CHANGE_TEMP_BUFFER: f32 = 5.0;
const HIGH_INERTIA_DAMPING: f32 = 0.2;
const MIN_STATE_SECONDS: f32 = 10.0;
const TARGET_DT_SCALING: f32 = 60.0;
const ACID_GAS_TEMP_FACTOR: f32 = 0.8;

#[derive(Debug, Clone)]
pub struct PhysicsState {
    pub width: usize,
    pub height: usize,
}

impl PhysicsState {
    pub fn new(width: usize, height: usize) -> Self {
        Self { width, height }
    }

    pub fn is_valid(&self, x: i32, y: i32) -> bool {
        x >= 0 && (x as usize) < self.width && y >= 0 && (y as usize) < self.height
    }

    pub fn handle_lifespan_and_burnout(&self, particle: &mut Particle, delta_time: f32) -> Option<Particle> {
        let mut needs_check = false;
        let mut is_burning_fuse = false;

        if particle.material_type == MaterialType::Fuse && particle.burning {
            needs_check = true;
            is_burning_fuse = true;
            if particle.life.is_none() {
                particle.life = Some(FUSE_BURN_LIFESPAN_SEC);
            }
        } else if particle.life.is_some() {
            needs_check = true;
        }

        if needs_check {
            if let Some(life_val) = particle.life {
                let new_life = life_val - delta_time;
                particle.life = Some(new_life);
                
                if is_burning_fuse {
                    particle.temp = (particle.temp + 5.0 * delta_time * TARGET_DT_SCALING).min(MAX_TEMP);
                }
                particle.invalidate_color_cache();

                if new_life <= 0.0 {
                    let (new_type, new_temp) = match particle.material_type {
                        MaterialType::Fire => (MaterialType::Smoke, (particle.temp * 0.6).min(400.0)),
                        MaterialType::Fuse => (MaterialType::Ash, (particle.temp * 0.5).max(AMBIENT_TEMP)),
                        MaterialType::Steam | MaterialType::Smoke | MaterialType::ToxicGas => {
                            (MaterialType::Empty, AMBIENT_TEMP)
                        }
                        _ => return None,
                    };

                    return Some(Particle::new(particle.x, particle.y, new_type, Some(new_temp)));
                }
            }
        }
        None
    }

    pub fn handle_state_changes_and_effects(
        &self,
        particle: &mut Particle,
        neighbors: &[Option<&Particle>],
        delta_time: f32,
    ) -> (Option<Particle>, Vec<(usize, usize, Particle)>) {
        let mut new_particles = Vec::new();
        let props = particle.get_properties();
        let dt_scale = delta_time * TARGET_DT_SCALING;

        // Ignition check
        if let Some(ignition_temp) = props.ignition_temp {
            if particle.temp >= ignition_temp && props.flammability > 0.0 {
                let mut external_ignition = false;
                let mut ignition_source_temp = particle.temp;

                // Check for ignition sources in neighbors
                for neighbor in neighbors.iter().flatten() {
                    match neighbor.material_type {
                        MaterialType::Fire | MaterialType::Lava => {
                            external_ignition = true;
                            ignition_source_temp = ignition_source_temp.max(neighbor.temp);
                            break;
                        }
                        MaterialType::Fuse if neighbor.burning => {
                            external_ignition = true;
                            ignition_source_temp = ignition_source_temp.max(neighbor.temp);
                            break;
                        }
                        _ => {}
                    }
                }

                match particle.material_type {
                    MaterialType::Plant | MaterialType::Wood | MaterialType::Coal | 
                    MaterialType::Oil | MaterialType::Gasoline => {
                        if external_ignition || particle.temp > ignition_temp + 100.0 {
                            let initial_fire_temp = ignition_source_temp.max(800.0);
                            let initial_fire_life = match particle.material_type {
                                MaterialType::Wood => 3.0,
                                MaterialType::Coal => 4.0,
                                _ => DEFAULT_FIRE_LIFESPAN_SEC,
                            };
                            let mut new_particle = Particle::new(
                                particle.x, 
                                particle.y, 
                                MaterialType::Fire, 
                                Some(initial_fire_temp)
                            );
                            new_particle.life = Some(initial_fire_life);
                            return (Some(new_particle), new_particles);
                        }
                    }
                    MaterialType::Gunpowder => {
                        if external_ignition || particle.temp > ignition_temp {
                            // Handle explosion
                            let explosion_particles = self.create_explosion(
                                particle.x, particle.y, props.explosive_yield.unwrap_or(4.0)
                            );
                            new_particles.extend(explosion_particles);
                            return (Some(Particle::new(particle.x, particle.y, MaterialType::Empty, None)), new_particles);
                        }
                    }
                    MaterialType::Fuse if !particle.burning => {
                        if external_ignition {
                            particle.burning = true;
                            particle.life = Some(FUSE_BURN_LIFESPAN_SEC);
                            particle.temp = particle.temp.max(ignition_temp + 50.0);
                            particle.invalidate_color_cache();
                        }
                    }
                    _ => {}
                }
            }
        }

        // Melting check
        if let Some(melt_temp) = props.melt_temp {
            if particle.temp >= melt_temp + PHASE_CHANGE_TEMP_BUFFER {
                let new_type = match particle.material_type {
                    MaterialType::Sand => MaterialType::Glass,
                    MaterialType::Glass => MaterialType::Lava,
                    MaterialType::Ice => MaterialType::Water,
                    _ => return (None, new_particles),
                };
                return (Some(Particle::new(particle.x, particle.y, new_type, Some(particle.temp))), new_particles);
            }
        }

        // Boiling check
        if let Some(boil_temp) = props.boil_temp {
            if particle.temp >= boil_temp + PHASE_CHANGE_TEMP_BUFFER {
                let new_type = match particle.material_type {
                    MaterialType::Water => MaterialType::Steam,
                    MaterialType::Acid => MaterialType::ToxicGas,
                    MaterialType::Slime => MaterialType::ToxicGas,
                    _ => return (None, new_particles),
                };
                return (Some(Particle::new(particle.x, particle.y, new_type, Some(particle.temp))), new_particles);
            }
        }

        // Freezing/Condensation check
        if let Some(freeze_temp) = props.freeze_temp {
            if particle.temp <= freeze_temp - PHASE_CHANGE_TEMP_BUFFER {
                let new_type = match particle.material_type {
                    MaterialType::Lava => MaterialType::Stone,
                    MaterialType::Water => MaterialType::Ice,
                    MaterialType::Steam if particle.time_in_state >= MIN_STATE_SECONDS => {
                        let condensation_chance = if particle.y < CONDENSATION_Y_LIMIT {
                            1.0
                        } else {
                            CONDENSATION_CHANCE_ANYWHERE_PER_SEC * delta_time
                        };
                        if rand::random::<f32>() < condensation_chance {
                            MaterialType::Water
                        } else {
                            return (None, new_particles);
                        }
                    }
                    _ => return (None, new_particles),
                };
                return (Some(Particle::new(particle.x, particle.y, new_type, Some(particle.temp))), new_particles);
            }
        }

        // Material-specific effects
        match particle.material_type {
            MaterialType::Fire => {
                // Fire effects handled in separate function
            }
            MaterialType::Acid => {
                if props.corrosive_power > 0.0 {
                    // Handle acid corrosion
                    for (i, neighbor) in neighbors.iter().enumerate() {
                        if let Some(neighbor) = neighbor {
                            let immune_materials = [
                                MaterialType::Empty, MaterialType::Acid, 
                                MaterialType::Glass, MaterialType::Generator
                            ];
                            if !immune_materials.contains(&neighbor.material_type) {
                                if rand::random::<f32>() < props.corrosive_power * dt_scale {
                                    let (nx, ny) = self.get_neighbor_coords(particle.x, particle.y, i);
                                    if neighbor.material_type == MaterialType::Stone && rand::random::<f32>() < 0.3 {
                                        new_particles.push((nx, ny, Particle::new(nx, ny, MaterialType::Sand, Some(neighbor.temp))));
                                    } else {
                                        new_particles.push((nx, ny, Particle::new(nx, ny, MaterialType::Empty, None)));
                                        // Create toxic gas
                                        let gas_temp = particle.temp * ACID_GAS_TEMP_FACTOR;
                                        if ny > 0 {
                                            new_particles.push((nx, ny - 1, Particle::new(nx, ny - 1, MaterialType::ToxicGas, Some(gas_temp))));
                                        }
                                    }
                                    if rand::random::<f32>() < 0.05 * dt_scale {
                                        return (Some(Particle::new(particle.x, particle.y, MaterialType::Empty, None)), new_particles);
                                    }
                                    break;
                                }
                            }
                        }
                    }
                }
            }
            MaterialType::Plant => {
                // Plant growth logic
                let mut has_adjacent_water = false;
                let mut empty_neighbors = Vec::new();
                
                for (i, neighbor) in neighbors.iter().enumerate() {
                    if let Some(neighbor) = neighbor {
                        if neighbor.material_type == MaterialType::Water {
                            has_adjacent_water = true;
                        }
                    } else {
                        empty_neighbors.push(i);
                    }
                }

                if has_adjacent_water && !empty_neighbors.is_empty() && 
                   AMBIENT_TEMP < particle.temp && particle.temp < 50.0 {
                    if rand::random::<f32>() < PLANT_GROWTH_CHANCE_PER_SEC * delta_time {
                        let neighbor_idx = empty_neighbors[rand::random::<usize>() % empty_neighbors.len()];
                        let (nx, ny) = self.get_neighbor_coords(particle.x, particle.y, neighbor_idx);
                        new_particles.push((nx, ny, Particle::new(nx, ny, MaterialType::Plant, Some(particle.temp))));
                    }
                }
            }
            _ => {}
        }

        (None, new_particles)
    }

    fn get_neighbor_coords(&self, x: usize, y: usize, neighbor_index: usize) -> (usize, usize) {
        let offsets = [
            (-1, -1), (0, -1), (1, -1),
            (-1,  0),          (1,  0),
            (-1,  1), (0,  1), (1,  1),
        ];
        let (dx, dy) = offsets[neighbor_index];
        ((x as i32 + dx) as usize, (y as i32 + dy) as usize)
    }

    fn create_explosion(&self, cx: usize, cy: usize, radius: f32) -> Vec<(usize, usize, Particle)> {
        let mut explosion_particles = Vec::new();
        let radius_sq = radius * radius;

        for dx in -(radius as i32)..=(radius as i32) {
            for dy in -(radius as i32)..=(radius as i32) {
                let dist_sq = (dx * dx + dy * dy) as f32;
                if dist_sq <= radius_sq {
                    let px = cx as i32 + dx;
                    let py = cy as i32 + dy;
                    if self.is_valid(px, py) {
                        let explosion_strength = (1.0 - (dist_sq.sqrt() / radius)).max(0.0);
                        if rand::random::<f32>() < explosion_strength * 0.95 {
                            if rand::random::<f32>() < 0.6 * explosion_strength {
                                let mut fire_particle = Particle::new(px as usize, py as usize, MaterialType::Fire, Some(800.0 + explosion_strength * 700.0));
                                fire_particle.life = Some(DEFAULT_FIRE_LIFESPAN_SEC * explosion_strength * 0.5);
                                explosion_particles.push((px as usize, py as usize, fire_particle));
                            } else {
                                let mut smoke_particle = Particle::new(px as usize, py as usize, MaterialType::Smoke, Some(400.0 * explosion_strength));
                                smoke_particle.life = Some(3.0 * explosion_strength);
                                explosion_particles.push((px as usize, py as usize, smoke_particle));
                            }
                        }
                    }
                }
            }
        }

        explosion_particles
    }

    pub fn update_temperature(&self, particle: &mut Particle, neighbors: &[Option<&Particle>], delta_time: f32) {
        if particle.material_type == MaterialType::Empty {
            return;
        }

        let props = particle.get_properties();
        let mut conductivity = props.conductivity;
        let dt_scale = delta_time * TARGET_DT_SCALING;

        // Adjust conductivity for specific materials
        match particle.material_type {
            MaterialType::Generator => conductivity *= 0.1,
            MaterialType::Stone | MaterialType::Glass => conductivity *= 0.3,
            _ => {}
        }

        let mut neighbor_temp_sum = 0.0;
        let mut neighbor_conductivity_sum = 0.0;
        let mut neighbor_count = 0;

        // Accumulate temperature and conductivity from neighbors
        for neighbor in neighbors.iter() {
            let (neighbor_temp, neighbor_conductivity) = if let Some(neighbor) = neighbor {
                (neighbor.temp, neighbor.get_properties().conductivity)
            } else {
                (AMBIENT_TEMP, get_material_properties(MaterialType::Empty).conductivity)
            };

            neighbor_temp_sum += neighbor_temp * neighbor_conductivity;
            neighbor_conductivity_sum += neighbor_conductivity;
            neighbor_count += 1;
        }

        let mut new_temp = particle.temp;

        // Calculate temperature change based on neighbors
        if neighbor_count > 0 && (conductivity > 0.0 || neighbor_conductivity_sum > 0.0) {
            let total_conductivity = conductivity + neighbor_conductivity_sum;
            if total_conductivity > 0.001 {
                let weighted_avg_temp = (particle.temp * conductivity + neighbor_temp_sum) / total_conductivity;
                let mut delta_temp = (weighted_avg_temp - particle.temp) * (conductivity * 0.8).min(0.5);

                // Apply inertia damping for specific materials
                if matches!(
                    particle.material_type,
                    MaterialType::Lava | MaterialType::Stone | MaterialType::Glass | MaterialType::Ice
                ) {
                    delta_temp *= HIGH_INERTIA_DAMPING;
                }

                // Scale delta by time and clamp magnitude
                delta_temp = delta_temp.max(-50.0).min(50.0) * dt_scale;
                new_temp = particle.temp + delta_temp;
            }
        }

        // Apply ambient cooling and heat generation
        new_temp += (AMBIENT_TEMP - new_temp) * COOLING_RATE * conductivity * dt_scale;
        if props.heat_generation > 0.0 {
            new_temp += props.heat_generation * dt_scale;
        }

        // Clamp temperature and update particle if changed
        new_temp = new_temp.max(-273.15).min(MAX_TEMP);
        if (new_temp - particle.temp).abs() > 0.01 {
            particle.temp = new_temp;
            particle.invalidate_color_cache();
        }
    }
}