// Brush management system
class BrushManager {
    constructor() {
        this.brushSize = 3;
        this.minBrushSize = 0;
        this.maxBrushSize = 20;
    }

    getBrushSize() {
        return this.brushSize;
    }

    setBrushSize(size) {
        this.brushSize = Math.max(this.minBrushSize, Math.min(this.maxBrushSize, size));
        this.updateUI();
    }

    adjustBrushSize(delta) {
        if (delta < 0) {
            this.setBrushSize(this.brushSize + 1);
        } else {
            this.setBrushSize(this.brushSize - 1);
        }
    }

    updateUI() {
        if (window.materialManager) {
            window.materialManager.updateUIText();
        }
    }
}