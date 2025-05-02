// File: constants.rs

// Grid and window dimensions
pub const GRID_WIDTH: usize = 200;
pub const GRID_HEIGHT: usize = 150;
pub const CELL_SIZE: usize = 4;
pub const WIDTH: u32 = (GRID_WIDTH * CELL_SIZE) as u32;
pub const HEIGHT: u32 = (GRID_HEIGHT * CELL_SIZE) as u32;
pub const UI_WIDTH: u32 = 200; // Width of the UI panel
pub const WINDOW_WIDTH: u32 = WIDTH + UI_WIDTH;

// Colors
pub const C_EMPTY: [u8; 4] = [0, 0, 0, 255];
pub const C_SAND: [u8; 4] = [194, 178, 128, 255];
pub const C_WATER: [u8; 4] = [50, 100, 200, 255];
pub const C_STONE: [u8; 4] = [100, 100, 100, 255];
pub const C_PLANT: [u8; 4] = [50, 150, 50, 255];
pub const C_FIRE: [u8; 4] = [255, 69, 0, 255];
pub const C_LAVA: [u8; 4] = [200, 50, 0, 255];
pub const C_BORDER: [u8; 4] = [100, 100, 100, 255];
pub const C_ERASER: [u8; 4] = [255, 0, 255, 255];
pub const C_UI_BG: [u8; 4] = [40, 40, 40, 255];
pub const C_UI_TEXT: [u8; 4] = [240, 240, 240, 255];
pub const C_UI_HIGHLIGHT: [u8; 4] = [100, 100, 160, 255];
pub const C_UI_BUTTON: [u8; 4] = [80, 80, 90, 255];
pub const C_UI_BUTTON_SELECTED: [u8; 4] = [100, 100, 120, 255];
pub const C_UI_BUTTON_BORDER: [u8; 4] = [160, 160, 180, 255];
pub const C_UI_CLEAR_BUTTON: [u8; 4] = [180, 60, 60, 255];
pub const C_UI_CLEAR_BUTTON_BORDER: [u8; 4] = [220, 100, 100, 255];

// Temperature constants
pub const AMBIENT_TEMP: f32 = 20.0;
pub const FIRE_START_TEMP: f32 = 1000.0;
pub const LAVA_START_TEMP: f32 = 800.0;
pub const MAX_TEMP: f32 = 3000.0;
pub const COOLING_RATE: f32 = 0.005;

