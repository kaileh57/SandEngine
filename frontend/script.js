// Canvas and UI elements
const canvas = document.getElementById('simulationCanvas');
const ctx = canvas.getContext('2d', { alpha: false });
const uiMaterialText = document.getElementById('material-text');
const uiCoordsText = document.getElementById('coords-text');
const connectionStatus = document.getElementById('connection-status');
const paletteDiv = document.getElementById('palette');
const clearButton = document.getElementById('clear-button');

// Constants
const CELL_SIZE = 4;
const GRID_WIDTH = 200;
const GRID_HEIGHT = 150;
const WIDTH = GRID_WIDTH * CELL_SIZE;
const HEIGHT = GRID_HEIGHT * CELL_SIZE;

// Setup canvas
canvas.width = WIDTH;
canvas.height = HEIGHT;
ctx.imageSmoothingEnabled = false;

// State
let socket = null;
let isDrawing = false;
let currentMaterial = 'Sand';
let currentMaterialType = 1; // Sand
let brushSize = 3;
let materials = new Map();
let simulationData = new Map();

// WebSocket connection
function connectWebSocket() {
    const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    const wsUrl = `${protocol}//${window.location.host}/ws`;
    
    socket = new WebSocket(wsUrl);
    
    socket.onopen = function() {
        console.log('Connected to sand engine server');
        connectionStatus.textContent = 'Status: Connected';
        connectionStatus.className = 'connected';
    };
    
    socket.onmessage = function(event) {
        try {
            const message = JSON.parse(event.data);
            handleServerMessage(message);
        } catch (error) {
            console.error('Failed to parse server message:', error);
        }
    };
    
    socket.onclose = function() {
        console.log('Disconnected from server');
        connectionStatus.textContent = 'Status: Disconnected';
        connectionStatus.className = 'disconnected';
        
        // Attempt to reconnect after 3 seconds
        setTimeout(connectWebSocket, 3000);
    };
    
    socket.onerror = function(error) {
        console.error('WebSocket error:', error);
        connectionStatus.textContent = 'Status: Error';
        connectionStatus.className = 'disconnected';
    };
}

function handleServerMessage(message) {
    switch (message.type) {
        case 'materials':
            handleMaterialsMessage(message.materials);
            break;
        case 'simulation_state':
            handleSimulationState(message);
            break;
        case 'delta_update':
            handleDeltaUpdate(message);
            break;
        case 'particle_info':
            handleParticleInfo(message);
            break;
    }
}

function handleMaterialsMessage(materialsArray) {
    materials.clear();
    for (const material of materialsArray) {
        materials.set(material.id, material);
    }
    populatePalette();
    updateUIText();
}

function handleSimulationState(state) {
    simulationData.clear();
    for (const [coords, particle] of Object.entries(state.particles)) {
        simulationData.set(coords, particle);
    }
    draw();
}

function handleDeltaUpdate(delta) {
    // Apply removed particles
    for (const coords of delta.removed) {
        simulationData.delete(coords);
    }
    
    // Apply added/changed particles
    for (const [coords, particle] of Object.entries(delta.added)) {
        simulationData.set(coords, particle);
    }
    
    draw();
}

function handleParticleInfo(info) {
    if (info.material) {
        const material = materials.get(info.material);
        const materialName = material ? material.name : 'Unknown';
        const tempText = info.temp ? ` | ${info.temp.toFixed(1)}°C` : '';
        const lifeText = info.life ? ` | Life: ${info.life.toFixed(1)}s` : '';
        const burningText = info.burning ? ' (Burning!)' : '';
        uiCoordsText.textContent = `Coords: (${info.x}, ${info.y}) | ${materialName}${burningText}${tempText}${lifeText}`;
    } else {
        uiCoordsText.textContent = `Coords: (${info.x}, ${info.y}) | Empty`;
    }
}

function sendMessage(message) {
    if (socket && socket.readyState === WebSocket.OPEN) {
        socket.send(JSON.stringify(message));
    }
}

function populatePalette() {
    paletteDiv.innerHTML = '';
    
    // Add eraser first
    const eraserMaterial = materials.get(99); // Eraser
    if (eraserMaterial) {
        const button = createMaterialButton(eraserMaterial, true);
        paletteDiv.appendChild(button);
    }
    
    // Add other materials (excluding Empty and Eraser)
    const sortedMaterials = Array.from(materials.values())
        .filter(m => m.id !== 0 && m.id !== 99) // Exclude Empty and Eraser
        .sort((a, b) => a.name.localeCompare(b.name));
    
    for (const material of sortedMaterials) {
        const button = createMaterialButton(material, false);
        paletteDiv.appendChild(button);
    }
}

