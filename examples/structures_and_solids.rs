use sand_engine::*;
use std::thread;
use std::time::Duration;

fn main() {
    println!("SandEngine - Structures and Solids Demo");
    println!("=====================================");
    
    // Create world components
    let mut chunk_manager = ChunkManager::new();
    let mut tile_entity_manager = TileEntityManager::new();
    
    println!("1. Creating structures...");
    
    // Spawn some structures
    let house = Structure::house();
    house.spawn(50, 50, &mut chunk_manager, &mut tile_entity_manager);
    println!("   ✓ House spawned at (50, 50)");
    
    let bridge = Structure::bridge();
    bridge.spawn(100, 30, &mut chunk_manager, &mut tile_entity_manager);
    println!("   ✓ Bridge spawned at (100, 30)");
    
    let tower = Structure::castle_tower();
    tower.spawn(150, 100, &mut chunk_manager, &mut tile_entity_manager);
    println!("   ✓ Castle tower spawned at (150, 100)");
    
    // Create some rigid bodies
    let rigid_box = Structure::rigid_box();
    rigid_box.spawn(80, 80, &mut chunk_manager, &mut tile_entity_manager);
    println!("   ✓ Rigid box spawned at (80, 80)");
    
    let rigid_platform = Structure::rigid_platform();
    rigid_platform.spawn(120, 60, &mut chunk_manager, &mut tile_entity_manager);
    println!("   ✓ Rigid platform spawned at (120, 60)");
    
    println!("\n2. Analyzing world state...");
    
    // Print world statistics
    println!("   Total particles: {}", chunk_manager.total_particles());
    println!("   Total chunks: {}", chunk_manager.chunk_count());
    println!("   Tile entities: {}", tile_entity_manager.count());
    
    // Test material properties
    println!("\n3. Testing material properties...");
    
    let materials = [
        ("Sand", MaterialType::Sand),
        ("Water", MaterialType::Water),
        ("Stone", MaterialType::Stone),
        ("Wood", MaterialType::Wood),
        ("Gold", MaterialType::Gold),
        ("Iron", MaterialType::Iron),
        ("Coal", MaterialType::Coal),
    ];
    
    for (name, material) in materials {
        let props = sand_engine::materials::get_material_properties(material);
        println!("   {}: density={:.2}, stationary={}, rigid_solid={}", 
                 name, props.density, props.is_stationary(material), props.is_rigid_solid(material));
    }
    
    println!("\n4. Simulating world generation...");
    
    // Create a world generator and generate a few chunks
    let world_generator = WorldGenerator::new(12345);
    
    // Generate some chunks around our structures
    for chunk_x in -1..=3 {
        for chunk_y in -1..=3 {
            world_generator.generate_chunk(
                (chunk_x, chunk_y),
                &mut chunk_manager,
                &mut tile_entity_manager,
            );
        }
    }
    
    println!("   ✓ Generated world chunks");
    println!("   Total particles after generation: {}", chunk_manager.total_particles());
    
    println!("\n5. Testing rigid body system...");
    
    // Test rigid body manager
    let mut rigid_body_manager = RigidBodyManager::new();
    println!("   ✓ Rigid body manager created");
    println!("   Note: Rigid body detection would find connected solid regions");
    
    println!("\n6. Running physics simulation...");
    
    // Run a few physics steps
    for step in 0..5 {
        // Update tile entities
        let nearby_particles = |_pos: (i64, i64)| Vec::new();
        let _effects = tile_entity_manager.update_all(0.016, nearby_particles);
        
        // Update rigid bodies
        rigid_body_manager.step();
        
        // Simple stats
        if step % 2 == 0 {
            println!("   Step {}: {} rigid bodies active", step, rigid_body_manager.rigid_body_count());
        }
        
        thread::sleep(Duration::from_millis(100));
    }
    
    println!("\n7. Testing save/load system...");
    
    // Test save/load
    let save_manager = SaveLoadManager::new("./test_saves").unwrap();
    
    let metadata = WorldMetadata {
        version: "1.0".to_string(),
        world_name: "structures_demo".to_string(),
        player_count: 1,
        total_playtime: 0.0,
        world_size: (200, 200),
        spawn_point: (100.0, 100.0),
        created_at: "2025-01-01T00:00:00Z".to_string(),
        difficulty: Difficulty::Normal,
        game_mode: GameMode::Creative,
        last_played: "2025-01-01T00:00:00Z".to_string(),
        seed: 12345,
    };
    
    let ecs = ECS::new();
    
    match save_manager.save_world(
        "structures_demo",
        &chunk_manager,
        &ecs,
        &tile_entity_manager,
        &world_generator,
        metadata,
    ) {
        Ok(_) => println!("   ✓ World saved successfully"),
        Err(e) => println!("   ✗ Save failed: {}", e),
    }
    
    // List saved worlds
    match save_manager.list_worlds() {
        Ok(worlds) => {
            println!("   Found {} saved worlds:", worlds.len());
            for world in worlds {
                println!("     - {}", world);
            }
        },
        Err(e) => println!("   ✗ Failed to list worlds: {}", e),
    }
    
    println!("\n8. Demonstrating material behavior...");
    
    // Add some falling and stationary materials
    let falling_materials = [
        (MaterialType::Sand, "Sand (falls)"),
        (MaterialType::Water, "Water (flows)"),
        (MaterialType::Ash, "Ash (falls)"),
    ];
    
    let stationary_materials = [
        (MaterialType::Stone, "Stone (stationary)"),
        (MaterialType::Wood, "Wood (stationary)"),
        (MaterialType::Gold, "Gold (stationary)"),
        (MaterialType::Iron, "Iron (stationary)"),
    ];
    
    println!("   Falling materials:");
    for (material, name) in falling_materials {
        let props = sand_engine::materials::get_material_properties(material);
        println!("     {} - density: {:.2}", name, props.density);
    }
    
    println!("   Stationary materials:");
    for (material, name) in stationary_materials {
        let props = sand_engine::materials::get_material_properties(material);
        println!("     {} - density: {:.2}", name, props.density);
    }
    
    println!("\n9. Final statistics:");
    println!("   Total particles: {}", chunk_manager.total_particles());
    println!("   Total chunks: {}", chunk_manager.chunk_count());
    println!("   Tile entities: {}", tile_entity_manager.count());
    println!("   Rigid bodies: {}", rigid_body_manager.rigid_body_count());
    
    println!("\n✓ Demo completed successfully!");
    println!("  The SandEngine now supports:");
    println!("  - Proper density-based physics");
    println!("  - Stationary solid materials");
    println!("  - Complex structures and rigid bodies");
    println!("  - World generation with biomes");
    println!("  - Save/load system");
    println!("  - Tile entities and interactive objects");
}