use sand_engine::{Simulation, MaterialType};
use std::thread;
use std::time::{Duration, Instant};

fn main() {
    println!("Testing server-style simulation...");
    
    let mut simulation = Simulation::new(200, 150);
    
    // Add some sand particles like in the server
    for x in 95..105 {
        for y in 5..10 {
            simulation.add_particle(x, y, MaterialType::Sand, None);
        }
    }
    
    println!("Added sand particles from (95,5) to (104,9)");
    
    // Check initial positions
    for y in 0..20 {
        for x in 95..105 {
            if let Some(particle) = simulation.get_particle(x, y) {
                if particle.material_type == MaterialType::Sand {
                    println!("Initial sand at ({}, {})", x, y);
                }
            }
        }
    }
    
    let start_time = Instant::now();
    let mut last_time = Instant::now();
    let frame_duration = Duration::from_millis(1000 / 60);
    
    // Run for 5 seconds like the server
    for frame in 0..300 {
        thread::sleep(frame_duration);
        
        let now = Instant::now();
        let delta_time = now.duration_since(last_time).as_secs_f32();
        last_time = now;
        
        // Clamp delta time like server
        let delta_time = delta_time.min(0.1);
        
        println!("Frame {}: delta_time = {:.6}", frame, delta_time);
        
        // Update simulation exactly like server
        simulation.update(delta_time);
        
        // Check particle positions every 60 frames
        if frame % 60 == 0 {
            println!("Frame {} - Checking particle positions:", frame);
            let mut found_particles = 0;
            for y in 0..50 {
                for x in 90..110 {
                    if let Some(particle) = simulation.get_particle(x, y) {
                        if particle.material_type == MaterialType::Sand {
                            println!("  Sand at ({}, {})", x, y);
                            found_particles += 1;
                        }
                    }
                }
            }
            println!("  Total sand particles found: {}", found_particles);
            
            if frame >= 120 {
                break; // Stop after 2 seconds to see movement
            }
        }
    }
    
    println!("Test complete!");
}