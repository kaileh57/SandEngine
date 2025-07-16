use std::collections::HashMap;
use std::marker::PhantomData;
use serde::{Deserialize, Serialize};

pub type EntityId = u32;

/// Entity Component System implementation based on the reference codebase
#[derive(Debug, Default)]
pub struct ECS {
    next_entity_id: EntityId,
    active_entities: Vec<EntityId>,
    freed_entities: Vec<EntityId>,
    
    // Component storage - each component type gets its own Vec<Option<T>>
    positions: Vec<Option<Position>>,
    velocities: Vec<Option<Velocity>>,
    healths: Vec<Option<Health>>,
    inventories: Vec<Option<Inventory>>,
    players: Vec<Option<Player>>,
    tile_entities: Vec<Option<TileEntityComponent>>,
}

/// Core component types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Velocity {
    pub dx: f64,
    pub dy: f64,
    pub dz: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Health {
    pub current: f32,
    pub max: f32,
    pub regeneration_rate: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Inventory {
    pub items: HashMap<String, u32>,
    pub max_capacity: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    pub name: String,
    pub level: u32,
    pub experience: u64,
    pub connection_id: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TileEntityComponent {
    pub tile_entity_type: String,
    pub data: HashMap<String, String>, // Generic key-value storage
}

impl ECS {
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new entity and return its ID
    pub fn create_entity(&mut self) -> EntityId {
        let entity_id = if let Some(freed_id) = self.freed_entities.pop() {
            freed_id
        } else {
            let id = self.next_entity_id;
            self.next_entity_id += 1;
            id
        };

        self.active_entities.push(entity_id);
        
        // Ensure all component vectors are large enough
        self.ensure_capacity(entity_id);
        
        entity_id
    }

    /// Remove an entity and all its components
    pub fn remove_entity(&mut self, entity_id: EntityId) -> bool {
        if let Some(pos) = self.active_entities.iter().position(|&id| id == entity_id) {
            self.active_entities.remove(pos);
            self.freed_entities.push(entity_id);
            
            // Clear all components for this entity
            if let Some(slot) = self.positions.get_mut(entity_id as usize) {
                *slot = None;
            }
            if let Some(slot) = self.velocities.get_mut(entity_id as usize) {
                *slot = None;
            }
            if let Some(slot) = self.healths.get_mut(entity_id as usize) {
                *slot = None;
            }
            if let Some(slot) = self.inventories.get_mut(entity_id as usize) {
                *slot = None;
            }
            if let Some(slot) = self.players.get_mut(entity_id as usize) {
                *slot = None;
            }
            if let Some(slot) = self.tile_entities.get_mut(entity_id as usize) {
                *slot = None;
            }
            
            true
        } else {
            false
        }
    }

    /// Check if an entity exists
    pub fn entity_exists(&self, entity_id: EntityId) -> bool {
        self.active_entities.contains(&entity_id)
    }

    /// Get all active entity IDs
    pub fn get_active_entities(&self) -> &[EntityId] {
        &self.active_entities
    }

    fn ensure_capacity(&mut self, entity_id: EntityId) {
        let required_size = (entity_id as usize) + 1;
        
        if self.positions.len() < required_size {
            self.positions.resize(required_size, None);
        }
        if self.velocities.len() < required_size {
            self.velocities.resize(required_size, None);
        }
        if self.healths.len() < required_size {
            self.healths.resize(required_size, None);
        }
        if self.inventories.len() < required_size {
            self.inventories.resize(required_size, None);
        }
        if self.players.len() < required_size {
            self.players.resize(required_size, None);
        }
        if self.tile_entities.len() < required_size {
            self.tile_entities.resize(required_size, None);
        }
    }

    // Component accessors - Position
    pub fn add_position(&mut self, entity_id: EntityId, position: Position) -> bool {
        if !self.entity_exists(entity_id) {
            return false;
        }
        self.ensure_capacity(entity_id);
        self.positions[entity_id as usize] = Some(position);
        true
    }

    pub fn get_position(&self, entity_id: EntityId) -> Option<&Position> {
        self.positions.get(entity_id as usize)?.as_ref()
    }

    pub fn get_position_mut(&mut self, entity_id: EntityId) -> Option<&mut Position> {
        self.positions.get_mut(entity_id as usize)?.as_mut()
    }

    pub fn remove_position(&mut self, entity_id: EntityId) -> Option<Position> {
        if let Some(slot) = self.positions.get_mut(entity_id as usize) {
            slot.take()
        } else {
            None
        }
    }

    // Component accessors - Velocity
    pub fn add_velocity(&mut self, entity_id: EntityId, velocity: Velocity) -> bool {
        if !self.entity_exists(entity_id) {
            return false;
        }
        self.ensure_capacity(entity_id);
        self.velocities[entity_id as usize] = Some(velocity);
        true
    }

    pub fn get_velocity(&self, entity_id: EntityId) -> Option<&Velocity> {
        self.velocities.get(entity_id as usize)?.as_ref()
    }

    pub fn get_velocity_mut(&mut self, entity_id: EntityId) -> Option<&mut Velocity> {
        self.velocities.get_mut(entity_id as usize)?.as_mut()
    }

    pub fn remove_velocity(&mut self, entity_id: EntityId) -> Option<Velocity> {
        if let Some(slot) = self.velocities.get_mut(entity_id as usize) {
            slot.take()
        } else {
            None
        }
    }

    // Component accessors - Health
    pub fn add_health(&mut self, entity_id: EntityId, health: Health) -> bool {
        if !self.entity_exists(entity_id) {
            return false;
        }
        self.ensure_capacity(entity_id);
        self.healths[entity_id as usize] = Some(health);
        true
    }

    pub fn get_health(&self, entity_id: EntityId) -> Option<&Health> {
        self.healths.get(entity_id as usize)?.as_ref()
    }

    pub fn get_health_mut(&mut self, entity_id: EntityId) -> Option<&mut Health> {
        self.healths.get_mut(entity_id as usize)?.as_mut()
    }

    // Component accessors - Player
    pub fn add_player(&mut self, entity_id: EntityId, player: Player) -> bool {
        if !self.entity_exists(entity_id) {
            return false;
        }
        self.ensure_capacity(entity_id);
        self.players[entity_id as usize] = Some(player);
        true
    }

    pub fn get_player(&self, entity_id: EntityId) -> Option<&Player> {
        self.players.get(entity_id as usize)?.as_ref()
    }

    pub fn get_player_mut(&mut self, entity_id: EntityId) -> Option<&mut Player> {
        self.players.get_mut(entity_id as usize)?.as_mut()
    }

    // Component accessors - TileEntity
    pub fn add_tile_entity(&mut self, entity_id: EntityId, tile_entity: TileEntityComponent) -> bool {
        if !self.entity_exists(entity_id) {
            return false;
        }
        self.ensure_capacity(entity_id);
        self.tile_entities[entity_id as usize] = Some(tile_entity);
        true
    }

    pub fn get_tile_entity(&self, entity_id: EntityId) -> Option<&TileEntityComponent> {
        self.tile_entities.get(entity_id as usize)?.as_ref()
    }

    pub fn get_tile_entity_mut(&mut self, entity_id: EntityId) -> Option<&mut TileEntityComponent> {
        self.tile_entities.get_mut(entity_id as usize)?.as_mut()
    }

    /// System iteration - get entities with position and velocity
    pub fn iter_position_velocity(&self) -> impl Iterator<Item = (EntityId, &Position, &Velocity)> {
        self.active_entities.iter().filter_map(move |&entity_id| {
            let position = self.get_position(entity_id)?;
            let velocity = self.get_velocity(entity_id)?;
            Some((entity_id, position, velocity))
        })
    }

    /// System iteration - get entities with position and velocity (mutable)
    pub fn iter_position_velocity_mut(&mut self) -> Vec<(EntityId, Position, Velocity)> {
        let mut results = Vec::new();
        for &entity_id in &self.active_entities.clone() {
            if let (Some(position), Some(velocity)) = (
                self.get_position(entity_id).cloned(),
                self.get_velocity(entity_id).cloned()
            ) {
                results.push((entity_id, position, velocity));
            }
        }
        results
    }

    /// System iteration - get all players
    pub fn iter_players(&self) -> impl Iterator<Item = (EntityId, &Player)> {
        self.active_entities.iter().filter_map(move |&entity_id| {
            let player = self.get_player(entity_id)?;
            Some((entity_id, player))
        })
    }

    /// Clear all entities and components
    pub fn clear(&mut self) {
        self.active_entities.clear();
        self.freed_entities.clear();
        self.next_entity_id = 0;
        
        self.positions.clear();
        self.velocities.clear();
        self.healths.clear();
        self.inventories.clear();
        self.players.clear();
        self.tile_entities.clear();
    }

    /// Get entity count
    pub fn entity_count(&self) -> usize {
        self.active_entities.len()
    }
}

/// Physics system for updating entity positions based on velocity
pub fn physics_system(ecs: &mut ECS, delta_time: f64) {
    let entities_with_movement = ecs.iter_position_velocity_mut();
    
    for (entity_id, mut position, velocity) in entities_with_movement {
        position.x += velocity.dx * delta_time;
        position.y += velocity.dy * delta_time;
        position.z += velocity.dz * delta_time;
        
        // Update the position in the ECS
        ecs.add_position(entity_id, position);
    }
}

/// Health regeneration system
pub fn health_regen_system(ecs: &mut ECS, delta_time: f64) {
    let active_entities = ecs.get_active_entities().to_vec();
    
    for entity_id in active_entities {
        if let Some(health) = ecs.get_health_mut(entity_id) {
            if health.current < health.max {
                health.current += health.regeneration_rate * delta_time as f32;
                health.current = health.current.min(health.max);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ecs_basic_operations() {
        let mut ecs = ECS::new();
        
        // Create entity
        let entity = ecs.create_entity();
        assert_eq!(entity, 0);
        assert!(ecs.entity_exists(entity));
        
        // Add components
        let position = Position { x: 10.0, y: 20.0, z: 0.0 };
        assert!(ecs.add_position(entity, position));
        
        let velocity = Velocity { dx: 1.0, dy: -1.0, dz: 0.0 };
        assert!(ecs.add_velocity(entity, velocity));
        
        // Retrieve components
        assert!(ecs.get_position(entity).is_some());
        assert!(ecs.get_velocity(entity).is_some());
        
        // Remove entity
        assert!(ecs.remove_entity(entity));
        assert!(!ecs.entity_exists(entity));
        assert!(ecs.get_position(entity).is_none());
    }

    #[test]
    fn test_physics_system() {
        let mut ecs = ECS::new();
        
        let entity = ecs.create_entity();
        ecs.add_position(entity, Position { x: 0.0, y: 0.0, z: 0.0 });
        ecs.add_velocity(entity, Velocity { dx: 10.0, dy: 5.0, dz: 0.0 });
        
        physics_system(&mut ecs, 1.0);
        
        let position = ecs.get_position(entity).unwrap();
        assert_eq!(position.x, 10.0);
        assert_eq!(position.y, 5.0);
    }

    #[test]
    fn test_player_creation() {
        let mut ecs = ECS::new();
        
        let player_entity = ecs.create_entity();
        let player = Player {
            name: "TestPlayer".to_string(),
            level: 1,
            experience: 0,
            connection_id: Some(1),
        };
        
        assert!(ecs.add_player(player_entity, player));
        
        let retrieved_player = ecs.get_player(player_entity).unwrap();
        assert_eq!(retrieved_player.name, "TestPlayer");
        assert_eq!(retrieved_player.level, 1);
    }
}