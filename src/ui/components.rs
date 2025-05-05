// ui/components.rs - UI component definitions

use crate::constants::*;
use crate::ui::text_renderer::TextRenderer;

#[derive(Debug, Clone, Copy)]
pub struct Rect {
    pub x: usize,
    pub y: usize,
    pub width: usize,
    pub height: usize,
}

impl Rect {
    pub fn new(x: usize, y: usize, width: usize, height: usize) -> Self {
        Self { x, y, width, height }
    }
    
    pub fn contains(&self, point_x: usize, point_y: usize) -> bool {
        point_x >= self.x && 
        point_x < self.x + self.width && 
        point_y >= self.y && 
        point_y < self.y + self.height
    }
}

#[derive(Debug, Clone)]
pub enum ButtonAction {
    SelectMaterial(usize),
    ClearSimulation,
    None,
}

#[derive(Debug)]
pub struct Button {
    pub rect: Rect,
    pub label: String,
    pub is_selected: bool,
    pub color: Option<[u8; 4]>,
    pub action: ButtonAction,
}

pub struct Panel {
    pub rect: Rect,
    pub title: String,
    pub padding: usize,
    pub background_color: [u8; 4],
}

impl Panel {
    pub fn draw(&self, frame: &mut [u8], text_renderer: &mut TextRenderer) {
        // Draw panel background
        for y in self.rect.y..self.rect.y + self.rect.height {
            for x in self.rect.x..self.rect.x + self.rect.width {
                let idx = (y * WINDOW_WIDTH as usize + x) * 4;
                if idx + 3 < frame.len() {
                    frame[idx..idx+4].copy_from_slice(&self.background_color);
                }
            }
        }
        
        // Draw panel title - centered
        if !self.title.is_empty() {
            // For main title, center it
            if self.title == "Sand Simulation" {
                text_renderer.draw_text(
                    frame,
                    self.rect.x + self.rect.width / 2 - 75, // Approximate centering
                    self.rect.y + 18,
                    &self.title,
                    24.0,
                    C_UI_TEXT,
                );
            } else {
                // Regular panel titles
                text_renderer.draw_text(
                    frame,
                    self.rect.x + self.padding,
                    self.rect.y + self.padding,
                    &self.title,
                    18.0,
                    C_UI_TEXT,
                );
            }
        }
    }
    
    pub fn content_rect(&self) -> Rect {
        let title_height = if self.title.is_empty() { 0 } else { 30 };
        Rect::new(
            self.rect.x + self.padding,
            self.rect.y + self.padding + title_height,
            self.rect.width - 2 * self.padding,
            self.rect.height - 2 * self.padding - title_height,
        )
    }
}

pub struct Slider {
    pub rect: Rect,
    pub value: f32,
    pub min_value: f32,
    pub max_value: f32,
    pub label: String,
}

impl Slider {
    pub fn draw(&self, frame: &mut [u8], text_renderer: &mut TextRenderer) {
        // Draw slider background
        for y in self.rect.y..self.rect.y + self.rect.height {
            for x in self.rect.x..self.rect.x + self.rect.width {
                let idx = (y * WINDOW_WIDTH as usize + x) * 4;
                if idx + 3 < frame.len() {
                    frame[idx..idx+4].copy_from_slice(&[60, 60, 70, 255]);
                }
            }
        }
        
        // Draw filled portion
        let filled_width = ((self.value - self.min_value) / (self.max_value - self.min_value) * self.rect.width as f32) as usize;
        for y in self.rect.y..self.rect.y + self.rect.height {
            for x in self.rect.x..self.rect.x + filled_width {
                let idx = (y * WINDOW_WIDTH as usize + x) * 4;
                if idx + 3 < frame.len() {
                    frame[idx..idx+4].copy_from_slice(&[180, 180, 200, 255]);
                }
            }
        }
        
        // Draw border
        self.draw_border(frame, [80, 80, 100, 255]);
    }
    
    fn draw_border(&self, frame: &mut [u8], color: [u8; 4]) {
        // Top and bottom borders
        for x in self.rect.x..self.rect.x + self.rect.width {
            let top_idx = (self.rect.y * WINDOW_WIDTH as usize + x) * 4;
            let bottom_idx = ((self.rect.y + self.rect.height - 1) * WINDOW_WIDTH as usize + x) * 4;
            
            if top_idx + 3 < frame.len() {
                frame[top_idx..top_idx+4].copy_from_slice(&color);
            }
            if bottom_idx + 3 < frame.len() {
                frame[bottom_idx..bottom_idx+4].copy_from_slice(&color);
            }
        }
        
        // Left and right borders
        for y in self.rect.y..self.rect.y + self.rect.height {
            let left_idx = (y * WINDOW_WIDTH as usize + self.rect.x) * 4;
            let right_idx = (y * WINDOW_WIDTH as usize + self.rect.x + self.rect.width - 1) * 4;
            
            if left_idx + 3 < frame.len() {
                frame[left_idx..left_idx+4].copy_from_slice(&color);
            }
            if right_idx + 3 < frame.len() {
                frame[right_idx..right_idx+4].copy_from_slice(&color);
            }
        }
    }
    
