use crate::materials::MaterialType;
use crate::particle::Particle;
use crate::chunk::ChunkManager;
use crate::tile_entity::{TileEntity, TileEntityManager};

/// Predefined structures that can be spawned in the world
#[derive(Debug, Clone)]
pub struct Structure {
    pub name: String,
    pub particles: Vec<StructureParticle>,
    pub tile_entities: Vec<StructureTileEntity>,
    pub width: usize,
    pub height: usize,
}

#[derive(Debug, Clone)]
pub struct StructureParticle {
    pub x: usize,
    pub y: usize,
    pub material: MaterialType,
    pub temp: Option<f32>,
}

#[derive(Debug, Clone)]
pub struct StructureTileEntity {
    pub x: i64,
    pub y: i64,
    pub entity_type: crate::tile_entity::TileEntityType,
}

impl Structure {
    /// Create a simple house structure
    pub fn house() -> Self {
        let mut particles = Vec::new();
        
        // Foundation (stone)
        for x in 0..12 {
            particles.push(StructureParticle {
                x, y: 0,
                material: MaterialType::Stone,
                temp: Some(20.0),
            });
        }
        
        // Walls (wood)
        for y in 1..8 {
            // Left wall
            particles.push(StructureParticle {
                x: 0, y,
                material: MaterialType::Wood,
                temp: Some(20.0),
            });
            // Right wall
            particles.push(StructureParticle {
                x: 11, y,
                material: MaterialType::Wood,
                temp: Some(20.0),
            });
        }
        
        // Back wall
        for x in 1..11 {
            particles.push(StructureParticle {
                x, y: 7,
                material: MaterialType::Wood,
                temp: Some(20.0),
            });
        }
        
        // Roof (stone)
        for x in 0..12 {
            particles.push(StructureParticle {
                x, y: 8,
                material: MaterialType::Stone,
                temp: Some(20.0),
            });
        }
        
        // Add a door (gap in front wall)
        // Door is at x=5,6 y=1,2,3
        
        // Add windows (gaps in walls)
        // Window in left wall at y=4
        // Window in right wall at y=4
        
        Self {
            name: "House".to_string(),
            particles,
            tile_entities: vec![
                StructureTileEntity {
                    x: 5,
                    y: 2,
                    entity_type: crate::tile_entity::TileEntityType::Chest,
                },
                StructureTileEntity {
                    x: 8,
                    y: 2,
                    entity_type: crate::tile_entity::TileEntityType::Furnace,
                },
                StructureTileEntity {
                    x: 2,
                    y: 2,
                    entity_type: crate::tile_entity::TileEntityType::Torch,
                },
            ],
            width: 12,
            height: 9,
        }
    }
    
    /// Create a bridge structure
    pub fn bridge() -> Self {
        let mut particles = Vec::new();
        
        // Bridge deck (wood)
        for x in 0..20 {
            particles.push(StructureParticle {
                x, y: 5,
                material: MaterialType::Wood,
                temp: Some(20.0),
            });
        }
        
        // Support pillars (stone)
        for pillar_x in [4, 9, 14].iter() {
            for y in 0..6 {
                particles.push(StructureParticle {
                    x: *pillar_x, y,
                    material: MaterialType::Stone,
                    temp: Some(20.0),
                });
            }
        }
        
        // Railings (wood)
        for x in 0..20 {
            particles.push(StructureParticle {
                x, y: 6,
                material: MaterialType::Wood,
                temp: Some(20.0),
            });
        }
        
        Self {
            name: "Bridge".to_string(),
            particles,
            tile_entities: vec![
                StructureTileEntity {
                    x: 0,
                    y: 6,
                    entity_type: crate::tile_entity::TileEntityType::Torch,
                },
                StructureTileEntity {
                    x: 19,
                    y: 6,
                    entity_type: crate::tile_entity::TileEntityType::Torch,
                },
            ],
            width: 20,
            height: 7,
        }
    }
    
