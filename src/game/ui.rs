// File: game/ui.rs
use crate::engine::material::MaterialType;
use crate::engine::simulation::SandSimulation;
use crate::engine::constants::{WIDTH, HEIGHT};
use crate::game::constants::*;
use crate::game::text_renderer::TextRenderer;
use crate::game::ui_components::*;
use crate::game::material_renderer::MaterialRenderer;

pub struct UI {
    text_renderer: TextRenderer,
    material_renderer: MaterialRenderer,
    panels: Vec<Panel>,
    buttons: Vec<Button>,
    sliders: Vec<Slider>,
    status_bar: StatusBar,
}

impl UI {
    pub fn new() -> Self {
        let mut ui = Self {
            text_renderer: TextRenderer::new(),
            material_renderer: MaterialRenderer::new(),
            panels: Vec::new(),
            buttons: Vec::new(),
            sliders: Vec::new(),
            status_bar: StatusBar {
                rect: Rect::new(WIDTH as usize, HEIGHT as usize - 30, UI_WIDTH as usize, 30),
                items: Vec::new(),
            },
        };
        
        ui.setup_panels();
        ui.setup_buttons();
        ui.setup_sliders();
        
        ui
    }
    
    fn setup_panels(&mut self) {
        // Main title panel
        self.panels.push(Panel {
            rect: Rect::new(WIDTH as usize, 0, UI_WIDTH as usize, 60),
            title: "Sand Simulation".to_string(),
            padding: 15,
            background_color: [30, 30, 35, 255],
        });
        
        // Materials panel
        self.panels.push(Panel {
            rect: Rect::new(WIDTH as usize, 60, UI_WIDTH as usize, 290),
            title: "Materials".to_string(),
            padding: 15,
            background_color: C_UI_BG,
        });
        
        // Brush settings panel
        self.panels.push(Panel {
            rect: Rect::new(WIDTH as usize, 350, UI_WIDTH as usize, 110),
            title: "Brush Settings".to_string(),
            padding: 15,
            background_color: C_UI_BG,
        });
        
        // Temperature display panel
        self.panels.push(Panel {
            rect: Rect::new(WIDTH as usize, 460, UI_WIDTH as usize, 110),
            title: "Temperature".to_string(),
            padding: 15,
            background_color: C_UI_BG,
        });
        
        // Controls panel
        self.panels.push(Panel {
            rect: Rect::new(WIDTH as usize, 570, UI_WIDTH as usize, 135),
            title: "Controls".to_string(),
            padding: 15,
            background_color: C_UI_BG,
        });
    }
    
    fn setup_buttons(&mut self) {
        let materials = [
            (MaterialType::Sand, "Sand", C_SAND),
            (MaterialType::Water, "Water", C_WATER),
            (MaterialType::Stone, "Stone", C_STONE),
            (MaterialType::Plant, "Plant", C_PLANT),
            (MaterialType::Fire, "Fire", C_FIRE),
            (MaterialType::Lava, "Lava", C_LAVA),
            (MaterialType::Eraser, "Eraser", C_ERASER),
        ];
        
        let button_height = 32;
        let button_spacing = 6;
        let start_y = 95;  // Start below the panel title
        let button_width = UI_WIDTH as usize - 30;  // Full width minus padding
        
        for (i, (material, name, color)) in materials.iter().enumerate() {
            let button_y = start_y + i * (button_height + button_spacing);
            self.buttons.push(Button {
                rect: Rect::new(WIDTH as usize + 15, button_y, button_width, button_height),
                label: name.to_string(),
                is_selected: i == 0,  // Select Sand by default
                color: Some(*color),
                action: ButtonAction::SelectMaterial(material.to_u8() as usize),
            });
        }
        
        // Clear button - positioned below controls panel
        self.buttons.push(Button {
            rect: Rect::new(WIDTH as usize + 60, 715, UI_WIDTH as usize - 120, 40),
            label: "CLEAR".to_string(),
            is_selected: false,
            color: None,
            action: ButtonAction::ClearSimulation,
        });
    }
    