function createMaterialButton(material, isEraser) {
    const button = document.createElement('button');
    button.textContent = material.name;
    button.dataset.materialId = material.id;
    button.style.backgroundColor = `rgb(${material.color.join(',')})`;
    
    // Calculate text color based on background brightness
    const brightness = (material.color[0] * 299 + material.color[1] * 587 + material.color[2] * 114) / 1000;
    button.style.color = brightness < 128 ? 'white' : '#111';
    
    if (isEraser) {
        button.classList.add('eraser');
    }
    
    button.title = `Select ${material.name}`;
    button.addEventListener('click', () => {
        currentMaterial = material.name;
        currentMaterialType = material.id;
        updateUIText();
    });
    
    return button;
}

function updateUIText() {
    uiMaterialText.textContent = `Brush: ${currentMaterial} (Size: ${brushSize})`;
    
    // Update button selection
    document.querySelectorAll('#palette button').forEach(button => {
        button.classList.toggle('selected', parseInt(button.dataset.materialId) === currentMaterialType);
    });
}

function draw() {
    // Clear canvas
    ctx.fillStyle = 'black';
    ctx.fillRect(0, 0, WIDTH, HEIGHT);
    
    // Draw particles
    for (const [coords, particle] of simulationData) {
        const [x, y] = coords.split(',').map(Number);
        const canvasX = x * CELL_SIZE;
        const canvasY = y * CELL_SIZE;
        
        ctx.fillStyle = `rgb(${particle.color.join(',')})`;
        ctx.fillRect(canvasX, canvasY, CELL_SIZE, CELL_SIZE);
    }
}

function getMousePos(canvas, evt) {
    const rect = canvas.getBoundingClientRect();
    return {
        x: evt.clientX - rect.left,
        y: evt.clientY - rect.top
    };
}

function handleDraw(event) {
    const pos = getMousePos(canvas, event);
    const gridX = Math.floor(pos.x / CELL_SIZE);
    const gridY = Math.floor(pos.y / CELL_SIZE);
    
    // Request particle info for display
    sendMessage({
        type: 'get_particle',
        x: gridX,
        y: gridY
    });
    
    if (!isDrawing) return;
    
    // Send paint command
    sendMessage({
        type: 'paint',
        x: gridX,
        y: gridY,
        material: currentMaterialType,
        brush_size: brushSize
    });
}

// Event listeners
canvas.addEventListener('mousedown', (e) => {
    if (e.button === 0) {
        isDrawing = true;
        handleDraw(e);
    }
});

canvas.addEventListener('mousemove', (e) => {
    if (isDrawing) {
        handleDraw(e);
    } else {
        const pos = getMousePos(canvas, e);
        const gridX = Math.floor(pos.x / CELL_SIZE);
        const gridY = Math.floor(pos.y / CELL_SIZE);
        
        if (gridX >= 0 && gridX < GRID_WIDTH && gridY >= 0 && gridY < GRID_HEIGHT) {
            sendMessage({
                type: 'get_particle',
                x: gridX,
                y: gridY
            });
        }
    }
});

canvas.addEventListener('mouseup', (e) => {
    if (e.button === 0) {
        isDrawing = false;
    }
});

canvas.addEventListener('mouseleave', () => {
    isDrawing = false;
    uiCoordsText.textContent = 'Coords: (--, --)';
});

canvas.addEventListener('contextmenu', (e) => e.preventDefault());

canvas.addEventListener('wheel', (e) => {
    e.preventDefault();
    if (e.deltaY < 0) {
        brushSize = Math.min(20, brushSize + 1);
    } else {
        brushSize = Math.max(0, brushSize - 1);
    }
    updateUIText();
});

window.addEventListener('keydown', (e) => {
    if (e.key === 'c' || e.key === 'C') {
        sendMessage({ type: 'clear' });
        console.log('Grid cleared by keypress');
    }
});

clearButton.addEventListener('click', () => {
    sendMessage({ type: 'clear' });
    console.log('Grid cleared by button');
});

// Initialize
connectWebSocket();