    /// Create a castle tower
    pub fn castle_tower() -> Self {
        let mut particles = Vec::new();
        
        // Tower base (stone)
        for x in 0..8 {
            for y in 0..2 {
                particles.push(StructureParticle {
                    x, y,
                    material: MaterialType::Stone,
                    temp: Some(20.0),
                });
            }
        }
        
        // Tower walls (stone)
        for y in 2..15 {
            // Left wall
            particles.push(StructureParticle {
                x: 0, y,
                material: MaterialType::Stone,
                temp: Some(20.0),
            });
            // Right wall
            particles.push(StructureParticle {
                x: 7, y,
                material: MaterialType::Stone,
                temp: Some(20.0),
            });
            // Back wall
            particles.push(StructureParticle {
                x: 3, y,
                material: MaterialType::Stone,
                temp: Some(20.0),
            });
            particles.push(StructureParticle {
                x: 4, y,
                material: MaterialType::Stone,
                temp: Some(20.0),
            });
        }
        
        // Battlements (stone)
        for x in [0, 2, 4, 6, 7].iter() {
            particles.push(StructureParticle {
                x: *x, y: 15,
                material: MaterialType::Stone,
                temp: Some(20.0),
            });
        }
        
        Self {
            name: "Castle Tower".to_string(),
            particles,
            tile_entities: vec![
                StructureTileEntity {
                    x: 2,
                    y: 3,
                    entity_type: crate::tile_entity::TileEntityType::Chest,
                },
                StructureTileEntity {
                    x: 5,
                    y: 3,
                    entity_type: crate::tile_entity::TileEntityType::Furnace,
                },
                StructureTileEntity {
                    x: 1,
                    y: 10,
                    entity_type: crate::tile_entity::TileEntityType::Torch,
                },
                StructureTileEntity {
                    x: 6,
                    y: 10,
                    entity_type: crate::tile_entity::TileEntityType::Torch,
                },
            ],
            width: 8,
            height: 16,
        }
    }
    
    /// Create a windmill
    pub fn windmill() -> Self {
        let mut particles = Vec::new();
        
        // Base (stone)
        for x in 0..6 {
            particles.push(StructureParticle {
                x, y: 0,
                material: MaterialType::Stone,
                temp: Some(20.0),
            });
        }
        
        // Tower (stone)
        for y in 1..10 {
            particles.push(StructureParticle {
                x: 0, y,
                material: MaterialType::Stone,
                temp: Some(20.0),
            });
            particles.push(StructureParticle {
                x: 5, y,
                material: MaterialType::Stone,
                temp: Some(20.0),
            });
        }
        
        // Top (stone)
        for x in 0..6 {
            particles.push(StructureParticle {
                x, y: 10,
                material: MaterialType::Stone,
                temp: Some(20.0),
            });
        }
        
        // Windmill blades (wood) - simplified cross pattern
        for i in 0..3 {
            particles.push(StructureParticle {
                x: 7 + i, y: 8,
                material: MaterialType::Wood,
                temp: Some(20.0),
            });
            particles.push(StructureParticle {
                x: 3, y: 12 + i,
                material: MaterialType::Wood,
                temp: Some(20.0),
            });
        }
        
        Self {
            name: "Windmill".to_string(),
            particles,
            tile_entities: vec![
                StructureTileEntity {
                    x: 2,
                    y: 2,
                    entity_type: crate::tile_entity::TileEntityType::Chest,
                },
                StructureTileEntity {
                    x: 3,
                    y: 2,
                    entity_type: crate::tile_entity::TileEntityType::Furnace,
                },
                StructureTileEntity {
                    x: 1,
                    y: 8,
                    entity_type: crate::tile_entity::TileEntityType::Torch,
                },
                StructureTileEntity {
                    x: 4,
                    y: 8,
                    entity_type: crate::tile_entity::TileEntityType::Torch,
                },
            ],
            width: 10,
            height: 15,
        }
    }
    
    /// Create a simple rigid body structure (will become a rigid body)
    pub fn rigid_box() -> Self {
        let mut particles = Vec::new();
        
        // Create a solid box of stone
        for x in 0..4 {
            for y in 0..4 {
                particles.push(StructureParticle {
                    x, y,
                    material: MaterialType::Stone,
                    temp: Some(20.0),
                });
            }
        }
        
        Self {
            name: "Rigid Box".to_string(),
            particles,
            tile_entities: vec![],
            width: 4,
            height: 4,
        }
    }
    
