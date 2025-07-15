# Sand Engine - Rust Backend

A high-performance particle physics simulation engine built in Rust with a web frontend, inspired by Noita's "Everything Falls" engine. This implementation focuses on realistic particle interactions, thermal dynamics, and chemical reactions.

## Features

### Core Simulation
- **22 Material Types**: Sand, Water, Fire, Lava, Steam, Oil, Acid, Coal, Gunpowder, Ice, Wood, Smoke, Toxic Gas, Slime, Gasoline, Plant, Stone, Glass, Generator, Fuse, Ash, and Eraser
- **Real-time Physics**: Density-based movement, viscosity, temperature propagation
- **Thermal Dynamics**: Melting, boiling, freezing, condensation with realistic temperature thresholds
- **Chemical Reactions**: Ignition, combustion, corrosion, explosions
- **Material Properties**: Each material has unique density, conductivity, flammability, and phase change temperatures

### Advanced Features
- **Temperature-based Color Rendering**: Visual feedback for heat with dynamic color changes
- **Life Cycles**: Timed materials like Fire, Steam, and Smoke with realistic lifespans
- **Plant Growth**: Organic spread mechanics with water dependency
- **Explosive Materials**: Gunpowder with radius-based damage and fire propagation
- **Corrosive Effects**: Acid dissolving materials with gas generation
- **Phase Changes**: Water ↔ Ice ↔ Steam transitions based on temperature
- **Realistic Movement**: Splash physics, powder piling, liquid flow with viscosity

### Technical Architecture
- **Rust Backend**: High-performance simulation engine running at 60 FPS
- **WebSocket Communication**: Real-time bidirectional data exchange
- **Web Frontend**: Canvas-based renderer with interactive painting tools
- **Modular Design**: Separate physics, materials, and rendering systems

## Quick Start

### Prerequisites
- Rust 1.70+ installed
- Modern web browser with WebSocket support (for web demo)

### Option 1: Web Demo with Interactive UI

1. **Clone the repository**:
   ```bash
   git clone <repository-url>
   cd SandEngine
   ```

2. **Run the web server**:
   ```bash
   cargo run --bin server
   ```

3. **Open your browser** and navigate to:
   ```
   http://localhost:3030
   ```

The simulation will start with an interactive canvas where you can paint different materials.

### Option 2: Game Engine Integration

For game engine integration, use the `PhysicsEngine` directly:

```rust
use sand_engine::{PhysicsEngine, MaterialType};

// Create physics engine
let mut engine = PhysicsEngine::new(200, 150);

// Game loop
loop {
    // Add particles based on player input
    engine.add_particle(mouse_x, mouse_y, MaterialType::Sand, None);
    
    // Update physics (call every frame)
    engine.update();
    
    // Get state for rendering
    let state = engine.get_state();
    render_particles(state);
}
```

3. **Run the example**:
   ```bash
   cargo run --example simple_engine
   ```

### Option 3: Standalone Physics Server

For headless simulation or external integration:

```bash
cargo run --bin physics_server
```

This runs a standalone physics simulation with performance logging.

## Usage

### Controls
- **Left Click + Drag**: Paint with selected material
- **Mouse Wheel**: Adjust brush size (0-20)
- **Hover**: View particle information (material, temperature, life)
- **C Key or Clear Button**: Clear the simulation
- **Material Palette**: Click to select different materials

### Material Interactions

#### Fire & Combustion
- **Ignition Sources**: Fire, Lava, burning Fuse
- **Flammable Materials**: Plant, Wood, Coal, Oil, Gasoline
- **Combustion Products**: Fire → Smoke, Wood → Ash
- **Heat Transfer**: High-temperature materials heat neighbors

#### Fluids & Flow
- **Liquids**: Water, Oil, Acid, Gasoline, Lava
- **Viscosity Effects**: Oil and Slime flow slower than Water
- **Pressure & Splash**: Solids falling into liquids create splash effects
- **Density Layering**: Lighter liquids float on heavier ones

#### Phase Changes
- **Melting**: Sand → Glass (1500°C), Ice → Water (1°C)
- **Boiling**: Water → Steam (100°C), Acid → Toxic Gas (200°C)
- **Freezing**: Water → Ice (0°C), Lava → Stone (1000°C)
- **Condensation**: Steam → Water (temperature and height dependent)

