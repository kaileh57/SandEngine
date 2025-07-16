use pixels::{Error, Pixels, SurfaceTexture};
use std::time::{Duration, Instant};
use winit::dpi::LogicalSize;
use winit::event::{Event, VirtualKeyCode, WindowEvent, ElementState, MouseButton};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::{Window, WindowBuilder};
use sand_engine::{Simulation, MaterialType};

const WIDTH: usize = 400;
const HEIGHT: usize = 300;
const TARGET_FPS: u64 = 60;
const FRAME_DURATION: Duration = Duration::from_millis(1000 / TARGET_FPS);

struct App {
    simulation: Simulation,
    current_material: MaterialType,
    brush_size: usize,
    mouse_pressed: bool,
    mouse_x: f32,
    mouse_y: f32,
}

impl App {
    fn new() -> Self {
        Self {
            simulation: Simulation::new(WIDTH, HEIGHT),
            current_material: MaterialType::Sand,
            brush_size: 3,
            mouse_pressed: false,
            mouse_x: 0.0,
            mouse_y: 0.0,
        }
    }

    fn update(&mut self, delta_time: f32) {
        // Handle painting
        if self.mouse_pressed {
            let x = (self.mouse_x as usize).min(WIDTH - 1);
            let y = (self.mouse_y as usize).min(HEIGHT - 1);
            self.paint_particles(x, y);
        }

        // Update simulation
        self.simulation.update(delta_time);
    }

    fn paint_particles(&mut self, center_x: usize, center_y: usize) {
        let start_x = center_x.saturating_sub(self.brush_size);
        let end_x = (center_x + self.brush_size).min(WIDTH - 1);
        let start_y = center_y.saturating_sub(self.brush_size);
        let end_y = (center_y + self.brush_size).min(HEIGHT - 1);
        let brush_size_sq = self.brush_size * self.brush_size;

        for x in start_x..=end_x {
            for y in start_y..=end_y {
                let dx = x as i32 - center_x as i32;
                let dy = y as i32 - center_y as i32;
                let dist_sq = (dx * dx + dy * dy) as usize;

                if dist_sq <= brush_size_sq {
                    self.simulation.add_particle(x, y, self.current_material, None);
                }
            }
        }
    }

    fn render(&self, frame: &mut [u8]) {
        // Clear frame to black
        for pixel in frame.chunks_exact_mut(4) {
            pixel[0] = 0;   // R
            pixel[1] = 0;   // G
            pixel[2] = 0;   // B
            pixel[3] = 255; // A
        }

        // Draw particles
        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                if let Some(particle_data) = self.simulation.get_particle_data(x, y) {
                    let (material, temp, _, _) = particle_data;
                    if material != MaterialType::Empty {
                        let color = get_material_color(material, temp);
                        let index = (y * WIDTH + x) * 4;
                        
                        if index + 3 < frame.len() {
                            frame[index] = color[0];     // R
                            frame[index + 1] = color[1]; // G
                            frame[index + 2] = color[2]; // B
                            frame[index + 3] = 255;      // A
                        }
                    }
                }
            }
        }
    }

    fn handle_key(&mut self, key: VirtualKeyCode) {
        match key {
            VirtualKeyCode::Key1 => self.current_material = MaterialType::Sand,
            VirtualKeyCode::Key2 => self.current_material = MaterialType::Water,
            VirtualKeyCode::Key3 => self.current_material = MaterialType::Stone,
            VirtualKeyCode::Key4 => self.current_material = MaterialType::Fire,
            VirtualKeyCode::Key5 => self.current_material = MaterialType::Oil,
            VirtualKeyCode::Key6 => self.current_material = MaterialType::Eraser,
            VirtualKeyCode::C => self.simulation.clear(),
            VirtualKeyCode::Equals | VirtualKeyCode::Plus => {
                self.brush_size = (self.brush_size + 1).min(10);
            }
            VirtualKeyCode::Minus => {
                self.brush_size = self.brush_size.saturating_sub(1).max(1);
            }
            _ => {}
        }
    }
}

fn get_material_color(material: MaterialType, temp: f32) -> [u8; 3] {
    match material {
        MaterialType::Sand => [194, 178, 128],
        MaterialType::Water => [64, 164, 223],
        MaterialType::Stone => [128, 128, 128],
        MaterialType::Fire => {
            // Animate fire color based on temperature
            let intensity = (temp / 1000.0).clamp(0.0, 1.0);
            [255, (100.0 + intensity * 155.0) as u8, (intensity * 50.0) as u8]
        }
        MaterialType::Oil => [101, 67, 33],
        MaterialType::Lava => [255, 69, 0],
        MaterialType::Steam => [200, 200, 255],
        MaterialType::Smoke => [64, 64, 64],
        MaterialType::Ice => [173, 216, 230],
        MaterialType::Wood => [139, 69, 19],
        MaterialType::Plant => [34, 139, 34],
        MaterialType::Glass => [173, 216, 230],
        MaterialType::Acid => [0, 255, 0],
        MaterialType::Coal => [36, 36, 36],
        MaterialType::Gunpowder => [64, 64, 64],
        MaterialType::ToxicGas => [128, 255, 0],
        MaterialType::Slime => [0, 255, 127],
        MaterialType::Gasoline => [255, 20, 147],
        MaterialType::Fuse => [139, 69, 19],
        MaterialType::Ash => [128, 128, 128],
        MaterialType::Gold => [255, 215, 0],
        MaterialType::Iron => [139, 139, 139],
        MaterialType::Generator => [255, 255, 0],
        MaterialType::Eraser => [0, 0, 0],
        MaterialType::Empty => [0, 0, 0],
    }
}

fn main() -> Result<(), Error> {
    let event_loop = EventLoop::new();
    let window = {
        let size = LogicalSize::new(WIDTH as f64, HEIGHT as f64);
        WindowBuilder::new()
            .with_title("Sand Engine - Native")
            .with_inner_size(size)
            .with_min_inner_size(size)
            .build(&event_loop)
            .unwrap()
    };

    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(WIDTH as u32, HEIGHT as u32, surface_texture)?
    };

    let mut app = App::new();
    let mut last_update = Instant::now();

    println!("Sand Engine - Native");
    println!("Controls:");
    println!("1-6: Select material (Sand, Water, Stone, Fire, Oil, Eraser)");
    println!("C: Clear simulation");
    println!("+/-: Adjust brush size");
    println!("Mouse: Paint particles");

    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => {
                    *control_flow = ControlFlow::Exit;
                }
                WindowEvent::KeyboardInput { input, .. } => {
                    if input.state == ElementState::Pressed {
                        if let Some(key) = input.virtual_keycode {
                            app.handle_key(key);
                        }
                    }
                }
                WindowEvent::MouseInput { state, button, .. } => {
                    if button == MouseButton::Left {
                        app.mouse_pressed = state == ElementState::Pressed;
                    }
                }
                WindowEvent::CursorMoved { position, .. } => {
                    app.mouse_x = position.x as f32;
                    app.mouse_y = position.y as f32;
                }
                _ => {}
            },
            Event::MainEventsCleared => {
                let now = Instant::now();
                let delta_time = now.duration_since(last_update).as_secs_f32();
                
                if delta_time >= FRAME_DURATION.as_secs_f32() {
                    app.update(delta_time);
                    app.render(pixels.frame_mut());
                    
                    if let Err(err) = pixels.render() {
                        eprintln!("pixels.render() failed: {err}");
                        *control_flow = ControlFlow::Exit;
                        return;
                    }
                    
                    last_update = now;
                }
                
                window.request_redraw();
            }
            _ => {}
        }
    });
}