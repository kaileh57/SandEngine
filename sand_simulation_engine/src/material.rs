// src/material.rs

/// Enum representing the different types of materials a particle can be.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MaterialType {
    EMPTY,
    SAND,
    WATER,
    STONE,
    PLANT,
    FIRE,
    LAVA,
    GLASS,
    STEAM,
    OIL,
    ACID,
    COAL,
    GUNPOWDER,
    ICE,
    WOOD,
    SMOKE,
    TOXIC_GAS,
    SLIME,
    GASOLINE,
    GENERATOR, // Special immovable particle, may generate other particles or heat
    FUSE,      // Burnable, propagates fire slowly
    ASH,
    ERASER,    // Special tool material to remove particles
}

/// Struct holding the physical and behavioral properties of a material.
/// These properties are generally static for each material type.
#[derive(Debug, Clone)]
pub struct MaterialProperties {
    /// User-friendly name of the material.
    pub name: &'static str,
    /// Density of the material. Higher values are heavier.
    /// Gases typically have negative values to indicate natural upward movement.
    pub density: f32,
    /// Thermal conductivity factor. Higher values transfer heat more readily.
    /// Typically ranges from 0.0 (insulator) to 1.0 (perfect conductor).
    pub thermal_conductivity: f32,
    /// How easily this material ignites or spreads fire.
    /// Typically a value from 0.0 (non-flammable) to 1.0 (highly flammable).
    pub flammability: f32,
    /// Temperature in Celsius at which the material melts. `None` if it doesn't melt.
    pub melt_temperature: Option<f32>,
    /// Temperature in Celsius at which the material boils. `None` if it doesn't boil.
    pub boil_temperature: Option<f32>,
    /// Temperature in Celsius at which the material freezes. `None` if it doesn't freeze.
    pub freeze_temperature: Option<f32>,
    /// Base color of the material as an RGB tuple (0-255 for each component).
    /// This color may be modified by temperature or other effects during rendering.
    pub base_color: (u8, u8, u8),
    /// Viscosity of the material, affecting its flow rate (especially for liquids).
    /// Higher values mean slower flow. Standard solids/powders usually have 1.0.
    pub viscosity: f32,
    /// Lifespan in seconds for temporary particles (e.g., FIRE, SMOKE). `None` if the material is permanent.
    pub lifespan_seconds: Option<f32>,
    /// Power of the material to corrode or dissolve other materials (e.g., ACID).
    /// Typically a value from 0.0 (non-corrosive) to 1.0.
    pub corrosive_power: f32,
    /// The yield or radius of an explosion if this material detonates (e.g., GUNPOWDER).
    /// `None` if the material is not explosive.
    pub explosive_yield: Option<f32>,
    /// Amount of heat this material generates passively per second.
    /// Positive values generate heat, negative values could (theoretically) absorb it.
    pub heat_generation: f32,
    /// Temperature in Celsius at which the material may spontaneously ignite or be ignited by a heat source.
    /// `None` if the material is not ignitable by temperature alone.
    pub ignition_temperature: Option<f32>,
}

