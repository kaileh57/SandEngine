// ui_helper.rs - Helper functions for UI integration with new material system

use crate::material_properties::MaterialType;
use crate::material;
use crate::material_converter::{to_new_material, to_old_material};
use crate::simulation::SandSimulation;

// This function converts the current_material in the simulation
// to the old material type used by the UI
pub fn get_old_material_for_ui(simulation: &SandSimulation) -> material::MaterialType {
    to_old_material(simulation.current_material)
}

// This function is used to update simulation.current_material 
// when a UI element selects a material
pub fn update_sim_material(simulation: &mut SandSimulation, ui_material: material::MaterialType) {
    simulation.current_material = to_new_material(ui_material);
}

// This function provides a material name for the UI
pub fn get_material_name(material: MaterialType) -> &'static str {
    material.get_name()
}

// This function provides material colors for the UI
pub fn get_material_color(material: MaterialType) -> [u8; 4] {
    material.get_color()
}

// This function is used to update the UI display with temperature
pub fn get_temp_display(simulation: &SandSimulation, x: usize, y: usize) -> (f32, &'static str) {
    let material = simulation.get(x, y);
    let temp = simulation.get_temp(x, y);
    
    (temp, material.get_name())
}

// This function provides a list of available materials for UI display
pub fn get_available_materials() -> Vec<(MaterialType, &'static str, [u8; 4])> {
    vec![
        (MaterialType::Sand, "Sand", MaterialType::Sand.get_color()),
        (MaterialType::Water, "Water", MaterialType::Water.get_color()),
        (MaterialType::Stone, "Stone", MaterialType::Stone.get_color()),
        (MaterialType::Plant, "Plant", MaterialType::Plant.get_color()),
        (MaterialType::Fire, "Fire", MaterialType::Fire.get_color()),
        (MaterialType::Lava, "Lava", MaterialType::Lava.get_color()),
        (MaterialType::Glass, "Glass", MaterialType::Glass.get_color()),
        (MaterialType::Ice, "Ice", MaterialType::Ice.get_color()),
        (MaterialType::Wood, "Wood", MaterialType::Wood.get_color()),
        (MaterialType::Coal, "Coal", MaterialType::Coal.get_color()),
        (MaterialType::Oil, "Oil", MaterialType::Oil.get_color()),
        (MaterialType::Acid, "Acid", MaterialType::Acid.get_color()),
        (MaterialType::Gunpowder, "Gunpowder", MaterialType::Gunpowder.get_color()),
        (MaterialType::Steam, "Steam", MaterialType::Steam.get_color()),
        (MaterialType::Fuse, "Fuse", MaterialType::Fuse.get_color()),
        (MaterialType::Generator, "Generator", MaterialType::Generator.get_color()),
        (MaterialType::Eraser, "Eraser", MaterialType::Eraser.get_color()),
    ]
}