// particle.rs - Particle data structure and behaviors

use crate::constants::*;
use crate::material_properties::MaterialType;
use crate::temperature::Temperature;

#[derive(Clone)]
pub struct Particle {
    // Core properties
    pub material: MaterialType,
    pub temperature: Temperature,
    pub processed: bool,
    pub moved_this_step: bool,
    
    // State tracking
    pub life_remaining: Option<f32>, // Life in seconds (for ephemeral particles)
    pub time_in_state: f32, // Time in current state (for tracking phase changes)
    pub burning: bool, // For burnable materials (fuse, wood, etc.)
    
    // Physics properties
    pub vel_x: f32,
    pub vel_y: f32,
}

impl Particle {
    pub fn new(material: MaterialType, temp: f32) -> Self {
        let mut particle = Self {
            material,
            temperature: Temperature::new(temp),
            processed: false,
            moved_this_step: false,
            life_remaining: None,
            time_in_state: 0.0,
            burning: false,
            vel_x: 0.0,
            vel_y: 0.0,
        };
        
        // Initialize properties based on material type
        particle.initialize_properties();
        
        particle
    }
    
    pub fn new_with_velocity(material: MaterialType, temp: f32, vel_x: f32, vel_y: f32) -> Self {
        let mut particle = Self::new(material, temp);
        particle.vel_x = vel_x;
        particle.vel_y = vel_y;
        particle
    }
    
    // Reset processed state for new simulation step
    pub fn reset_processed(&mut self) {
        self.processed = false;
        self.moved_this_step = false;
    }
    
    // Initialize properties based on material type
    pub fn initialize_properties(&mut self) {
        // Set appropriate initial temperature based on material
        match self.material {
            MaterialType::Fire => {
                self.temperature.set(800.0_f32.max(self.temperature.get()));
                self.life_remaining = Some(FIRE_LIFESPAN);
            },
            MaterialType::Lava => {
                self.temperature.set(1500.0_f32.max(self.temperature.get()));
            },
            MaterialType::Steam => {
                self.temperature.set(110.0_f32.max(self.temperature.get()));
                self.life_remaining = Some(STEAM_LIFESPAN);
            },
            MaterialType::Smoke => {
                self.life_remaining = Some(SMOKE_LIFESPAN);
            },
            MaterialType::Ice => {
                self.temperature.set((-5.0_f32).min(self.temperature.get()));
            },
            _ => {
                // Other materials just keep their temperature
            }
        }
        
        // Reset time in state
        self.time_in_state = 0.0;
    }
    
    // Change material type and handle any related property changes
    pub fn change_material(&mut self, new_material: MaterialType, new_temp: Option<f32>) {
        let old_material = self.material;
        let old_temp = self.temperature.get();
        
        // Update material
        self.material = new_material;
        
        // Update temperature if provided
        if let Some(temp) = new_temp {
            self.temperature.set(temp);
        }
        
        // Re-initialize properties for the new material
        self.initialize_properties();
        
        // Handle temperature transitions if no specific temp was provided
        if new_temp.is_none() {
            self.apply_phase_change_temperature(old_material, old_temp);
        }
    }
    
    // Adjust temperature for phase changes when no explicit temperature is given
    fn apply_phase_change_temperature(&mut self, old_material: MaterialType, old_temp: f32) {
        match (old_material, self.material) {
            (MaterialType::Steam, MaterialType::Water) => {
                // Steam condensing to water
                self.temperature.set((old_temp - 20.0).max(AMBIENT_TEMP).min(99.0));
            },
            (MaterialType::Ice, MaterialType::Water) => {
                // Ice melting to water
                self.temperature.set((old_temp + 5.0).max(1.0).min(AMBIENT_TEMP));
            },
            (MaterialType::Glass, MaterialType::Lava) => {
                // Glass melting to lava
                self.temperature.set((old_temp + 50.0).max(1800.0));
            },
            (MaterialType::Lava, MaterialType::Stone) => {
                // Lava cooling to stone
                self.temperature.set((old_temp - 100.0).min(999.0));
            },
            (MaterialType::Sand, MaterialType::Glass) => {
                // Sand melting to glass
                self.temperature.set((old_temp + 20.0).max(1500.0));
            },
            (MaterialType::Fuse, MaterialType::Ash) => {
                // Burned fuse to ash
                self.temperature.set(old_temp * 0.5);
            },
            (MaterialType::Fire, MaterialType::Smoke) => {
                // Fire burning out to smoke
                self.temperature.set(old_temp * 0.6);
            },
            _ => {
                // Default: keep the old temperature
                self.temperature.set(old_temp);
            }
        }
    }
    
    // Update life (for particles with limited lifespan)
    pub fn update_life(&mut self, delta_time: f32) -> bool {
        if let Some(life) = &mut self.life_remaining {
            *life -= delta_time;
            
            // Handle burning fuse
            if self.material == MaterialType::Fuse && self.burning {
                self.temperature.add(5.0 * delta_time * TARGET_DT_SCALING);
            }
            
            // Return true if life is depleted
            if *life <= 0.0 {
                return true;
            }
        }
        false
    }
    
    // Get appropriate successor material when life depletes
    pub fn get_successor_material(&self) -> (MaterialType, f32) {
        match self.material {
            MaterialType::Fire => (MaterialType::Smoke, self.temperature.get() * 0.6),
            MaterialType::Fuse => (MaterialType::Ash, self.temperature.get() * 0.5),
            MaterialType::Steam => (MaterialType::Empty, AMBIENT_TEMP),
            MaterialType::Smoke => (MaterialType::Empty, AMBIENT_TEMP),
            _ => (MaterialType::Empty, AMBIENT_TEMP),
        }
    }
    
    // Increase time in current state
    pub fn increment_time_in_state(&mut self, delta_time: f32) {
        self.time_in_state += delta_time;
    }
    
    // Check if particle is lighter than another material type
    pub fn is_lighter_than(&self, other_material: MaterialType) -> bool {
        let self_props = self.material.get_properties();
        let other_props = other_material.get_properties();
        
        self_props.density < other_props.density
    }
    
    // Check if particle is heavier than another material type
    pub fn is_heavier_than(&self, other_material: MaterialType) -> bool {
        let self_props = self.material.get_properties();
        let other_props = other_material.get_properties();
        
        self_props.density > other_props.density
    }
}