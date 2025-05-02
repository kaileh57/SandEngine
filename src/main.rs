use pixels::{Error, Pixels, SurfaceTexture};
use rand::prelude::*;
use winit::dpi::{LogicalSize, PhysicalPosition};
use winit::event::{Event, VirtualKeyCode, ElementState, MouseButton};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use winit::event::WindowEvent;

const GRID_WIDTH: usize = 200;
const GRID_HEIGHT: usize = 150;
const CELL_SIZE: usize = 4;
const WIDTH: u32 = (GRID_WIDTH * CELL_SIZE) as u32;
const HEIGHT: u32 = (GRID_HEIGHT * CELL_SIZE) as u32;

// Particle types
const EMPTY: u8 = 0;
const SAND: u8 = 1;

// Colors
const C_EMPTY: [u8; 4] = [0, 0, 0, 255];
const C_SAND: [u8; 4] = [194, 178, 128, 255];
const C_BORDER: [u8; 4] = [100, 100, 100, 255];

struct SandSimulation {
    // Use a flat vector for better cache locality
    grid: Vec<u8>,
    updated: Vec<bool>,
    brush_size: usize,
}

impl SandSimulation {
    fn new() -> Self {
        Self {
            grid: vec![EMPTY; GRID_WIDTH * GRID_HEIGHT],
            updated: vec![false; GRID_WIDTH * GRID_HEIGHT],
            brush_size: 3,
        }
    }

    #[inline]
    fn get_index(x: usize, y: usize) -> usize {
        y * GRID_WIDTH + x
    }

    #[inline]
    fn get(&self, x: usize, y: usize) -> u8 {
        if x < GRID_WIDTH && y < GRID_HEIGHT {
            self.grid[Self::get_index(x, y)]
        } else {
            0 // Out of bounds, return EMPTY
        }
    }

    #[inline]
    fn set(&mut self, x: usize, y: usize, value: u8) {
        if x < GRID_WIDTH && y < GRID_HEIGHT {
            let idx = Self::get_index(x, y);
            self.grid[idx] = value;
        }
    }

    #[inline]
    fn is_updated(&self, x: usize, y: usize) -> bool {
        if x < GRID_WIDTH && y < GRID_HEIGHT {
            self.updated[Self::get_index(x, y)]
        } else {
            true // Out of bounds is treated as already updated
        }
    }

    #[inline]
    fn set_updated(&mut self, x: usize, y: usize, value: bool) {
        if x < GRID_WIDTH && y < GRID_HEIGHT {
            let idx = Self::get_index(x, y);
            self.updated[idx] = value;
        }
    }

    fn clear(&mut self) {
        for i in 0..self.grid.len() {
            self.grid[i] = EMPTY;
        }
    }

    fn update(&mut self) {
        // Reset update flags
        for i in 0..self.updated.len() {
            self.updated[i] = false;
        }

        // Process from bottom to top, shuffling column order for natural flow
        let mut columns: Vec<usize> = (0..GRID_WIDTH).collect();
        columns.shuffle(&mut rand::thread_rng());

        for y in (0..GRID_HEIGHT).rev() {
            for &x in &columns {
                if self.get(x, y) == SAND && !self.is_updated(x, y) {
                    self.update_sand(x, y);
                }
            }
        }
    }

    fn update_sand(&mut self, x: usize, y: usize) {
        // Mark as updated
        self.set_updated(x, y, true);

        // Try to move down
        if y < GRID_HEIGHT - 1 && self.get(x, y + 1) == EMPTY {
            self.set(x, y + 1, SAND);
            self.set(x, y, EMPTY);
            self.set_updated(x, y + 1, true);
            return;
        }

        // Try to move diagonally
        if y < GRID_HEIGHT - 1 {
            let left_first = rand::thread_rng().gen_bool(0.5);
            
            if left_first {
                // Try left diagonal first
                if x > 0 && self.get(x - 1, y + 1) == EMPTY {
                    self.set(x - 1, y + 1, SAND);
                    self.set(x, y, EMPTY);
                    self.set_updated(x - 1, y + 1, true);
                    return;
                }
                // Then right diagonal
                if x < GRID_WIDTH - 1 && self.get(x + 1, y + 1) == EMPTY {
                    self.set(x + 1, y + 1, SAND);
                    self.set(x, y, EMPTY);
                    self.set_updated(x + 1, y + 1, true);
                    return;
                }
            } else {
                // Try right diagonal first
                if x < GRID_WIDTH - 1 && self.get(x + 1, y + 1) == EMPTY {
                    self.set(x + 1, y + 1, SAND);
                    self.set(x, y, EMPTY);
                    self.set_updated(x + 1, y + 1, true);
                    return;
                }
                // Then left diagonal
                if x > 0 && self.get(x - 1, y + 1) == EMPTY {
                    self.set(x - 1, y + 1, SAND);
                    self.set(x, y, EMPTY);
                    self.set_updated(x - 1, y + 1, true);
                    return;
                }
            }
        }
    }

