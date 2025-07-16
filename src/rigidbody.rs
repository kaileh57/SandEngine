use crate::materials::{MaterialType, get_material_properties};
use crate::chunk::{ChunkManager, ChunkKey, CHUNK_SIZE};
use nalgebra::{Point2, Vector2, UnitComplex};
use rapier2d::prelude::*;
use std::collections::{HashMap, VecDeque};

const PHYSICS_SCALE: f32 = 0.1; // Scale factor for physics world (pixels to meters)

#[derive(Debug, Clone)]
pub struct RigidBodyData {
    pub pixels: Vec<(i32, i32, MaterialType)>, // World coordinates and material
    pub center_of_mass: (f32, f32),
    pub mass: f32,
    pub handle: Option<RigidBodyHandle>,
    pub collider_handle: Option<ColliderHandle>,
    pub chunk_origin: ChunkKey,
}

impl RigidBodyData {
    pub fn new(pixels: Vec<(i32, i32, MaterialType)>, chunk_origin: ChunkKey) -> Self {
        // Calculate center of mass
        let mut total_mass = 0.0;
        let mut com_x = 0.0;
        let mut com_y = 0.0;
        
        for (x, y, material) in &pixels {
            let props = get_material_properties(*material);
            let mass = props.density;
            total_mass += mass;
            com_x += *x as f32 * mass;
            com_y += *y as f32 * mass;
        }
        
        if total_mass > 0.0 {
            com_x /= total_mass;
            com_y /= total_mass;
        }
        
        Self {
            pixels,
            center_of_mass: (com_x, com_y),
            mass: total_mass,
            handle: None,
            collider_handle: None,
            chunk_origin,
        }
    }
}

pub struct RigidBodyManager {
    pub physics_world: RigidBodySet,
    pub collider_set: ColliderSet,
    pub integration_parameters: IntegrationParameters,
    pub physics_pipeline: PhysicsPipeline,
    pub island_manager: IslandManager,
    pub broad_phase: BroadPhase,
    pub narrow_phase: NarrowPhase,
    pub impulse_joint_set: ImpulseJointSet,
    pub multibody_joint_set: MultibodyJointSet,
    pub ccd_solver: CCDSolver,
    pub physics_hooks: (),
    pub event_handler: (),
    pub rigid_bodies: HashMap<RigidBodyHandle, RigidBodyData>,
    pub gravity: Vector2<f32>,
}

impl RigidBodyManager {
    pub fn new() -> Self {
        let mut integration_parameters = IntegrationParameters::default();
        integration_parameters.dt = 1.0 / 60.0; // 60 FPS
        
        Self {
            physics_world: RigidBodySet::new(),
            collider_set: ColliderSet::new(),
            integration_parameters,
            physics_pipeline: PhysicsPipeline::new(),
            island_manager: IslandManager::new(),
            broad_phase: BroadPhase::new(),
            narrow_phase: NarrowPhase::new(),
            impulse_joint_set: ImpulseJointSet::new(),
            multibody_joint_set: MultibodyJointSet::new(),
            ccd_solver: CCDSolver::new(),
            physics_hooks: (),
            event_handler: (),
            rigid_bodies: HashMap::new(),
            gravity: Vector2::new(0.0, 9.81 * 10.0), // Stronger gravity for falling sand
        }
    }

    pub fn step(&mut self) {
        self.physics_pipeline.step(
            &self.gravity,
            &self.integration_parameters,
            &mut self.island_manager,
            &mut self.broad_phase,
            &mut self.narrow_phase,
            &mut self.physics_world,
            &mut self.collider_set,
            &mut self.impulse_joint_set,
            &mut self.multibody_joint_set,
            &mut self.ccd_solver,
            None,
            &self.physics_hooks,
            &self.event_handler,
        );
    }

