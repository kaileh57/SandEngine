//! Defines the `Particle` struct, representing a single element in the simulation.
//!
//! Each particle has properties like its material type, temperature, and state flags.
//! It interacts with other particles and the environment based on these properties and
//! the rules defined in the `simulation_engine`.

use crate::material::{MaterialType, MaterialProperties, get_material_properties};
use rand; // For random numbers, e.g., in fire flicker

// Constants
/// Default ambient temperature for new particles and surroundings.
const AMBIENT_TEMP: f32 = 20.0;
/// Maximum temperature a particle can reach.
const MAX_TEMP: f32 = 3000.0;

/// Represents a single particle in the simulation grid.
///
/// Particles have a type, temperature, and other state information that dictates
/// their behavior and appearance.
#[derive(Debug, Clone)]
pub struct Particle {
    /// The x-coordinate of the particle in the grid. Can be read for game logic or rendering.
    pub x: i32,
    /// The y-coordinate of the particle in the grid. Can be read for game logic or rendering.
    pub y: i32,
    /// The type of material this particle is made of (e.g., SAND, WATER). Can be read.
    pub material_type: MaterialType,
    /// Current temperature of the particle in Celsius. Can be read.
    pub temperature: f32,
    /// Remaining lifespan in seconds for temporary particles (e.g., FIRE, STEAM).
    /// `None` if the particle is permanent or its lifespan is not currently active. Can be read.
    pub life_remaining_seconds: Option<f32>,
    /// Flag indicating if the particle is currently burning (e.g., a FUSE particle). Can be read.
    pub burning: bool,
    /// Tracks how long (in seconds) a particle has been in its current material state.
    /// Used for state-duration specific logic, like minimum time for LAVA/STEAM before changing. Can be read.
    pub time_in_state_seconds: f32,
    /// Cached color of the particle as an RGB tuple `(u8, u8, u8)`.
    /// `None` if the color needs recalculation. Used internally for rendering optimization.
    pub color_cache: Option<(u8, u8, u8)>,
    /// Flag used by the simulation loop to ensure each particle is updated once per step.
    /// Reset at the beginning of each simulation step. Read by the engine.
    pub processed_this_step: bool,
    /// Flag used by physics logic, e.g., for liquid wave propagation or other momentum effects.
    /// Reset at the beginning of each simulation step. Read by the engine.
    pub moved_this_step: bool,
}

impl Particle {
    /// Creates a new particle with the given properties.
    ///
    /// After basic initialization, it calls `init_properties_after_change` to set
    /// initial temperature boosts (e.g., FIRE starts hot), lifespan based on material, etc.
    ///
    /// # Arguments
    /// * `x` - The x-coordinate of the particle.
    /// * `y` - The y-coordinate of the particle.
    /// * `material_type` - The `MaterialType` of the particle.
    /// * `initial_temp` - The initial temperature for the particle.
    ///
    /// # Returns
    /// A new `Particle` instance.
    pub fn new(x: i32, y: i32, material_type: MaterialType, initial_temp: f32) -> Self {
        let mut particle = Particle {
            x,
            y,
            material_type,
            temperature: initial_temp, 
            life_remaining_seconds: None, 
            burning: false,
            time_in_state_seconds: 0.0, 
            color_cache: None,
            processed_this_step: false,
            moved_this_step: false,
        };
        particle.init_properties_after_change(); 
        particle
    }

    /// Initializes or re-initializes particle properties based on its current `material_type`.
    ///
    /// This is a private helper called when a particle is created or changes type.
    /// It applies initial temperature boosts for certain materials (e.g., FIRE, LAVA start hot),
    /// sets the lifespan based on material properties, resets `time_in_state_seconds`,
    /// and invalidates the color cache. The temperature is clamped to valid min/max values.
    fn init_properties_after_change(&mut self) {
        let props = self.get_material_properties();
        let mut target_temp = self.temperature;

        match self.material_type {
            MaterialType::FIRE => target_temp = target_temp.max(800.0),
            MaterialType::LAVA => target_temp = target_temp.max(1800.0),
            MaterialType::STEAM => target_temp = target_temp.max(101.0),
            MaterialType::GENERATOR => target_temp = target_temp.max(300.0),
            MaterialType::ICE => target_temp = target_temp.min(-5.0),
            MaterialType::SAND if self.temperature > 1500.0 => target_temp = target_temp.max(1500.0), 
            MaterialType::STONE if self.temperature > 1000.0 => target_temp = target_temp.max(1000.0), 
            _ => {}
        }

        self.temperature = target_temp.clamp(-273.15, MAX_TEMP);
        self.life_remaining_seconds = props.lifespan_seconds;
        self.time_in_state_seconds = 0.0; 
        self.invalidate_color_cache();
    }

