use crate::chunk::{ChunkManager, ChunkKey, CHUNK_SIZE};
use crate::materials::MaterialType;
use crate::particle::Particle;
use crate::tile_entity::{TileEntity, TileEntityManager};
use noise::{NoiseFn, Perlin, Seedable};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// World generation system based on biomes and features
#[derive(Debug, Clone)]
pub struct WorldGenerator {
    seed: u64,
    noise_terrain: Perlin,
    noise_caves: Perlin,
    noise_ores: Perlin,
    noise_temperature: Perlin,
    noise_humidity: Perlin,
    biome_registry: BiomeRegistry,
    feature_registry: FeatureRegistry,
}

impl WorldGenerator {
    pub fn new(seed: u64) -> Self {
        let mut terrain_noise = Perlin::new(seed as u32);
        terrain_noise = terrain_noise.set_seed(seed as u32);
        
        let mut caves_noise = Perlin::new((seed + 1) as u32);
        caves_noise = caves_noise.set_seed((seed + 1) as u32);
        
        let mut ores_noise = Perlin::new((seed + 2) as u32);
        ores_noise = ores_noise.set_seed((seed + 2) as u32);
        
        let mut temp_noise = Perlin::new((seed + 3) as u32);
        temp_noise = temp_noise.set_seed((seed + 3) as u32);
        
        let mut humidity_noise = Perlin::new((seed + 4) as u32);
        humidity_noise = humidity_noise.set_seed((seed + 4) as u32);

        Self {
            seed,
            noise_terrain: terrain_noise,
            noise_caves: caves_noise,
            noise_ores: ores_noise,
            noise_temperature: temp_noise,
            noise_humidity: humidity_noise,
            biome_registry: BiomeRegistry::default(),
            feature_registry: FeatureRegistry::default(),
        }
    }

    /// Generate a chunk at the given coordinates
    pub fn get_seed(&self) -> u64 {
        self.seed
    }

    pub fn generate_chunk(&self, chunk_key: ChunkKey, chunk_manager: &mut ChunkManager, tile_entity_manager: &mut TileEntityManager) {
        let (chunk_x, chunk_y) = chunk_key;
        let mut rng = ChaCha8Rng::seed_from_u64(self.seed + (chunk_x as u64 * 1000000) + chunk_y as u64);
        
        // Generate terrain heightmap
        let mut heightmap = vec![vec![0; CHUNK_SIZE]; CHUNK_SIZE];
        let mut biome_map = vec![vec![BiomeType::Plains; CHUNK_SIZE]; CHUNK_SIZE];
        
        for local_y in 0..CHUNK_SIZE {
            for local_x in 0..CHUNK_SIZE {
                let world_x = chunk_x as f64 * CHUNK_SIZE as f64 + local_x as f64;
                let world_y = chunk_y as f64 * CHUNK_SIZE as f64 + local_y as f64;
                
                // Generate height using noise
                let height_noise = self.noise_terrain.get([world_x * 0.01, world_y * 0.01]);
                let height = ((height_noise + 1.0) * 0.5 * 30.0) as i32 + 20; // Height between 20-50
                heightmap[local_y][local_x] = height;
                
                // Determine biome based on temperature and humidity
                let temperature = self.noise_temperature.get([world_x * 0.005, world_y * 0.005]);
                let humidity = self.noise_humidity.get([world_x * 0.005, world_y * 0.005]);
                biome_map[local_y][local_x] = self.determine_biome(temperature, humidity);
            }
        }
        
        // Generate terrain layers
        for local_y in 0..CHUNK_SIZE {
            for local_x in 0..CHUNK_SIZE {
                let world_x = chunk_x as i64 * CHUNK_SIZE as i64 + local_x as i64;
                let world_y = chunk_y as i64 * CHUNK_SIZE as i64 + local_y as i64;
                let height = heightmap[local_y][local_x];
                let biome = biome_map[local_y][local_x];
                
                // Generate vertical column
                for depth in 0..CHUNK_SIZE {
                    let world_z = world_y; // In 2D, z is just y
                    let absolute_depth = world_z as i32;
                    
                    let material = if absolute_depth > height {
                        // Above ground - air or water
                        if absolute_depth < 25 {
                            Some(MaterialType::Water) // Sea level
                        } else {
                            None // Air
                        }
                    } else if absolute_depth > height - 3 {
                        // Surface layer
                        self.get_surface_material(biome, &mut rng)
                    } else if absolute_depth > height - 10 {
                        // Subsurface layer
                        self.get_subsurface_material(biome, &mut rng)
                    } else {
                        // Deep layer
                        self.get_deep_material(world_x, world_z, &mut rng)
                    };
                    
                    if let Some(mat) = material {
                        let particle = Particle::new(world_x as usize, world_z as usize, mat, None);
                        chunk_manager.set_particle(world_x, world_z, particle);
                    }
                }
            }
        }
        
        // Generate caves
        self.generate_caves(chunk_key, chunk_manager);
        
        // Generate ore deposits
        self.generate_ores(chunk_key, chunk_manager, &mut rng);
        
        // Generate structures and features
        self.generate_features(chunk_key, chunk_manager, tile_entity_manager, &biome_map, &mut rng);
    }