    fn setup_sliders(&mut self) {
        // Brush size slider
        self.sliders.push(Slider {
            rect: Rect::new(WIDTH as usize + 15, 410, UI_WIDTH as usize - 30, 12),
            value: 3.0,
            min_value: 0.0,
            max_value: 20.0,
            label: "Brush Size".to_string(),
        });
    }
    
    pub fn handle_click(&mut self, simulation: &mut SandSimulation, x: usize, y: usize) {
        // Convert to UI-relative coordinates
        let ui_x = x;
        let ui_y = y;
        
        // Check button clicks
        for i in 0..self.buttons.len() {
            if self.buttons[i].rect.contains(ui_x, ui_y) {
                // Clone the action before mutating the buttons
                let action = self.buttons[i].action.clone();
                match action {
                    ButtonAction::SelectMaterial(material_id) => {
                        // Deselect all material buttons
                        for j in 0..self.buttons.len() {
                            if matches!(self.buttons[j].action, ButtonAction::SelectMaterial(_)) {
                                self.buttons[j].is_selected = false;
                            }
                        }
                        // Select this button
                        self.buttons[i].is_selected = true;
                        simulation.current_material = MaterialType::from_u8(material_id as u8);
                    },
                    ButtonAction::ClearSimulation => {
                        simulation.clear();
                    },
                    ButtonAction::None => {},
                }
                return;
            }
        }
        
        // Check slider clicks
        for slider in &mut self.sliders {
            if slider.handle_click(ui_x, ui_y) {
                simulation.brush_size = slider.value as usize;
                return;
            }
        }
    }
    
    pub fn draw(&mut self, frame: &mut [u8], simulation: &SandSimulation) {
        // Draw panels
        for panel in &self.panels {
            panel.draw(frame, &mut self.text_renderer);
        }
        
        // Draw material buttons
        for i in 0..self.buttons.len() - 1 {
            self.buttons[i].draw(frame, &mut self.text_renderer);
        }
        
        // Draw brush size info above slider
        self.text_renderer.draw_text(
            frame,
            WIDTH as usize + 15,
            390,
            &format!("Size: {}", simulation.brush_size),
            16.0,
            C_UI_TEXT,
        );
        
        // Draw sliders
        for slider in &self.sliders {
            slider.draw(frame, &mut self.text_renderer);
        }
        
        // Draw hint text below slider
        self.text_renderer.draw_text(
            frame,
            WIDTH as usize + 15,
            430,
            "Use mouse wheel to adjust",
            14.0,
            [150, 150, 150, 255],
        );
        
        // Draw temperature display
        self.draw_temperature_display(frame, simulation);
        
        // Draw controls info
        self.draw_controls_info(frame);
        
        // Update and draw status bar
        self.update_status_bar(simulation);
        self.status_bar.draw(frame, &mut self.text_renderer);
        
        // Draw the clear button last (on top)
        self.buttons.last().unwrap().draw(frame, &mut self.text_renderer);
    }
    
