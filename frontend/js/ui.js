// UI management and controls
class UIManager {
    constructor() {
        this.clearButton = document.getElementById('clear-button');
        this.setupEventListeners();
    }

    setupEventListeners() {
        // Clear button
        this.clearButton.addEventListener('click', () => {
            this.clearSimulation();
        });

        // Keyboard shortcuts
        window.addEventListener('keydown', (e) => {
            this.handleKeyPress(e);
        });
    }

    handleKeyPress(e) {
        switch (e.key.toLowerCase()) {
            case 'c':
                this.clearSimulation();
                break;
            case 'escape':
                if (window.structureManager && window.structureManager.isInStructureMode()) {
                    window.structureManager.exitStructureMode();
                }
                break;
            case 's':
                if (window.structureManager) {
                    window.structureManager.toggleStructureMode();
                }
                break;
            case '1':
            case '2':
            case '3':
            case '4':
            case '5':
            case '6':
            case '7':
            case '8':
            case '9':
                this.selectMaterialByNumber(parseInt(e.key));
                break;
        }
    }

    selectMaterialByNumber(number) {
        if (!window.materialManager) return;
        
        const buttons = document.querySelectorAll('#palette button');
        if (buttons.length >= number) {
            const button = buttons[number - 1];
            const materialId = parseInt(button.dataset.materialId);
            window.materialManager.setCurrentMaterial(materialId);
        }
    }

    clearSimulation() {
        if (window.wsManager) {
            window.wsManager.send({ type: 'clear' });
            console.log('Grid cleared');
        }
    }

    showNotification(message, type = 'info') {
        const notification = document.createElement('div');
        notification.className = `notification ${type}`;
        notification.textContent = message;
        
        document.body.appendChild(notification);
        
        // Auto-remove after 3 seconds
        setTimeout(() => {
            if (notification.parentNode) {
                notification.parentNode.removeChild(notification);
            }
        }, 3000);
    }

    updateConnectionStatus(connected) {
        const status = document.getElementById('connection-status');
        if (connected) {
            status.textContent = 'Status: Connected';
            status.className = 'connected';
        } else {
            status.textContent = 'Status: Disconnected';
            status.className = 'disconnected';
        }
    }
}