    fn determine_biome(&self, temperature: f64, humidity: f64) -> BiomeType {
        match (temperature, humidity) {
            (t, h) if t > 0.6 && h < -0.2 => BiomeType::Desert,
            (t, h) if t > 0.3 && h > 0.5 => BiomeType::Jungle,
            (t, h) if t < -0.3 => BiomeType::Tundra,
            (t, h) if t > 0.1 && h > 0.1 => BiomeType::Forest,
            (t, h) if h < -0.5 => BiomeType::Wasteland,
            _ => BiomeType::Plains,
        }
    }

    fn get_surface_material(&self, biome: BiomeType, rng: &mut ChaCha8Rng) -> Option<MaterialType> {
        match biome {
            BiomeType::Desert => Some(MaterialType::Sand),
            BiomeType::Plains => {
                if rng.gen::<f32>() < 0.1 {
                    Some(MaterialType::Plant)
                } else {
                    Some(MaterialType::Sand)
                }
            },
            BiomeType::Forest => {
                if rng.gen::<f32>() < 0.3 {
                    Some(MaterialType::Wood)
                } else if rng.gen::<f32>() < 0.5 {
                    Some(MaterialType::Plant)
                } else {
                    Some(MaterialType::Sand)
                }
            },
            BiomeType::Jungle => {
                if rng.gen::<f32>() < 0.6 {
                    Some(MaterialType::Plant)
                } else {
                    Some(MaterialType::Sand)
                }
            },
            BiomeType::Tundra => Some(MaterialType::Ice),
            BiomeType::Wasteland => {
                if rng.gen::<f32>() < 0.1 {
                    Some(MaterialType::Ash)
                } else {
                    Some(MaterialType::Stone)
                }
            },
        }
    }

    fn get_subsurface_material(&self, _biome: BiomeType, rng: &mut ChaCha8Rng) -> Option<MaterialType> {
        if rng.gen::<f32>() < 0.8 {
            Some(MaterialType::Sand)
        } else {
            Some(MaterialType::Stone)
        }
    }

    fn get_deep_material(&self, world_x: i64, world_y: i64, rng: &mut ChaCha8Rng) -> Option<MaterialType> {
        // Check for ore generation
        let ore_noise = self.noise_ores.get([world_x as f64 * 0.02, world_y as f64 * 0.02]);
        
        if ore_noise > 0.7 {
            // Rare ores
            if rng.gen::<f32>() < 0.1 {
                Some(MaterialType::Gold)
            } else {
                Some(MaterialType::Iron)
            }
        } else if ore_noise > 0.4 {
            Some(MaterialType::Coal)
        } else {
            Some(MaterialType::Stone)
        }
    }

    fn generate_caves(&self, chunk_key: ChunkKey, chunk_manager: &mut ChunkManager) {
        let (chunk_x, chunk_y) = chunk_key;
        
        for local_y in 0..CHUNK_SIZE {
            for local_x in 0..CHUNK_SIZE {
                let world_x = chunk_x as f64 * CHUNK_SIZE as f64 + local_x as f64;
                let world_y = chunk_y as f64 * CHUNK_SIZE as f64 + local_y as f64;
                
                let cave_noise = self.noise_caves.get([world_x * 0.03, world_y * 0.03]);
                
                if cave_noise > 0.6 && world_y < 40.0 { // Caves only below surface
                    // Remove material to create cave
                    let world_x_i64 = chunk_x as i64 * CHUNK_SIZE as i64 + local_x as i64;
                    let world_y_i64 = chunk_y as i64 * CHUNK_SIZE as i64 + local_y as i64;
                    chunk_manager.remove_particle(world_x_i64, world_y_i64);
                }
            }
        }
    }

