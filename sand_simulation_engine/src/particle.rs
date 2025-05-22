// src/particle.rs

use crate::material::{MaterialType, MaterialProperties, get_material_properties};
use rand; // For random numbers, e.g., in fire flicker

// Constants
const AMBIENT_TEMP: f32 = 20.0;
const MAX_TEMP: f32 = 3000.0;
// const ABSOLUTE_ZERO: f32 = -273.15; // Defined inline in clamp

#[derive(Debug, Clone)]
pub struct Particle {
    pub x: i32,
    pub y: i32,
    pub material_type: MaterialType,
    pub temperature: f32,
    pub life_remaining_seconds: Option<f32>,
    pub burning: bool,
    pub time_in_state_seconds: f32,
    pub color_cache: Option<(u8, u8, u8)>,
    pub processed_this_step: bool,
    pub moved_this_step: bool,
}

impl Particle {
    pub fn new(x: i32, y: i32, material_type: MaterialType, initial_temp: f32) -> Self {
        let mut particle = Particle {
            x,
            y,
            material_type,
            temperature: initial_temp, // Set initial temp first
            life_remaining_seconds: None, // Will be set by init_properties_after_change
            burning: false,
            time_in_state_seconds: 0.0, // Will be set by init_properties_after_change
            color_cache: None,
            processed_this_step: false,
            moved_this_step: false,
        };
        particle.init_properties_after_change(); // Adjust temp, set lifespan, etc.
        particle
    }

    // Private helper, similar to JS initProperties
    fn init_properties_after_change(&mut self) {
        let props = self.get_material_properties();
        let mut target_temp = self.temperature;

        // Initial temp boosts from JS initProperties
        match self.material_type {
            MaterialType::FIRE => target_temp = target_temp.max(800.0),
            MaterialType::LAVA => target_temp = target_temp.max(1800.0),
            MaterialType::STEAM => target_temp = target_temp.max(101.0),
            MaterialType::GENERATOR => target_temp = target_temp.max(300.0),
            MaterialType::ICE => target_temp = target_temp.min(-5.0),
            MaterialType::SAND if self.temperature > 1500.0 => target_temp = target_temp.max(1500.0), // Temp retention
            MaterialType::STONE if self.temperature > 1000.0 => target_temp = target_temp.max(1000.0), // Temp retention
            _ => {}
        }

        self.temperature = target_temp.clamp(-273.15, MAX_TEMP);
        self.life_remaining_seconds = props.lifespan_seconds;
        self.time_in_state_seconds = 0.0;
        self.invalidate_color_cache();
    }

    pub fn get_material_properties(&self) -> MaterialProperties {
        get_material_properties(self.material_type)
    }

    pub fn invalidate_color_cache(&mut self) {
        self.color_cache = None;
    }

    pub fn change_type(&mut self, new_type: MaterialType, new_temp_override: Option<f32>) {
        let old_type = self.material_type;
        let old_temp = self.temperature;
        // Keep current x, y. JS version does this too.
        // let current_x = self.x;
        // let current_y = self.y;

        self.material_type = new_type;
        if let Some(temp_override) = new_temp_override {
            self.temperature = temp_override;
        }
        
        self.init_properties_after_change(); // This resets lifespan, time_in_state, and applies initial temp boosts for new_type

        // self.x = current_x; // Ensure position is maintained if init_properties somehow changed it (it shouldn't)
        // self.y = current_y;

        if new_temp_override.is_none() {
            let mut needs_phase_temp_adjust = true;
            match new_type {
                MaterialType::FIRE | MaterialType::LAVA | MaterialType::STEAM | MaterialType::GENERATOR | MaterialType::ICE => {
                    needs_phase_temp_adjust = false; // Temps handled by init_properties_after_change
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
                    _ => { self.temperature = old_temp; } // Default keep old temp
                }
                self.temperature = self.temperature.clamp(-273.15, MAX_TEMP);
            }
        }
        // time_in_state_seconds is reset by init_properties_after_change
        // invalidate_color_cache is also called by init_properties_after_change
        // No need to call invalidate_color_cache explicitly here as init_properties_after_change does it.
    }

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
                            if max_life > 0.0 { // Avoid division by zero
                                let life_factor = (current_life / max_life).max(0.0);
                                let fade = 0.6 * (1.0 - life_factor);
                                let gray = 80.0; // Target gray color for fading
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
                _ => { // Default temperature tint for other materials
                    // Check if it's NOT one of the types that have custom color logic already
                    if material_type != MaterialType::FIRE && material_type != MaterialType::LAVA &&
                       material_type != MaterialType::STEAM && material_type != MaterialType::SMOKE &&
                       material_type != MaterialType::TOXIC_GAS && material_type != MaterialType::GENERATOR &&
                       !(material_type == MaterialType::FUSE && self.burning) { // FUSE non-burning gets default tint
                        temp_factor = ((self.temperature - AMBIENT_TEMP) / 150.0).max(-0.5).min(1.5);
                        r_f32 = (r_f32 + temp_factor * 25.0);
                        g_f32 = (g_f32 + temp_factor * 15.0);
                        b_f32 = (b_f32 + temp_factor * 10.0 - temp_factor.abs() * 15.0);
                    }
                }
            }
        }

        // Clamp all color components to valid u8 range before converting
        let final_r = r_f32.max(0.0).min(255.0).floor() as u8;
        let final_g = g_f32.max(0.0).min(255.0).floor() as u8;
        let final_b = b_f32.max(0.0).min(255.0).floor() as u8;
        
        let final_color = (final_r, final_g, final_b);
        self.color_cache = Some(final_color);
        final_color
    }
}
