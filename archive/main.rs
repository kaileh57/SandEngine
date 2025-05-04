// File: main.rs
use pixels::{Error, Pixels, SurfaceTexture};
use winit::dpi::LogicalSize;
use winit::event::{Event, VirtualKeyCode, ElementState, MouseButton};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use winit::event::WindowEvent;

// Import from our library
mod engine;
mod game;
use engine::simulation::SandSimulation;
use engine::material::MaterialType;
use engine::constants::{GRID_WIDTH, GRID_HEIGHT, CELL_SIZE, WIDTH, HEIGHT};
use game::constants::{UI_WIDTH, WINDOW_WIDTH};
use game::ui::UI;
use game::material_renderer::MaterialRenderer;

fn main() -> Result<(), Error> {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Sand Simulation")
        .with_inner_size(LogicalSize::new(WINDOW_WIDTH, HEIGHT))
        .with_resizable(true)
        .build(&event_loop)
        .unwrap();

    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(WINDOW_WIDTH, HEIGHT, surface_texture)?
    };

    let mut simulation = SandSimulation::new();
    let mut ui = UI::new();
    let material_renderer = MaterialRenderer::new();
    let mut is_drawing = false;
    let mut last_cursor_pos = (0, 0);
    let mut last_screen_pos = (0.0, 0.0); // Track raw screen coordinates
    let simulation_speed = 3; // Simulation speed multiplier (2-3x faster)
    
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
                        if input.state == ElementState::Pressed {
                            match key_code {
                                VirtualKeyCode::Escape => {
                                    *control_flow = ControlFlow::Exit;
                                },
                                VirtualKeyCode::C => {
                                    simulation.clear();
                                },
                                VirtualKeyCode::Key1 => {
                                    simulation.current_material = MaterialType::Sand;
                                },
                                VirtualKeyCode::Key2 => {
                                    simulation.current_material = MaterialType::Water;
                                },
                                VirtualKeyCode::Key3 => {
                                    simulation.current_material = MaterialType::Stone;
                                },
                                VirtualKeyCode::Key4 => {
                                    simulation.current_material = MaterialType::Plant;
                                },
                                VirtualKeyCode::Key5 => {
                                    simulation.current_material = MaterialType::Fire;
                                },
                                VirtualKeyCode::Key6 => {
                                    simulation.current_material = MaterialType::Lava;
                                },
                                VirtualKeyCode::E => {
                                    simulation.current_material = MaterialType::Eraser;
                                },
                                _ => (),
                            }
                        }
                    }
                },
                WindowEvent::MouseInput { state, button, .. } => {
                    if button == MouseButton::Left {
                        is_drawing = state == ElementState::Pressed;
                        
                        // Use the last known screen position
                        let physical_pos = (last_screen_pos.0 as f32, last_screen_pos.1 as f32);
                        
                        match pixels.window_pos_to_pixel(physical_pos) {
                            Ok((pixel_x, pixel_y)) => {
                                // Check if click was in the UI area
                                if pixel_x >= WIDTH as usize {
                                    // Handle UI click
                                    if state == ElementState::Pressed {
                                        ui.handle_click(&mut simulation, pixel_x, pixel_y);
                                    }
                                    is_drawing = false; // Don't draw when clicking UI
                                } else {
                                    // When pressing in simulation area, add material at cursor
                                    if is_drawing {
                                        let x = (pixel_x / CELL_SIZE).min(GRID_WIDTH - 1);
                                        let y = (pixel_y / CELL_SIZE).min(GRID_HEIGHT - 1);
                                        simulation.add_material(x, y, simulation.brush_size, simulation.current_material);
                                    }
                                }
                            }
                            Err(_) => {}
                        }
                    }
                },
                WindowEvent::CursorMoved { position, .. } => {
                    // Store the raw screen position
                    last_screen_pos = (position.x, position.y);
                    
                    // Convert PhysicalPosition<f64> to (f32, f32) for window_pos_to_pixel
                    let physical_pos = (position.x as f32, position.y as f32);
                    
                    // Use pixels helper function to map window physical pos to pixel buffer pos
                    match pixels.window_pos_to_pixel(physical_pos) {
                        Ok((pixel_x, pixel_y)) => {
                            // Only handle mouse input in the simulation area
                            if pixel_x < WIDTH as usize {
                                // Map buffer pixel position to grid cell
                                let x = (pixel_x / CELL_SIZE).min(GRID_WIDTH - 1);
                                let y = (pixel_y / CELL_SIZE).min(GRID_HEIGHT - 1);

                                last_cursor_pos = (x, y);
                                simulation.cursor_pos = (x, y);

                                if is_drawing {
                                    simulation.add_material(x, y, simulation.brush_size, simulation.current_material);
                                }
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
                                ui.update_brush_size(&mut simulation, 1);
                            } else if y < 0.0 {
                                ui.update_brush_size(&mut simulation, -1);
                            }
                        },
                        _ => (),
                    }
                },
                _ => (),
            },
            Event::RedrawRequested(_) => {
                // Add material continuously while drawing, even if mouse isn't moving
                if is_drawing {
                    simulation.add_material(last_cursor_pos.0, last_cursor_pos.1, simulation.brush_size, simulation.current_material);
                }
                
                // Run simulation multiple times per frame for increased speed
                for _ in 0..simulation_speed {
                    simulation.update();
                }
                
                // Update the frame
                let frame = pixels.frame_mut();
                
                // First clear the frame
                for pixel in frame.chunks_exact_mut(4) {
                    pixel.copy_from_slice(&game::constants::C_EMPTY);
                }
                
                // Draw the simulation
                material_renderer.draw(&simulation, frame);
                
                // Draw the UI on top
                ui.draw(frame, &simulation);
                
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