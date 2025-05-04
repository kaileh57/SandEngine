// File: game/material_renderer.rs
use crate::engine::material::MaterialType;
use crate::engine::constants::*;
use crate::engine::simulation::SandSimulation;
use crate::game::constants::*;

pub struct MaterialRenderer;

impl MaterialRenderer {
    pub fn new() -> Self {
        Self {}
    }
    
    pub fn get_material_color(&self, material: MaterialType) -> [u8; 4] {
        match material {
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
    
    pub fn get_material_name(&self, material: MaterialType) -> &'static str {
        match material {
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
    
    // Helper method to get temperature color modification
    pub fn get_temp_color_modifier(&self, simulation: &SandSimulation, x: usize, y: usize) -> [i16; 3] {
        let temp = simulation.get_temp(x, y);
        let material = simulation.get(x, y);
        
        // No temperature visualization for empty cells
        if material == MaterialType::Empty {
            return [0, 0, 0];
        }
        
        // Special cases for fire/lava which have their own visualization
        if material == MaterialType::Fire || material == MaterialType::Lava {
            return [0, 0, 0];
        }
        
        // Calculate temperature factor (-1.0 to 1.0)
        let temp_factor = ((temp - AMBIENT_TEMP) / 150.0).max(-1.0).min(1.0);
        
        // Red increases with temperature, blue decreases
        let r_mod = (temp_factor * 50.0) as i16;
        let g_mod = (temp_factor * 20.0) as i16;
        let b_mod = (-temp_factor * 30.0) as i16;
        
        [r_mod, g_mod, b_mod]
    }

    pub fn draw(&self, simulation: &SandSimulation, frame: &mut [u8]) {
        for y in 0..GRID_HEIGHT {
            for x in 0..GRID_WIDTH {
                let material = simulation.get(x, y);
                let base_color = self.get_material_color(material);
                
                // Apply temperature color modification
                let temp_mod = self.get_temp_color_modifier(simulation, x, y);
                
                let r = (base_color[0] as i16 + temp_mod[0]).max(0).min(255) as u8;
                let g = (base_color[1] as i16 + temp_mod[1]).max(0).min(255) as u8;
                let b = (base_color[2] as i16 + temp_mod[2]).max(0).min(255) as u8;
                let a = base_color[3];
                
                // Draw the cell (scaled by CELL_SIZE)
                for dy in 0..CELL_SIZE {
                    for dx in 0..CELL_SIZE {
                        let px = x * CELL_SIZE + dx;
                        let py = y * CELL_SIZE + dy;
                        
                        // Calculate index in frame buffer
                        let idx = ((py * WINDOW_WIDTH as usize) + px) * 4;
                        
                        if idx + 3 < frame.len() {
                            frame[idx] = r;
                            frame[idx + 1] = g;
                            frame[idx + 2] = b;
                            frame[idx + 3] = a;
                        }
                    }
                }
            }
        }
        
        // Draw a border around the simulation area
        for y in 0..HEIGHT as usize {
            for x in 0..WIDTH as usize {
                // Draw border (1 pixel width)
                if x < 1 || x >= WIDTH as usize - 1 || 
                   y < 1 || y >= HEIGHT as usize - 1 {
                    let idx = (y * WINDOW_WIDTH as usize + x) * 4;
                    if idx + 3 < frame.len() {
                        frame[idx..idx+4].copy_from_slice(&C_BORDER);
                    }
                }
            }
        }
    }
} 