use crate::materials::{get_material_properties, Material, MaterialType};
use rand::Rng;
use serde::{Deserialize, Serialize};

const AMBIENT_TEMP: f32 = 20.0;
const MAX_TEMP: f32 = 3000.0;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Particle {
    pub x: usize,
    pub y: usize,
    pub material_type: MaterialType,
    pub temp: f32,
    pub initial_temp: f32,
    pub processed: bool,
    pub life: Option<f32>,
    pub time_in_state: f32,
    pub moved_this_step: bool,
    pub burning: bool,
    // Performance optimizations from reference project
    pub dynamic: bool, // Whether this particle needs frequent updates
    pub settled_frames: u8, // How many frames it's been stationary
    #[serde(skip)]
    color_cache: Option<[u8; 3]>,
}

impl Particle {
    #[inline(always)]
    fn is_material_dynamic(material_type: MaterialType) -> bool {
        // Static materials that don't need frequent updates
        !matches!(material_type, 
            MaterialType::Empty | MaterialType::Stone | MaterialType::Generator |
            MaterialType::Glass | MaterialType::Ice | MaterialType::Wood
        )
    }

    pub fn new(x: usize, y: usize, material_type: MaterialType, temp: Option<f32>) -> Self {
        let temp = temp.unwrap_or(AMBIENT_TEMP);
        let mut particle = Self {
            x,
            y,
            material_type,
            temp,
            initial_temp: temp,
            processed: false,
            life: None,
            time_in_state: 0.0,
            moved_this_step: false,
            burning: false,
            dynamic: Self::is_material_dynamic(material_type),
            settled_frames: 0,
            color_cache: None,
        };
        particle.init_properties();
        particle
    }

    pub fn init_properties(&mut self) {
        let props = get_material_properties(self.material_type);
        
        // Set initial temperatures based on material type
        let target_temp = match self.material_type {
            MaterialType::Fire => self.temp.max(800.0),
            MaterialType::Lava => self.temp.max(1800.0),
            MaterialType::Steam => self.temp.max(101.0),
            MaterialType::Generator => self.temp.max(300.0),
            MaterialType::Ice => self.temp.min(-5.0),
            MaterialType::Sand if self.temp > 1500.0 => self.temp.max(1500.0),
            MaterialType::Stone if self.temp > 1000.0 => self.temp.max(1000.0),
            _ => self.temp,
        };

        self.temp = target_temp.max(-273.15).min(MAX_TEMP);
        self.life = props.life_seconds;
        self.time_in_state = 0.0;
        self.invalidate_color_cache();
    }

    pub fn get_properties(&self) -> Material {
        get_material_properties(self.material_type)
    }

    pub fn invalidate_color_cache(&mut self) {
        self.color_cache = None;
    }

