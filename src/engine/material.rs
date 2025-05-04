// File: engine/material.rs

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
}

// Material properties - physics related only
pub struct MaterialProperties {
    pub name: &'static str,
    pub density: f32,
    pub flammable: bool,
    pub viscosity: f32,
    pub has_gravity: bool,
} 