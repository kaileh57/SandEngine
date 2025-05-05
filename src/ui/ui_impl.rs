// ui/ui_impl.rs - Main UI implementation for the sand simulation

use crate::constants::*;
use crate::material::MaterialType;
use crate::simulation::SandSimulation;
use super::components::{Button, Panel, Rect, Slider, StatusBar, ButtonAction};
use super::text_renderer::TextRenderer;

pub struct UI {
    text_renderer: TextRenderer,
    panels: Vec<Panel>,
    buttons: Vec<Button>,
    sliders: Vec<Slider>,
    status_bar: StatusBar,
}

impl UI {
    pub fn new() -> Self {
        let mut ui = Self {
            text_renderer: TextRenderer::new(),
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
        // Check button clicks
        for i in 0..self.buttons.len() {
            if self.buttons[i].rect.contains(x, y) {
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
            if slider.handle_click(x, y) {
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
        
        // Update and draw temperature display
        self.draw_temperature_display(frame, simulation);
        
        // Draw controls info
        self.draw_controls_info(frame);
        
        // Draw clear button
        self.buttons[self.buttons.len() - 1].draw(frame, &mut self.text_renderer);
        
        // Update status bar
        self.update_status_bar(simulation);
        self.status_bar.draw(frame, &mut self.text_renderer);
    }
    
    fn draw_temperature_display(&mut self, frame: &mut [u8], simulation: &SandSimulation) {
        let start_y = 495;
        if simulation.cursor_pos.0 < GRID_WIDTH && simulation.cursor_pos.1 < GRID_HEIGHT {
            let material = simulation.get(simulation.cursor_pos.0, simulation.cursor_pos.1);
            let temp = simulation.get_temp(simulation.cursor_pos.0, simulation.cursor_pos.1);
            
            if material != MaterialType::Empty {
                self.text_renderer.draw_text(
                    frame,
                    WIDTH as usize + 15,
                    start_y,
                    material.get_name(),
                    16.0,
                    C_UI_TEXT,
                );
                
                // Draw temperature value
                let temp_text = format!("{:.1}°C", temp);
                self.text_renderer.draw_text(
                    frame,
                    WIDTH as usize + 15,
                    start_y + 22,
                    &temp_text,
                    16.0,
                    C_UI_TEXT,
                );
                
                // Temperature bar
                let bar_y = start_y + 45;
                let bar_width = UI_WIDTH as usize - 30;
                let temp_factor = ((temp - AMBIENT_TEMP) / 1000.0).max(0.0).min(1.0);
                let temp_width = (temp_factor * bar_width as f32) as usize;
                
                // Draw temperature bar background
                for y in bar_y..bar_y + 10 {
                    for x in (WIDTH as usize + 15)..(WIDTH as usize + 15 + bar_width) {
                        let idx = (y * WINDOW_WIDTH as usize + x) * 4;
                        if idx + 3 < frame.len() {
                            frame[idx..idx+4].copy_from_slice(&[40, 40, 45, 255]);
                        }
                    }
                }
                
                // Draw temperature fill
                for y in bar_y..bar_y + 10 {
                    for x in (WIDTH as usize + 15)..(WIDTH as usize + 15 + temp_width) {
                        let idx = (y * WINDOW_WIDTH as usize + x) * 4;
                        if idx + 3 < frame.len() {
                            let progress = (x - (WIDTH as usize + 15)) as f32 / bar_width as f32;
                            let r = (progress * 255.0) as u8;
                            let b = (255.0 - progress * 255.0) as u8;
                            frame[idx..idx+4].copy_from_slice(&[r, 50, b, 255]);
                        }
                    }
                }
            } else {
                self.text_renderer.draw_text(
                    frame,
                    WIDTH as usize + 15,
                    start_y,
                    "Empty",
                    16.0,
                    C_UI_TEXT,
                );
                
                self.text_renderer.draw_text(
                    frame,
                    WIDTH as usize + 15,
                    start_y + 22,
                    "20.0°C",
                    16.0,
                    C_UI_TEXT,
                );
            }
        }
    }
    
    fn draw_controls_info(&mut self, frame: &mut [u8]) {
        let start_y = 605;
        let line_height = 20;
        
        let controls = [
            "Mouse: Draw",
            "1-6/E: Select material",
            "Wheel: Adjust brush size",
            "C: Clear simulation",
            "Esc: Quit",
        ];
        
        for (i, text) in controls.iter().enumerate() {
            self.text_renderer.draw_text(
                frame,
                WIDTH as usize + 15,
                start_y + i * line_height,
                text,
                15.0,
                C_UI_TEXT,
            );
        }
    }
    
    fn update_status_bar(&mut self, simulation: &SandSimulation) {
        self.status_bar.items.clear();
        
        // Cursor position
        self.status_bar.items.push((
            "Pos:".to_string(),
            format!("({}, {})", simulation.cursor_pos.0, simulation.cursor_pos.1),
        ));
        
        // Current material
        self.status_bar.items.push((
            "Mat:".to_string(),
            simulation.current_material.get_name().to_string(),
        ));
    }
    
    pub fn update_brush_size(&mut self, simulation: &mut SandSimulation, delta: i32) {
        let new_size = (simulation.brush_size as i32 + delta).clamp(0, 20) as usize;
        simulation.brush_size = new_size;
        
        // Update slider
        if let Some(slider) = self.sliders.get_mut(0) {
            slider.value = new_size as f32;
        }
    }
}