    /// Create a complex rigid body structure (will become a rigid body)
    pub fn rigid_platform() -> Self {
        let mut particles = Vec::new();
        
        // Create a platform with legs
        // Platform top (wood)
        for x in 0..8 {
            particles.push(StructureParticle {
                x, y: 4,
                material: MaterialType::Wood,
                temp: Some(20.0),
            });
        }
        
        // Legs (stone)
        for leg_x in [0, 3, 5, 7].iter() {
            for y in 0..4 {
                particles.push(StructureParticle {
                    x: *leg_x, y,
                    material: MaterialType::Stone,
                    temp: Some(20.0),
                });
            }
        }
        
        Self {
            name: "Rigid Platform".to_string(),
            particles,
            tile_entities: vec![],
            width: 8,
            height: 5,
        }
    }
    
    /// Spawn this structure in the world
    pub fn spawn(&self, center_x: i64, center_y: i64, chunk_manager: &mut ChunkManager, tile_entity_manager: &mut TileEntityManager) {
        let offset_x = center_x - (self.width as i64 / 2);
        let offset_y = center_y - (self.height as i64 / 2);
        
        // Place particles
        for particle_data in &self.particles {
            let world_x = offset_x + particle_data.x as i64;
            let world_y = offset_y + particle_data.y as i64;
            
            let particle = Particle::new(
                world_x as usize,
                world_y as usize,
                particle_data.material,
                particle_data.temp,
            );
            
            chunk_manager.set_particle(world_x, world_y, particle);
        }
        
        // Place tile entities
        for tile_data in &self.tile_entities {
            let world_x = offset_x + tile_data.x;
            let world_y = offset_y + tile_data.y;
            
            let tile_entity = match tile_data.entity_type {
                crate::tile_entity::TileEntityType::Chest => {
                    TileEntity::new_chest((world_x, world_y), 100)
                },
                crate::tile_entity::TileEntityType::Furnace => {
                    TileEntity::new_furnace((world_x, world_y))
                },
                crate::tile_entity::TileEntityType::Torch => {
                    TileEntity::new_torch((world_x, world_y))
                },
                _ => TileEntity::new_chest((world_x, world_y), 50), // Default fallback
            };
            
            tile_entity_manager.add_tile_entity(tile_entity);
        }
    }
    
    /// Get all available structures
    pub fn get_all_structures() -> Vec<Structure> {
        vec![
            Structure::house(),
            Structure::bridge(),
            Structure::castle_tower(),
            Structure::windmill(),
            Structure::rigid_box(),
            Structure::rigid_platform(),
        ]
    }
    
    /// Get a structure by name
    pub fn get_by_name(name: &str) -> Option<Structure> {
        match name {
            "House" => Some(Structure::house()),
            "Bridge" => Some(Structure::bridge()),
            "Castle Tower" => Some(Structure::castle_tower()),
            "Windmill" => Some(Structure::windmill()),
            "Rigid Box" => Some(Structure::rigid_box()),
            "Rigid Platform" => Some(Structure::rigid_platform()),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chunk::ChunkManager;
    use crate::tile_entity::TileEntityManager;
    
    #[test]
    fn test_structure_creation() {
        let house = Structure::house();
        assert_eq!(house.name, "House");
        assert!(house.particles.len() > 0);
        assert!(house.tile_entities.len() > 0);
    }
    
    #[test]
    fn test_structure_spawning() {
        let mut chunk_manager = ChunkManager::new();
        let mut tile_entity_manager = TileEntityManager::new();
        
        let house = Structure::house();
        house.spawn(50, 50, &mut chunk_manager, &mut tile_entity_manager);
        
        // Should have spawned particles
        assert!(chunk_manager.total_particles() > 0);
        
        // Should have spawned tile entities
        assert!(tile_entity_manager.get_tile_entities().len() > 0);
    }
    
    #[test]
    fn test_get_all_structures() {
        let structures = Structure::get_all_structures();
        assert!(structures.len() >= 6);
    }
    
    #[test]
    fn test_get_by_name() {
        let house = Structure::get_by_name("House");
        assert!(house.is_some());
        assert_eq!(house.unwrap().name, "House");
        
        let invalid = Structure::get_by_name("Invalid");
        assert!(invalid.is_none());
    }
}