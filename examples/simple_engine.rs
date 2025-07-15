use sand_engine::{PhysicsEngine, MaterialType};
use std::time::Duration;
use std::thread;

fn main() {
    println!("Sand Engine - Game Engine Integration Example");
    println!("Creating 100x75 physics simulation...");
    
    let mut engine = PhysicsEngine::new(100, 75);
    
    // Add some initial materials
    println!("Adding demo materials...");
    
    // Add sand at the top
    for x in 40..60 {
        engine.add_particle(x, 5, MaterialType::Sand, None);
    }
    
    // Add water pool
    for x in 30..70 {
        for y in 60..65 {
            engine.add_particle(x, y, MaterialType::Water, None);
        }
    }
    
    // Add some fire
    engine.paint_material(50, 20, MaterialType::Fire, 2);
    
    // Add wood to burn
    for x in 45..55 {
        for y in 25..30 {
            engine.add_particle(x, y, MaterialType::Wood, None);
        }
    }
    
    println!("Starting simulation...");
    
    // Run simulation for 10 seconds
    let start_time = std::time::Instant::now();
    let mut frame_count = 0;
    
    while start_time.elapsed() < Duration::from_secs(10) {
        // Update physics (this would be called from your game loop)
        engine.update();
        frame_count += 1;
        
        // Print stats every 60 frames (about 1 second)
        if frame_count % 60 == 0 {
            let stats = engine.stats();
            println!("Frame {}: {} particles active", 
                stats.frame_count, stats.particle_count);
            
            // Example: Check what's at a specific position
            if let Some((material, temp, _, _)) = engine.get_particle_data(50, 40) {
                println!("  Position (50,40): {:?} at {:.1}°C", material, temp);
            }
        }
        
        // Sleep to maintain roughly 60 FPS
        thread::sleep(Duration::from_millis(16));
    }
    
    println!("Simulation complete!");
    println!("Final stats: {:?}", engine.stats());
    
    // Example: Get final state for rendering
    println!("\nGetting simulation state for rendering...");
    let state = engine.get_state();
    let mut particle_types = std::collections::HashMap::new();
    
    for row in state {
        for cell in row {
            if let Some((material, _, _)) = cell {
                *particle_types.entry(material).or_insert(0) += 1;
            }
        }
    }
    
    println!("Final particle distribution:");
    for (material, count) in particle_types {
        println!("  {:?}: {}", material, count);
    }
}