# Sand Simulation Game

## Overview

This is a falling sand simulation game built in Rust. It allows users to place various materials onto a canvas and watch them interact based on physics and thermal properties.

## Features

*   **Variety of Materials:** Includes a diverse set of materials such as Sand, Water, Stone, Plant, Fire, Lava, Acid, Oil, Steam, and more, each with unique behaviors.
*   **Physics Simulation:** Implements gravity, density-based interactions (e.g., heavier particles sink below lighter ones), and particle displacement.
*   **Thermal Simulation:** Particles have temperatures, exchange heat with neighbors, and can undergo phase changes (e.g., Water to Steam, Sand to Glass).
*   **Interactive "Painting":** Users can select materials and "paint" them onto the simulation grid using a circular brush.

## Engine

The game is built upon the `sand_simulation_engine` Rust library, which provides the core simulation logic.

## How to Run

### Prerequisites

*   **Rust:** Ensure you have Rust installed. You can install it via [rustup](https://rustup.rs/).

### Steps

1.  **Clone the Repository (if applicable):**
    If you obtained this as part of a larger repository, ensure the entire project is cloned.
    ```bash
    # git clone <repository_url>
    # cd <repository_name>
    ```
2.  **Navigate to the Game Directory:**
    ```bash
    cd sand_game
    ```
3.  **Build and Run:**
    It is recommended to run in release mode for better performance.
    ```bash
    cargo run --release
    ```

## Controls

### Mouse

*   **Left Click + Drag:** Draw particles with the selected material and brush size.
*   **Mouse Wheel:** Adjust the brush size.

### Keyboard

*   **`1`**: Select SAND
*   **`2`**: Select WATER
*   **`3`**: Select STONE
*   **`4`**: Select ACID
*   **`5`**: Select GENERATOR
*   **`0`**: Select ERASER tool
*   **`Up Arrow` / `Equals (=)` / `NumpadAdd (+)`**: Increase brush size.
*   **`Down Arrow` / `Minus (-)` / `NumpadSubtract (-)`**: Decrease brush size.
*   **`C`**: Clear the entire simulation grid.

## Interface

*   **Window Title:** Dynamically displays the currently selected brush material, brush size, and the grid coordinates of the cell under the mouse cursor.
*   **Console Output:** When the mouse cursor is over the simulation area, detailed information about the particle under the cursor (material type, temperature, lifespan, burning status) is logged to the console.

## Future Enhancements (Optional)

*   Additional materials and more complex interactions.
*   A graphical user interface (GUI) for material selection and simulation controls.
*   Further performance optimizations for larger grid sizes.
*   Saving and loading simulation states.
