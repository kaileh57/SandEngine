use sand_engine::{Simulation, MaterialType};
use std::time::{Duration, Instant};
use std::thread;
use tracing::{info, error};

const SIMULATION_WIDTH: usize = 200;
const SIMULATION_HEIGHT: usize = 150;
const TARGET_FPS: u64 = 60;
const FRAME_DURATION: Duration = Duration::from_millis(1000 / TARGET_FPS);

struct PhysicsServer {
    simulation: Simulation,
    running: bool,
    frame_count: u64,
    last_stats_time: Instant,
}

impl PhysicsServer {
    fn new(width: usize, height: usize) -> Self {
        Self {
            simulation: Simulation::new(width, height),
            running: false,
            frame_count: 0,
            last_stats_time: Instant::now(),
        }
    }

    fn start(&mut self) {
        info!("Starting physics server with {}x{} grid", SIMULATION_WIDTH, SIMULATION_HEIGHT);
        self.running = true;
        self.last_stats_time = Instant::now();
        
        let mut last_time = Instant::now();

        while self.running {
            let frame_start = Instant::now();
            
            // Calculate delta time
            let now = Instant::now();
            let delta_time = now.duration_since(last_time).as_secs_f32();
            last_time = now;
            
            // Clamp delta time to avoid large jumps
            let delta_time = delta_time.min(0.1);
            
            // Update simulation
            self.simulation.update(delta_time);
            self.frame_count += 1;
            
            // Print stats every second
            if now.duration_since(self.last_stats_time) >= Duration::from_secs(1) {
                let particle_count = self.count_particles();
                info!("FPS: {}, Active particles: {}", self.frame_count, particle_count);
                self.frame_count = 0;
                self.last_stats_time = now;
            }
            
            // Sleep to maintain target framerate
            let frame_time = frame_start.elapsed();
            if frame_time < FRAME_DURATION {
                thread::sleep(FRAME_DURATION - frame_time);
            }
        }
    }

    fn stop(&mut self) {
        self.running = false;
        info!("Physics server stopped");
    }

    fn add_material(&mut self, x: usize, y: usize, material: MaterialType, brush_size: usize) -> bool {
        let start_x = x.saturating_sub(brush_size);
        let end_x = (x + brush_size).min(self.simulation.width.saturating_sub(1));
        let start_y = y.saturating_sub(brush_size);
        let end_y = (y + brush_size).min(self.simulation.height.saturating_sub(1));
        let brush_size_sq = brush_size * brush_size;
        
        let mut placed = false;
        for px in start_x..=end_x {
            for py in start_y..=end_y {
                let dx = px as i32 - x as i32;
                let dy = py as i32 - y as i32;
                let dist_sq = (dx * dx + dy * dy) as usize;
                
                if dist_sq <= brush_size_sq {
                    if self.simulation.add_particle(px, py, material, None) {
                        placed = true;
                    }
                }
            }
        }
        placed
    }

    fn clear(&mut self) {
        self.simulation.clear();
        info!("Simulation cleared");
    }

    fn count_particles(&self) -> usize {
        let mut count = 0;
        for y in 0..self.simulation.height {
            for x in 0..self.simulation.width {
                if self.simulation.get_particle(x, y).is_some() {
                    count += 1;
                }
            }
        }
        count
    }

    fn get_particle_info(&self, x: usize, y: usize) -> Option<(MaterialType, f32, Option<f32>, bool)> {
        self.simulation.get_particle_data(x, y)
    }

    // API methods for external integration
    pub fn get_simulation_state(&self) -> Vec<Vec<Option<(MaterialType, f32)>>> {
        let mut state = Vec::with_capacity(self.simulation.height);
        for y in 0..self.simulation.height {
            let mut row = Vec::with_capacity(self.simulation.width);
            for x in 0..self.simulation.width {
                if let Some(particle) = self.simulation.get_particle(x, y) {
                    row.push(Some((particle.material_type, particle.temp)));
                } else {
                    row.push(None);
                }
            }
            state.push(row);
        }
        state
    }
}

