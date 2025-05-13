use winit::{
    event::{Event, VirtualKeyCode, KeyboardInput, ElementState, MouseButton, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use pixels::{Pixels, SurfaceTexture};
use std::time::{Duration, Instant};

mod material;
mod chunk;
mod world;
mod simulation;
mod optimization;

use world::World;
use material::MaterialInstance;

const WIDTH: u32 = 800;
const HEIGHT: u32 = 600;
const MAX_FPS: u64 = 60;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BrushType {
    Sand,
    Stone,
    Eraser,
}

fn main() {
    // Initialize window and event loop
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Falling Sand Engine")
        .with_inner_size(winit::dpi::LogicalSize::new(WIDTH, HEIGHT))
        .build(&event_loop)
        .unwrap();

    // Initialize pixels framebuffer
    let surface_texture = SurfaceTexture::new(WIDTH, HEIGHT, &window);
    let mut pixels = Pixels::new(WIDTH, HEIGHT, surface_texture).unwrap();

    // Create our world
    let mut world = World::new(WIDTH, HEIGHT);
    
    // Track mouse state
    let mut mouse_position = (0, 0);
    let mut mouse_pressed = false;
    let mut brush_type = BrushType::Sand;
    let mut brush_radius = 5;
    
    // Time tracking
    let mut last_update = Instant::now();
    let update_interval = Duration::from_micros(1_000_000 / MAX_FPS);

    // Main event loop
    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::WindowEvent { window_id, event } if window_id == window.id() => {
                match event {
                    WindowEvent::CloseRequested => {
                        *control_flow = ControlFlow::Exit;
                    },
                    
                    // Track mouse position
                    WindowEvent::CursorMoved { position, .. } => {
                        mouse_position = (position.x as i32, position.y as i32);
                    },
                    
                    // Track mouse buttons
                    WindowEvent::MouseInput { state, button: MouseButton::Left, .. } => {
                        mouse_pressed = state == ElementState::Pressed;
                    },
                    
                    // Handle keyboard input
                    WindowEvent::KeyboardInput { 
                        input: KeyboardInput { 
                            virtual_keycode: Some(key), 
                            state: ElementState::Pressed, 
                            .. 
                        }, 
                        .. 
                    } => {
                        match key {
                            VirtualKeyCode::Key1 => {
                                brush_type = BrushType::Sand;
                                println!("Selected: Sand");
                            },
                            VirtualKeyCode::Key2 => {
                                brush_type = BrushType::Stone;
                                println!("Selected: Stone");
                            },
                            VirtualKeyCode::Key0 => {
                                brush_type = BrushType::Eraser;
                                println!("Selected: Eraser");
                            },
                            VirtualKeyCode::Plus | VirtualKeyCode::Equals => {
                                brush_radius = (brush_radius + 1).min(20);
                                println!("Brush size: {}", brush_radius);
                            },
                            VirtualKeyCode::Minus => {
                                brush_radius = (brush_radius - 1).max(1);
                                println!("Brush size: {}", brush_radius);
                            },
                            VirtualKeyCode::Space => {
                                // Drop a big pile of sand
                                world.create_sand_circle(
                                    WIDTH as i32 / 2,
                                    HEIGHT as i32 / 3,
                                    30
                                );
                            },
                            _ => (),
                        }
                    },
                    
                    _ => (),
                }
            },
            
            Event::MainEventsCleared => {
                // Handle mouse drawing
                if mouse_pressed {
                    match brush_type {
                        BrushType::Sand => {
                            let sand_id = 1; // Assuming sand is at index 1
                            if let Some(sand) = world.materials.create_instance(sand_id) {
                                world.create_material_circle(
                                    mouse_position.0,
                                    mouse_position.1,
                                    brush_radius,
                                    sand
                                );
                            }
                        },
                        BrushType::Stone => {
                            let stone_id = 2; // Assuming stone is at index 2
                            if let Some(stone) = world.materials.create_instance(stone_id) {
                                world.create_material_circle(
                                    mouse_position.0,
                                    mouse_position.1,
                                    brush_radius,
                                    stone
                                );
                            }
                        },
                        BrushType::Eraser => {
                            let air = MaterialInstance::air();
                            world.create_material_circle(
                                mouse_position.0,
                                mouse_position.1,
                                brush_radius,
                                air
                            );
                        },
                    }
                }
                
                // Throttle updates to our target FPS
                let now = Instant::now();
                let elapsed = now.duration_since(last_update);
                
                if elapsed >= update_interval {
                    // Update simulation
                    world.update_optimized();
                    last_update = now;
                    
                    // Render
                    let frame = pixels.get_frame_mut();
                    world.render(frame);
                    if let Err(err) = pixels.render() {
                        eprintln!("Render error: {}", err);
                        *control_flow = ControlFlow::Exit;
                    }
                }
                
                // Throttle the event loop
                *control_flow = ControlFlow::WaitUntil(
                    last_update + update_interval
                );
            },
            
            _ => (),
        }
    });
}
