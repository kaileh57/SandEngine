use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MaterialType {
    Empty = 0,
    Sand = 1,
    Water = 2,
    Stone = 3,
    Plant = 4,
    Fire = 5,
    Lava = 6,
    Glass = 7,
    Steam = 8,
    Oil = 9,
    Acid = 10,
    Coal = 11,
    Gunpowder = 12,
    Ice = 13,
    Wood = 14,
    Smoke = 15,
    ToxicGas = 16,
    Slime = 17,
    Gasoline = 18,
    Generator = 19,
    Fuse = 20,
    Ash = 21,
    Gold = 22,
    Iron = 23,
    Eraser = 99,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Material {
    pub density: f32,
    pub conductivity: f32,
    pub flammability: f32,
    pub melt_temp: Option<f32>,
    pub boil_temp: Option<f32>,
    pub freeze_temp: Option<f32>,
    pub base_color: [u8; 3],
    pub name: String,
    pub viscosity: f32,
    pub life_seconds: Option<f32>,
    pub corrosive_power: f32,
    pub explosive_yield: Option<f32>,
    pub heat_generation: f32,
    pub ignition_temp: Option<f32>,
}

impl Material {
    pub fn new(
        density: f32,
        conductivity: f32,
        flammability: f32,
        melt_temp: Option<f32>,
        boil_temp: Option<f32>,
        freeze_temp: Option<f32>,
        base_color: [u8; 3],
        name: &str,
        viscosity: f32,
        life_seconds: Option<f32>,
        corrosive_power: f32,
        explosive_yield: Option<f32>,
        heat_generation: f32,
        ignition_temp: Option<f32>,
    ) -> Self {
        Self {
            density,
            conductivity,
            flammability,
            melt_temp,
            boil_temp,
            freeze_temp,
            base_color,
            name: name.to_string(),
            viscosity,
            life_seconds,
            corrosive_power,
            explosive_yield,
            heat_generation,
            ignition_temp,
        }
    }

    pub fn is_liquid(&self, material_type: MaterialType) -> bool {
        matches!(
            material_type,
            MaterialType::Water | MaterialType::Oil | MaterialType::Acid | MaterialType::Gasoline | MaterialType::Lava
        )
    }

    pub fn is_powder(&self, material_type: MaterialType) -> bool {
        matches!(
            material_type,
            MaterialType::Sand | MaterialType::Ash | MaterialType::Gunpowder
        )
    }

    pub fn is_rigid_solid(&self, material_type: MaterialType) -> bool {
        matches!(
            material_type,
            MaterialType::Stone | MaterialType::Glass | MaterialType::Wood | MaterialType::Ice |
            MaterialType::Gold | MaterialType::Iron | MaterialType::Coal
        )
    }

    pub fn is_gas(&self, material_type: MaterialType) -> bool {
        self.density < 0.0 || matches!(
            material_type,
            MaterialType::Steam | MaterialType::Smoke | MaterialType::ToxicGas
        )
    }

    pub fn is_stationary(&self, material_type: MaterialType) -> bool {
        // Materials that don't fall or move (except when part of rigid bodies)
        matches!(
            material_type,
            MaterialType::Stone | MaterialType::Wood | MaterialType::Ice | 
            MaterialType::Gold | MaterialType::Iron | MaterialType::Coal |
            MaterialType::Generator
        )
    }
}

pub fn get_material_properties(material_type: MaterialType) -> Material {
    match material_type {
        MaterialType::Empty => Material::new(
            0.0, 0.1, 0.0, None, None, None, [0, 0, 0], "Empty", 1.0, None, 0.0, None, 0.0, None
        ),
        MaterialType::Sand => Material::new(
            1.6, 0.3, 0.0, Some(1500.0), None, None, [194, 178, 128], "Sand", 1.0, None, 0.0, None, 0.0, None
        ),
        MaterialType::Water => Material::new(
            1.0, 0.6, 0.0, None, Some(100.0), Some(0.0), [50, 100, 200], "Water", 1.0, None, 0.0, None, 0.0, None
        ),
        MaterialType::Stone => Material::new(
            2.7, 0.2, 0.0, None, None, None, [100, 100, 100], "Stone", 1.0, None, 0.0, None, 0.0, None
        ),
        MaterialType::Plant => Material::new(
            0.4, 0.1, 0.4, Some(200.0), None, None, [50, 150, 50], "Plant", 1.0, None, 0.0, None, 0.0, Some(150.0)
        ),
        MaterialType::Fire => Material::new(
            -2.0, 0.9, 0.0, None, None, None, [255, 69, 0], "Fire", 1.0, Some(1.0), 0.0, None, 0.0, None
        ),
        MaterialType::Lava => Material::new(
            3.2, 0.8, 0.0, Some(1800.0), None, Some(1000.0), [200, 50, 0], "Lava", 5.0, None, 0.0, None, 0.0, None
        ),
        MaterialType::Glass => Material::new(
            2.5, 0.4, 0.0, Some(1800.0), None, None, [210, 230, 240], "Glass", 1.0, None, 0.0, None, 0.0, None
        ),
        MaterialType::Steam => Material::new(
            -5.0, 0.7, 0.0, None, None, Some(99.0), [180, 180, 190], "Steam", 1.0, Some(10.0), 0.0, None, 0.0, None
        ),
        MaterialType::Oil => Material::new(
            0.8, 0.4, 0.9, None, Some(300.0), None, [80, 70, 20], "Oil", 3.0, None, 0.0, None, 0.0, Some(200.0)
        ),
        MaterialType::Acid => Material::new(
            1.8, 0.5, 0.0, None, Some(200.0), None, [100, 255, 100], "Acid", 1.0, None, 0.15, None, 0.0, None
        ),
        MaterialType::Coal => Material::new(
            1.3, 0.2, 1.0, Some(800.0), None, None, [40, 40, 40], "Coal", 1.0, None, 0.0, None, 0.0, Some(250.0)
        ),
        MaterialType::Gunpowder => Material::new(
            1.7, 0.1, 1.0, None, None, None, [60, 60, 70], "Gunpowder", 1.0, None, 0.0, Some(4.0), 0.0, Some(150.0)
        ),
        MaterialType::Ice => Material::new(
            0.92, 0.01, 0.0, Some(1.0), None, None, [170, 200, 255], "Ice", 1.0, None, 0.0, None, 0.0, None
        ),
        MaterialType::Wood => Material::new(
            0.6, 0.2, 0.6, Some(400.0), None, None, [139, 69, 19], "Wood", 1.0, None, 0.0, None, 0.0, Some(200.0)
        ),
        MaterialType::Smoke => Material::new(
            -3.0, 0.1, 0.0, None, None, None, [150, 150, 150], "Smoke", 1.0, Some(3.0), 0.0, None, 0.0, None
        ),
        MaterialType::ToxicGas => Material::new(
            -4.0, 0.1, 0.1, None, None, None, [150, 200, 150], "Toxic Gas", 1.0, Some(5.0), 0.02, None, 0.0, None
        ),
        MaterialType::Slime => Material::new(
            3.2, 0.3, 0.1, None, Some(150.0), None, [100, 200, 100], "Slime", 10.0, None, 0.0, None, 0.0, None
        ),
        MaterialType::Gasoline => Material::new(
            0.8, 0.5, 1.0, None, Some(80.0), None, [255, 223, 186], "Gasoline", 2.0, None, 0.0, None, 0.0, Some(100.0)
        ),
        MaterialType::Generator => Material::new(
            100.0, 0.9, 0.0, None, None, None, [255, 0, 0], "Generator", 1.0, None, 0.0, None, 5.0, None
        ),
        MaterialType::Fuse => Material::new(
            5.0, 0.2, 1.0, Some(150.0), None, None, [100, 80, 60], "Fuse", 1.0, None, 0.0, None, 0.0, Some(150.0)
        ),
        MaterialType::Ash => Material::new(
            0.9, 0.2, 0.0, None, None, None, [90, 90, 90], "Ash", 1.0, None, 0.0, None, 0.0, None
        ),
        MaterialType::Gold => Material::new(
            19.3, 0.8, 0.0, Some(1064.0), None, None, [255, 215, 0], "Gold", 1.0, None, 0.0, None, 0.0, None
        ),
        MaterialType::Iron => Material::new(
            7.9, 0.7, 0.0, Some(1538.0), None, None, [139, 139, 139], "Iron", 1.0, None, 0.0, None, 0.0, None
        ),
        MaterialType::Eraser => Material::new(
            0.0, 0.0, 0.0, None, None, None, [255, 0, 255], "Eraser", 1.0, None, 0.0, None, 0.0, None
        ),
    }
}