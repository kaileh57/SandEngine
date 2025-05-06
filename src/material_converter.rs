// material_converter.rs - Helper to convert between old and new material types
// This is a temporary module to help with transition from the old system

use crate::material;
use crate::material_properties;

/// Convert from old material type to new material type
pub fn to_new_material(old: material::MaterialType) -> material_properties::MaterialType {
    match old {
        material::MaterialType::Empty => material_properties::MaterialType::Empty,
        material::MaterialType::Sand => material_properties::MaterialType::Sand,
        material::MaterialType::Water => material_properties::MaterialType::Water,
        material::MaterialType::Stone => material_properties::MaterialType::Stone,
        material::MaterialType::Plant => material_properties::MaterialType::Plant,
        material::MaterialType::Fire => material_properties::MaterialType::Fire,
        material::MaterialType::Lava => material_properties::MaterialType::Lava,
        material::MaterialType::Eraser => material_properties::MaterialType::Eraser,
    }
}

/// Convert from new material type to old material type
pub fn to_old_material(new: material_properties::MaterialType) -> material::MaterialType {
    match new {
        material_properties::MaterialType::Empty => material::MaterialType::Empty,
        material_properties::MaterialType::Sand => material::MaterialType::Sand,
        material_properties::MaterialType::Water => material::MaterialType::Water,
        material_properties::MaterialType::Stone => material::MaterialType::Stone,
        material_properties::MaterialType::Plant => material::MaterialType::Plant,
        material_properties::MaterialType::Fire => material::MaterialType::Fire,
        material_properties::MaterialType::Lava => material::MaterialType::Lava,
        material_properties::MaterialType::Eraser => material::MaterialType::Eraser,
        // Map new materials to closest old equivalent
        material_properties::MaterialType::Glass => material::MaterialType::Stone,
        material_properties::MaterialType::Steam => material::MaterialType::Water,
        material_properties::MaterialType::Smoke => material::MaterialType::Empty,
        material_properties::MaterialType::Ice => material::MaterialType::Water,
        material_properties::MaterialType::Wood => material::MaterialType::Plant,
        material_properties::MaterialType::Coal => material::MaterialType::Stone,
        material_properties::MaterialType::Oil => material::MaterialType::Water,
        material_properties::MaterialType::Acid => material::MaterialType::Water,
        material_properties::MaterialType::Gunpowder => material::MaterialType::Sand,
        material_properties::MaterialType::ToxicGas => material::MaterialType::Empty,
        material_properties::MaterialType::Ash => material::MaterialType::Sand,
        material_properties::MaterialType::Fuse => material::MaterialType::Plant,
        material_properties::MaterialType::Generator => material::MaterialType::Stone,
    }
}