    fn draw_temperature_display(&mut self, frame: &mut [u8], simulation: &SandSimulation) {
        // Only display temperature if we have a valid cursor position in the simulation area
        let (x, y) = simulation.cursor_pos;
        if x < WIDTH as usize / SandSimulation::get_cell_size() && y < HEIGHT as usize / SandSimulation::get_cell_size() {
            let temp = simulation.get_temp(x, y);
            let temp_bar_width = UI_WIDTH as usize - 30;
            let temp_bar_rect = Rect::new(WIDTH as usize + 15, 510, temp_bar_width, 20);
            
            // Draw temperature bar background
            for y in temp_bar_rect.y..temp_bar_rect.y + temp_bar_rect.height {
                for x in temp_bar_rect.x..temp_bar_rect.x + temp_bar_rect.width {
                    let idx = (y * WINDOW_WIDTH as usize + x) * 4;
                    if idx + 3 < frame.len() {
                        frame[idx..idx+4].copy_from_slice(&[60, 60, 70, 255]);
                    }
                }
            }
            
            // Draw temperature bar (normalized to 0-3000)
            let filled_width = ((temp / 3000.0) * temp_bar_width as f32).min(temp_bar_width as f32) as usize;
            
            for y in temp_bar_rect.y..temp_bar_rect.y + temp_bar_rect.height {
                for x in temp_bar_rect.x..temp_bar_rect.x + filled_width {
                    let normalized_pos = (x - temp_bar_rect.x) as f32 / temp_bar_width as f32;
                    
                    // Color gradient from blue (cool) to red (hot)
                    let r = (normalized_pos * 255.0) as u8;
                    let g = ((1.0 - normalized_pos) * 100.0) as u8;
                    let b = ((1.0 - normalized_pos) * 255.0) as u8;
                    
                    let idx = (y * WINDOW_WIDTH as usize + x) * 4;
                    if idx + 3 < frame.len() {
                        frame[idx] = r;
                        frame[idx + 1] = g;
                        frame[idx + 2] = b;
                        frame[idx + 3] = 255;
                    }
                }
            }
            
            // Draw temperature bar border
            for x in temp_bar_rect.x..temp_bar_rect.x + temp_bar_rect.width {
                let top_idx = (temp_bar_rect.y * WINDOW_WIDTH as usize + x) * 4;
                let bottom_idx = ((temp_bar_rect.y + temp_bar_rect.height - 1) * WINDOW_WIDTH as usize + x) * 4;
                
                if top_idx + 3 < frame.len() {
                    frame[top_idx..top_idx+4].copy_from_slice(&[120, 120, 140, 255]);
                }
                if bottom_idx + 3 < frame.len() {
                    frame[bottom_idx..bottom_idx+4].copy_from_slice(&[120, 120, 140, 255]);
                }
            }
            
            for y in temp_bar_rect.y..temp_bar_rect.y + temp_bar_rect.height {
                let left_idx = (y * WINDOW_WIDTH as usize + temp_bar_rect.x) * 4;
                let right_idx = (y * WINDOW_WIDTH as usize + temp_bar_rect.x + temp_bar_rect.width - 1) * 4;
                
                if left_idx + 3 < frame.len() {
                    frame[left_idx..left_idx+4].copy_from_slice(&[120, 120, 140, 255]);
                }
                if right_idx + 3 < frame.len() {
                    frame[right_idx..right_idx+4].copy_from_slice(&[120, 120, 140, 255]);
                }
            }
            
            // Display temperature text
            self.text_renderer.draw_text(
                frame,
                WIDTH as usize + 15,
                490,
                &format!("Temperature: {:.1}°C", temp),
                16.0,
                C_UI_TEXT,
            );
        }
    }
    
    fn draw_controls_info(&mut self, frame: &mut [u8]) {
        let controls = [
            "1-6: Select materials",
            "E: Eraser",
            "C: Clear simulation",
            "Mouse wheel: Adjust brush size",
            "ESC: Exit",
        ];
        
        for (i, control) in controls.iter().enumerate() {
            self.text_renderer.draw_text(
                frame,
                WIDTH as usize + 15,
                600 + i * 20,
                control,
                14.0,
                C_UI_TEXT,
            );
        }
    }
    
    fn update_status_bar(&mut self, simulation: &SandSimulation) {
        let (x, y) = simulation.cursor_pos;
        let material = simulation.get(x, y);
        let material_name = self.material_renderer.get_material_name(material);
        let temp = simulation.get_temp(x, y);
        
        self.status_bar.items.clear();
        self.status_bar.items.push(("Position: ".to_string(), format!("({}, {})", x, y)));
        self.status_bar.items.push(("Material: ".to_string(), material_name.to_string()));
        self.status_bar.items.push(("Temp: ".to_string(), format!("{:.1}°C", temp)));
    }
    
    pub fn update_brush_size(&mut self, simulation: &mut SandSimulation, delta: i32) {
        let current_size = simulation.brush_size as i32;
        let new_size = (current_size + delta).max(1).min(20) as usize;
        simulation.brush_size = new_size;
        
        // Update slider to match
        if let Some(slider) = self.sliders.first_mut() {
            slider.value = new_size as f32;
        }
    }
} 