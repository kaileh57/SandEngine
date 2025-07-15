use sand_engine::{Simulation, MaterialType};

fn main() {
    println!("Creating simulation...");
    let mut simulation = Simulation::new(200, 150);
    
    // Add sand particles
    println!("Adding sand particles...");
    for x in 95..105 {
        for y in 5..10 {
            simulation.add_particle(x, y, MaterialType::Sand, None);
        }
    }
    
    println!("Initial state:");
    let mut initial_count = 0;
    for y in 0..50 {
        for x in 90..110 {
            if let Some(particle) = simulation.get_particle(x, y) {
                if particle.material_type == MaterialType::Sand {
                    println!("  Sand at ({}, {})", x, y);
                    initial_count += 1;
                }
            }
        }
    }
    println!("Initial sand count: {}", initial_count);
    
    // Test single update
    println!("\nRunning single update with delta_time = 0.016667...");
    simulation.update(0.016667);
    
    println!("After 1 update:");
    let mut count_after_1 = 0;
    for y in 0..50 {
        for x in 90..110 {
            if let Some(particle) = simulation.get_particle(x, y) {
                if particle.material_type == MaterialType::Sand {
                    println!("  Sand at ({}, {})", x, y);
                    count_after_1 += 1;
                }
            }
        }
    }
    println!("Sand count after 1 update: {}", count_after_1);
    
    // Test several more updates
    println!("\nRunning 10 more updates...");
    for i in 0..10 {
        simulation.update(0.016667);
        if i % 5 == 4 {
            let mut count = 0;
            for y in 0..50 {
                for x in 90..110 {
                    if let Some(particle) = simulation.get_particle(x, y) {
                        if particle.material_type == MaterialType::Sand {
                            count += 1;
                        }
                    }
                }
            }
            println!("After {} more updates: {} sand particles", i + 1, count);
        }
    }
    
    println!("\nFinal particle positions:");
    for y in 0..50 {
        for x in 90..110 {
            if let Some(particle) = simulation.get_particle(x, y) {
                if particle.material_type == MaterialType::Sand {
                    println!("  Sand at ({}, {})", x, y);
                }
            }
        }
    }
}