/// Returns the static properties for a given `MaterialType`.
/// This function acts as a central repository for defining the characteristics of each material.
pub fn get_material_properties(material_type: MaterialType) -> MaterialProperties {
    match material_type {
        MaterialType::EMPTY => MaterialProperties {
            name: "Empty",
            density: 0.0,
            thermal_conductivity: 0.1,
            flammability: 0.0,
            melt_temperature: None,
            boil_temperature: None,
            freeze_temperature: None,
            base_color: (0, 0, 0), 
            viscosity: 1.0,
            lifespan_seconds: None,
            corrosive_power: 0.0,
            explosive_yield: None,
            heat_generation: 0.0,
            ignition_temperature: None,
        },
        MaterialType::SAND => MaterialProperties {
            name: "Sand",
            density: 5.0,
            thermal_conductivity: 0.3,
            flammability: 0.0,
            melt_temperature: Some(1500.0),
            boil_temperature: None,
            freeze_temperature: None,
            base_color: (194, 178, 128), 
            viscosity: 1.0,
            lifespan_seconds: None,
            corrosive_power: 0.0,
            explosive_yield: None,
            heat_generation: 0.0,
            ignition_temperature: None,
        },
        MaterialType::WATER => MaterialProperties {
            name: "Water",
            density: 3.0,
            thermal_conductivity: 0.6,
            flammability: 0.0,
            melt_temperature: None,
            boil_temperature: Some(100.0),
            freeze_temperature: Some(0.0),
            base_color: (50, 100, 200), 
            viscosity: 1.0,
            lifespan_seconds: None,
            corrosive_power: 0.0,
            explosive_yield: None,
            heat_generation: 0.0,
            ignition_temperature: None,
        },
        MaterialType::STONE => MaterialProperties {
            name: "Stone",
            density: 10.0,
            thermal_conductivity: 0.2,
            flammability: 0.0,
            melt_temperature: None, 
            boil_temperature: None,
            freeze_temperature: None,
            base_color: (100, 100, 100), 
            viscosity: 1.0,
            lifespan_seconds: None,
            corrosive_power: 0.0,
            explosive_yield: None,
            heat_generation: 0.0,
            ignition_temperature: None,
        },
        MaterialType::PLANT => MaterialProperties {
            name: "Plant",
            density: 0.1,
            thermal_conductivity: 0.1,
            flammability: 0.4,
            melt_temperature: Some(200.0), 
            boil_temperature: None,
            freeze_temperature: None,
            base_color: (50, 150, 50), 
            viscosity: 1.0,
            lifespan_seconds: None,
            corrosive_power: 0.0,
            explosive_yield: None,
            heat_generation: 0.0,
            ignition_temperature: Some(150.0),
        },
        MaterialType::FIRE => MaterialProperties {
            name: "Fire",
            density: -2.0, // Negative density for upward movement
            thermal_conductivity: 0.9,
            flammability: 0.0, // Fire itself is not flammable, it *is* fire.
            melt_temperature: None,
            boil_temperature: None,
            freeze_temperature: None,
            base_color: (255, 69, 0), 
            viscosity: 1.0,
            lifespan_seconds: Some(1.0), // Fire burns out
            corrosive_power: 0.0,
            explosive_yield: None,
            heat_generation: 0.0, // Heat is an intrinsic property, managed by simulation logic
            ignition_temperature: None,
        },
        MaterialType::LAVA => MaterialProperties {
            name: "Lava",
            density: 8.0,
            thermal_conductivity: 0.8,
            flammability: 0.0,
            melt_temperature: Some(1800.0), // Can re-melt from solidified state (e.g. Glass) if applicable
            boil_temperature: None,
            freeze_temperature: Some(1000.0), // Solidifies to STONE
            base_color: (200, 50, 0), 
            viscosity: 5.0, // More viscous than water
            lifespan_seconds: None,
            corrosive_power: 0.0,
            explosive_yield: None,
            heat_generation: 0.0, // Similar to fire, intrinsic heat
            ignition_temperature: None,
        },
        MaterialType::GLASS => MaterialProperties {
            name: "Glass",
            density: 9.0,
            thermal_conductivity: 0.4,
            flammability: 0.0,
            melt_temperature: Some(1800.0), // Melts to LAVA
            boil_temperature: None,
            freeze_temperature: None,
            base_color: (210, 230, 240), 
            viscosity: 1.0,
            lifespan_seconds: None,
            corrosive_power: 0.0,
            explosive_yield: None,
            heat_generation: 0.0,
            ignition_temperature: None,
        },
        MaterialType::STEAM => MaterialProperties {
            name: "Steam",
            density: -5.0, // Gas, moves upwards
            thermal_conductivity: 0.7,
            flammability: 0.0,
            melt_temperature: None,
            boil_temperature: None, 
            freeze_temperature: Some(99.0), // Condenses to WATER slightly below 100C
            base_color: (180, 180, 190), 
            viscosity: 1.0,
            lifespan_seconds: Some(10.0), // Steam dissipates
            corrosive_power: 0.0,
            explosive_yield: None,
            heat_generation: 0.0,
            ignition_temperature: None,
        },
        MaterialType::OIL => MaterialProperties {
            name: "Oil",
            density: 2.0,
            thermal_conductivity: 0.4,
            flammability: 0.9,
            melt_temperature: None,
            boil_temperature: Some(300.0),
            freeze_temperature: None,
            base_color: (80, 70, 20), 
            viscosity: 3.0, // More viscous than water
            lifespan_seconds: None,
            corrosive_power: 0.0,
            explosive_yield: None,
            heat_generation: 0.0,
            ignition_temperature: Some(200.0),
        },
        MaterialType::ACID => MaterialProperties {
            name: "Acid",
            density: 3.5,
            thermal_conductivity: 0.5,
            flammability: 0.0,
            melt_temperature: None,
            boil_temperature: Some(200.0), // Boils into TOXIC_GAS
            freeze_temperature: None,
            base_color: (100, 255, 100), 
            viscosity: 1.0,
            lifespan_seconds: None,
            corrosive_power: 0.15, // Corrodes other materials
            explosive_yield: None,
            heat_generation: 0.0,
            ignition_temperature: None,
        },
        MaterialType::COAL => MaterialProperties {
            name: "Coal",
            density: 4.0,
            thermal_conductivity: 0.2,
            flammability: 1.0,
            melt_temperature: Some(800.0), // Represents burning point rather than true melting
            boil_temperature: None,
            freeze_temperature: None,
            base_color: (40, 40, 40), 
            viscosity: 1.0,
            lifespan_seconds: None,
            corrosive_power: 0.0,
            explosive_yield: None,
            heat_generation: 0.0,
            ignition_temperature: Some(250.0),
        },
        MaterialType::GUNPOWDER => MaterialProperties {
            name: "Gunpowder",
            density: 4.5,
            thermal_conductivity: 0.1,
            flammability: 1.0, // Highly flammable, leading to explosion
            melt_temperature: None, 
            boil_temperature: None,
            freeze_temperature: None,
            base_color: (60, 60, 70), 
            viscosity: 1.0,
            lifespan_seconds: None,
            corrosive_power: 0.0,
            explosive_yield: Some(4.0), // Explosion radius/power
            heat_generation: 0.0,
            ignition_temperature: Some(150.0),
        },
        MaterialType::ICE => MaterialProperties {
            name: "Ice",
            density: 2.9, // Slightly less dense than water if considering precise values, but simplified here
            thermal_conductivity: 0.01, // Poor conductor
            flammability: 0.0,
            melt_temperature: Some(1.0), // Melts to WATER slightly above 0C
            boil_temperature: None,
            freeze_temperature: None, 
            base_color: (170, 200, 255), 
            viscosity: 1.0,
            lifespan_seconds: None,
            corrosive_power: 0.0,
            explosive_yield: None,
            heat_generation: 0.0,
            ignition_temperature: None,
        },
        MaterialType::WOOD => MaterialProperties {
            name: "Wood",
            density: 0.7, // Lighter than water
            thermal_conductivity: 0.2,
            flammability: 0.6,
            melt_temperature: Some(400.0), // Represents burning point
            boil_temperature: None,
            freeze_temperature: None,
            base_color: (139, 69, 19), 
            viscosity: 1.0,
            lifespan_seconds: None,
            corrosive_power: 0.0,
            explosive_yield: None,
            heat_generation: 0.0,
            ignition_temperature: Some(200.0),
        },
        MaterialType::SMOKE => MaterialProperties {
            name: "Smoke",
            density: -3.0, // Gas, moves upwards
            thermal_conductivity: 0.1,
            flammability: 0.0,
            melt_temperature: None,
            boil_temperature: None,
            freeze_temperature: None, 
            base_color: (150, 150, 150), 
            viscosity: 1.0,
            lifespan_seconds: Some(3.0), // Smoke dissipates
            corrosive_power: 0.0,
            explosive_yield: None,
            heat_generation: 0.0,
            ignition_temperature: None,
        },
        MaterialType::TOXIC_GAS => MaterialProperties {
            name: "Toxic Gas",
            density: -4.0, // Gas, moves upwards
            thermal_conductivity: 0.1,
            flammability: 0.1, // Slightly flammable
            melt_temperature: None,
            boil_temperature: None,
            freeze_temperature: None, 
            base_color: (150, 200, 150), 
            viscosity: 1.0,
            lifespan_seconds: Some(5.0), // Dissipates
            corrosive_power: 0.02, // Slightly corrosive (e.g., to plants)
            explosive_yield: None,
            heat_generation: 0.0,
            ignition_temperature: None, 
        },
        MaterialType::SLIME => MaterialProperties {
            name: "Slime",
            density: 3.2,
            thermal_conductivity: 0.3,
            flammability: 0.1,
            melt_temperature: None,
            boil_temperature: Some(150.0), // Boils into TOXIC_GAS
            freeze_temperature: None,
            base_color: (100, 200, 100), 
            viscosity: 10.0, // Very viscous
            lifespan_seconds: None,
            corrosive_power: 0.0,
            explosive_yield: None,
            heat_generation: 0.0,
            ignition_temperature: None,
        },
        MaterialType::GASOLINE => MaterialProperties {
            name: "Gasoline",
            density: 0.8, // Lighter than water
            thermal_conductivity: 0.5,
            flammability: 1.0, // Highly flammable
            melt_temperature: None,
            boil_temperature: Some(80.0), // Evaporates/boils easily
            freeze_temperature: None,
            base_color: (255, 223, 186), 
            viscosity: 2.0, // Less viscous than oil
            lifespan_seconds: None,
            corrosive_power: 0.0,
            explosive_yield: None, 
            heat_generation: 0.0,
            ignition_temperature: Some(100.0),
        },
        MaterialType::GENERATOR => MaterialProperties {
            name: "Generator",
            density: 100.0, // Very dense, effectively immovable
            thermal_conductivity: 0.9,
            flammability: 0.0,
            melt_temperature: None,
            boil_temperature: None,
            freeze_temperature: None,
            base_color: (255, 0, 0), 
            viscosity: 1.0,
            lifespan_seconds: None,
            corrosive_power: 0.0,
            explosive_yield: None,
            heat_generation: 5.0, // Passively generates heat
            ignition_temperature: None,
        },
        MaterialType::FUSE => MaterialProperties {
            name: "Fuse",
            density: 5.0,
            thermal_conductivity: 0.2,
            flammability: 1.0, 
            melt_temperature: Some(150.0), // Represents its burning consumption point
            boil_temperature: None,
            freeze_temperature: None,
            base_color: (100, 80, 60), 
            viscosity: 1.0,
            lifespan_seconds: None, // Actual burn duration handled by FUSE_BURN_LIFESPAN_SEC in engine
            corrosive_power: 0.0,
            explosive_yield: None,
            heat_generation: 0.0, 
            ignition_temperature: Some(150.0),
        },
        MaterialType::ASH => MaterialProperties {
            name: "Ash",
            density: 4.8,
            thermal_conductivity: 0.2,
            flammability: 0.0,
            melt_temperature: None,
            boil_temperature: None,
            freeze_temperature: None,
            base_color: (90, 90, 90), 
            viscosity: 1.0,
            lifespan_seconds: None,
            corrosive_power: 0.0,
            explosive_yield: None,
            heat_generation: 0.0,
            ignition_temperature: None,
        },
        MaterialType::ERASER => MaterialProperties {
            name: "Eraser",
            density: 0.0, 
            thermal_conductivity: 0.0,
            flammability: 0.0,
            melt_temperature: None,
            boil_temperature: None,
            freeze_temperature: None,
            base_color: (255,0,255), 
            viscosity: 1.0,
            lifespan_seconds: None,
            corrosive_power: 0.0,
            explosive_yield: None,
            heat_generation: 0.0,
            ignition_temperature: None,
        },
    }
}