    pub fn create_rigid_body_from_pixels(
        &mut self,
        pixels: Vec<(i32, i32, MaterialType)>,
        chunk_origin: ChunkKey,
    ) -> Option<RigidBodyHandle> {
        if pixels.is_empty() {
            return None;
        }

        let mut body_data = RigidBodyData::new(pixels, chunk_origin);
        
        // Create rigid body at center of mass
        let rigid_body = RigidBodyBuilder::dynamic()
            .translation(Vector2::new(
                body_data.center_of_mass.0 * PHYSICS_SCALE,
                body_data.center_of_mass.1 * PHYSICS_SCALE,
            ))
            .build();
        
        let handle = self.physics_world.insert(rigid_body);
        body_data.handle = Some(handle);
        
        // Create collider from pixel data
        if let Some(collider_handle) = self.create_collider_from_pixels(&body_data, handle) {
            body_data.collider_handle = Some(collider_handle);
        }
        
        self.rigid_bodies.insert(handle, body_data);
        Some(handle)
    }

    fn create_collider_from_pixels(
        &mut self,
        body_data: &RigidBodyData,
        rigid_body_handle: RigidBodyHandle,
    ) -> Option<ColliderHandle> {
        // Convert pixels to relative coordinates from center of mass
        let mut relative_pixels = Vec::new();
        for (x, y, _) in &body_data.pixels {
            let rel_x = *x as f32 - body_data.center_of_mass.0;
            let rel_y = *y as f32 - body_data.center_of_mass.1;
            relative_pixels.push((rel_x, rel_y));
        }

        // Create convex hull from pixels
        if let Some(convex_hull) = self.create_convex_hull(&relative_pixels) {
            let collider = ColliderBuilder::convex_hull(&convex_hull)
                .unwrap_or_else(|| {
                    // Fallback to bounding box if convex hull fails
                    let (min_x, max_x) = relative_pixels.iter()
                        .map(|(x, _)| *x)
                        .fold((f32::INFINITY, f32::NEG_INFINITY), |(min, max), x| (min.min(x), max.max(x)));
                    let (min_y, max_y) = relative_pixels.iter()
                        .map(|(_, y)| *y)
                        .fold((f32::INFINITY, f32::NEG_INFINITY), |(min, max), y| (min.min(y), max.max(y)));
                    
                    ColliderBuilder::cuboid(
                        (max_x - min_x) * 0.5 * PHYSICS_SCALE,
                        (max_y - min_y) * 0.5 * PHYSICS_SCALE,
                    )
                })
                .density(body_data.mass / body_data.pixels.len() as f32)
                .build();

            Some(self.collider_set.insert_with_parent(
                collider,
                rigid_body_handle,
                &mut self.physics_world,
            ))
        } else {
            None
        }
    }

    fn create_convex_hull(&self, pixels: &[(f32, f32)]) -> Option<Vec<Point2<f32>>> {
        if pixels.len() < 3 {
            return None;
        }

        // Simple convex hull algorithm - gift wrapping
        let mut hull = Vec::new();
        
        // Find the leftmost point
        let mut leftmost = 0;
        for i in 1..pixels.len() {
            if pixels[i].0 < pixels[leftmost].0 {
                leftmost = i;
            }
        }
        
        let mut p = leftmost;
        loop {
            hull.push(Point2::new(pixels[p].0 * PHYSICS_SCALE, pixels[p].1 * PHYSICS_SCALE));
            
            let mut q = (p + 1) % pixels.len();
            for i in 0..pixels.len() {
                if self.orientation(&pixels[p], &pixels[i], &pixels[q]) == 2 {
                    q = i;
                }
            }
            
            p = q;
            if p == leftmost {
                break;
            }
        }
        
        if hull.len() >= 3 {
            Some(hull)
        } else {
            None
        }
    }

    fn orientation(&self, p: &(f32, f32), q: &(f32, f32), r: &(f32, f32)) -> i32 {
        let val = (q.1 - p.1) * (r.0 - q.0) - (q.0 - p.0) * (r.1 - q.1);
        if val == 0.0 {
            0 // Collinear
        } else if val > 0.0 {
            1 // Clockwise
        } else {
            2 // Counterclockwise
        }
    }

