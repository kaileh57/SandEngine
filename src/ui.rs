// File: ui.rs
use crate::constants::*;
use crate::material::MaterialType;
use crate::simulation::SandSimulation;

pub struct UI {
    // UI state
    button_hover: Option<MaterialType>,
}

struct UIButton {
    x: usize,
    y: usize,
    width: usize,
    height: usize,
    material: MaterialType,
    label: &'static str,
}

struct UIElement {
    x: usize,
    y: usize,
    width: usize,
    height: usize,
    elem_type: UIElementType,
}

enum UIElementType {
    MaterialButton(MaterialType),
    ClearButton,
    BrushSize,
    InfoPanel,
}

impl UI {
    pub fn new() -> Self {
        Self {
            button_hover: None,
        }
    }

    pub fn handle_click(&mut self, simulation: &mut SandSimulation, x: usize, y: usize) {
        // Convert to UI-relative coordinates
        let ui_x = x - WIDTH as usize;
        
        // Material buttons section
        if y >= 50 && y < 230 {
            let materials = [
                MaterialType::Sand,
                MaterialType::Water,
                MaterialType::Stone,
                MaterialType::Plant,
                MaterialType::Fire,
                MaterialType::Lava,
                MaterialType::Eraser
            ];
            
            let button_height = 22;
            let padding = 4;
            
            for (i, &mat) in materials.iter().enumerate() {
                let button_y = 50 + i * (button_height + padding);
                if y >= button_y && y < button_y + button_height && 
                   ui_x >= 10 && ui_x < UI_WIDTH as usize - 10 {
                    simulation.current_material = mat;
                    break;
                }
            }
        }
        
        // Brush size slider
        if y >= 240 && y < 260 {
            if ui_x >= 10 && ui_x < UI_WIDTH as usize - 10 {
                // Calculate brush size from click position
                let max_width = (UI_WIDTH as usize) - 20;
                let click_pos = ui_x - 10;
                let brush_size = ((click_pos as f32 / max_width as f32) * 20.0).round() as usize;
                simulation.brush_size = brush_size.max(1).min(20);
            }
        }
        
        // Clear button
        if y >= 350 && y < 380 {
            if ui_x >= 30 && ui_x < UI_WIDTH as usize - 30 {
                simulation.clear();
            }
        }
    }

    pub fn draw(&self, frame: &mut [u8], simulation: &SandSimulation) {
        // Draw UI panel background
        for y in 0..HEIGHT as usize {
            for x in WIDTH as usize..WINDOW_WIDTH as usize {
                let idx = (y * WINDOW_WIDTH as usize + x) * 4;
                if idx + 3 < frame.len() {
                    frame[idx..idx+4].copy_from_slice(&C_UI_BG);
                }
            }
        }

        // Draw title
        draw_text(frame, WIDTH as usize + 50, 15, "Sand Simulation", C_UI_TEXT);
        
        // Draw divider line
        for x in WIDTH as usize..WINDOW_WIDTH as usize {
            let idx = (40 * WINDOW_WIDTH as usize + x) * 4;
            if idx + 3 < frame.len() {
                frame[idx..idx+4].copy_from_slice(&[60, 60, 70, 255]);
            }
        }
        
        // Draw material buttons
        self.draw_material_buttons(frame, simulation);
        
        // Draw brush size control
        self.draw_brush_size_control(frame, simulation);
        
        // Draw temperature display
        self.draw_temperature_display(frame, simulation);
        
        // Draw controls info
        self.draw_controls_info(frame);
        
        // Draw clear button
        self.draw_clear_button(frame);
    }
    
