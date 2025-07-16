// Structure placement system
class StructureManager {
    constructor() {
        this.structures = new Map();
        this.structurePanel = null;
        this.isStructureMode = false;
        this.selectedStructure = null;
        this.previewCanvas = null;
        this.previewCtx = null;
        this.setupStructurePanel();
    }

    setupStructurePanel() {
        // Create structure panel
        this.structurePanel = document.createElement('div');
        this.structurePanel.id = 'structure-panel';
        this.structurePanel.className = 'panel-section';
        this.structurePanel.innerHTML = `
            <h3>Structures</h3>
            <button id="toggle-structure-mode" class="mode-button">Structure Mode: OFF</button>
            <div id="structure-list"></div>
            <div id="structure-preview"></div>
        `;

        // Add to right panel
        const rightPanel = document.getElementById('right-panel');
        rightPanel.appendChild(this.structurePanel);

        // Setup event listeners
        this.setupEventListeners();
    }

    setupEventListeners() {
        const toggleButton = document.getElementById('toggle-structure-mode');
        toggleButton.addEventListener('click', () => {
            this.toggleStructureMode();
        });

        // Listen for escape key to exit structure mode
        document.addEventListener('keydown', (e) => {
            if (e.key === 'Escape' && this.isStructureMode) {
                this.exitStructureMode();
            }
        });
    }

    setStructures(structuresArray) {
        this.structures.clear();
        for (const structure of structuresArray) {
            this.structures.set(structure.name, structure);
        }
        this.populateStructureList();
    }

    populateStructureList() {
        const structureList = document.getElementById('structure-list');
        structureList.innerHTML = '';

        if (this.structures.size === 0) {
            structureList.innerHTML = '<p class="no-structures">No structures available</p>';
            return;
        }

        for (const [name, structure] of this.structures) {
            const button = this.createStructureButton(structure);
            structureList.appendChild(button);
        }
    }

    createStructureButton(structure) {
        const button = document.createElement('button');
        button.className = 'structure-button';
        button.textContent = structure.name;
        button.dataset.structureName = structure.name;
        
        // Add structure info
        const infoDiv = document.createElement('div');
        infoDiv.className = 'structure-info';
        infoDiv.innerHTML = `
            <small>
                ${structure.width}×${structure.height}<br>
                ${structure.particle_count} particles<br>
                ${structure.tile_entity_count} entities
            </small>
        `;
        button.appendChild(infoDiv);

        button.addEventListener('click', () => {
            this.selectStructure(structure);
        });

        return button;
    }

    selectStructure(structure) {
        this.selectedStructure = structure;
        
        // Update UI
        document.querySelectorAll('.structure-button').forEach(btn => {
            btn.classList.toggle('selected', btn.dataset.structureName === structure.name);
        });

        // Show preview
        this.showStructurePreview(structure);
        
        // Update status
        this.updateStructureModeStatus();
    }

    showStructurePreview(structure) {
        const previewDiv = document.getElementById('structure-preview');
        previewDiv.innerHTML = `
            <div class="preview-header">
                <h4>${structure.name}</h4>
                <p>Size: ${structure.width} × ${structure.height}</p>
                <p>Particles: ${structure.particle_count}</p>
                <p>Entities: ${structure.tile_entity_count}</p>
            </div>
            <div class="preview-instructions">
                <p>Click on canvas to place structure</p>
                <p>Press ESC to cancel</p>
            </div>
        `;
    }

    toggleStructureMode() {
        if (this.isStructureMode) {
            this.exitStructureMode();
        } else {
            this.enterStructureMode();
        }
    }

    enterStructureMode() {
        this.isStructureMode = true;
        this.updateStructureModeStatus();
        
        // Change canvas cursor
        const canvas = document.getElementById('simulationCanvas');
        canvas.style.cursor = 'crosshair';
        
        // Show structure panel
        this.structurePanel.classList.add('active');
        
        // Disable regular painting
        window.paintingEnabled = false;
    }

    exitStructureMode() {
        this.isStructureMode = false;
        this.selectedStructure = null;
        this.updateStructureModeStatus();
        
        // Reset canvas cursor
        const canvas = document.getElementById('simulationCanvas');
        canvas.style.cursor = 'crosshair';
        
        // Hide structure panel active state
        this.structurePanel.classList.remove('active');
        
        // Re-enable regular painting
        window.paintingEnabled = true;
        
        // Clear selection
        document.querySelectorAll('.structure-button').forEach(btn => {
            btn.classList.remove('selected');
        });
        
        // Clear preview
        document.getElementById('structure-preview').innerHTML = '';
    }

    updateStructureModeStatus() {
        const toggleButton = document.getElementById('toggle-structure-mode');
        const materialText = document.getElementById('material-text');
        
        if (this.isStructureMode) {
            toggleButton.textContent = 'Structure Mode: ON';
            toggleButton.classList.add('active');
            
            if (this.selectedStructure) {
                materialText.textContent = `Structure: ${this.selectedStructure.name}`;
                materialText.style.color = '#ff9800';
            } else {
                materialText.textContent = 'Select a structure to place';
                materialText.style.color = '#ff9800';
            }
        } else {
            toggleButton.textContent = 'Structure Mode: OFF';
            toggleButton.classList.remove('active');
            
            // Reset material text
            if (window.materialManager) {
                window.materialManager.updateUIText();
            }
            materialText.style.color = '#4CAF50';
        }
    }

    handleCanvasClick(gridX, gridY) {
        if (!this.isStructureMode || !this.selectedStructure) {
            return false;
        }

        // Send structure placement command
        if (window.wsManager) {
            window.wsManager.send({
                type: 'place_structure',
                structure_name: this.selectedStructure.name,
                x: gridX,
                y: gridY
            });
        }

        return true; // Handled
    }

    isInStructureMode() {
        return this.isStructureMode;
    }

    getSelectedStructure() {
        return this.selectedStructure;
    }
}