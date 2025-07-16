use crate::materials::MaterialType;
use crate::particle::Particle;
use ahash::AHashMap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Tile entity system for complex objects that need more than just material data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TileEntity {
    pub tile_type: TileEntityType,
    pub position: (i64, i64),
    pub data: TileEntityData,
    pub active: bool,
    pub update_timer: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TileEntityType {
    Chest,
    Furnace,
    Generator,
    Pipe,
    Pump,
    Torch,
    Spawner,
    Reactor,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TileEntityData {
    Chest {
        inventory: HashMap<MaterialType, u32>,
        max_capacity: u32,
    },
    Furnace {
        fuel: Option<MaterialType>,
        fuel_amount: u32,
        input_material: Option<MaterialType>,
        input_amount: u32,
        output_material: Option<MaterialType>,
        output_amount: u32,
        temperature: f32,
        smelting_progress: f32,
    },
    Generator {
        fuel: Option<MaterialType>,
        fuel_amount: u32,
        power_output: f32,
        efficiency: f32,
        heat_generation: f32,
    },
    Pipe {
        fluid_type: Option<MaterialType>,
        fluid_amount: u32,
        flow_rate: f32,
        pressure: f32,
        connections: Vec<(i64, i64)>, // Connected pipe positions
    },
    Pump {
        input_fluid: Option<MaterialType>,
        output_fluid: Option<MaterialType>,
        flow_rate: f32,
        power_consumption: f32,
        suction_range: u32,
    },
    Torch {
        fuel_type: MaterialType,
        fuel_remaining: f32,
        light_radius: u32,
        heat_output: f32,
    },
    Spawner {
        spawn_material: MaterialType,
        spawn_rate: f32,
        spawn_amount: u32,
        spawn_radius: u32,
        energy_cost: f32,
    },
    Reactor {
        fuel_rods: Vec<(MaterialType, f32)>, // (material, remaining_fuel)
        moderator: Option<MaterialType>,
        coolant: Option<MaterialType>,
        temperature: f32,
        pressure: f32,
        power_output: f32,
        waste_products: HashMap<MaterialType, u32>,
    },
    Custom {
        properties: HashMap<String, String>,
    },
}

impl TileEntity {
    pub fn new_chest(position: (i64, i64), capacity: u32) -> Self {
        Self {
            tile_type: TileEntityType::Chest,
            position,
            data: TileEntityData::Chest {
                inventory: HashMap::new(),
                max_capacity: capacity,
            },
            active: true,
            update_timer: 0.0,
        }
    }

    pub fn new_furnace(position: (i64, i64)) -> Self {
        Self {
            tile_type: TileEntityType::Furnace,
            position,
            data: TileEntityData::Furnace {
                fuel: None,
                fuel_amount: 0,
                input_material: None,
                input_amount: 0,
                output_material: None,
                output_amount: 0,
                temperature: 20.0,
                smelting_progress: 0.0,
            },
            active: true,
            update_timer: 0.0,
        }
    }

    pub fn new_generator(position: (i64, i64), power_output: f32) -> Self {
        Self {
            tile_type: TileEntityType::Generator,
            position,
            data: TileEntityData::Generator {
                fuel: None,
                fuel_amount: 0,
                power_output,
                efficiency: 1.0,
                heat_generation: power_output * 0.1,
            },
            active: true,
            update_timer: 0.0,
        }
    }

    pub fn new_torch(position: (i64, i64)) -> Self {
        Self {
            tile_type: TileEntityType::Torch,
            position,
            data: TileEntityData::Torch {
                fuel_type: MaterialType::Wood,
                fuel_remaining: 100.0,
                light_radius: 8,
                heat_output: 50.0,
            },
            active: true,
            update_timer: 0.0,
        }
    }

    pub fn new_spawner(position: (i64, i64), material: MaterialType, rate: f32) -> Self {
        Self {
            tile_type: TileEntityType::Spawner,
            position,
            data: TileEntityData::Spawner {
                spawn_material: material,
                spawn_rate: rate,
                spawn_amount: 1,
                spawn_radius: 2,
                energy_cost: 1.0,
            },
            active: true,
            update_timer: 0.0,
        }
    }

    /// Update the tile entity logic
    pub fn update(&mut self, delta_time: f32, surrounding_particles: &[(i64, i64, &Particle)]) -> Vec<TileEntityEffect> {
        self.update_timer += delta_time;
        
        if !self.active {
            return Vec::new();
        }

        match &self.data {
            TileEntityData::Furnace { temperature, fuel_amount, smelting_progress, .. } => {
                Self::update_furnace_static(delta_time, *temperature, *fuel_amount, *smelting_progress)
            },
            TileEntityData::Generator { fuel_amount, heat_generation, .. } => {
                Self::update_generator_static(delta_time, *fuel_amount, *heat_generation, surrounding_particles)
            },
            TileEntityData::Torch { fuel_remaining, heat_output, light_radius, .. } => {
                Self::update_torch_static(delta_time, *fuel_remaining, *heat_output, *light_radius)
            },
            TileEntityData::Spawner { spawn_material, spawn_rate, spawn_amount, spawn_radius, .. } => {
                Self::update_spawner_static(delta_time, *spawn_material, *spawn_rate, *spawn_amount, *spawn_radius)
            },
            TileEntityData::Reactor { temperature, pressure, power_output, .. } => {
                Self::update_reactor_static(delta_time, *temperature, *pressure, *power_output)
            },
            _ => Vec::new(),
        }
    }

    fn update_furnace(&mut self, delta_time: f32, temperature: &mut f32, fuel_amount: &mut u32, smelting_progress: &mut f32) -> Vec<TileEntityEffect> {
        let mut effects = Vec::new();

        // Consume fuel to maintain temperature
        if *fuel_amount > 0 && *temperature < 1000.0 {
            *fuel_amount = fuel_amount.saturating_sub(1);
            *temperature += 100.0 * delta_time;
            effects.push(TileEntityEffect::HeatGeneration {
                position: self.position,
                heat_amount: 50.0,
                radius: 3,
            });
        } else {
            // Cool down when no fuel
            *temperature -= 20.0 * delta_time;
            *temperature = temperature.max(20.0);
        }

        // Smelting logic
        if *temperature > 500.0 {
            *smelting_progress += delta_time * 0.1;
            if *smelting_progress >= 1.0 {
                *smelting_progress = 0.0;
                effects.push(TileEntityEffect::MaterialConversion {
                    position: self.position,
                    from_material: MaterialType::Sand, // Example conversion
                    to_material: MaterialType::Glass,
                    amount: 1,
                });
            }
        }

        effects
    }

    fn update_generator(&mut self, delta_time: f32, fuel_amount: &mut u32, heat_generation: f32, _surrounding_particles: &[(i64, i64, &Particle)]) -> Vec<TileEntityEffect> {
        let mut effects = Vec::new();

        if *fuel_amount > 0 {
            *fuel_amount = fuel_amount.saturating_sub(1);
            effects.push(TileEntityEffect::HeatGeneration {
                position: self.position,
                heat_amount: heat_generation,
                radius: 5,
            });
            effects.push(TileEntityEffect::ParticleSpawn {
                position: (self.position.0, self.position.1 - 1),
                material: MaterialType::Smoke,
                amount: 1,
            });
        }

        effects
    }

    fn update_torch(&mut self, delta_time: f32, fuel_remaining: &mut f32, heat_output: f32, light_radius: u32) -> Vec<TileEntityEffect> {
        let mut effects = Vec::new();

        if *fuel_remaining > 0.0 {
            *fuel_remaining -= delta_time;
            effects.push(TileEntityEffect::LightGeneration {
                position: self.position,
                intensity: 1.0,
                radius: light_radius,
            });
            effects.push(TileEntityEffect::HeatGeneration {
                position: self.position,
                heat_amount: heat_output,
                radius: 2,
            });
            
            // Occasional spark particles
            if rand::random::<f32>() < 0.1 {
                effects.push(TileEntityEffect::ParticleSpawn {
                    position: (self.position.0 + rand::random::<i64>() % 3 - 1, self.position.1 - 1),
                    material: MaterialType::Fire,
                    amount: 1,
                });
            }
        } else {
            self.active = false;
        }

        effects
    }

    fn update_spawner(&mut self, delta_time: f32, spawn_material: MaterialType, spawn_rate: f32, spawn_amount: u32, spawn_radius: u32) -> Vec<TileEntityEffect> {
        let mut effects = Vec::new();

        if self.update_timer >= 1.0 / spawn_rate {
            self.update_timer = 0.0;
            
            for _ in 0..spawn_amount {
                let offset_x = (rand::random::<i64>() % (spawn_radius as i64 * 2 + 1)) - spawn_radius as i64;
                let offset_y = (rand::random::<i64>() % (spawn_radius as i64 * 2 + 1)) - spawn_radius as i64;
                
                effects.push(TileEntityEffect::ParticleSpawn {
                    position: (self.position.0 + offset_x, self.position.1 + offset_y),
                    material: spawn_material,
                    amount: 1,
                });
            }
        }

        effects
    }

    fn update_reactor(&mut self, _delta_time: f32, temperature: &mut f32, pressure: &mut f32, power_output: &mut f32) -> Vec<TileEntityEffect> {
        let mut effects = Vec::new();

        // Simplified reactor physics
        *temperature += 10.0; // Heat generation from nuclear reactions
        *pressure = *temperature / 100.0;
        *power_output = *temperature * 0.1;

        // Safety systems
        if *temperature > 2000.0 {
            effects.push(TileEntityEffect::Explosion {
                position: self.position,
                radius: 10,
                power: (*temperature / 100.0) as u32,
            });
            self.active = false;
        }

        effects
    }

    /// Add items to a chest
    pub fn add_to_inventory(&mut self, material: MaterialType, amount: u32) -> u32 {
        if let TileEntityData::Chest { inventory, max_capacity } = &mut self.data {
            let current_total: u32 = inventory.values().sum();
            let can_add = (*max_capacity).saturating_sub(current_total).min(amount);
            
            if can_add > 0 {
                *inventory.entry(material).or_insert(0) += can_add;
            }
            can_add
        } else {
            0
        }
    }

    /// Remove items from inventory
    pub fn remove_from_inventory(&mut self, material: MaterialType, amount: u32) -> u32 {
        if let TileEntityData::Chest { inventory, .. } = &mut self.data {
            if let Some(current_amount) = inventory.get_mut(&material) {
                let can_remove = (*current_amount).min(amount);
                *current_amount -= can_remove;
                if *current_amount == 0 {
                    inventory.remove(&material);
                }
                can_remove
            } else {
                0
            }
        } else {
            0
        }
    }

    pub fn get_position(&self) -> (i64, i64) {
        self.position
    }

    pub fn is_active(&self) -> bool {
        self.active
    }

    pub fn set_active(&mut self, active: bool) {
        self.active = active;
    }

    // Static helper methods to avoid borrowing issues
    fn update_furnace_static(_delta_time: f32, temperature: f32, fuel_amount: u32, smelting_progress: f32) -> Vec<TileEntityEffect> {
        let mut effects = Vec::new();
        
        if fuel_amount > 0 {
            effects.push(TileEntityEffect::HeatGeneration {
                position: (0, 0), // Position will be set by caller
                heat_amount: temperature,
                radius: 3,
            });
        }
        
        effects
    }

    fn update_generator_static(_delta_time: f32, fuel_amount: u32, heat_generation: f32, _surrounding_particles: &[(i64, i64, &crate::particle::Particle)]) -> Vec<TileEntityEffect> {
        let mut effects = Vec::new();
        
        if fuel_amount > 0 {
            effects.push(TileEntityEffect::HeatGeneration {
                position: (0, 0), // Position will be set by caller
                heat_amount: heat_generation,
                radius: 5,
            });
        }
        
        effects
    }

    fn update_torch_static(_delta_time: f32, fuel_remaining: f32, heat_output: f32, light_radius: u32) -> Vec<TileEntityEffect> {
        let mut effects = Vec::new();
        
        if fuel_remaining > 0.0 {
            effects.push(TileEntityEffect::HeatGeneration {
                position: (0, 0), // Position will be set by caller
                heat_amount: heat_output,
                radius: 2,
            });
            
            effects.push(TileEntityEffect::LightGeneration {
                position: (0, 0), // Position will be set by caller
                intensity: fuel_remaining / 100.0,
                radius: light_radius,
            });
        }
        
        effects
    }

    fn update_spawner_static(_delta_time: f32, spawn_material: MaterialType, spawn_rate: f32, spawn_amount: u32, spawn_radius: u32) -> Vec<TileEntityEffect> {
        let mut effects = Vec::new();
        
        // Simplified spawning logic
        if spawn_rate > 0.0 {
            effects.push(TileEntityEffect::ParticleSpawn {
                position: (0, 0), // Position will be set by caller
                material: spawn_material,
                amount: spawn_amount,
            });
        }
        
        effects
    }

    fn update_reactor_static(_delta_time: f32, temperature: f32, pressure: f32, power_output: f32) -> Vec<TileEntityEffect> {
        let mut effects = Vec::new();
        
        if temperature > 1000.0 && pressure > 50.0 {
            effects.push(TileEntityEffect::HeatGeneration {
                position: (0, 0), // Position will be set by caller
                heat_amount: power_output,
                radius: 8,
            });
        }
        
        effects
    }
}

/// Effects that tile entities can produce
#[derive(Debug, Clone)]
pub enum TileEntityEffect {
    ParticleSpawn {
        position: (i64, i64),
        material: MaterialType,
        amount: u32,
    },
    HeatGeneration {
        position: (i64, i64),
        heat_amount: f32,
        radius: u32,
    },
    LightGeneration {
        position: (i64, i64),
        intensity: f32,
        radius: u32,
    },
    MaterialConversion {
        position: (i64, i64),
        from_material: MaterialType,
        to_material: MaterialType,
        amount: u32,
    },
    Explosion {
        position: (i64, i64),
        radius: u32,
        power: u32,
    },
    FluidFlow {
        from_position: (i64, i64),
        to_position: (i64, i64),
        material: MaterialType,
        amount: u32,
    },
}

/// Manager for all tile entities in the world
#[derive(Debug, Default)]
pub struct TileEntityManager {
    entities: AHashMap<(i64, i64), TileEntity>,
    update_order: Vec<(i64, i64)>,
}

impl TileEntityManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_tile_entity(&mut self, tile_entity: TileEntity) {
        let position = tile_entity.position;
        self.entities.insert(position, tile_entity);
        self.update_order.push(position);
    }

    pub fn remove_tile_entity(&mut self, position: (i64, i64)) -> Option<TileEntity> {
        self.update_order.retain(|&pos| pos != position);
        self.entities.remove(&position)
    }

    pub fn get_tile_entity(&self, position: (i64, i64)) -> Option<&TileEntity> {
        self.entities.get(&position)
    }

    pub fn get_tile_entity_mut(&mut self, position: (i64, i64)) -> Option<&mut TileEntity> {
        self.entities.get_mut(&position)
    }

    /// Update all tile entities and return their effects
    pub fn update_all(&mut self, delta_time: f32, get_surrounding_particles: impl Fn((i64, i64)) -> Vec<(i64, i64, Particle)>) -> Vec<TileEntityEffect> {
        let mut all_effects = Vec::new();
        
        for &position in &self.update_order.clone() {
            if let Some(tile_entity) = self.entities.get_mut(&position) {
                let surrounding = get_surrounding_particles(position);
                let surrounding_refs: Vec<(i64, i64, &Particle)> = surrounding.iter()
                    .map(|(x, y, p)| (*x, *y, p))
                    .collect();
                
                let effects = tile_entity.update(delta_time, &surrounding_refs);
                all_effects.extend(effects);
                
                // Remove inactive tile entities
                if !tile_entity.is_active() {
                    self.remove_tile_entity(position);
                }
            }
        }
        
        all_effects
    }

    pub fn get_all_positions(&self) -> impl Iterator<Item = (i64, i64)> + '_ {
        self.entities.keys().copied()
    }

    pub fn count(&self) -> usize {
        self.entities.len()
    }

    pub fn clear(&mut self) {
        self.entities.clear();
        self.update_order.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tile_entity_creation() {
        let chest = TileEntity::new_chest((10, 10), 100);
        assert_eq!(chest.position, (10, 10));
        assert!(chest.is_active());
        
        if let TileEntityData::Chest { max_capacity, .. } = chest.data {
            assert_eq!(max_capacity, 100);
        } else {
            panic!("Expected chest data");
        }
    }

    #[test]
    fn test_chest_inventory() {
        let mut chest = TileEntity::new_chest((0, 0), 10);
        
        // Add items
        let added = chest.add_to_inventory(MaterialType::Sand, 5);
        assert_eq!(added, 5);
        
        // Try to add more than capacity
        let added = chest.add_to_inventory(MaterialType::Water, 8);
        assert_eq!(added, 5); // Should only add 5 to reach capacity
        
        // Remove items
        let removed = chest.remove_from_inventory(MaterialType::Sand, 3);
        assert_eq!(removed, 3);
    }

    #[test]
    fn test_tile_entity_manager() {
        let mut manager = TileEntityManager::new();
        
        let torch = TileEntity::new_torch((5, 5));
        manager.add_tile_entity(torch);
        
        assert_eq!(manager.count(), 1);
        assert!(manager.get_tile_entity((5, 5)).is_some());
        
        let removed = manager.remove_tile_entity((5, 5));
        assert!(removed.is_some());
        assert_eq!(manager.count(), 0);
    }

    #[test]
    fn test_furnace_update() {
        let mut furnace = TileEntity::new_furnace((0, 0));
        
        // Add fuel to the furnace manually for testing
        if let TileEntityData::Furnace { fuel_amount, .. } = &mut furnace.data {
            *fuel_amount = 100;
        }
        
        let effects = furnace.update(1.0, &[]);
        assert!(!effects.is_empty());
        
        // Should generate heat
        assert!(effects.iter().any(|effect| matches!(effect, TileEntityEffect::HeatGeneration { .. })));
    }
}