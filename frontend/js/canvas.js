// Canvas rendering and interaction
class CanvasManager {
    constructor() {
        this.canvas = document.getElementById('simulationCanvas');
        this.ctx = this.canvas.getContext('2d', { alpha: false });
        this.simulationData = new Map();
        
        // Constants
        this.CELL_SIZE = 4;
        this.GRID_WIDTH = 200;
        this.GRID_HEIGHT = 150;
        this.WIDTH = this.GRID_WIDTH * this.CELL_SIZE;
        this.HEIGHT = this.GRID_HEIGHT * this.CELL_SIZE;
        
        this.setupCanvas();
        this.setupEventListeners();
    }

    setupCanvas() {
        this.canvas.width = this.WIDTH;
        this.canvas.height = this.HEIGHT;
        this.ctx.imageSmoothingEnabled = false;
    }

    setupEventListeners() {
        this.canvas.addEventListener('mousedown', (e) => this.handleMouseDown(e));
        this.canvas.addEventListener('mousemove', (e) => this.handleMouseMove(e));
        this.canvas.addEventListener('mouseup', (e) => this.handleMouseUp(e));
        this.canvas.addEventListener('mouseleave', () => this.handleMouseLeave());
        this.canvas.addEventListener('contextmenu', (e) => e.preventDefault());
        this.canvas.addEventListener('wheel', (e) => this.handleWheel(e));
    }

    handleMouseDown(e) {
        if (e.button === 0) { // Left click
            const pos = this.getMousePos(e);
            const gridX = Math.floor(pos.x / this.CELL_SIZE);
            const gridY = Math.floor(pos.y / this.CELL_SIZE);
            
            // Check if structure manager wants to handle this
            if (window.structureManager && window.structureManager.handleCanvasClick(gridX, gridY)) {
                return;
            }
            
            // Regular painting
            window.isDrawing = true;
            this.handleDraw(e);
        }
    }

    handleMouseMove(e) {
        if (window.isDrawing && window.paintingEnabled !== false) {
            this.handleDraw(e);
        } else {
            this.updateCoordsDisplay(e);
        }
    }

    handleMouseUp(e) {
        if (e.button === 0) {
            window.isDrawing = false;
        }
    }

    handleMouseLeave() {
        window.isDrawing = false;
        const coordsText = document.getElementById('coords-text');
        coordsText.textContent = 'Coords: (--, --)';
    }

    handleWheel(e) {
        e.preventDefault();
        if (window.brushManager) {
            window.brushManager.adjustBrushSize(e.deltaY);
        }
    }

    handleDraw(event) {
        const pos = this.getMousePos(event);
        const gridX = Math.floor(pos.x / this.CELL_SIZE);
        const gridY = Math.floor(pos.y / this.CELL_SIZE);
        
        if (!window.isDrawing) {
            this.requestParticleInfo(gridX, gridY);
            return;
        }
        
        // Send paint command
        if (window.wsManager && window.materialManager) {
            const material = window.materialManager.getCurrentMaterial();
            const brushSize = window.brushManager ? window.brushManager.getBrushSize() : 3;
            
            window.wsManager.send({
                type: 'paint',
                x: gridX,
                y: gridY,
                material: material.id,
                brush_size: brushSize
            });
        }
    }

    updateCoordsDisplay(event) {
        const pos = this.getMousePos(event);
        const gridX = Math.floor(pos.x / this.CELL_SIZE);
        const gridY = Math.floor(pos.y / this.CELL_SIZE);
        
        if (gridX >= 0 && gridX < this.GRID_WIDTH && gridY >= 0 && gridY < this.GRID_HEIGHT) {
            this.requestParticleInfo(gridX, gridY);
        }
    }

    requestParticleInfo(gridX, gridY) {
        if (window.wsManager) {
            window.wsManager.send({
                type: 'get_particle',
                x: gridX,
                y: gridY
            });
        }
    }

    getMousePos(event) {
        const rect = this.canvas.getBoundingClientRect();
        return {
            x: event.clientX - rect.left,
            y: event.clientY - rect.top
        };
    }

    handleSimulationState(state) {
        this.simulationData.clear();
        for (const [coords, particle] of Object.entries(state.particles)) {
            this.simulationData.set(coords, particle);
        }
        this.draw();
    }

    handleDeltaUpdate(delta) {
        // Apply removed particles
        for (const coords of delta.removed) {
            this.simulationData.delete(coords);
        }
        
        // Apply added/changed particles
        for (const [coords, particle] of Object.entries(delta.added)) {
            this.simulationData.set(coords, particle);
        }
        
        this.draw();
    }

    handleParticleInfo(info) {
        const coordsText = document.getElementById('coords-text');
        
        if (info.material && window.materialManager) {
            const material = window.materialManager.getMaterial(info.material);
            const materialName = material ? material.name : 'Unknown';
            const tempText = info.temp ? ` | ${info.temp.toFixed(1)}°C` : '';
            const lifeText = info.life ? ` | Life: ${info.life.toFixed(1)}s` : '';
            const burningText = info.burning ? ' (Burning!)' : '';
            
            let propertiesText = '';
            if (material) {
                const props = [];
                if (material.is_stationary) props.push('Stationary');
                if (material.is_rigid_solid) props.push('Rigid');
                if (material.is_liquid) props.push('Liquid');
                if (material.is_gas) props.push('Gas');
                if (material.is_powder) props.push('Powder');
                
                if (props.length > 0) {
                    propertiesText = ` [${props.join(', ')}]`;
                }
            }
            
            coordsText.innerHTML = `
                <div>Coords: (${info.x}, ${info.y})</div>
                <div>${materialName}${burningText}${propertiesText}</div>
                <div>Density: ${material ? material.density.toFixed(2) : 'N/A'}${tempText}${lifeText}</div>
            `;
        } else {
            coordsText.textContent = `Coords: (${info.x}, ${info.y}) | Empty`;
        }
    }

    draw() {
        // Clear canvas
        this.ctx.fillStyle = 'black';
        this.ctx.fillRect(0, 0, this.WIDTH, this.HEIGHT);
        
        // Draw particles
        for (const [coords, particle] of this.simulationData) {
            const [x, y] = coords.split(',').map(Number);
            const canvasX = x * this.CELL_SIZE;
            const canvasY = y * this.CELL_SIZE;
            
            this.ctx.fillStyle = `rgb(${particle.color.join(',')})`;
            this.ctx.fillRect(canvasX, canvasY, this.CELL_SIZE, this.CELL_SIZE);
        }
    }
}