use env_logger::Env;
use log::info; // Ensure info is used
use pixels::{Pixels, SurfaceTexture};
use sand_simulation_engine::simulation_engine::SimulationEngine;
use sand_simulation_engine::material::MaterialType; 
use winit::dpi::LogicalSize;
use winit::event::{Event, WindowEvent, KeyboardInput, ElementState, MouseButton, MouseScrollDelta, VirtualKeyCode};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;

const SCREEN_WIDTH: u32 = 800; 
const SCREEN_HEIGHT: u32 = 600; 
const SIM_WIDTH: usize = 200; 
const SIM_HEIGHT: usize = 150;
const LAVA_BRUSH_TEMP_GAME: f32 = 2500.0; 

// Helper function to get material name
fn get_material_name(material_type: MaterialType) -> &'static str {
    match material_type {
        MaterialType::EMPTY => "Empty",
        MaterialType::SAND => "Sand",
        MaterialType::WATER => "Water",
        MaterialType::STONE => "Stone",
        MaterialType::PLANT => "Plant",
        MaterialType::FIRE => "Fire",
        MaterialType::LAVA => "Lava",
        MaterialType::GLASS => "Glass",
        MaterialType::STEAM => "Steam",
        MaterialType::OIL => "Oil",
        MaterialType::ACID => "Acid",
        MaterialType::COAL => "Coal",
        MaterialType::GUNPOWDER => "Gunpowder",
        MaterialType::ICE => "Ice",
        MaterialType::WOOD => "Wood",
        MaterialType::SMOKE => "Smoke",
        MaterialType::TOXIC_GAS => "Toxic Gas",
        MaterialType::SLIME => "Slime",
        MaterialType::GASOLINE => "Gasoline",
        MaterialType::GENERATOR => "Generator",
        MaterialType::FUSE => "Fuse",
        MaterialType::ASH => "Ash",
        MaterialType::ERASER => "Eraser",
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    info!("Starting Sand Game");

    let event_loop = EventLoop::new();
    let window = {
        let size = LogicalSize::new(SCREEN_WIDTH as f64, SCREEN_HEIGHT as f64);
        WindowBuilder::new()
            .with_title("Sand Simulation Game") // Initial title
            .with_inner_size(size)
            .with_min_inner_size(size)
            .build(&event_loop)?
    };

    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(SIM_WIDTH as u32, SIM_HEIGHT as u32, surface_texture)?
    };

    let mut game_engine = SimulationEngine::new(SIM_WIDTH, SIM_HEIGHT);
    let mut last_update = std::time::Instant::now();

    let mut current_material = MaterialType::SAND;
    let mut brush_size: i32 = 3;
    let mut is_mouse_down = false;
    let mut logical_mouse_pos: (i32, i32) = (0,0); 


    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::WindowEvent { window_id, event } if window_id == window.id() => {
                match event {
                    WindowEvent::CloseRequested => {
                        *control_flow = ControlFlow::Exit;
                    }
                    WindowEvent::Resized(new_size) => {
                        if pixels.resize_surface(new_size.width, new_size.height).is_err() {
                            log::error!("pixels.resize_surface failed");
                            *control_flow = ControlFlow::Exit;
                            return;
                        }
                    }
                    WindowEvent::KeyboardInput { input: KeyboardInput { state, virtual_keycode, .. }, .. } => {
                        if state == ElementState::Pressed {
                            if let Some(keycode) = virtual_keycode {
                                match keycode {
                                    VirtualKeyCode::Key1 => { current_material = MaterialType::SAND; info!("Brush: SAND"); }
                                    VirtualKeyCode::Key2 => { current_material = MaterialType::WATER; info!("Brush: WATER"); }
                                    VirtualKeyCode::Key3 => { current_material = MaterialType::STONE; info!("Brush: STONE"); }
                                    VirtualKeyCode::Key4 => { current_material = MaterialType::ACID; info!("Brush: ACID"); }
                                    VirtualKeyCode::Key5 => { current_material = MaterialType::GENERATOR; info!("Brush: GENERATOR"); }
                                    VirtualKeyCode::Key0 => { current_material = MaterialType::ERASER; info!("Brush: ERASER"); }
                                    VirtualKeyCode::C => { game_engine.clear_grid(); info!("Grid cleared"); }
                                    VirtualKeyCode::Up | VirtualKeyCode::Equals | VirtualKeyCode::NumpadAdd => {
                                        brush_size = (brush_size + 1).min(20); info!("Brush size: {}", brush_size);
                                    }
                                    VirtualKeyCode::Down | VirtualKeyCode::Minus | VirtualKeyCode::NumpadSubtract => {
                                        brush_size = (brush_size - 1).max(0); info!("Brush size: {}", brush_size);
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                    WindowEvent::CursorMoved { position, .. } => {
                        if let Some(pos_in_buffer) = pixels.window_pos_to_pixel(position.into()) {
                            logical_mouse_pos = (pos_in_buffer.0 as i32, pos_in_buffer.1 as i32);
                            if is_mouse_down {
                                if logical_mouse_pos.0 >= 0 && logical_mouse_pos.0 < SIM_WIDTH as i32 &&
                                   logical_mouse_pos.1 >= 0 && logical_mouse_pos.1 < SIM_HEIGHT as i32 {
                                    let temp_override = if current_material == MaterialType::LAVA { Some(LAVA_BRUSH_TEMP_GAME) } else { None };
                                    game_engine.place_particle_circle(logical_mouse_pos.0, logical_mouse_pos.1, brush_size, current_material, temp_override);
                                }
                            }
                            // Log particle info under cursor
                            if logical_mouse_pos.0 >= 0 && logical_mouse_pos.0 < SIM_WIDTH as i32 &&
                               logical_mouse_pos.1 >= 0 && logical_mouse_pos.1 < SIM_HEIGHT as i32 {
                                if let Some(info_str) = game_engine.get_particle_info_for_display(logical_mouse_pos.0, logical_mouse_pos.1) {
                                    info!("{}", info_str);
                                }
                            } else {
                                info!("Cursor outside simulation area");
                            }

                        } else {
                            logical_mouse_pos = (-1, -1); 
                            info!("Cursor outside simulation area");
                        }
                    }
                    WindowEvent::MouseInput { state, button, .. } => {
                        if button == MouseButton::Left {
                            match state {
                                ElementState::Pressed => {
                                    is_mouse_down = true;
                                    if logical_mouse_pos.0 >= 0 && logical_mouse_pos.0 < SIM_WIDTH as i32 &&
                                       logical_mouse_pos.1 >= 0 && logical_mouse_pos.1 < SIM_HEIGHT as i32 {
                                        let temp_override = if current_material == MaterialType::LAVA { Some(LAVA_BRUSH_TEMP_GAME) } else { None };
                                        game_engine.place_particle_circle(logical_mouse_pos.0, logical_mouse_pos.1, brush_size, current_material, temp_override);
                                    }
                                }
                                ElementState::Released => {
                                    is_mouse_down = false;
                                }
                            }
                        }
                    }
                    WindowEvent::MouseWheel { delta, .. } => {
                        match delta {
                            MouseScrollDelta::LineDelta(_, y_scroll) => {
                                if y_scroll > 0.0 {
                                    brush_size = (brush_size + 1).min(20);
                                } else if y_scroll < 0.0 {
                                    brush_size = (brush_size - 1).max(0);
                                }
                                info!("Brush size: {}", brush_size);
                            }
                            MouseScrollDelta::PixelDelta(_) => {}
                        }
                    }
                    _ => {}
                }
            }
            Event::MainEventsCleared => { 
                let now = std::time::Instant::now();
                let delta_time = now.duration_since(last_update).as_secs_f32();
                last_update = now;

                // Update window title
                let material_name_str = get_material_name(current_material);
                let title = format!(
                    "Sand Game | Brush: {} (Size: {}) | Cell: ({}, {})",
                    material_name_str,
                    brush_size,
                    logical_mouse_pos.0, 
                    logical_mouse_pos.1
                );
                window.set_title(&title);

                game_engine.update(delta_time);
                window.request_redraw(); 
            }
            Event::RedrawRequested(_) => {
                let frame = pixels.get_frame_mut();
                for (i, pixel_rgba_chunk) in frame.chunks_exact_mut(4).enumerate() {
                    let gx = i % SIM_WIDTH; 
                    let gy = i / SIM_WIDTH; 
                    let mut display_color = [0x00, 0x00, 0x00, 0xff]; 
                    if let Some(particle) = game_engine.grid.get_particle_mut(gx as i32, gy as i32) {
                        if particle.material_type != MaterialType::EMPTY { 
                            let color_tuple = particle.get_color(); 
                            display_color = [color_tuple.0, color_tuple.1, color_tuple.2, 0xff];
                        }
                    }
                    pixel_rgba_chunk.copy_from_slice(&display_color);
                }

                if pixels.render().is_err() {
                    log::error!("pixels.render failed");
                    *control_flow = ControlFlow::Exit;
                }
            }
            _ => {}
        }
    });
}
