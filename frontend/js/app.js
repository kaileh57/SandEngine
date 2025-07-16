// Main application initialization and coordination
class SandEngineApp {
    constructor() {
        this.initialized = false;
        this.setupGlobalVariables();
        this.init();
    }

    setupGlobalVariables() {
        // Global state variables
        window.isDrawing = false;
        window.paintingEnabled = true;
    }

    init() {
        // Initialize all managers
        this.initializeManagers();
        
        // Setup message handlers
        this.setupMessageHandlers();
        
        // Connect to server
        window.wsManager.connect();
        
        this.initialized = true;
        console.log('SandEngine client initialized');
    }

    initializeManagers() {
        // Initialize all manager classes
        window.wsManager = new WebSocketManager();
        window.materialManager = new MaterialManager();
        window.structureManager = new StructureManager();
        window.canvasManager = new CanvasManager();
        window.brushManager = new BrushManager();
        window.uiManager = new UIManager();
    }

    setupMessageHandlers() {
        // Register WebSocket message handlers
        window.wsManager.onMessage('materials', (message) => {
            window.materialManager.setMaterials(message.materials);
        });

        window.wsManager.onMessage('structures', (message) => {
            window.structureManager.setStructures(message.structures);
        });

        window.wsManager.onMessage('simulation_state', (message) => {
            window.canvasManager.handleSimulationState(message);
        });

        window.wsManager.onMessage('delta_update', (message) => {
            window.canvasManager.handleDeltaUpdate(message);
        });

        window.wsManager.onMessage('particle_info', (message) => {
            window.canvasManager.handleParticleInfo(message);
        });

        window.wsManager.onMessage('structure_placed', (message) => {
            if (message.success) {
                window.uiManager.showNotification(`Structure "${message.structure_name}" placed successfully!`, 'success');
            } else {
                window.uiManager.showNotification(`Failed to place structure: ${message.error}`, 'error');
            }
        });

        window.wsManager.onMessage('error', (message) => {
            window.uiManager.showNotification(`Error: ${message.message}`, 'error');
        });
    }
}

// Initialize the application when DOM is loaded
document.addEventListener('DOMContentLoaded', () => {
    new SandEngineApp();
});