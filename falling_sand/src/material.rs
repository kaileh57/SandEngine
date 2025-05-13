use std::fmt::Debug;

/// Types of physical behaviors a material can have
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicsType {
    Air,    // Gas-like behavior, minimal interactions
    Sand,   // Falls and piles up
    Solid,  // Does not move, acts as boundary
}

/// Color representation for rendering
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const TRANSPARENT: Color = Color { r: 0, g: 0, b: 0, a: 0 };
    pub const BLACK: Color = Color { r: 0, g: 0, b: 0, a: 255 };
    
    pub fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }
    
    pub fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }
}

/// Represents a material type in the simulation
#[derive(Debug, Clone)]
pub struct Material {
    pub name: String,
    pub default_color: Color,
    pub physics: PhysicsType,
}

/// An instance of a material at a specific location
#[derive(Clone, Debug)]
pub struct MaterialInstance {
    pub material_id: usize,
    pub physics: PhysicsType,
    pub color: Color,
}

impl MaterialInstance {
    /// Create a new instance of air
    pub fn air() -> Self {
        Self {
            material_id: 0, // Assuming 0 is reserved for air
            physics: PhysicsType::Air,
            color: Color::TRANSPARENT,
        }
    }
    
    /// Check if this material should be simulated
    pub fn is_active(&self) -> bool {
        match self.physics {
            PhysicsType::Air => false, // Air doesn't move on its own
            PhysicsType::Sand => true, // Sand can fall
            PhysicsType::Solid => false, // Solids don't move
        }
    }
}

/// Registry of all available materials
pub struct MaterialRegistry {
    materials: Vec<Material>,
}

impl MaterialRegistry {
    pub fn new() -> Self {
        let mut registry = Self { materials: Vec::new() };
        
        // Register default materials
        registry.register(Material {
            name: "Air".to_string(),
            default_color: Color::TRANSPARENT,
            physics: PhysicsType::Air,
        });
        
        registry.register(Material {
            name: "Sand".to_string(),
            default_color: Color::rgb(194, 178, 128), // Sand color
            physics: PhysicsType::Sand,
        });
        
        registry.register(Material {
            name: "Stone".to_string(),
            default_color: Color::rgb(128, 128, 128), // Grey color
            physics: PhysicsType::Solid,
        });
        
        registry
    }
    
    pub fn register(&mut self, material: Material) -> usize {
        let id = self.materials.len();
        self.materials.push(material);
        id
    }
    
    pub fn get(&self, id: usize) -> Option<&Material> {
        self.materials.get(id)
    }
    
    pub fn create_instance(&self, id: usize) -> Option<MaterialInstance> {
        self.get(id).map(|material| {
            MaterialInstance {
                material_id: id,
                physics: material.physics,
                color: material.default_color,
            }
        })
    }
} 