// Helper functions for UI
pub fn font_8x8() -> [(u8, u64); 128] {
    [
        // Space (1)
        (32, 0x0000000000000000),
        
        // Numbers 0-9 (10)
        (48, 0x1c22222a32222200), // 0
        (49, 0x0818080808080c00), // 1
        (50, 0x3c0204081020221c), // 2
        (51, 0x1c22201820201e00), // 3
        (52, 0x20203e2224283000), // 4
        (53, 0x1e2020203c02223c), // 5
        (54, 0x1c22023e22221c00), // 6
        (55, 0x3e22040810202000), // 7
        (56, 0x1c22221c22221c00), // 8
        (57, 0x1c22221e02041800), // 9
        
        // Letters A-Z (26)
        (65, 0x1c22223e22222200), // A
        (66, 0x3c22223c22223c00), // B
        (67, 0x1c22202020221c00), // C
        (68, 0x3c22222222223c00), // D
        (69, 0x3e20203c20203e00), // E
        (70, 0x3e20203c20202000), // F
        (71, 0x1c22202e22221c00), // G
        (72, 0x2222223e22222200), // H
        (73, 0x3e08080808083e00), // I
        (74, 0x0202020222221c00), // J
        (75, 0x22242830242c2200), // K
        (76, 0x2020202020203e00), // L
        (77, 0x4163554941414100), // M
        (78, 0x22322a2a26222200), // N
        (79, 0x1c22222222221c00), // O
        (80, 0x3c22223c20202000), // P
        (81, 0x1c222222222a1d00), // Q
        (82, 0x3c22223c28242200), // R
        (83, 0x1c22201c02221c00), // S
        (84, 0x3e08080808080800), // T
        (85, 0x2222222222221c00), // U
        (86, 0x2222222214080000), // V
        (87, 0x4141414155634100), // W
        (88, 0x2222140814222200), // X
        (89, 0x2222140808080800), // Y
        (90, 0x3e02040810203e00), // Z
        
        // Lowercase a-z (26)
        (97, 0x001c023e22221e00),  // a
        (98, 0x2020203c22223c00),  // b
        (99, 0x001c2020201c0000),  // c
        (100, 0x0202023e22221e00), // d
        (101, 0x001c223e201c0000), // e
        (102, 0x0c10103c10101000), // f
        (103, 0x001e2222221e0238), // g
        (104, 0x2020203c22222200), // h
        (105, 0x0008001808083e00), // i
        (106, 0x0004000c04040438), // j
        (107, 0x2020242830282400), // k
        (108, 0x1818080808083e00), // l
        (109, 0x00665a5a42424200), // m
        (110, 0x003c2222222200),   // n
        (111, 0x001c2222221c0000), // o
        (112, 0x003c22223c202000), // p
        (113, 0x001e22221e020200), // q
        (114, 0x003a1c2020200000), // r
        (115, 0x001e201c021c0000), // s
        (116, 0x08083e0808081c00), // t
        (117, 0x002222222a1c0000), // u
        (118, 0x0022222214080000), // v
        (119, 0x0022225a5a240000), // w
        (120, 0x0022140814220000), // x
        (121, 0x0022221e02041800), // y
        (122, 0x003e0408103e0000), // z
        
        // Special characters (9)
        (40, 0x0408101010080400),  // (
        (41, 0x2010080808102000),  // )
        (43, 0x0008083e08080000),  // +
        (45, 0x0000003e00000000),  // -
        (46, 0x0000000000030300),  // .
        (47, 0x0002040810204000),  // /
        (58, 0x0000180018000000),  // :
        (59, 0x0000180018100000),  // ;
        (176, 0x00081c3e1c080000), // ° (degree symbol)
        
        // Exactly 56 padding entries to reach 128 total elements
        (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0),
        (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0),
        (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0),
        (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0),
        (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0),
        (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0),
        (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0)
    ]
}

// Helper function to draw characters with our bitmap font
pub fn draw_char(frame: &mut [u8], x: usize, y: usize, ch: char, color: [u8; 4]) {
    let font = font_8x8();
    
    // Find the glyph for this character
    let glyph = font.iter()
        .find(|&(c, _)| *c as char == ch)
        .map(|&(_, g)| g)
        .unwrap_or(0);
    
    // Draw each pixel of the 8x8 glyph
    for row in 0..8 {
        for col in 0..8 {
            // Check if this bit is set in the glyph
            let bit = (glyph >> (8 * (7 - row) + (7 - col))) & 1;
            if bit == 1 {
                // Calculate pixel position in frame
                let px = x + col;
                let py = y + row;
                let idx = (py * WINDOW_WIDTH as usize + px) * 4;
                
                // Set pixel color if within bounds
                if idx + 3 < frame.len() {
                    frame[idx..idx+4].copy_from_slice(&color);
                }
            }
        }
    }
}

pub fn draw_text(frame: &mut [u8], x: usize, y: usize, text: &str, color: [u8; 4]) {
    for (i, ch) in text.chars().enumerate() {
        draw_char(frame, x + i * 8, y, ch, color);
    }
}

// --- Fast Random Number Generator ---
pub struct FastRand {
    seed: u32,
}

impl FastRand {
    pub fn new(seed: u32) -> Self {
        Self { seed: if seed == 0 { 1 } else { seed } } // Ensure seed is never 0
    }

    pub fn rand(&mut self) -> u32 {
        // Xorshift algorithm
        let mut x = self.seed;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.seed = x;
        x
    }

    pub fn rand_range(&mut self, min: u32, max: u32) -> u32 {
        if min >= max {
            return min;
        }
        min + (self.rand() % (max - min + 1))
    }

    pub fn rand_bool(&mut self, probability: f32) -> bool {
        // Scale probability to u32 range for comparison
        let threshold = (probability.max(0.0).min(1.0) * (u32::MAX as f32)) as u32;
        self.rand() < threshold
    }
}