/// Checks if a material type is considered a liquid for movement purposes.
/// Liquids typically flow and can be pushed by falling solids.
pub fn is_liquid(material_type: MaterialType) -> bool {
    matches!(material_type, MaterialType::WATER | MaterialType::OIL | MaterialType::ACID | MaterialType::GASOLINE | MaterialType::LAVA)
}

/// Checks if a material type behaves like a powder or granular solid.
/// Powders can form piles and may flow differently from rigid solids.
pub fn is_powder(material_type: MaterialType) -> bool {
    matches!(material_type, MaterialType::SAND | MaterialType::ASH | MaterialType::GUNPOWDER | MaterialType::COAL)
}

/// Checks if a material type is a generally rigid solid.
/// Rigid solids often have more restricted movement (e.g., no easy diagonal piling).
pub fn is_rigid_solid(material_type: MaterialType) -> bool {
    matches!(material_type, MaterialType::STONE | MaterialType::GLASS | MaterialType::WOOD | MaterialType::ICE)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sand_properties() {
        let props = get_material_properties(MaterialType::SAND);
        assert_eq!(props.name, "Sand");
        assert_eq!(props.density, 5.0);
        assert_eq!(props.melt_temperature, Some(1500.0));
    }

    #[test]
    fn test_water_properties() {
        let props = get_material_properties(MaterialType::WATER);
        assert_eq!(props.name, "Water");
        assert!(props.boil_temperature.is_some());
        assert_eq!(props.boil_temperature.unwrap(), 100.0);
    }

    #[test]
    fn test_material_type_helpers() {
        assert!(is_liquid(MaterialType::WATER));
        assert!(!is_liquid(MaterialType::SAND));
        assert!(is_powder(MaterialType::SAND));
        assert!(!is_powder(MaterialType::STONE));
        assert!(is_rigid_solid(MaterialType::STONE));
        assert!(!is_rigid_solid(MaterialType::WATER));
    }
}