    pub fn update_rigid_body_positions(&mut self, chunk_manager: &mut ChunkManager) {
        let mut bodies_to_remove = Vec::new();
        
        for (handle, body_data) in &self.rigid_bodies {
            if let Some(rigid_body) = self.physics_world.get(*handle) {
                let position = rigid_body.translation();
                let rotation = rigid_body.rotation();
                
                // Check if body has moved significantly
                let world_pos = (position.x / PHYSICS_SCALE, position.y / PHYSICS_SCALE);
                let distance_moved = ((world_pos.0 - body_data.center_of_mass.0).powi(2) + 
                                     (world_pos.1 - body_data.center_of_mass.1).powi(2)).sqrt();
                
                if distance_moved > 0.5 || rotation.angle().abs() > 0.1 {
                    // Body has moved significantly, update particle positions
                    self.update_particle_positions_from_rigid_body(chunk_manager, body_data, position, rotation);
                    
                    // If body is moving very slowly, consider making it static
                    if rigid_body.linvel().magnitude() < 0.1 && rigid_body.angvel().abs() < 0.1 {
                        bodies_to_remove.push(*handle);
                    }
                }
            }
        }
        
        // Remove slow/stopped bodies to improve performance
        for handle in bodies_to_remove {
            self.remove_rigid_body(handle);
        }
    }

    fn update_particle_positions_from_rigid_body(
        &self,
        chunk_manager: &mut ChunkManager,
        body_data: &RigidBodyData,
        position: &Vector2<f32>,
        rotation: &UnitComplex<f32>,
    ) {
        // Clear old particle positions
        for (old_x, old_y, _) in &body_data.pixels {
            chunk_manager.remove_particle(*old_x as i64, *old_y as i64);
        }
        
        // Set new particle positions
        for (rel_x, rel_y, material) in &body_data.pixels {
            let relative_pos = Vector2::new(
                *rel_x as f32 - body_data.center_of_mass.0,
                *rel_y as f32 - body_data.center_of_mass.1,
            );
            
            let rotated_pos = rotation * relative_pos;
            let new_world_pos = Vector2::new(
                position.x / PHYSICS_SCALE + rotated_pos.x,
                position.y / PHYSICS_SCALE + rotated_pos.y,
            );
            
            chunk_manager.add_particle(
                new_world_pos.x as i64,
                new_world_pos.y as i64,
                *material,
                None,
            );
        }
    }

    pub fn remove_rigid_body(&mut self, handle: RigidBodyHandle) {
        if let Some(body_data) = self.rigid_bodies.remove(&handle) {
            // Remove collider
            if let Some(collider_handle) = body_data.collider_handle {
                self.collider_set.remove(
                    collider_handle,
                    &mut self.island_manager,
                    &mut self.physics_world,
                    false,
                );
            }
            
            // Remove rigid body
            self.physics_world.remove(
                handle,
                &mut self.island_manager,
                &mut self.collider_set,
                &mut self.impulse_joint_set,
                &mut self.multibody_joint_set,
                false,
            );
        }
    }

    pub fn clear(&mut self) {
        self.rigid_bodies.clear();
        // Create new instances to effectively "clear" them
        self.physics_world = RigidBodySet::new();
        self.collider_set = ColliderSet::new();
        self.impulse_joint_set = ImpulseJointSet::new();
        self.multibody_joint_set = MultibodyJointSet::new();
        self.island_manager = IslandManager::new();
        self.broad_phase = BroadPhase::new();
        self.narrow_phase = NarrowPhase::new();
    }

    pub fn rigid_body_count(&self) -> usize {
        self.rigid_bodies.len()
    }
}

impl Default for RigidBodyManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Analyzes chunks to find connected solid regions that should become rigid bodies
pub struct RigidBodyAnalyzer;

