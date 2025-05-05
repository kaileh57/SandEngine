// ui/text_renderer.rs - Font rendering for UI

use fontdue::{Font, FontSettings};
use fontdue::layout::{CoordinateSystem, Layout, LayoutSettings, TextStyle};
use crate::constants::WINDOW_WIDTH;

pub struct TextRenderer {
    font: Font,
    layout: Layout,
}

impl TextRenderer {
    pub fn new() -> Self {
        // Include the default font (Roboto Regular)
        let font_data = include_bytes!("../../assets/Roboto-Regular.ttf");
        let font = Font::from_bytes(font_data as &[u8], FontSettings::default()).unwrap();
        
        let mut layout = Layout::new(CoordinateSystem::PositiveYDown);
        layout.reset(&LayoutSettings {
            ..LayoutSettings::default()
        });
        
        Self {
            font,
            layout,
        }
    }
    
    pub fn draw_text(
        &mut self,
        frame: &mut [u8],
        x: usize,
        y: usize,
        text: &str,
        size: f32,
        color: [u8; 4],
    ) {
        // Layout the text
        self.layout.reset(&LayoutSettings {
            ..LayoutSettings::default()
        });
        
        self.layout.append(&[&self.font], &TextStyle::new(text, size, 0));
        
        // Render each glyph
        for glyph in self.layout.glyphs() {
            let (metrics, bitmap) = self.font.rasterize(glyph.parent, glyph.key.px);
            
            // Skip glyphs with zero width or height
            if metrics.width == 0 || metrics.height == 0 {
                continue;
            }
            
            for (bitmap_y, row) in bitmap.chunks(metrics.width).enumerate() {
                for (bitmap_x, &coverage) in row.iter().enumerate() {
                    if coverage > 0 {
                        let px = x + glyph.x as usize + bitmap_x;
                        let py = y + glyph.y as usize + bitmap_y;
                        
                        if px < WINDOW_WIDTH as usize && py * WINDOW_WIDTH as usize + px < frame.len() / 4 {
                            let idx = (py * WINDOW_WIDTH as usize + px) * 4;
                            
                            if idx + 3 < frame.len() {
                                let alpha = (coverage as f32 / 255.0) * (color[3] as f32 / 255.0);
                                
                                // Alpha blending
                                let existing_r = frame[idx] as f32 / 255.0;
                                let existing_g = frame[idx + 1] as f32 / 255.0;
                                let existing_b = frame[idx + 2] as f32 / 255.0;
                                
                                let new_r = color[0] as f32 / 255.0;
                                let new_g = color[1] as f32 / 255.0;
                                let new_b = color[2] as f32 / 255.0;
                                
                                frame[idx] = ((new_r * alpha + existing_r * (1.0 - alpha)) * 255.0) as u8;
                                frame[idx + 1] = ((new_g * alpha + existing_g * (1.0 - alpha)) * 255.0) as u8;
                                frame[idx + 2] = ((new_b * alpha + existing_b * (1.0 - alpha)) * 255.0) as u8;
                                frame[idx + 3] = 255;
                            }
                        }
                    }
                }
            }
        }
    }
    
    #[allow(dead_code)]
    pub fn measure_text(&mut self, text: &str, size: f32) -> (usize, usize) {
        self.layout.reset(&LayoutSettings {
            ..LayoutSettings::default()
        });
        
        self.layout.append(&[&self.font], &TextStyle::new(text, size, 0));
        
        let mut width = 0;
        let mut height = 0;
        
        for glyph in self.layout.glyphs() {
            let glyph_right = glyph.x as usize + glyph.width;
            let glyph_bottom = glyph.y as usize + glyph.height;
            
            width = width.max(glyph_right);
            height = height.max(glyph_bottom);
        }
        
        (width, height)
    }
}