    fn generate_ores(&self, chunk_key: ChunkKey, chunk_manager: &mut ChunkManager, rng: &mut ChaCha8Rng) {
        let (chunk_x, chunk_y) = chunk_key;
        
        // Generate ore veins
        let num_veins = rng.gen_range(0..3);
        
        for _ in 0..num_veins {
            let start_x = rng.gen_range(0..CHUNK_SIZE);
            let start_y = rng.gen_range(0..CHUNK_SIZE);
            let vein_size = rng.gen_range(3..8);
            let ore_type = if rng.gen::<f32>() < 0.1 {
                MaterialType::Gold
            } else {
                MaterialType::Iron
            };
            
            // Generate vein using random walk
            let mut current_x = start_x;
            let mut current_y = start_y;
            
            for _ in 0..vein_size {
                let world_x = chunk_x as i64 * CHUNK_SIZE as i64 + current_x as i64;
                let world_y = chunk_y as i64 * CHUNK_SIZE as i64 + current_y as i64;
                
                if chunk_manager.get_particle(world_x, world_y).is_some() {
                    let particle = Particle::new(world_x as usize, world_y as usize, ore_type, None);
                    chunk_manager.set_particle(world_x, world_y, particle);
                }
                
                // Random walk
                current_x = (current_x as i32 + rng.gen_range(-1..=1)).max(0).min(CHUNK_SIZE as i32 - 1) as usize;
                current_y = (current_y as i32 + rng.gen_range(-1..=1)).max(0).min(CHUNK_SIZE as i32 - 1) as usize;
            }
        }
    }

    fn generate_features(&self, chunk_key: ChunkKey, chunk_manager: &mut ChunkManager, tile_entity_manager: &mut TileEntityManager, biome_map: &[Vec<BiomeType>], rng: &mut ChaCha8Rng) {
        let (chunk_x, chunk_y) = chunk_key;
        
        // Generate structures based on biome
        for local_y in (0..CHUNK_SIZE).step_by(8) {
            for local_x in (0..CHUNK_SIZE).step_by(8) {
                let biome = biome_map[local_y][local_x];
                let world_x = chunk_x as i64 * CHUNK_SIZE as i64 + local_x as i64;
                let world_y = chunk_y as i64 * CHUNK_SIZE as i64 + local_y as i64;
                
                match biome {
                    BiomeType::Forest => {
                        if rng.gen::<f32>() < 0.1 {
                            self.generate_tree(world_x, world_y, chunk_manager);
                        }
                    },
                    BiomeType::Desert => {
                        if rng.gen::<f32>() < 0.05 {
                            // Generate oasis
                            self.generate_oasis(world_x, world_y, chunk_manager);
                        }
                    },
                    BiomeType::Wasteland => {
                        if rng.gen::<f32>() < 0.02 {
                            // Generate toxic waste generator
                            let generator = TileEntity::new_spawner((world_x, world_y), MaterialType::ToxicGas, 0.5);
                            tile_entity_manager.add_tile_entity(generator);
                        }
                    },
                    BiomeType::Tundra => {
                        if rng.gen::<f32>() < 0.03 {
                            // Generate ice formations
                            self.generate_ice_formation(world_x, world_y, chunk_manager);
                        }
                    },
                    _ => {},
                }
                
                // Random loot chests
                if rng.gen::<f32>() < 0.001 {
                    let chest = TileEntity::new_chest((world_x, world_y), 50);
                    tile_entity_manager.add_tile_entity(chest);
                }
                
                // Random torches in caves
                if rng.gen::<f32>() < 0.02 && world_y < 40 {
                    if chunk_manager.get_particle(world_x, world_y).is_none() {
                        let torch = TileEntity::new_torch((world_x, world_y));
                        tile_entity_manager.add_tile_entity(torch);
                    }
                }
            }
        }
    }

    fn generate_tree(&self, center_x: i64, center_y: i64, chunk_manager: &mut ChunkManager) {
        // Simple tree generation
        let trunk_height = 5;
        let crown_radius = 3;
        
        // Generate trunk
        for y in 0..trunk_height {
            let particle = Particle::new((center_x) as usize, (center_y + y) as usize, MaterialType::Wood, None);
            chunk_manager.set_particle(center_x, center_y + y, particle);
        }
        
        // Generate crown
        for dy in -crown_radius..=crown_radius {
            for dx in -crown_radius..=crown_radius {
                if dx * dx + dy * dy <= crown_radius * crown_radius {
                    let leaf_x = center_x + dx;
                    let leaf_y = center_y + trunk_height + dy;
                    
                    if rand::random::<f32>() < 0.7 {
                        let particle = Particle::new(leaf_x as usize, leaf_y as usize, MaterialType::Plant, None);
                        chunk_manager.set_particle(leaf_x, leaf_y, particle);
                    }
                }
            }
        }
    }