    fn draw_material_buttons(&self, frame: &mut [u8], simulation: &SandSimulation) {
        // Material selection title
        draw_text(frame, WIDTH as usize + 15, 50, "Materials:", C_UI_TEXT);
        
        // Material palette buttons
        let materials = [
            MaterialType::Sand,
            MaterialType::Water, 
            MaterialType::Stone,
            MaterialType::Plant,
            MaterialType::Fire,
            MaterialType::Lava,
            MaterialType::Eraser
        ];
        
        let button_height = 22;
        let padding = 4;
        
        for (i, &mat) in materials.iter().enumerate() {
            let button_y = 60 + i * (button_height + padding);
            let is_selected = mat == simulation.current_material;
            
            // Button background
            for y in button_y..button_y + button_height {
                for x in (WIDTH as usize + 10)..(WINDOW_WIDTH as usize - 10) {
                    let idx = (y * WINDOW_WIDTH as usize + x) * 4;
                    
                    if idx + 3 < frame.len() {
                        if is_selected {
                            frame[idx..idx+4].copy_from_slice(&C_UI_BUTTON_SELECTED);
                        } else {
                            frame[idx..idx+4].copy_from_slice(&C_UI_BUTTON);
                        }
                    }
                }
            }
            
            // Material color sample
            for y in button_y + 2..button_y + button_height - 2 {
                for x in (WIDTH as usize + 15)..(WIDTH as usize + 30) {
                    let idx = (y * WINDOW_WIDTH as usize + x) * 4;
                    
                    if idx + 3 < frame.len() {
                        frame[idx..idx+4].copy_from_slice(&mat.get_color());
                    }
                }
            }
            
            // Button text
            draw_text(frame, WIDTH as usize + 35, button_y + 7, mat.get_name(), C_UI_TEXT);
            
            // Button border when selected
            if is_selected {
                for y in button_y..button_y + button_height {
                    for x in (WIDTH as usize + 10)..(WINDOW_WIDTH as usize - 10) {
                        if y == button_y || y == button_y + button_height - 1 ||
                           x == WIDTH as usize + 10 || x == WINDOW_WIDTH as usize - 11 {
                            let idx = (y * WINDOW_WIDTH as usize + x) * 4;
                            
                            if idx + 3 < frame.len() {
                                frame[idx..idx+4].copy_from_slice(&C_UI_BUTTON_BORDER);
                            }
                        }
                    }
                }
            }
            
            // Keyboard shortcut
            let shortcut = match mat {
                MaterialType::Sand => "1",
                MaterialType::Water => "2",
                MaterialType::Stone => "3",
                MaterialType::Plant => "4",
                MaterialType::Fire => "5",
                MaterialType::Lava => "6",
                MaterialType::Eraser => "E",
                _ => "",
            };
            
            if !shortcut.is_empty() {
                draw_text(frame, 
                         WINDOW_WIDTH as usize - 25, 
                         button_y + 7, 
                         shortcut, 
                         C_UI_TEXT);
            }
        }
    }
    
    fn draw_brush_size_control(&self, frame: &mut [u8], simulation: &SandSimulation) {
        // Brush size title
        draw_text(frame, WIDTH as usize + 15, 240, "Brush Size:", C_UI_TEXT);
        
        // Brush size bar
        let bar_y = 250;
        let max_width = (UI_WIDTH as usize) - 20;
        let brush_width = ((simulation.brush_size as f32 / 20.0) * max_width as f32) as usize;
        
        // Draw background bar
        for y in bar_y..bar_y + 10 {
            for x in (WIDTH as usize + 10)..(WINDOW_WIDTH as usize - 10) {
                let idx = (y * WINDOW_WIDTH as usize + x) * 4;
                
                if idx + 3 < frame.len() {
                    frame[idx..idx+4].copy_from_slice(&[60, 60, 70, 255]);
                }
            }
        }
        
        // Draw filled portion
        for y in bar_y..bar_y + 10 {
            for x in (WIDTH as usize + 10)..(WIDTH as usize + 10 + brush_width) {
                let idx = (y * WINDOW_WIDTH as usize + x) * 4;
                
                if idx + 3 < frame.len() {
                    frame[idx..idx+4].copy_from_slice(&[180, 180, 200, 255]);
                }
            }
        }
        
        // Draw tick marks
        for tick in 0..=5 {
            let tick_x = WIDTH as usize + 10 + (tick * max_width / 5);
            for y in [bar_y - 1, bar_y + 10] {
                let idx = (y * WINDOW_WIDTH as usize + tick_x) * 4;
                
                if idx + 3 < frame.len() {
                    frame[idx..idx+4].copy_from_slice(&[150, 150, 170, 255]);
                }
            }
        }
        
        // Draw brush size number
        let size_text = format!("Size: {}", simulation.brush_size);
        draw_text(frame, WIDTH as usize + 15, bar_y + 15, &size_text, C_UI_TEXT);
        
        // Draw hint
        draw_text(frame, WIDTH as usize + 15, bar_y + 28, "Use mouse wheel to adjust", [150, 150, 150, 255]);
    }
    