    pub fn get_color(&mut self) -> [u8; 3] {
        if let Some(cached_color) = self.color_cache {
            return cached_color;
        }

        let props = self.get_properties();
        let mut r = props.base_color[0] as f32;
        let mut g = props.base_color[1] as f32;
        let mut b = props.base_color[2] as f32;

        if self.material_type != MaterialType::Empty {
            match self.material_type {
                MaterialType::Fire => {
                    let mut rng = rand::thread_rng();
                    let flicker = rng.gen_range(0.85..1.15);
                    let temp_factor = ((self.temp - 500.0) / 600.0).max(0.0).min(1.0);
                    r = (props.base_color[0] as f32 * flicker + temp_factor * 60.0).min(255.0);
                    g = (props.base_color[1] as f32 * flicker * (1.0 - temp_factor * 0.6)).min(255.0);
                    b = (props.base_color[2] as f32 * flicker * (1.0 - temp_factor)).max(0.0);
                }
                MaterialType::Lava => {
                    let temp_factor = ((self.temp - 1000.0) / 800.0).max(0.0).min(1.0);
                    r = (props.base_color[0] as f32 + temp_factor * 50.0).min(255.0);
                    g = (props.base_color[1] as f32 + temp_factor * 70.0).min(255.0);
                    b = (props.base_color[2] as f32 * (1.0 - temp_factor * 0.5)).max(0.0);
                }
                MaterialType::Generator => {
                    let temp_factor = ((self.temp - 300.0) / 1000.0).max(0.0).min(1.0);
                    r = (props.base_color[0] as f32 + temp_factor * 50.0).min(255.0);
                    g = (props.base_color[1] as f32 * (1.0 - temp_factor * 0.8)).max(0.0);
                    b = (props.base_color[2] as f32 * (1.0 - temp_factor * 0.8)).max(0.0);
                }
                MaterialType::Steam | MaterialType::Smoke | MaterialType::ToxicGas => {
                    if let Some(max_life) = props.life_seconds {
                        if let Some(current_life) = self.life {
                            if max_life > 0.0 {
                                let life_factor = (current_life / max_life).max(0.0);
                                let fade = 0.6 * (1.0 - life_factor);
                                let gray = 80.0;
                                r = r * life_factor + gray * fade;
                                g = g * life_factor + gray * fade;
                                b = b * life_factor + gray * fade;
                            }
                        }
                    }
                }
                MaterialType::Fuse if self.burning => {
                    r = (r + 100.0).min(255.0);
                    g = (g + 50.0).min(255.0);
                    b = (b - 20.0).max(0.0);
                }
                _ => {
                    // Temperature-based color adjustment for other materials
                    if !matches!(
                        self.material_type,
                        MaterialType::Fire | MaterialType::Lava | MaterialType::Steam | 
                        MaterialType::Smoke | MaterialType::ToxicGas | MaterialType::Generator
                    ) {
                        let temp_factor = ((self.temp - AMBIENT_TEMP) / 150.0).max(-0.5).min(1.5);
                        r = (r + temp_factor * 25.0).max(0.0).min(255.0);
                        g = (g + temp_factor * 15.0).max(0.0).min(255.0);
                        b = (b + temp_factor * 10.0 - temp_factor.abs() * 15.0).max(0.0).min(255.0);
                    }
                }
            }
        }

        let color = [r as u8, g as u8, b as u8];
        self.color_cache = Some(color);
        color
    }

    pub fn change_type(&mut self, new_type: MaterialType, new_temp: Option<f32>) {
        let old_temp = self.temp;
        let current_x = self.x;
        let current_y = self.y;

        self.material_type = new_type;
        if let Some(temp) = new_temp {
            self.temp = temp;
        }

        // Update dynamic flag when material changes
        self.dynamic = Self::is_material_dynamic(new_type);
        self.settled_frames = 0; // Reset settled counter on material change

        self.init_properties();

        // Restore position
        self.x = current_x;
        self.y = current_y;

        // Apply temperature adjustments for phase changes
        if new_temp.is_none() {
            let needs_phase_temp_adjust = !matches!(
                new_type,
                MaterialType::Fire | MaterialType::Lava | MaterialType::Steam | 
                MaterialType::Generator | MaterialType::Ice
            );

            if needs_phase_temp_adjust {
                self.temp = match new_type {
                    MaterialType::Water => match self.material_type {
                        MaterialType::Steam => (old_temp - 20.0).max(AMBIENT_TEMP).min(99.0),
                        MaterialType::Ice => (old_temp + 5.0).max(1.0).min(AMBIENT_TEMP),
                        _ => old_temp,
                    },
                    MaterialType::Lava => (old_temp + 50.0).max(1800.0),
                    MaterialType::Stone => (old_temp - 100.0).min(999.0),
                    MaterialType::Glass => (old_temp + 20.0).max(1500.0),
                    MaterialType::Ash => (old_temp * 0.5).max(AMBIENT_TEMP),
                    MaterialType::Smoke => (old_temp * 0.6).max(AMBIENT_TEMP),
                    _ => old_temp,
                };
                self.temp = self.temp.max(-273.15).min(MAX_TEMP);
            }
        }

        self.time_in_state = 0.0;
        self.invalidate_color_cache();
    }
}