#### Chemical Reactions
- **Acid Corrosion**: Dissolves most materials, creates Toxic Gas
- **Explosions**: Gunpowder creates fire and pressure waves
- **Plant Growth**: Spreads near Water sources in suitable temperatures
- **Generators**: Immovable heat sources for experiments

## Architecture

### Core Library (Rust)
```
src/
├── lib.rs              # Library exports
├── engine.rs           # High-level PhysicsEngine API
├── materials.rs        # Material definitions and properties
├── particle.rs         # Particle struct and behavior
├── physics.rs          # Physics calculations and state changes
├── simulation.rs       # Low-level simulation grid management
└── bin/
    ├── server.rs       # WebSocket server and HTTP endpoints
    └── physics_server.rs # Standalone physics server
```

### Integration Options

#### 1. **PhysicsEngine** (Recommended for Games)
High-level API for game integration:
```rust
use sand_engine::PhysicsEngine;
let mut engine = PhysicsEngine::new(width, height);
engine.update(); // Call every frame
let state = engine.get_state(); // For rendering
```

#### 2. **Simulation** (Low-level Control)
Direct access to simulation internals:
```rust
use sand_engine::Simulation;
let mut sim = Simulation::new(width, height);
sim.update(delta_time);
```

#### 3. **WebSocket Server** (Remote Integration)
For web frontends or remote applications:
- Real-time WebSocket communication
- HTTP endpoints for static assets
- JSON message protocol

#### 4. **C FFI** (External Language Integration)
Export functions for integration with C/C++ engines:
```c
PhysicsServer* create_physics_server(int width, int height);
void update_physics(PhysicsServer* server, float delta_time);
```

### Frontend (Web Demo)
```
frontend/
├── index.html          # HTML structure
├── style.css           # Styling and layout
└── script.js           # WebSocket client and canvas rendering
```

### Key Components

- **PhysicsEngine**: High-level game integration API
- **Simulation**: Low-level particle grid and update loop
- **Physics**: Handles temperature, state changes, and interactions
- **Materials**: Defines properties for all 22 material types
- **Particle**: Individual particle state and rendering
- **Server**: WebSocket communication and web serving

## Performance

- **60 FPS**: Consistent frame rate with 200×150 grid (30,000 cells)
- **Real-time Physics**: Sub-millisecond particle updates
- **Efficient Communication**: Delta-compressed state updates
- **Memory Optimized**: Sparse particle storage, only active cells tracked

## Material Properties Reference

| Material  | Density | Temp Effects | Special Properties |
|-----------|---------|-------------|-------------------|
| Sand      | 5.0     | Melts→Glass | Powder physics |
| Water     | 3.0     | Boils→Steam, Freezes→Ice | Extinguishes fire |
| Fire      | -2.0    | Heat source | Limited lifespan, spreads |
| Lava      | 8.0     | Extreme heat | Ignites materials |
| Oil       | 2.0     | Flammable | High viscosity |
| Acid      | 3.5     | Corrosive | Dissolves materials |
| Gunpowder | 4.5     | Explosive | Chain reactions |
| Plant     | 0.1     | Grows | Requires water |

## Future Enhancements

This particle engine is designed to eventually support Noita-style "everything falls" mechanics:

- **Terrain Destruction**: Destructible world geometry
- **Fluid Pressure**: Realistic pressure-based fluid flow
- **Electrical Systems**: Conductive materials and electricity
- **Advanced Chemistry**: Complex multi-step reactions
- **Performance Scaling**: Support for larger world sizes
- **Save/Load**: Persistent world states
- **Multiplayer**: Shared simulation instances

## Contributing

The codebase is modular and extensible. To add new materials:

1. Add the material type to `MaterialType` enum in `materials.rs`
2. Define properties in `get_material_properties()`
3. Add any special behavior in `physics.rs`
4. Update frontend material list handling

## License

This project is open source. See LICENSE file for details.

## Inspiration

Inspired by:
- **Noita**: "Everything Falls" particle physics
- **Powder Toy**: Classic falling sand simulation
- **Sandspiel**: Modern web-based particle playground

Built with performance and extensibility in mind for future game development projects.