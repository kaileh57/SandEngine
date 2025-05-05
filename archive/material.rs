// File: material.rs
use crate::constants::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MaterialType {
    Empty,
    Sand,
    Water,
    Stone,
    Plant,
    Fire,
    Lava,
    Eraser,
}

impl MaterialType {
    pub fn to_u8(self) -> u8 {
        match self {
            MaterialType::Empty => 0,
            MaterialType::Sand => 1,
            MaterialType::Water => 2,
            MaterialType::Stone => 3,
            MaterialType::Plant => 4,
            MaterialType::Fire => 5,
            MaterialType::Lava => 6,
            MaterialType::Eraser => 99,
        }
    }
    
    pub fn from_u8(value: u8) -> Self {
        match value {
            1 => MaterialType::Sand,
            2 => MaterialType::Water,
            3 => MaterialType::Stone,
            4 => MaterialType::Plant,
            5 => MaterialType::Fire,
            6 => MaterialType::Lava,
            99 => MaterialType::Eraser,
            _ => MaterialType::Empty,
        }
    }
    
    pub fn get_name(self) -> &'static str {
        match self {
            MaterialType::Empty => "Empty",
            MaterialType::Sand => "Sand",
            MaterialType::Water => "Water",
            MaterialType::Stone => "Stone",
            MaterialType::Plant => "Plant",
            MaterialType::Fire => "Fire",
            MaterialType::Lava => "Lava",
            MaterialType::Eraser => "Eraser",
        }
    }
    
    pub fn get_color(self) -> [u8; 4] {
        match self {
            MaterialType::Empty => C_EMPTY,
            MaterialType::Sand => C_SAND,
            MaterialType::Water => C_WATER,
            MaterialType::Stone => C_STONE,
            MaterialType::Plant => C_PLANT,
            MaterialType::Fire => C_FIRE,
            MaterialType::Lava => C_LAVA,
            MaterialType::Eraser => C_ERASER,
        }
    }
    
    pub fn get_properties(self) -> MaterialProperties {
        match self {
            MaterialType::Empty => MaterialProperties {
                name: "Empty",
                density: 0.0,
                flammable: false,
                viscosity: 0.0,
                has_gravity: false,
                color: C_EMPTY,
            },
            MaterialType::Sand => MaterialProperties {
                name: "Sand",
                density: 5.0,
                flammable: false,
                viscosity: 0.0,
                has_gravity: true,
                color: C_SAND,
            },
            MaterialType::Water => MaterialProperties {
                name: "Water",
                density: 3.0,
                flammable: false,
                viscosity: 1.0,
                has_gravity: true,
                color: C_WATER,
            },
            MaterialType::Stone => MaterialProperties {
                name: "Stone",
                density: 10.0,
                flammable: false,
                viscosity: 0.0,
                has_gravity: true,
                color: C_STONE,
            },
            MaterialType::Plant => MaterialProperties {
                name: "Plant",
                density: 0.1,
                flammable: true,
                viscosity: 0.0,
                has_gravity: false,
                color: C_PLANT,
            },
            MaterialType::Fire => MaterialProperties {
                name: "Fire",
                density: -2.0,
                flammable: false,
                viscosity: 0.0,
                has_gravity: false,
                color: C_FIRE,
            },
            MaterialType::Lava => MaterialProperties {
                name: "Lava",
                density: 8.0,
                flammable: false,
                viscosity: 5.0,
                has_gravity: true,
                color: C_LAVA,
            },
            MaterialType::Eraser => MaterialProperties {
                name: "Eraser",
                density: 0.0,
                flammable: false,
                viscosity: 0.0,
                has_gravity: false,
                color: C_ERASER,
            },
        }
    }
}

// Material properties
pub struct MaterialProperties {
    pub name: &'static str,
    pub density: f32,
    pub flammable: bool,
    pub viscosity: f32,
    pub has_gravity: bool,
    pub color: [u8; 4],
}