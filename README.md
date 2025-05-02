# Sand Engine

A fast and efficient sand particle simulation written in Rust.

## Overview

This project implements a simple falling sand simulation where particles follow basic physics rules:
- Sand falls downward with gravity
- When blocked, sand tries to move diagonally
- Random column processing creates natural flow patterns

## Controls

- **Left Mouse Button**: Click and drag to add sand
- **Mouse Wheel**: Adjust brush size
- **C Key**: Clear the simulation
- **Escape**: Exit the application

## Building and Running

Make sure you have Rust installed on your system. Then run:

```
cargo run --release
```

The `--release` flag ensures optimal performance.

## Implementation Details

- Optimized using a flat array for better cache locality
- Implements sand particles that exhibit natural falling behavior
- Uses pixels crate for fast rendering
- Uses winit for window management and user input

## License

MIT