    /// Retrieves the static material properties for this particle's type.
    ///
    /// # Returns
    /// A `MaterialProperties` struct associated with the particle's `material_type`.
    pub fn get_material_properties(&self) -> MaterialProperties {
        get_material_properties(self.material_type)
    }

    /// Invalidates the cached color of the particle.
    ///
    /// This forces a recalculation of the particle's color the next time `get_color()` is called.
    /// It should be used whenever a particle's state changes in a way that might affect its appearance.
    pub fn invalidate_color_cache(&mut self) {
        self.color_cache = None;
    }

    /// Changes the particle's `material_type` and adjusts its properties accordingly.
    ///
    /// This method updates the particle's type and then calls `init_properties_after_change`
    /// to apply default settings for the new type (like initial temperature boosts or lifespan).
    /// If `new_temp_override` is `None`, specific temperature adjustments for phase changes
    /// (e.g., water condensing from steam) are applied based on the old and new types.
    ///
    /// # Arguments
    /// * `new_type` - The new `MaterialType` for the particle.
    /// * `new_temp_override` - An optional temperature to set for the particle. If `Some`, this
    ///   temperature is set before `init_properties_after_change` is called (which might further
    ///   adjust it, e.g., for FIRE). If `None`, phase-change specific temperature adjustments are applied.
    pub fn change_type(&mut self, new_type: MaterialType, new_temp_override: Option<f32>) {
        let old_type = self.material_type;
        let old_temp = self.temperature;
        
        self.material_type = new_type;
        if let Some(temp_override) = new_temp_override {
            self.temperature = temp_override;
        }
        
        self.init_properties_after_change(); 

        if new_temp_override.is_none() {
            let mut needs_phase_temp_adjust = true;
            match new_type {
                MaterialType::FIRE | MaterialType::LAVA | MaterialType::STEAM | MaterialType::GENERATOR | MaterialType::ICE => {
                    needs_phase_temp_adjust = false; 
                }
                _ => {}
            }

            if needs_phase_temp_adjust {
                match (new_type, old_type) {
                    (MaterialType::WATER, MaterialType::STEAM) => self.temperature = (old_temp - 20.0).min(99.0).max(AMBIENT_TEMP),
                    (MaterialType::WATER, MaterialType::ICE) => self.temperature = (old_temp + 5.0).max(1.0).min(AMBIENT_TEMP),
                    (MaterialType::LAVA, MaterialType::GLASS) => self.temperature = (old_temp + 50.0).max(1800.0), 
                    (MaterialType::STONE, MaterialType::LAVA) => self.temperature = (old_temp - 100.0).min(999.0),
                    (MaterialType::GLASS, MaterialType::SAND) => self.temperature = (old_temp + 20.0).max(1500.0),
                    (MaterialType::ASH, MaterialType::FUSE) => self.temperature = (old_temp * 0.5).max(AMBIENT_TEMP),
                    (MaterialType::SMOKE, MaterialType::FIRE) => self.temperature = (old_temp * 0.6).max(AMBIENT_TEMP),
                    _ => { self.temperature = old_temp; } 
                }
                self.temperature = self.temperature.clamp(-273.15, MAX_TEMP);
            }
        }
    }