    fn draw_temperature_display(&self, frame: &mut [u8], simulation: &SandSimulation) {
        // Temperature display title
        draw_text(frame, WIDTH as usize + 15, 290, "Temperature:", C_UI_TEXT);
        
        if simulation.cursor_pos.0 < GRID_WIDTH && simulation.cursor_pos.1 < GRID_HEIGHT {
            let material = simulation.get(simulation.cursor_pos.0, simulation.cursor_pos.1);
            let temp = simulation.get_temp(simulation.cursor_pos.0, simulation.cursor_pos.1);
            
            // Draw material at cursor
            let material_name = material.get_name();
            draw_text(frame, WIDTH as usize + 15, 305, material_name, C_UI_TEXT);
            
            // Draw material color at cursor
            for y in 305..315 {
                for x in (WIDTH as usize + 100)..(WIDTH as usize + 115) {
                    let idx = (y * WINDOW_WIDTH as usize + x) * 4;
                    
                    if idx + 3 < frame.len() {
                        frame[idx..idx+4].copy_from_slice(&material.get_color());
                    }
                }
            }
            
            // Draw box around material color
            for y in 305..315 {
                for x in (WIDTH as usize + 100)..(WIDTH as usize + 115) {
                    if y == 305 || y == 314 || x == WIDTH as usize + 100 || x == WIDTH as usize + 114 {
                        let idx = (y * WINDOW_WIDTH as usize + x) * 4;
                        
                        if idx + 3 < frame.len() {
                            frame[idx..idx+4].copy_from_slice(&[120, 120, 140, 255]);
                        }
                    }
                }
            }
            
            // Draw temperature value
            let temp_text = format!("{:.1}°C", temp);
            draw_text(frame, WIDTH as usize + 15, 320, &temp_text, C_UI_TEXT);
            
            // Temperature bar
            let bar_y = 325;
            let bar_height = 8;
            let max_width = (UI_WIDTH as usize) - 30;
            
            // Temperature normalized to 0.0-1.0 (from AMBIENT_TEMP to 1000°C)
            let temp_factor = ((temp - AMBIENT_TEMP) / 1000.0).max(0.0).min(1.0);
            let temp_width = (temp_factor * max_width as f32) as usize;
            
            // Draw temperature bar background
            for y in bar_y..bar_y + bar_height {
                for x in (WIDTH as usize + 15)..(WIDTH as usize + 15 + max_width) {
                    let idx = (y * WINDOW_WIDTH as usize + x) * 4;
                    
                    if idx + 3 < frame.len() {
                        frame[idx..idx+4].copy_from_slice(&[40, 40, 45, 255]);
                    }
                }
            }
            
            // Draw temperature bar fill with gradient from blue to red
            for y in bar_y..bar_y + bar_height {
                for x in (WIDTH as usize + 15)..(WIDTH as usize + 15 + temp_width) {
                    let idx = (y * WINDOW_WIDTH as usize + x) * 4;
                    
                    if idx + 3 < frame.len() {
                        // Gradient from blue (cold) to red (hot)
                        let progress = (x - (WIDTH as usize + 15)) as f32 / temp_width as f32;
                        let r = (progress * 255.0) as u8;
                        let b = (255.0 - progress * 255.0) as u8;
                        frame[idx..idx+4].copy_from_slice(&[r, 50, b, 255]);
                    }
                }
            }
            
            // Draw border around temperature bar
            for y in bar_y..bar_y + bar_height {
                for x in (WIDTH as usize + 15)..(WIDTH as usize + 15 + max_width) {
                    if y == bar_y || y == bar_y + bar_height - 1 || 
                       x == WIDTH as usize + 15 || x == WIDTH as usize + 15 + max_width - 1 {
                        let idx = (y * WINDOW_WIDTH as usize + x) * 4;
                        
                        if idx + 3 < frame.len() {
                            frame[idx..idx+4].copy_from_slice(&[80, 80, 100, 255]);
                        }
                    }
                }
            }
        }
    }
    
    fn draw_controls_info(&self, frame: &mut [u8]) {
        // Controls section title
        draw_text(frame, WIDTH as usize + 15, 350, "Controls:", C_UI_TEXT);
        
        // Controls info
        draw_text(frame, WIDTH as usize + 15, 365, "Mouse: Draw", C_UI_TEXT);
        draw_text(frame, WIDTH as usize + 15, 380, "1-6/E: Select material", C_UI_TEXT);
        draw_text(frame, WIDTH as usize + 15, 395, "C: Clear simulation", C_UI_TEXT);
        draw_text(frame, WIDTH as usize + 15, 410, "Esc: Quit", C_UI_TEXT);
    }
    
    fn draw_clear_button(&self, frame: &mut [u8]) {
        // Clear button
        let button_y = 435;
        let button_height = 30;
        
        // Button background
        for y in button_y..button_y + button_height {
            for x in (WIDTH as usize + 30)..(WINDOW_WIDTH as usize - 30) {
                let idx = (y * WINDOW_WIDTH as usize + x) * 4;
                
                if idx + 3 < frame.len() {
                    frame[idx..idx+4].copy_from_slice(&C_UI_CLEAR_BUTTON);
                }
            }
        }
        
        // Button border
        for y in button_y..button_y + button_height {
            for x in (WIDTH as usize + 30)..(WINDOW_WIDTH as usize - 30) {
                if y == button_y || y == button_y + button_height - 1 || 
                   x == WIDTH as usize + 30 || x == WINDOW_WIDTH as usize - 31 {
                    let idx = (y * WINDOW_WIDTH as usize + x) * 4;
                    
                    if idx + 3 < frame.len() {
                        frame[idx..idx+4].copy_from_slice(&C_UI_CLEAR_BUTTON_BORDER);
                    }
                }
            }
        }
        
        // Button text
        draw_text(frame, WIDTH as usize + 75, button_y + 10, "CLEAR", C_UI_TEXT);
    }
}