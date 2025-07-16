use crate::chunk::{ChunkManager, ChunkKey};
use crate::ecs::ECS;
use crate::materials::MaterialType;
use crate::particle::Particle;
use crate::tile_entity::{TileEntity, TileEntityManager};
use crate::world_generation::{BiomeType, WorldGenerator};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::{Path, PathBuf};
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;

/// World save/load system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldSave {
    pub metadata: WorldMetadata,
    pub chunks: Vec<ChunkSave>,
    pub entities: ECSSnapshot,
    pub tile_entities: Vec<TileEntity>,
    pub world_generator_seed: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldMetadata {
    pub world_name: String,
    pub version: String,
    pub created_at: String,
    pub last_played: String,
    pub player_count: u32,
    pub total_playtime: f64,
    pub world_size: (i32, i32), // Min/Max chunk coordinates
    pub spawn_point: (f64, f64),
    pub difficulty: Difficulty,
    pub game_mode: GameMode,
    pub seed: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkSave {
    pub chunk_key: ChunkKey,
    pub particles: Vec<ParticleSave>,
    pub biome_data: HashMap<(usize, usize), BiomeType>,
    pub last_updated: String,
    pub generation_stage: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParticleSave {
    pub local_x: usize,
    pub local_y: usize,
    pub material_type: MaterialType,
    pub temp: f32,
    pub life: Option<f32>,
    pub burning: bool,
    pub time_in_state: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ECSSnapshot {
    pub entities: Vec<EntitySnapshot>,
    pub next_entity_id: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntitySnapshot {
    pub entity_id: u32,
    pub position: Option<(f64, f64, f64)>,
    pub velocity: Option<(f64, f64, f64)>,
    pub health: Option<(f32, f32, f32)>, // current, max, regen_rate
    pub player_data: Option<PlayerData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerData {
    pub name: String,
    pub level: u32,
    pub experience: u64,
    pub connection_id: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Difficulty {
    Peaceful,
    Easy,
    Normal,
    Hard,
    Hardcore,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GameMode {
    Survival,
    Creative,
    Adventure,
    Spectator,
}

/// Save/Load manager
pub struct SaveLoadManager {
    save_directory: PathBuf,
    compression_level: Compression,
}

impl SaveLoadManager {
    pub fn new(save_directory: impl AsRef<Path>) -> std::io::Result<Self> {
        let save_dir = save_directory.as_ref().to_path_buf();
        fs::create_dir_all(&save_dir)?;
        
        Ok(Self {
            save_directory: save_dir,
            compression_level: Compression::default(),
        })
    }

    /// Save a complete world
    pub fn save_world(
        &self,
        world_name: &str,
        chunk_manager: &ChunkManager,
        ecs: &ECS,
        tile_entity_manager: &TileEntityManager,
        world_generator: &WorldGenerator,
        metadata: WorldMetadata,
    ) -> Result<(), SaveLoadError> {
        let world_dir = self.save_directory.join(world_name);
        fs::create_dir_all(&world_dir)?;

        // Save metadata
        self.save_metadata(&world_dir, &metadata)?;

        // Save chunks
        self.save_chunks(&world_dir, chunk_manager)?;

        // Save ECS data
        self.save_ecs(&world_dir, ecs)?;

        // Save tile entities
        self.save_tile_entities(&world_dir, tile_entity_manager)?;

        // Save world generator data
        self.save_world_generator_data(&world_dir, world_generator)?;

        Ok(())
    }

    /// Load a complete world
    pub fn load_world(
        &self,
        world_name: &str,
    ) -> Result<WorldSave, SaveLoadError> {
        let world_dir = self.save_directory.join(world_name);
        
        if !world_dir.exists() {
            return Err(SaveLoadError::WorldNotFound(world_name.to_string()));
        }

        // Load metadata
        let metadata = self.load_metadata(&world_dir)?;

        // Load chunks
        let chunks = self.load_chunks(&world_dir)?;

        // Load ECS data
        let entities = self.load_ecs(&world_dir)?;

        // Load tile entities
        let tile_entities = self.load_tile_entities(&world_dir)?;

        // Load world generator seed
        let world_generator_seed = self.load_world_generator_data(&world_dir)?;

        Ok(WorldSave {
            metadata,
            chunks,
            entities,
            tile_entities,
            world_generator_seed,
        })
    }

    /// Apply loaded world data to game systems
    pub fn apply_world_save(
        world_save: &WorldSave,
        chunk_manager: &mut ChunkManager,
        ecs: &mut ECS,
        tile_entity_manager: &mut TileEntityManager,
    ) -> Result<(), SaveLoadError> {
        // Clear existing data
        chunk_manager.clear();
        ecs.clear();
        tile_entity_manager.clear();

        // Apply chunks
        for chunk_save in &world_save.chunks {
            Self::apply_chunk_save(chunk_save, chunk_manager)?;
        }

        // Apply ECS data
        Self::apply_ecs_snapshot(&world_save.entities, ecs)?;

        // Apply tile entities
        for tile_entity in &world_save.tile_entities {
            tile_entity_manager.add_tile_entity(tile_entity.clone());
        }

        Ok(())
    }

    fn save_metadata(&self, world_dir: &Path, metadata: &WorldMetadata) -> Result<(), SaveLoadError> {
        let metadata_path = world_dir.join("metadata.json");
        let file = File::create(metadata_path)?;
        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, metadata)?;
        Ok(())
    }

    fn load_metadata(&self, world_dir: &Path) -> Result<WorldMetadata, SaveLoadError> {
        let metadata_path = world_dir.join("metadata.json");
        let file = File::open(metadata_path)?;
        let reader = BufReader::new(file);
        let metadata = serde_json::from_reader(reader)?;
        Ok(metadata)
    }

    fn save_chunks(&self, world_dir: &Path, chunk_manager: &ChunkManager) -> Result<(), SaveLoadError> {
        let chunks_dir = world_dir.join("chunks");
        fs::create_dir_all(&chunks_dir)?;

        for (chunk_key, chunk) in chunk_manager.chunks_iter() {
            let chunk_save = ChunkSave::from_chunk(*chunk_key, chunk);
            let chunk_filename = format!("chunk_{}_{}.dat", chunk_key.0, chunk_key.1);
            let chunk_path = chunks_dir.join(chunk_filename);
            
            // Save with compression
            let file = File::create(chunk_path)?;
            let encoder = GzEncoder::new(file, self.compression_level);
            let writer = BufWriter::new(encoder);
            bincode::serialize_into(writer, &chunk_save)?;
        }

        Ok(())
    }

    fn load_chunks(&self, world_dir: &Path) -> Result<Vec<ChunkSave>, SaveLoadError> {
        let chunks_dir = world_dir.join("chunks");
        let mut chunks = Vec::new();

        if chunks_dir.exists() {
            for entry in fs::read_dir(chunks_dir)? {
                let entry = entry?;
                let path = entry.path();
                
                if path.extension().and_then(|s| s.to_str()) == Some("dat") {
                    let file = File::open(path)?;
                    let decoder = GzDecoder::new(file);
                    let reader = BufReader::new(decoder);
                    let chunk_save: ChunkSave = bincode::deserialize_from(reader)?;
                    chunks.push(chunk_save);
                }
            }
        }

        Ok(chunks)
    }

    fn save_ecs(&self, world_dir: &Path, ecs: &ECS) -> Result<(), SaveLoadError> {
        let ecs_snapshot = ECSSnapshot::from_ecs(ecs);
        let ecs_path = world_dir.join("entities.dat");
        
        let file = File::create(ecs_path)?;
        let encoder = GzEncoder::new(file, self.compression_level);
        let writer = BufWriter::new(encoder);
        bincode::serialize_into(writer, &ecs_snapshot)?;
        
        Ok(())
    }

    fn load_ecs(&self, world_dir: &Path) -> Result<ECSSnapshot, SaveLoadError> {
        let ecs_path = world_dir.join("entities.dat");
        
        if ecs_path.exists() {
            let file = File::open(ecs_path)?;
            let decoder = GzDecoder::new(file);
            let reader = BufReader::new(decoder);
            let snapshot = bincode::deserialize_from(reader)?;
            Ok(snapshot)
        } else {
            Ok(ECSSnapshot {
                entities: Vec::new(),
                next_entity_id: 0,
            })
        }
    }

    fn save_tile_entities(&self, world_dir: &Path, tile_entity_manager: &TileEntityManager) -> Result<(), SaveLoadError> {
        let tile_entities: Vec<TileEntity> = tile_entity_manager.get_all_positions()
            .filter_map(|pos| tile_entity_manager.get_tile_entity(pos).cloned())
            .collect();
        
        let tile_entities_path = world_dir.join("tile_entities.dat");
        let file = File::create(tile_entities_path)?;
        let encoder = GzEncoder::new(file, self.compression_level);
        let writer = BufWriter::new(encoder);
        bincode::serialize_into(writer, &tile_entities)?;
        
        Ok(())
    }

    fn load_tile_entities(&self, world_dir: &Path) -> Result<Vec<TileEntity>, SaveLoadError> {
        let tile_entities_path = world_dir.join("tile_entities.dat");
        
        if tile_entities_path.exists() {
            let file = File::open(tile_entities_path)?;
            let decoder = GzDecoder::new(file);
            let reader = BufReader::new(decoder);
            let tile_entities = bincode::deserialize_from(reader)?;
            Ok(tile_entities)
        } else {
            Ok(Vec::new())
        }
    }

    fn save_world_generator_data(&self, world_dir: &Path, world_generator: &WorldGenerator) -> Result<(), SaveLoadError> {
        let generator_data = world_generator.get_seed();
        let generator_path = world_dir.join("generator.json");
        
        let file = File::create(generator_path)?;
        let writer = BufWriter::new(file);
        serde_json::to_writer(writer, &generator_data)?;
        
        Ok(())
    }

    fn load_world_generator_data(&self, world_dir: &Path) -> Result<u64, SaveLoadError> {
        let generator_path = world_dir.join("generator.json");
        
        if generator_path.exists() {
            let file = File::open(generator_path)?;
            let reader = BufReader::new(file);
            let seed = serde_json::from_reader(reader)?;
            Ok(seed)
        } else {
            Ok(0) // Default seed
        }
    }

    fn apply_chunk_save(chunk_save: &ChunkSave, chunk_manager: &mut ChunkManager) -> Result<(), SaveLoadError> {
        // Get chunk key
        let chunk_key = chunk_save.chunk_key;
        
        // Clear existing particles by key
        chunk_manager.clear_chunk(chunk_key);
        
        // Apply saved particles
        for particle_save in &chunk_save.particles {
            let particle = Particle::new(
                particle_save.local_x,
                particle_save.local_y,
                particle_save.material_type,
                Some(particle_save.temp),
            );
            
            // Calculate world position directly
            let world_x = chunk_key.0 as i64 * crate::chunk::CHUNK_SIZE as i64 + particle_save.local_x as i64;
            let world_y = chunk_key.1 as i64 * crate::chunk::CHUNK_SIZE as i64 + particle_save.local_y as i64;
            
            chunk_manager.set_particle(world_x, world_y, particle);
        }

        Ok(())
    }

    fn apply_ecs_snapshot(snapshot: &ECSSnapshot, ecs: &mut ECS) -> Result<(), SaveLoadError> {
        use crate::ecs::{Position, Velocity, Health, Player};
        
        for entity_snapshot in &snapshot.entities {
            let entity_id = ecs.create_entity();
            
            if let Some((x, y, z)) = entity_snapshot.position {
                ecs.add_position(entity_id, Position { x, y, z });
            }
            
            if let Some((dx, dy, dz)) = entity_snapshot.velocity {
                ecs.add_velocity(entity_id, Velocity { dx, dy, dz });
            }
            
            if let Some((current, max, regen)) = entity_snapshot.health {
                ecs.add_health(entity_id, Health {
                    current,
                    max,
                    regeneration_rate: regen,
                });
            }
            
            if let Some(player_data) = &entity_snapshot.player_data {
                ecs.add_player(entity_id, Player {
                    name: player_data.name.clone(),
                    level: player_data.level,
                    experience: player_data.experience,
                    connection_id: player_data.connection_id,
                });
            }
        }

        Ok(())
    }

    /// List all available worlds
    pub fn list_worlds(&self) -> Result<Vec<String>, SaveLoadError> {
        let mut worlds = Vec::new();
        
        if self.save_directory.exists() {
            for entry in fs::read_dir(&self.save_directory)? {
                let entry = entry?;
                let path = entry.path();
                
                if path.is_dir() {
                    if let Some(world_name) = path.file_name().and_then(|s| s.to_str()) {
                        // Check if it's a valid world directory
                        let metadata_path = path.join("metadata.json");
                        if metadata_path.exists() {
                            worlds.push(world_name.to_string());
                        }
                    }
                }
            }
        }

        Ok(worlds)
    }

    /// Delete a world
    pub fn delete_world(&self, world_name: &str) -> Result<(), SaveLoadError> {
        let world_dir = self.save_directory.join(world_name);
        
        if world_dir.exists() {
            fs::remove_dir_all(world_dir)?;
            Ok(())
        } else {
            Err(SaveLoadError::WorldNotFound(world_name.to_string()))
        }
    }

    /// Get world metadata without loading the entire world
    pub fn get_world_metadata(&self, world_name: &str) -> Result<WorldMetadata, SaveLoadError> {
        let world_dir = self.save_directory.join(world_name);
        
        if !world_dir.exists() {
            return Err(SaveLoadError::WorldNotFound(world_name.to_string()));
        }

        self.load_metadata(&world_dir)
    }
}

impl ChunkSave {
    fn from_chunk(chunk_key: ChunkKey, chunk: &crate::chunk::Chunk) -> Self {
        let mut particles = Vec::new();
        let mut biome_data = HashMap::new();
        
        for y in 0..crate::chunk::CHUNK_SIZE {
            for x in 0..crate::chunk::CHUNK_SIZE {
                if let Some(particle) = chunk.get_particle(x, y) {
                    particles.push(ParticleSave {
                        local_x: x,
                        local_y: y,
                        material_type: particle.material_type,
                        temp: particle.temp,
                        life: particle.life,
                        burning: particle.burning,
                        time_in_state: particle.time_in_state,
                    });
                }
                
                // For now, assume Plains biome - this could be expanded
                biome_data.insert((x, y), BiomeType::Plains);
            }
        }

        Self {
            chunk_key,
            particles,
            biome_data,
            last_updated: chrono::Utc::now().to_rfc3339(),
            generation_stage: 100, // Fully generated
        }
    }
}

impl ECSSnapshot {
    fn from_ecs(ecs: &ECS) -> Self {
        let mut entities = Vec::new();
        
        for &entity_id in ecs.get_active_entities() {
            let position = ecs.get_position(entity_id).map(|p| (p.x, p.y, p.z));
            let velocity = ecs.get_velocity(entity_id).map(|v| (v.dx, v.dy, v.dz));
            let health = ecs.get_health(entity_id).map(|h| (h.current, h.max, h.regeneration_rate));
            let player_data = ecs.get_player(entity_id).map(|p| PlayerData {
                name: p.name.clone(),
                level: p.level,
                experience: p.experience,
                connection_id: p.connection_id,
            });

            entities.push(EntitySnapshot {
                entity_id,
                position,
                velocity,
                health,
                player_data,
            });
        }

        Self {
            entities,
            next_entity_id: ecs.get_active_entities().len() as u32,
        }
    }
}


#[derive(Debug)]
pub enum SaveLoadError {
    IoError(std::io::Error),
    SerializationError(serde_json::Error),
    BinarySerializationError(bincode::Error),
    WorldNotFound(String),
    CorruptedData(String),
}

impl From<std::io::Error> for SaveLoadError {
    fn from(error: std::io::Error) -> Self {
        SaveLoadError::IoError(error)
    }
}

impl From<serde_json::Error> for SaveLoadError {
    fn from(error: serde_json::Error) -> Self {
        SaveLoadError::SerializationError(error)
    }
}

impl From<bincode::Error> for SaveLoadError {
    fn from(error: bincode::Error) -> Self {
        SaveLoadError::BinarySerializationError(error)
    }
}

impl std::fmt::Display for SaveLoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SaveLoadError::IoError(e) => write!(f, "IO error: {}", e),
            SaveLoadError::SerializationError(e) => write!(f, "Serialization error: {}", e),
            SaveLoadError::BinarySerializationError(e) => write!(f, "Binary serialization error: {}", e),
            SaveLoadError::WorldNotFound(name) => write!(f, "World '{}' not found", name),
            SaveLoadError::CorruptedData(msg) => write!(f, "Corrupted data: {}", msg),
        }
    }
}

impl std::error::Error for SaveLoadError {}

#[cfg(test)]
mod tests {
    use super::*;
    // use tempfile::TempDir; // TODO: Add tempfile dependency for testing

    #[test]
    #[ignore] // TODO: Enable when tempfile dependency is added
    fn test_save_load_manager_creation() {
        // let temp_dir = TempDir::new().unwrap();
        // let manager = SaveLoadManager::new(temp_dir.path()).unwrap();
        // assert!(manager.save_directory.exists());
    }

    #[test]
    fn test_world_metadata_serialization() {
        let metadata = WorldMetadata {
            world_name: "TestWorld".to_string(),
            version: "1.0.0".to_string(),
            created_at: "2024-01-01T00:00:00Z".to_string(),
            last_played: "2024-01-01T01:00:00Z".to_string(),
            player_count: 1,
            total_playtime: 3600.0,
            world_size: (-10, 10),
            spawn_point: (0.0, 0.0),
            difficulty: Difficulty::Normal,
            game_mode: GameMode::Survival,
            seed: 12345,
        };

        let json = serde_json::to_string(&metadata).unwrap();
        let deserialized: WorldMetadata = serde_json::from_str(&json).unwrap();
        
        assert_eq!(metadata.world_name, deserialized.world_name);
        assert_eq!(metadata.seed, deserialized.seed);
    }

    #[test]
    #[ignore] // TODO: Enable when tempfile dependency is added
    fn test_list_worlds() {
        // let temp_dir = TempDir::new().unwrap();
        // let manager = SaveLoadManager::new(temp_dir.path()).unwrap();
        // 
        // // Initially empty
        // let worlds = manager.list_worlds().unwrap();
        // assert!(worlds.is_empty());
    }
}