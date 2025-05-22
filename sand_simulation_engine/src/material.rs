// src/material.rs

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
    GENERATOR,
    FUSE,
    ASH,
    ERASER,
}

#[derive(Debug, Clone)]
pub struct MaterialProperties {
    pub name: &'static str,
    pub density: f32,
    pub thermal_conductivity: f32,
    pub flammability: f32,
    pub melt_temperature: Option<f32>,
    pub boil_temperature: Option<f32>,
    pub freeze_temperature: Option<f32>,
    pub base_color: (u8, u8, u8),
    pub viscosity: f32,
    pub lifespan_seconds: Option<f32>,
    pub corrosive_power: f32,
    pub explosive_yield: Option<f32>,
    pub heat_generation: f32,
    pub ignition_temperature: Option<f32>,
}

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
            base_color: (0, 0, 0), // C_EMPTY
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
            base_color: (194, 178, 128), // C_SAND
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
            base_color: (50, 100, 200), // C_WATER
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
            melt_temperature: None, // Original JS: null, but LAVA turns to STONE at 1000, implies STONE can melt? For now, stick to JS.
            boil_temperature: None,
            freeze_temperature: None,
            base_color: (100, 100, 100), // C_STONE
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
            melt_temperature: Some(200.0), // Original JS: 200
            boil_temperature: None,
            freeze_temperature: None,
            base_color: (50, 150, 50), // C_PLANT
            viscosity: 1.0,
            lifespan_seconds: None,
            corrosive_power: 0.0,
            explosive_yield: None,
            heat_generation: 0.0,
            ignition_temperature: Some(150.0),
        },
        MaterialType::FIRE => MaterialProperties {
            name: "Fire",
            density: -2.0,
            thermal_conductivity: 0.9,
            flammability: 0.0,
            melt_temperature: None,
            boil_temperature: None,
            freeze_temperature: None,
            base_color: (255, 69, 0), // C_FIRE
            viscosity: 1.0,
            lifespan_seconds: Some(1.0),
            corrosive_power: 0.0,
            explosive_yield: None,
            heat_generation: 0.0, // Heat generation is handled by simulation logic, not base property here
            ignition_temperature: None,
        },
        MaterialType::LAVA => MaterialProperties {
            name: "Lava",
            density: 8.0,
            thermal_conductivity: 0.8,
            flammability: 0.0,
            melt_temperature: Some(1800.0), // Technically, it's already molten. This could be its "superheat" point or solidification point if cooled significantly below its "freeze_temperature".
            boil_temperature: None,
            freeze_temperature: Some(1000.0), // Solidifies to STONE
            base_color: (200, 50, 0), // C_LAVA
            viscosity: 5.0,
            lifespan_seconds: None,
            corrosive_power: 0.0,
            explosive_yield: None,
            heat_generation: 0.0, // Similar to FIRE, heat is part of its nature
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
            base_color: (210, 230, 240), // C_GLASS
            viscosity: 1.0,
            lifespan_seconds: None,
            corrosive_power: 0.0,
            explosive_yield: None,
            heat_generation: 0.0,
            ignition_temperature: None,
        },
        MaterialType::STEAM => MaterialProperties {
            name: "Steam",
            density: -5.0,
            thermal_conductivity: 0.7,
            flammability: 0.0,
            melt_temperature: None,
            boil_temperature: None, // Already gas
            freeze_temperature: Some(99.0), // Condenses to WATER
            base_color: (180, 180, 190), // C_STEAM
            viscosity: 1.0,
            lifespan_seconds: Some(10.0),
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
            base_color: (80, 70, 20), // C_OIL
            viscosity: 3.0,
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
            base_color: (100, 255, 100), // C_ACID
            viscosity: 1.0,
            lifespan_seconds: None,
            corrosive_power: 0.15,
            explosive_yield: None,
            heat_generation: 0.0,
            ignition_temperature: None,
        },
        MaterialType::COAL => MaterialProperties {
            name: "Coal",
            density: 4.0,
            thermal_conductivity: 0.2,
            flammability: 1.0,
            melt_temperature: Some(800.0), // Burns to FIRE/ASH, not melts in a typical sense
            boil_temperature: None,
            freeze_temperature: None,
            base_color: (40, 40, 40), // C_COAL
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
            flammability: 1.0,
            melt_temperature: None, // Explodes, doesn't melt
            boil_temperature: None,
            freeze_temperature: None,
            base_color: (60, 60, 70), // C_GUNPOWDER
            viscosity: 1.0,
            lifespan_seconds: None,
            corrosive_power: 0.0,
            explosive_yield: Some(4.0),
            heat_generation: 0.0,
            ignition_temperature: Some(150.0),
        },
        MaterialType::ICE => MaterialProperties {
            name: "Ice",
            density: 2.9,
            thermal_conductivity: 0.01,
            flammability: 0.0,
            melt_temperature: Some(1.0), // Melts to WATER
            boil_temperature: None,
            freeze_temperature: None, // Already solid
            base_color: (170, 200, 255), // C_ICE
            viscosity: 1.0,
            lifespan_seconds: None,
            corrosive_power: 0.0,
            explosive_yield: None,
            heat_generation: 0.0,
            ignition_temperature: None,
        },
        MaterialType::WOOD => MaterialProperties {
            name: "Wood",
            density: 0.7,
            thermal_conductivity: 0.2,
            flammability: 0.6,
            melt_temperature: Some(400.0), // Burns to FIRE/ASH
            boil_temperature: None,
            freeze_temperature: None,
            base_color: (139, 69, 19), // C_WOOD
            viscosity: 1.0,
            lifespan_seconds: None,
            corrosive_power: 0.0,
            explosive_yield: None,
            heat_generation: 0.0,
            ignition_temperature: Some(200.0),
        },
        MaterialType::SMOKE => MaterialProperties {
            name: "Smoke",
            density: -3.0,
            thermal_conductivity: 0.1,
            flammability: 0.0,
            melt_temperature: None,
            boil_temperature: None,
            freeze_temperature: None, // Dissipates
            base_color: (150, 150, 150), // C_SMOKE
            viscosity: 1.0,
            lifespan_seconds: Some(3.0),
            corrosive_power: 0.0,
            explosive_yield: None,
            heat_generation: 0.0,
            ignition_temperature: None,
        },
        MaterialType::TOXIC_GAS => MaterialProperties {
            name: "Toxic Gas",
            density: -4.0,
            thermal_conductivity: 0.1,
            flammability: 0.1, // Slightly flammable? JS says 0.1
            melt_temperature: None,
            boil_temperature: None,
            freeze_temperature: None, // Dissipates
            base_color: (150, 200, 150), // C_TOXIC_GAS
            viscosity: 1.0,
            lifespan_seconds: Some(5.0),
            corrosive_power: 0.02, // Corrosive power from JS
            explosive_yield: None,
            heat_generation: 0.0,
            ignition_temperature: None, // JS suggests no specific ignition temp, but it is flammable
        },
        MaterialType::SLIME => MaterialProperties {
            name: "Slime",
            density: 3.2,
            thermal_conductivity: 0.3,
            flammability: 0.1,
            melt_temperature: None,
            boil_temperature: Some(150.0), // Boils into TOXIC_GAS
            freeze_temperature: None,
            base_color: (100, 200, 100), // C_SLIME
            viscosity: 10.0,
            lifespan_seconds: None,
            corrosive_power: 0.0,
            explosive_yield: None,
            heat_generation: 0.0,
            ignition_temperature: None,
        },
        MaterialType::GASOLINE => MaterialProperties {
            name: "Gasoline",
            density: 0.8,
            thermal_conductivity: 0.5,
            flammability: 1.0,
            melt_temperature: None,
            boil_temperature: Some(80.0), // Evaporates/boils easily
            freeze_temperature: None,
            base_color: (255, 223, 186), // C_GASOLINE
            viscosity: 2.0,
            lifespan_seconds: None,
            corrosive_power: 0.0,
            explosive_yield: None, // Could consider adding one if it makes sense later
            heat_generation: 0.0,
            ignition_temperature: Some(100.0),
        },
        MaterialType::GENERATOR => MaterialProperties {
            name: "Generator",
            density: 100.0, // Very dense, immovable
            thermal_conductivity: 0.9,
            flammability: 0.0,
            melt_temperature: None,
            boil_temperature: None,
            freeze_temperature: None,
            base_color: (255, 0, 0), // C_GENERATOR
            viscosity: 1.0,
            lifespan_seconds: None,
            corrosive_power: 0.0,
            explosive_yield: None,
            heat_generation: 5.0, // Generates heat
            ignition_temperature: None,
        },
        MaterialType::FUSE => MaterialProperties {
            name: "Fuse",
            density: 5.0,
            thermal_conductivity: 0.2,
            flammability: 1.0, // It's very flammable
            melt_temperature: Some(150.0), // "Melts" or rather, burns away
            boil_temperature: None,
            freeze_temperature: None,
            base_color: (100, 80, 60), // C_FUSE
            viscosity: 1.0,
            lifespan_seconds: None, // Lifespan is handled by burning logic (FUSE_BURN_LIFESPAN_SEC)
            corrosive_power: 0.0,
            explosive_yield: None,
            heat_generation: 0.0, // Heat is generated when burning, not intrinsically
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
            base_color: (90, 90, 90), // C_ASH
            viscosity: 1.0,
            lifespan_seconds: None,
            corrosive_power: 0.0,
            explosive_yield: None,
            heat_generation: 0.0,
            ignition_temperature: None,
        },
        MaterialType::ERASER => MaterialProperties {
            name: "Eraser",
            density: 0.0, // Similar to EMPTY for interaction, but distinct type
            thermal_conductivity: 0.0,
            flammability: 0.0,
            melt_temperature: None,
            boil_temperature: None,
            freeze_temperature: None,
            base_color: (255,0,255), // C_ERASER
            viscosity: 1.0,
            lifespan_seconds: None,
            corrosive_power: 0.0,
            explosive_yield: None,
            heat_generation: 0.0,
            ignition_temperature: None,
        },
    }
}

// Helper functions to check material properties
// Ensure MaterialType is in scope (it is, as it's defined in this file)

pub fn is_liquid(material_type: MaterialType) -> bool {
    matches!(material_type, MaterialType::WATER | MaterialType::OIL | MaterialType::ACID | MaterialType::GASOLINE | MaterialType::LAVA)
}

pub fn is_powder(material_type: MaterialType) -> bool {
    matches!(material_type, MaterialType::SAND | MaterialType::ASH | MaterialType::GUNPOWDER | MaterialType::COAL)
}

pub fn is_rigid_solid(material_type: MaterialType) -> bool {
    matches!(material_type, MaterialType::STONE | MaterialType::GLASS | MaterialType::WOOD | MaterialType::ICE)
}