impl RigidBodyAnalyzer {
    pub fn find_rigid_body_candidates(
        chunk_manager: &ChunkManager,
        chunk_key: ChunkKey,
    ) -> Vec<Vec<(i32, i32, MaterialType)>> {
        let mut candidates = Vec::new();
        
        if let Some(chunk) = chunk_manager.get_chunk(chunk_key) {
            let mut visited = vec![vec![false; CHUNK_SIZE]; CHUNK_SIZE];
            
            for y in 0..CHUNK_SIZE {
                for x in 0..CHUNK_SIZE {
                    if !visited[y][x] {
                        if let Some(particle) = chunk.get_particle(x, y) {
                            if Self::is_solid_material(particle.material_type) {
                                let region = Self::flood_fill_solid_region(
                                    chunk_manager,
                                    chunk_key,
                                    x,
                                    y,
                                    &mut visited,
                                );
                                
                                // Only create rigid bodies for regions with enough mass
                                if region.len() >= 4 {
                                    candidates.push(region);
                                }
                            }
                        }
                    }
                }
            }
        }
        
        candidates
    }

    fn is_solid_material(material: MaterialType) -> bool {
        matches!(
            material,
            MaterialType::Stone | MaterialType::Wood | MaterialType::Glass | 
            MaterialType::Ice | MaterialType::Coal
        )
    }

    fn flood_fill_solid_region(
        chunk_manager: &ChunkManager,
        chunk_key: ChunkKey,
        start_x: usize,
        start_y: usize,
        visited: &mut Vec<Vec<bool>>,
    ) -> Vec<(i32, i32, MaterialType)> {
        let mut region = Vec::new();
        let mut queue = VecDeque::new();
        queue.push_back((start_x, start_y));
        
        if let Some(chunk) = chunk_manager.get_chunk(chunk_key) {
            while let Some((x, y)) = queue.pop_front() {
                if x >= CHUNK_SIZE || y >= CHUNK_SIZE || visited[y][x] {
                    continue;
                }
                
                if let Some(particle) = chunk.get_particle(x, y) {
                    if Self::is_solid_material(particle.material_type) {
                        visited[y][x] = true;
                        
                        let (world_x, world_y) = chunk.world_pos(x, y);
                        region.push((world_x as i32, world_y as i32, particle.material_type));
                        
                        // Check neighbors
                        for dx in -1..=1 {
                            for dy in -1..=1 {
                                if dx == 0 && dy == 0 {
                                    continue;
                                }
                                
                                let nx = x as i32 + dx;
                                let ny = y as i32 + dy;
                                
                                if nx >= 0 && nx < CHUNK_SIZE as i32 && ny >= 0 && ny < CHUNK_SIZE as i32 {
                                    queue.push_back((nx as usize, ny as usize));
                                }
                            }
                        }
                    }
                }
            }
        }
        
        region
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::particle::Particle;

    #[test]
    fn test_rigid_body_creation() {
        let mut manager = RigidBodyManager::new();
        
        let pixels = vec![
            (0, 0, MaterialType::Stone),
            (1, 0, MaterialType::Stone),
            (0, 1, MaterialType::Stone),
            (1, 1, MaterialType::Stone),
        ];
        
        let handle = manager.create_rigid_body_from_pixels(pixels, (0, 0));
        assert!(handle.is_some());
        
        let handle = handle.unwrap();
        assert!(manager.rigid_bodies.contains_key(&handle));
        assert_eq!(manager.rigid_body_count(), 1);
        
        manager.remove_rigid_body(handle);
        assert_eq!(manager.rigid_body_count(), 0);
    }

    #[test]
    fn test_rigid_body_analyzer() {
        let mut chunk_manager = ChunkManager::new();
        
        // Create a solid region
        for x in 10..14 {
            for y in 10..14 {
                chunk_manager.add_particle(x, y, MaterialType::Stone, None);
            }
        }
        
        let chunk_key = ChunkManager::world_to_chunk_pos(10, 10);
        let candidates = RigidBodyAnalyzer::find_rigid_body_candidates(&chunk_manager, chunk_key);
        
        assert!(!candidates.is_empty());
        assert!(candidates[0].len() >= 4);
    }

    #[test]
    fn test_physics_step() {
        let mut manager = RigidBodyManager::new();
        
        // Create a rigid body
        let pixels = vec![(0, 0, MaterialType::Stone)];
        let handle = manager.create_rigid_body_from_pixels(pixels, (0, 0));
        assert!(handle.is_some());
        
        // Step the physics
        manager.step();
        
        // Body should still exist
        assert_eq!(manager.rigid_body_count(), 1);
    }
}