fn main() {
    // Initialize tracing
    tracing_subscriber::fmt::init();
    
    let mut physics_server = PhysicsServer::new(SIMULATION_WIDTH, SIMULATION_HEIGHT);
    
    // Spawn a demo thread that adds some initial particles
    let handle = {
        let mut server_ref = PhysicsServer::new(SIMULATION_WIDTH, SIMULATION_HEIGHT);
        
        thread::spawn(move || {
            thread::sleep(Duration::from_secs(1));
            
            // Add some demo materials
            info!("Adding demo materials...");
            
            // Add some sand at the top
            for x in 80..120 {
                server_ref.add_material(x, 10, MaterialType::Sand, 1);
            }
            
            // Add water source
            for x in 90..110 {
                for y in 20..25 {
                    server_ref.add_material(x, y, MaterialType::Water, 0);
                }
            }
            
            // Add fire source
            server_ref.add_material(100, 40, MaterialType::Fire, 2);
            
            // Add some wood to burn
            for x in 95..105 {
                for y in 45..50 {
                    server_ref.add_material(x, y, MaterialType::Wood, 0);
                }
            }
            
            // Add lava at bottom
            for x in 70..130 {
                server_ref.add_material(x, 140, MaterialType::Lava, 1);
            }
            
            info!("Demo materials added. Simulation running...");
        })
    };
    
    // For a real integration, you'd expose these methods via an API:
    // - physics_server.add_material(x, y, material, brush_size)
    // - physics_server.get_simulation_state()
    // - physics_server.get_particle_info(x, y)
    // - physics_server.clear()
    
    info!("Physics server starting. Press Ctrl+C to stop.");
    
    // Start the main simulation loop
    physics_server.start();
    
    handle.join().unwrap_or_else(|e| {
        error!("Demo thread panicked: {:?}", e);
    });
}

// Example integration functions that could be exposed via FFI or other means:

#[no_mangle]
pub extern "C" fn create_physics_server(width: usize, height: usize) -> *mut PhysicsServer {
    let server = Box::new(PhysicsServer::new(width, height));
    Box::into_raw(server)
}

#[no_mangle]
pub extern "C" fn destroy_physics_server(server: *mut PhysicsServer) {
    if !server.is_null() {
        unsafe {
            let _ = Box::from_raw(server);
        }
    }
}

#[no_mangle]
pub extern "C" fn update_physics(server: *mut PhysicsServer, delta_time: f32) {
    if !server.is_null() {
        unsafe {
            (*server).simulation.update(delta_time);
        }
    }
}

#[no_mangle]
pub extern "C" fn add_particle_to_sim(
    server: *mut PhysicsServer, 
    x: usize, 
    y: usize, 
    material: u32, 
    brush_size: usize
) -> bool {
    if !server.is_null() {
        unsafe {
            // Convert u32 to MaterialType safely
            let mat_type = match material {
                0 => MaterialType::Empty,
                1 => MaterialType::Sand,
                2 => MaterialType::Water,
                3 => MaterialType::Stone,
                4 => MaterialType::Plant,
                5 => MaterialType::Fire,
                6 => MaterialType::Lava,
                7 => MaterialType::Glass,
                8 => MaterialType::Steam,
                9 => MaterialType::Oil,
                10 => MaterialType::Acid,
                11 => MaterialType::Coal,
                12 => MaterialType::Gunpowder,
                13 => MaterialType::Ice,
                14 => MaterialType::Wood,
                15 => MaterialType::Smoke,
                16 => MaterialType::ToxicGas,
                17 => MaterialType::Slime,
                18 => MaterialType::Gasoline,
                19 => MaterialType::Generator,
                20 => MaterialType::Fuse,
                21 => MaterialType::Ash,
                99 => MaterialType::Eraser,
                _ => return false, // Invalid material type
            };
            return (*server).add_material(x, y, mat_type, brush_size);
        }
    }
    false
}

#[no_mangle]
pub extern "C" fn clear_simulation(server: *mut PhysicsServer) {
    if !server.is_null() {
        unsafe {
            (*server).clear();
        }
    }
}