    pub fn handle_click(&mut self, x: usize, y: usize) -> bool {
        if self.rect.contains(x, y) {
            let relative_x = x - self.rect.x;
            self.value = self.min_value + (relative_x as f32 / self.rect.width as f32) * (self.max_value - self.min_value);
            self.value = self.value.clamp(self.min_value, self.max_value);
            true
        } else {
            false
        }
    }
}

impl Button {
    pub fn draw(&self, frame: &mut [u8], text_renderer: &mut TextRenderer) {
        // Draw button background
        let bg_color = if self.is_selected { C_UI_BUTTON_SELECTED } else { C_UI_BUTTON };
        for y in self.rect.y..self.rect.y + self.rect.height {
            for x in self.rect.x..self.rect.x + self.rect.width {
                let idx = (y * WINDOW_WIDTH as usize + x) * 4;
                if idx + 3 < frame.len() {
                    frame[idx..idx+4].copy_from_slice(&bg_color);
                }
            }
        }
        
        // Draw color sample if present
        let text_offset = if let Some(color) = self.color {
            let color_box_size = self.rect.height - 8;
            let color_box_x = self.rect.x + 4;
            let color_box_y = self.rect.y + 4;
            
            for y in color_box_y..color_box_y + color_box_size {
                for x in color_box_x..color_box_x + color_box_size {
                    let idx = (y * WINDOW_WIDTH as usize + x) * 4;
                    if idx + 3 < frame.len() {
                        frame[idx..idx+4].copy_from_slice(&color);
                    }
                }
            }
            
            color_box_size + 8
        } else {
            4
        };
        
        // Draw button text
        text_renderer.draw_text(
            frame,
            self.rect.x + text_offset,
            self.rect.y + (self.rect.height - 14) / 2,  // Center vertically
            &self.label,
            14.0,
            C_UI_TEXT,
        );
        
        // Draw border for selected button
        if self.is_selected {
            self.draw_border(frame, C_UI_BUTTON_BORDER);
        }
    }
    
    fn draw_border(&self, frame: &mut [u8], color: [u8; 4]) {
        // Top and bottom borders
        for x in self.rect.x..self.rect.x + self.rect.width {
            let top_idx = (self.rect.y * WINDOW_WIDTH as usize + x) * 4;
            let bottom_idx = ((self.rect.y + self.rect.height - 1) * WINDOW_WIDTH as usize + x) * 4;
            
            if top_idx + 3 < frame.len() {
                frame[top_idx..top_idx+4].copy_from_slice(&color);
            }
            if bottom_idx + 3 < frame.len() {
                frame[bottom_idx..bottom_idx+4].copy_from_slice(&color);
            }
        }
        
        // Left and right borders
        for y in self.rect.y..self.rect.y + self.rect.height {
            let left_idx = (y * WINDOW_WIDTH as usize + self.rect.x) * 4;
            let right_idx = (y * WINDOW_WIDTH as usize + self.rect.x + self.rect.width - 1) * 4;
            
            if left_idx + 3 < frame.len() {
                frame[left_idx..left_idx+4].copy_from_slice(&color);
            }
            if right_idx + 3 < frame.len() {
                frame[right_idx..right_idx+4].copy_from_slice(&color);
            }
        }
    }
}

pub struct StatusBar {
    pub rect: Rect,
    pub items: Vec<(String, String)>,
}

impl StatusBar {
    pub fn draw(&self, frame: &mut [u8], text_renderer: &mut TextRenderer) {
        // Draw background
        for y in self.rect.y..self.rect.y + self.rect.height {
            for x in self.rect.x..self.rect.x + self.rect.width {
                let idx = (y * WINDOW_WIDTH as usize + x) * 4;
                if idx + 3 < frame.len() {
                    frame[idx..idx+4].copy_from_slice(&[50, 50, 55, 255]);
                }
            }
        }
        
        // Draw items with better spacing
        let mut x_offset = self.rect.x + 10;
        for (key, value) in &self.items {
            // Draw key
            text_renderer.draw_text(
                frame,
                x_offset,
                self.rect.y + 6,  // Center vertically
                key,
                12.0,
                [180, 180, 180, 255],
            );
            
            x_offset += 40;  // Adjust for key width
            
            // Draw value
            text_renderer.draw_text(
                frame,
                x_offset,
                self.rect.y + 6,  // Center vertically
                value,
                12.0,
                C_UI_TEXT,
            );
            
            x_offset += 100;  // Increased space between items
        }
    }
}