    fn draw(&self, frame: &mut [u8]) {
        for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {
            let screen_x = i % WIDTH as usize;
            let screen_y = i / WIDTH as usize;
            let x = screen_x / CELL_SIZE;
            let y = screen_y / CELL_SIZE;
            
            // Draw border (1 pixel width)
            if screen_x < CELL_SIZE || screen_x >= WIDTH as usize - CELL_SIZE || 
               screen_y < CELL_SIZE || screen_y >= HEIGHT as usize - CELL_SIZE {
                pixel.copy_from_slice(&C_BORDER);
                continue;
            }
            
            let color = match self.get(x, y) {
                SAND => C_SAND,
                _ => C_EMPTY,
            };
            
            pixel.copy_from_slice(&color);
        }
    }

    fn add_sand(&mut self, x: usize, y: usize, brush_size: usize) {
        let start_x = x.saturating_sub(brush_size);
        let end_x = (x + brush_size).min(GRID_WIDTH - 1);
        let start_y = y.saturating_sub(brush_size);
        let end_y = (y + brush_size).min(GRID_HEIGHT - 1);
        
        let brush_size_squared = (brush_size * brush_size) as isize;
        
        for cy in start_y..=end_y {
            for cx in start_x..=end_x {
                let dx = cx as isize - x as isize;
                let dy = cy as isize - y as isize;
                if dx * dx + dy * dy <= brush_size_squared {
                    self.set(cx, cy, SAND);
                }
            }
        }
    }
}

fn main() -> Result<(), Error> {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Sand Simulation")
        .with_inner_size(LogicalSize::new(WIDTH, HEIGHT))
        .with_resizable(true)
        .build(&event_loop)
        .unwrap();

    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(WIDTH, HEIGHT, surface_texture)?
    };

    let mut simulation = SandSimulation::new();
    let mut is_drawing = false;
    let mut last_cursor_pos = (0, 0);
    
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => {
                    *control_flow = ControlFlow::Exit;
                },
                WindowEvent::Resized(new_size) => {
                    if let Err(err) = pixels.resize_surface(new_size.width, new_size.height) {
                        eprintln!("pixels.resize_surface error: {err}");
                        *control_flow = ControlFlow::Exit;
                    }
                },
                WindowEvent::KeyboardInput { input, .. } => {
                    if let Some(key_code) = input.virtual_keycode {
                        match key_code {
                            VirtualKeyCode::Escape => {
                                *control_flow = ControlFlow::Exit;
                            },
                            VirtualKeyCode::C => {
                                simulation.clear();
                            },
                            _ => (),
                        }
                    }
                },
                WindowEvent::MouseInput { state, button, .. } => {
                    if button == MouseButton::Left {
                        is_drawing = state == ElementState::Pressed;
                        // When pressing, immediately add sand at the last known position
                        if is_drawing {
                            simulation.add_sand(last_cursor_pos.0, last_cursor_pos.1, simulation.brush_size);
                        }
                    }
                },
                WindowEvent::CursorMoved { position, .. } => {
                    // Convert PhysicalPosition<f64> to (f32, f32) for window_pos_to_pixel
                    let physical_pos = (position.x as f32, position.y as f32);
                    
                    // Use pixels helper function to map window physical pos to pixel buffer pos
                    match pixels.window_pos_to_pixel(physical_pos) {
                        Ok((pixel_x, pixel_y)) => {
                            // Map buffer pixel position to grid cell
                            let x = (pixel_x / CELL_SIZE).min(GRID_WIDTH - 1);
                            let y = (pixel_y / CELL_SIZE).min(GRID_HEIGHT - 1);

                            last_cursor_pos = (x, y);

                            if is_drawing {
                                simulation.add_sand(x, y, simulation.brush_size);
                            }
                        }
                        Err(_) => {
                            // Cursor might be outside the window's drawing surface,
                            // ignore this position.
                        }
                    }
                },
                WindowEvent::MouseWheel { delta, .. } => {
                    match delta {
                        winit::event::MouseScrollDelta::LineDelta(_, y) => {
                            if y > 0.0 {
                                simulation.brush_size = (simulation.brush_size + 1).min(20);
                            } else if y < 0.0 {
                                simulation.brush_size = simulation.brush_size.saturating_sub(1);
                            }
                        },
                        _ => (),
                    }
                },
                _ => (),
            },
            Event::RedrawRequested(_) => {
                // Add sand continuously while drawing, even if mouse isn't moving
                if is_drawing {
                    simulation.add_sand(last_cursor_pos.0, last_cursor_pos.1, simulation.brush_size);
                }
                
                simulation.update();
                simulation.draw(pixels.frame_mut());
                if let Err(err) = pixels.render() {
                    eprintln!("pixels.render error: {err}");
                    *control_flow = ControlFlow::Exit;
                }
            },
            Event::MainEventsCleared => {
                window.request_redraw();
            },
            _ => (),
        }
    });
}
