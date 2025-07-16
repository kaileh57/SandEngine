use futures_util::{SinkExt, StreamExt};
use sand_engine::{Simulation, MaterialType, Particle};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::time;
use tracing::{info, warn, error};
use warp::Filter;

const SIMULATION_WIDTH: usize = 200;
const SIMULATION_HEIGHT: usize = 150;
const TARGET_FPS: u64 = 60;
const FRAME_DURATION: Duration = Duration::from_millis(1000 / TARGET_FPS);
const BROADCAST_FPS: u64 = 30; // Broadcast at 30 FPS for smoother updates
const BROADCAST_INTERVAL: u64 = TARGET_FPS / BROADCAST_FPS;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ClientMessage {
    #[serde(rename = "paint")]
    Paint {
        x: usize,
        y: usize,
        material: MaterialType,
        brush_size: usize,
    },
    #[serde(rename = "clear")]
    Clear,
    #[serde(rename = "get_particle")]
    GetParticle { x: usize, y: usize },
    #[serde(rename = "place_structure")]
    PlaceStructure { structure_name: String, x: usize, y: usize },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ServerMessage {
    #[serde(rename = "simulation_state")]
    SimulationState {
        width: usize,
        height: usize,
        particles: HashMap<String, ParticleData>,
    },
    #[serde(rename = "delta_update")]
    DeltaUpdate {
        added: HashMap<String, ParticleData>,
        removed: Vec<String>,
    },
    #[serde(rename = "particle_info")]
    ParticleInfo {
        x: usize,
        y: usize,
        material: Option<MaterialType>,
        temp: Option<f32>,
        life: Option<f32>,
        burning: Option<bool>,
    },
    #[serde(rename = "materials")]
    Materials { materials: Vec<MaterialInfo> },
    #[serde(rename = "structures")]
    Structures { structures: Vec<StructureInfo> },
    #[serde(rename = "structure_placed")]
    StructurePlaced { success: bool, structure_name: String, error: Option<String> },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ParticleData {
    pub material: MaterialType,
    pub temp: f32,
    pub color: [u8; 3],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterialInfo {
    pub id: MaterialType,
    pub name: String,
    pub color: [u8; 3],
    pub density: f32,
    pub is_liquid: bool,
    pub is_powder: bool,
    pub is_rigid_solid: bool,
    pub is_gas: bool,
    pub is_stationary: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructureInfo {
    pub name: String,
    pub width: usize,
    pub height: usize,
    pub particle_count: usize,
    pub tile_entity_count: usize,
}

type Clients = Arc<Mutex<Vec<tokio::sync::mpsc::UnboundedSender<String>>>>;

#[derive(Debug)]
struct SimulationState {
    last_state: HashMap<String, ParticleData>,
    full_update_counter: u64,
}

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt::init();
    
    let simulation = Arc::new(Mutex::new(Simulation::new(SIMULATION_WIDTH, SIMULATION_HEIGHT)));
    let clients: Clients = Arc::new(Mutex::new(Vec::new()));
    let sim_state = Arc::new(Mutex::new(SimulationState {
        last_state: HashMap::new(),
        full_update_counter: 0,
    }));
    
    // Clone for the simulation loop
    let sim_for_loop = Arc::clone(&simulation);
    let clients_for_loop = Arc::clone(&clients);
    let state_for_loop = Arc::clone(&sim_state);
    
    // Start simulation loop
    tokio::spawn(async move {
        simulation_loop(sim_for_loop, clients_for_loop, state_for_loop).await;
    });
    
    // Static file serving
    let static_files = warp::path::end()
        .and(warp::get())
        .map(|| warp::reply::html(include_str!("../../frontend/index.html")));
    
    let css = warp::path("style.css")
        .and(warp::get())
        .map(|| {
            warp::reply::with_header(
                include_str!("../../frontend/style.css"),
                "content-type",
                "text/css"
            )
        });
    
    // JavaScript modules
    let js_websocket = warp::path!("js" / "websocket.js")
        .and(warp::get())
        .map(|| {
            warp::reply::with_header(
                include_str!("../../frontend/js/websocket.js"),
                "content-type",
                "application/javascript"
            )
        });
    
    let js_materials = warp::path!("js" / "materials.js")
        .and(warp::get())
        .map(|| {
            warp::reply::with_header(
                include_str!("../../frontend/js/materials.js"),
                "content-type",
                "application/javascript"
            )
        });
    
    let js_structures = warp::path!("js" / "structures.js")
        .and(warp::get())
        .map(|| {
            warp::reply::with_header(
                include_str!("../../frontend/js/structures.js"),
                "content-type",
                "application/javascript"
            )
        });
    
    let js_canvas = warp::path!("js" / "canvas.js")
        .and(warp::get())
        .map(|| {
            warp::reply::with_header(
                include_str!("../../frontend/js/canvas.js"),
                "content-type",
                "application/javascript"
            )
        });
    
    let js_brush = warp::path!("js" / "brush.js")
        .and(warp::get())
        .map(|| {
            warp::reply::with_header(
                include_str!("../../frontend/js/brush.js"),
                "content-type",
                "application/javascript"
            )
        });
    
    let js_ui = warp::path!("js" / "ui.js")
        .and(warp::get())
        .map(|| {
            warp::reply::with_header(
                include_str!("../../frontend/js/ui.js"),
                "content-type",
                "application/javascript"
            )
        });
    
    let js_app = warp::path!("js" / "app.js")
        .and(warp::get())
        .map(|| {
            warp::reply::with_header(
                include_str!("../../frontend/js/app.js"),
                "content-type",
                "application/javascript"
            )
        });
    
    let favicon = warp::path("favicon.ico")
        .and(warp::get())
        .map(|| {
            // Simple 1x1 transparent PNG
            use base64::Engine;
            let favicon_bytes = base64::engine::general_purpose::STANDARD.decode("iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNkYPhfDwAChwGA60e6kgAAAABJRU5ErkJggg==").unwrap_or_default();
            warp::reply::with_header(
                favicon_bytes,
                "content-type",
                "image/x-icon"
            )
        });
    
    // WebSocket endpoint
    let simulation_for_ws = Arc::clone(&simulation);
    let clients_for_ws = Arc::clone(&clients);
    
    let websocket = warp::path("ws")
        .and(warp::ws())
        .map(move |ws: warp::ws::Ws| {
            let simulation = Arc::clone(&simulation_for_ws);
            let clients = Arc::clone(&clients_for_ws);
            ws.on_upgrade(move |websocket| handle_websocket(websocket, simulation, clients))
        });
    
    let routes = static_files.or(css)
        .or(js_websocket).or(js_materials).or(js_structures)
        .or(js_canvas).or(js_brush).or(js_ui).or(js_app)
        .or(favicon).or(websocket);
    
    
    warp::serve(routes)
        .run(([127, 0, 0, 1], 3030))
        .await;
}

async fn simulation_loop(simulation: Arc<Mutex<Simulation>>, clients: Clients, sim_state: Arc<Mutex<SimulationState>>) {
    let mut last_time = Instant::now();
    let mut interval = time::interval(FRAME_DURATION);
    let mut frame_count = 0u64;
    
    loop {
        interval.tick().await;
        frame_count += 1;
        
        let now = Instant::now();
        let delta_time = now.duration_since(last_time).as_secs_f32();
        last_time = now;
        
        // Clamp delta time to avoid large jumps
        let delta_time = delta_time.min(0.1);
        
        // Update simulation
        {
            let mut sim = simulation.lock().unwrap();
            sim.update(delta_time);
        }
        
        // Only broadcast every BROADCAST_INTERVAL frames to reduce network load
        if frame_count % BROADCAST_INTERVAL == 0 {
            // Only broadcast if we have clients
            let should_broadcast = {
                let clients_lock = clients.lock().unwrap();
                !clients_lock.is_empty()
            };
            
            if should_broadcast {
                // Create delta update
                let message = {
                    let sim = simulation.lock().unwrap();
                    let mut state = sim_state.lock().unwrap();
                    create_delta_update(&*sim, &mut state)
                };
                
                if let Some(msg) = message {
                    broadcast_to_clients(&clients, &msg).await;
                }
            }
        }
    }
}

fn create_simulation_state_message(simulation: &Simulation) -> ServerMessage {
    let mut particles = HashMap::new();
    
    // Only scan a smaller area or use sparse representation for better performance
    for y in 0..simulation.height {
        for x in 0..simulation.width {
            if let Some(particle_data) = simulation.get_particle_data(x, y) {
                let (material, temp, _life, _burning) = particle_data;
                if material != MaterialType::Empty {
                    // Create color only once per material type for performance
                    let color = match material {
                        MaterialType::Sand => [194, 178, 128],
                        MaterialType::Water => [64, 164, 223],
                        MaterialType::Fire => [255, 100, 0],
                        MaterialType::Stone => [128, 128, 128],
                        MaterialType::Lava => [255, 69, 0],
                        _ => {
                            let mut temp_particle = Particle::new(x, y, material, Some(temp));
                            temp_particle.get_color()
                        }
                    };
                    
                    particles.insert(
                        format!("{},{}", x, y),
                        ParticleData {
                            material,
                            temp,
                            color,
                        }
                    );
                }
            }
        }
    }
    
    ServerMessage::SimulationState {
        width: simulation.width,
        height: simulation.height,
        particles,
    }
}

fn create_delta_update(simulation: &Simulation, state: &mut SimulationState) -> Option<ServerMessage> {
    // Send full update every 60 frames (2 seconds at 30 FPS) to sync
    if state.full_update_counter % 60 == 0 {
        state.full_update_counter += 1;
        let full_state = create_simulation_state_message(simulation);
        
        // Update last_state to current state
        if let ServerMessage::SimulationState { particles, .. } = &full_state {
            state.last_state = particles.clone();
        }
        
        return Some(full_state);
    }
    
    state.full_update_counter += 1;
    
    // Get current particles - optimized to only scan dirty regions
    let mut current_particles = HashMap::new();
    
    // Use dirty region optimization: only scan areas that likely changed
    let chunk_size = 16; // Match simulation chunk size
    let chunks_x = (simulation.width + chunk_size - 1) / chunk_size;
    let chunks_y = (simulation.height + chunk_size - 1) / chunk_size;
    
    // Quick scan to find regions with particles (sparse grid optimization)
    let mut active_regions = Vec::new();
    for chunk_y in 0..chunks_y {
        for chunk_x in 0..chunks_x {
            let start_x = chunk_x * chunk_size;
            let end_x = ((chunk_x + 1) * chunk_size).min(simulation.width);
            let start_y = chunk_y * chunk_size;
            let end_y = ((chunk_y + 1) * chunk_size).min(simulation.height);
            
            // Quick check if chunk has any particles
            let mut has_particles = false;
            'chunk_check: for y in start_y..end_y {
                for x in start_x..end_x {
                    if simulation.get_particle_data(x, y).is_some() {
                        has_particles = true;
                        break 'chunk_check;
                    }
                }
            }
            
            if has_particles {
                active_regions.push((start_x, start_y, end_x, end_y));
            }
        }
    }
    
    // Only scan active regions
    for (start_x, start_y, end_x, end_y) in active_regions {
        for y in start_y..end_y {
            for x in start_x..end_x {
                if let Some(particle_data) = simulation.get_particle_data(x, y) {
                    let (material, temp, _life, _burning) = particle_data;
                    if material != MaterialType::Empty {
                        let color = get_fast_material_color(material);
                        let key = format!("{},{}", x, y);
                        current_particles.insert(key, ParticleData {
                            material,
                            temp,
                            color,
                        });
                    }
                }
            }
        }
    }
    
    // Calculate deltas
    let mut added = HashMap::new();
    let mut removed = Vec::new();
    
    // Find added/changed particles
    for (key, particle) in &current_particles {
        if !state.last_state.contains_key(key) || 
           state.last_state.get(key) != Some(particle) {
            added.insert(key.clone(), particle.clone());
        }
    }
    
    // Find removed particles
    for key in state.last_state.keys() {
        if !current_particles.contains_key(key) {
            removed.push(key.clone());
        }
    }
    
    // Update last state
    state.last_state = current_particles;
    
    // Only send delta if there are changes
    if !added.is_empty() || !removed.is_empty() {
        Some(ServerMessage::DeltaUpdate { added, removed })
    } else {
        None
    }
}

fn get_fast_material_color(material: MaterialType) -> [u8; 3] {
    // Optimized color lookup without temperature calculation
    match material {
        MaterialType::Sand => [194, 178, 128],
        MaterialType::Water => [64, 164, 223],
        MaterialType::Stone => [128, 128, 128],
        MaterialType::Fire => [255, 100, 0],
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

async fn broadcast_to_clients(clients: &Clients, message: &ServerMessage) {
    let message_json = match serde_json::to_string(message) {
        Ok(json) => json,
        Err(_) => return,
    };
    
    let mut clients_lock = clients.lock().unwrap();
    let client_count = clients_lock.len();
    let mut to_remove = Vec::new();
    
    for (i, client) in clients_lock.iter().enumerate() {
        if let Err(_) = client.send(message_json.clone()) {
            to_remove.push(i);
        }
    }
    
    // Remove disconnected clients (in reverse order to maintain indices)
    for &i in to_remove.iter().rev() {
        clients_lock.remove(i);
    }
}

async fn handle_websocket(
    websocket: warp::ws::WebSocket,
    simulation: Arc<Mutex<Simulation>>,
    clients: Clients,
) {
    let (mut ws_sender, mut ws_receiver) = websocket.split();
    
    // Create a channel for this client
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<String>();
    
    // Add this client to the list
    {
        let mut clients_lock = clients.lock().unwrap();
        clients_lock.push(tx.clone());
    }
    
    // Spawn a task to handle outgoing messages for this client
    let outgoing_task = tokio::spawn(async move {
        while let Some(message) = rx.recv().await {
            if let Err(_) = ws_sender.send(warp::ws::Message::text(message)).await {
                break;
            }
        }
    });
    
    // Send initial materials list
    let materials_message = ServerMessage::Materials {
        materials: get_materials_info(),
    };
    
    if let Ok(json) = serde_json::to_string(&materials_message) {
        let _ = tx.send(json);
    }
    
    // Send structures list
    let structures_message = ServerMessage::Structures {
        structures: get_structures_info(),
    };
    
    if let Ok(json) = serde_json::to_string(&structures_message) {
        let _ = tx.send(json);
    }
    
    
    // Handle incoming messages
    while let Some(result) = ws_receiver.next().await {
        match result {
            Ok(msg) => {
                if let Ok(text) = msg.to_str() {
                    match serde_json::from_str::<ClientMessage>(text) {
                        Ok(client_message) => {
                            handle_client_message(client_message, &simulation).await;
                        }
                        Err(_) => {}
                    }
                } else if msg.is_close() {
                    break;
                }
            }
            Err(_) => {
                break;
            }
        }
    }
    
    outgoing_task.abort();
}

async fn handle_client_message(
    message: ClientMessage,
    simulation: &Arc<Mutex<Simulation>>,
) {
    match message {
        ClientMessage::Paint { x, y, material, brush_size } => {
            
            let mut sim = simulation.lock().unwrap();
            
            let start_x = x.saturating_sub(brush_size);
            let end_x = (x + brush_size).min(sim.width.saturating_sub(1));
            let start_y = y.saturating_sub(brush_size);
            let end_y = (y + brush_size).min(sim.height.saturating_sub(1));
            let brush_size_sq = brush_size * brush_size;
            
            let mut placed_count = 0;
            for px in start_x..=end_x {
                for py in start_y..=end_y {
                    let dx = px as i32 - x as i32;
                    let dy = py as i32 - y as i32;
                    let dist_sq = (dx * dx + dy * dy) as usize;
                    
                    if dist_sq <= brush_size_sq {
                        // Check if we can paint here (don't overwrite generators unless erasing)
                        if let Some(existing_data) = sim.get_particle_data(px, py) {
                            if existing_data.0 == MaterialType::Generator && material != MaterialType::Eraser {
                                continue;
                            }
                        }
                        
                        if sim.add_particle(px, py, material, None) {
                            placed_count += 1;
                        }
                    }
                }
            }
            
        }
        ClientMessage::Clear => {
            let mut sim = simulation.lock().unwrap();
            sim.clear();
        }
        ClientMessage::GetParticle { x: _, y: _ } => {
            // For now, we'll just ignore this since we're broadcasting full state
            // In a more optimized version, we'd send individual particle info
        }
        ClientMessage::PlaceStructure { structure_name, x, y } => {
            let mut sim = simulation.lock().unwrap();
            
            // Try to place the structure
            match sand_engine::Structure::get_by_name(&structure_name) {
                Some(structure) => {
                    // Convert coordinates to world coordinates
                    let world_x = x as i64;
                    let world_y = y as i64;
                    
                    // For now, we'll just add the structure particles to the simulation
                    // In a more complete implementation, we'd use the chunk manager
                    let mut particles_placed = 0;
                    
                    for particle_data in &structure.particles {
                        let particle_x = (world_x + particle_data.x as i64) as usize;
                        let particle_y = (world_y + particle_data.y as i64) as usize;
                        
                        // Check bounds
                        if particle_x < sim.width && particle_y < sim.height {
                            if sim.add_particle(particle_x, particle_y, particle_data.material, particle_data.temp) {
                                particles_placed += 1;
                            }
                        }
                    }
                    
                    println!("Placed structure '{}' at ({}, {}) with {} particles", 
                             structure_name, x, y, particles_placed);
                }
                None => {
                    println!("Unknown structure: {}", structure_name);
                }
            }
        }
    }
}

fn get_materials_info() -> Vec<MaterialInfo> {
    use sand_engine::materials::get_material_properties;
    
    let materials = [
        MaterialType::Sand, MaterialType::Water, MaterialType::Stone, MaterialType::Plant,
        MaterialType::Fire, MaterialType::Lava, MaterialType::Glass, MaterialType::Steam,
        MaterialType::Oil, MaterialType::Acid, MaterialType::Coal, MaterialType::Gunpowder,
        MaterialType::Ice, MaterialType::Wood, MaterialType::Smoke, MaterialType::ToxicGas,
        MaterialType::Slime, MaterialType::Gasoline, MaterialType::Generator, MaterialType::Fuse,
        MaterialType::Ash, MaterialType::Gold, MaterialType::Iron, MaterialType::Eraser,
    ];
    
    materials.iter().map(|&material_type| {
        let props = get_material_properties(material_type);
        MaterialInfo {
            id: material_type,
            name: props.name.clone(),
            color: props.base_color,
            density: props.density,
            is_liquid: props.is_liquid(material_type),
            is_powder: props.is_powder(material_type),
            is_rigid_solid: props.is_rigid_solid(material_type),
            is_gas: props.is_gas(material_type),
            is_stationary: props.is_stationary(material_type),
        }
    }).collect()
}

fn get_structures_info() -> Vec<StructureInfo> {
    use sand_engine::Structure;
    
    Structure::get_all_structures().iter().map(|structure| {
        StructureInfo {
            name: structure.name.clone(),
            width: structure.width,
            height: structure.height,
            particle_count: structure.particles.len(),
            tile_entity_count: structure.tile_entities.len(),
        }
    }).collect()
}