    /// Calculates and returns the RGB color of the particle for rendering.
    ///
    /// This method uses a cached color if available. If not, it calculates the color based
    /// on the particle's `material_type`, `temperature`, `life_remaining_seconds` (for fading effects),
    /// and `burning` status. The calculated color is then cached for future calls.
    ///
    /// Note: This method takes `&mut self` because it updates the internal `color_cache`.
    ///
    /// # Returns
    /// An RGB tuple `(u8, u8, u8)` representing the particle's color.
    pub fn get_color(&mut self) -> (u8, u8, u8) {
        if let Some(cached_color) = self.color_cache {
            return cached_color;
        }

        let props = self.get_material_properties();
        let base_color_arr = props.base_color;
        let mut r_f32 = base_color_arr.0 as f32;
        let mut g_f32 = base_color_arr.1 as f32;
        let mut b_f32 = base_color_arr.2 as f32;

        let material_type = self.material_type;

        if material_type != MaterialType::EMPTY {
            let mut temp_factor: f32;
            match material_type {
                MaterialType::FIRE => {
                    let flicker = rand::random::<f32>() * 0.3 + 0.85; 
                    temp_factor = ((self.temperature - 500.0) / 600.0).max(0.0).min(1.0); 
                    r_f32 = (base_color_arr.0 as f32 * flicker + temp_factor * 60.0).min(255.0);
                    g_f32 = (base_color_arr.1 as f32 * flicker * (1.0 - temp_factor * 0.6)).min(255.0);
                    b_f32 = (base_color_arr.2 as f32 * flicker * (1.0 - temp_factor)).max(0.0);
                }
                MaterialType::LAVA => {
                    temp_factor = ((self.temperature - 1000.0) / 800.0).max(0.0).min(1.0);
                    r_f32 = (base_color_arr.0 as f32 + temp_factor * 50.0).min(255.0);
                    g_f32 = (base_color_arr.1 as f32 + temp_factor * 70.0).min(255.0);
                    b_f32 = (base_color_arr.2 as f32 * (1.0 - temp_factor * 0.5)).max(0.0);
                }
                MaterialType::GENERATOR => {
                    temp_factor = ((self.temperature - 300.0) / 1000.0).max(0.0).min(1.0);
                    r_f32 = (base_color_arr.0 as f32 + temp_factor * 50.0).min(255.0);
                    g_f32 = (base_color_arr.1 as f32 * (1.0 - temp_factor * 0.8)).max(0.0);
                    b_f32 = (base_color_arr.2 as f32 * (1.0 - temp_factor * 0.8)).max(0.0);
                }
                MaterialType::STEAM | MaterialType::SMOKE | MaterialType::TOXIC_GAS => {
                    if let Some(max_life) = props.lifespan_seconds {
                        if let Some(current_life) = self.life_remaining_seconds {
                            if max_life > 0.0 { 
                                let life_factor = (current_life / max_life).max(0.0); 
                                let fade = 0.6 * (1.0 - life_factor); 
                                let gray = 80.0; 
                                r_f32 = r_f32 * life_factor + gray * fade;
                                g_f32 = g_f32 * life_factor + gray * fade;
                                b_f32 = b_f32 * life_factor + gray * fade;
                            }
                        }
                    }
                }
                MaterialType::FUSE if self.burning => {
                    r_f32 = (r_f32 + 100.0).min(255.0);
                    g_f32 = (g_f32 + 50.0).min(255.0);
                    b_f32 = (b_f32 - 20.0).max(0.0);
                }
                _ => { 
                    if material_type != MaterialType::FIRE && material_type != MaterialType::LAVA &&
                       material_type != MaterialType::STEAM && material_type != MaterialType::SMOKE &&
                       material_type != MaterialType::TOXIC_GAS && material_type != MaterialType::GENERATOR &&
                       !(material_type == MaterialType::FUSE && self.burning) { 
                        temp_factor = ((self.temperature - AMBIENT_TEMP) / 150.0).max(-0.5).min(1.5);
                        r_f32 = (r_f32 + temp_factor * 25.0);
                        g_f32 = (g_f32 + temp_factor * 15.0);
                        b_f32 = (b_f32 + temp_factor * 10.0 - temp_factor.abs() * 15.0);
                    }
                }
            }
        }

        let final_r = r_f32.max(0.0).min(255.0).floor() as u8;
        let final_g = g_f32.max(0.0).min(255.0).floor() as u8;
        let final_b = b_f32.max(0.0).min(255.0).floor() as u8;
        
        let final_color = (final_r, final_g, final_b);
        self.color_cache = Some(final_color); 
        final_color
    }
}

#[cfg(test)]
mod tests {
    use super::*; 
    use crate::material::MaterialType; 

    #[test]
    fn test_particle_creation() {
        let p = Particle::new(0, 0, MaterialType::SAND, AMBIENT_TEMP);
        assert_eq!(p.material_type, MaterialType::SAND);
        assert_eq!(p.temperature, AMBIENT_TEMP);
        assert!(!p.burning);
    }

    #[test]
    fn test_fire_particle_hot_creation() {
        let p = Particle::new(0, 0, MaterialType::FIRE, AMBIENT_TEMP); 
        assert_eq!(p.material_type, MaterialType::FIRE);
        assert_eq!(p.temperature, 800.0); 
        assert!(p.life_remaining_seconds.is_some());
    }

    #[test]
    fn test_particle_change_type_phase() {
        let mut p = Particle::new(0, 0, MaterialType::ICE, -5.0);
        p.change_type(MaterialType::WATER, Some(10.0));
        assert_eq!(p.material_type, MaterialType::WATER);
        assert_eq!(p.temperature, 10.0); 

        let mut steam = Particle::new(0,0, MaterialType::STEAM, 110.0);
        steam.change_type(MaterialType::WATER, None); 
        assert_eq!(steam.material_type, MaterialType::WATER);
        assert_eq!(steam.temperature, 90.0);
    }
}