    fn generate_oasis(&self, center_x: i64, center_y: i64, chunk_manager: &mut ChunkManager) {
        let radius = 4;
        
        for dy in -radius..=radius {
            for dx in -radius..=radius {
                let distance_sq = dx * dx + dy * dy;
                if distance_sq <= radius * radius {
                    let x = center_x + dx;
                    let y = center_y + dy;
                    
                    let material = if distance_sq <= 2 {
                        MaterialType::Water
                    } else {
                        MaterialType::Plant
                    };
                    
                    let particle = Particle::new(x as usize, y as usize, material, None);
                    chunk_manager.set_particle(x, y, particle);
                }
            }
        }
    }

    fn generate_ice_formation(&self, center_x: i64, center_y: i64, chunk_manager: &mut ChunkManager) {
        let height = rand::random::<i64>() % 5 + 3;
        
        for y in 0..height {
            let width = ((height - y) as f64 * 0.5 + 1.0) as i64;
            for dx in -width..=width {
                let x = center_x + dx;
                let particle = Particle::new(x as usize, (center_y + y) as usize, MaterialType::Ice, Some(-10.0));
                chunk_manager.set_particle(x, center_y + y, particle);
            }
        }
    }
}

/// Biome types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BiomeType {
    Plains,
    Desert,
    Forest,
    Jungle,
    Tundra,
    Wasteland,
}

/// Biome registry for managing biome properties
#[derive(Debug, Clone)]
pub struct BiomeRegistry {
    biomes: HashMap<BiomeType, BiomeProperties>,
}

#[derive(Debug, Clone)]
pub struct BiomeProperties {
    pub name: String,
    pub temperature_range: (f64, f64),
    pub humidity_range: (f64, f64),
    pub common_materials: Vec<MaterialType>,
    pub rare_materials: Vec<MaterialType>,
    pub structure_chance: f64,
}

/// Feature registry for managing world features
#[derive(Debug, Clone, Default)]
pub struct FeatureRegistry {
    features: HashMap<String, FeatureTemplate>,
}

#[derive(Debug, Clone)]
pub struct FeatureTemplate {
    pub name: String,
    pub biome_restrictions: Vec<BiomeType>,
    pub rarity: f64,
    pub min_size: (u32, u32),
    pub max_size: (u32, u32),
    pub materials: Vec<MaterialType>,
}

impl Default for BiomeRegistry {
    fn default() -> Self {
        let mut registry = Self {
            biomes: HashMap::new(),
        };
        
        registry.biomes.insert(BiomeType::Plains, BiomeProperties {
            name: "Plains".to_string(),
            temperature_range: (-0.2, 0.4),
            humidity_range: (-0.3, 0.3),
            common_materials: vec![MaterialType::Sand, MaterialType::Plant],
            rare_materials: vec![MaterialType::Wood],
            structure_chance: 0.1,
        });
        
        registry.biomes.insert(BiomeType::Desert, BiomeProperties {
            name: "Desert".to_string(),
            temperature_range: (0.6, 1.0),
            humidity_range: (-1.0, -0.2),
            common_materials: vec![MaterialType::Sand],
            rare_materials: vec![MaterialType::Glass],
            structure_chance: 0.05,
        });
        
        registry.biomes.insert(BiomeType::Forest, BiomeProperties {
            name: "Forest".to_string(),
            temperature_range: (0.1, 0.6),
            humidity_range: (0.1, 0.8),
            common_materials: vec![MaterialType::Wood, MaterialType::Plant],
            rare_materials: vec![MaterialType::Coal],
            structure_chance: 0.2,
        });
        
        registry
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tile_entity::TileEntityManager;

    #[test]
    fn test_world_generator_creation() {
        let generator = WorldGenerator::new(12345);
        assert_eq!(generator.seed, 12345);
    }

    #[test]
    fn test_biome_determination() {
        let generator = WorldGenerator::new(0);
        
        assert_eq!(generator.determine_biome(0.8, -0.5), BiomeType::Desert);
        assert_eq!(generator.determine_biome(0.5, 0.7), BiomeType::Jungle);
        assert_eq!(generator.determine_biome(-0.5, 0.0), BiomeType::Tundra);
        assert_eq!(generator.determine_biome(0.2, 0.2), BiomeType::Forest);
        assert_eq!(generator.determine_biome(0.0, 0.0), BiomeType::Plains);
    }

    #[test]
    fn test_chunk_generation() {
        let generator = WorldGenerator::new(54321);
        let mut chunk_manager = ChunkManager::new();
        let mut tile_entity_manager = TileEntityManager::new();
        
        generator.generate_chunk((0, 0), &mut chunk_manager, &mut tile_entity_manager);
        
        // Should have generated some particles
        assert!(chunk_manager.total_particles() > 0);
        
        // Should have some chunks
        assert!(chunk_manager.chunk_count() > 0);
    }
}