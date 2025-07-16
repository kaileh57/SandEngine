// Material management system
class MaterialManager {
    constructor() {
        this.materials = new Map();
        this.currentMaterial = 'Sand';
        this.currentMaterialType = 1;
        this.paletteDiv = document.getElementById('palette');
        this.uiMaterialText = document.getElementById('material-text');
    }

    setMaterials(materialsArray) {
        this.materials.clear();
        for (const material of materialsArray) {
            this.materials.set(material.id, material);
        }
        this.populatePalette();
        this.updateUIText();
    }

    getMaterial(id) {
        return this.materials.get(id);
    }

    getCurrentMaterial() {
        return {
            name: this.currentMaterial,
            id: this.currentMaterialType
        };
    }

    setCurrentMaterial(materialId) {
        const material = this.materials.get(materialId);
        if (material) {
            this.currentMaterial = material.name;
            this.currentMaterialType = material.id;
            this.updateUIText();
        }
    }

    populatePalette() {
        this.paletteDiv.innerHTML = '';
        
        // Add eraser first
        const eraserMaterial = this.materials.get(99); // Eraser
        if (eraserMaterial) {
            const button = this.createMaterialButton(eraserMaterial, true);
            this.paletteDiv.appendChild(button);
        }
        
        // Group materials by type
        const groupedMaterials = this.groupMaterials();
        
        // Add each group
        for (const [groupName, materials] of groupedMaterials) {
            if (materials.length > 0) {
                const groupLabel = document.createElement('div');
                groupLabel.className = 'material-group-label';
                groupLabel.textContent = groupName;
                this.paletteDiv.appendChild(groupLabel);
                
                for (const material of materials) {
                    const button = this.createMaterialButton(material, false);
                    this.paletteDiv.appendChild(button);
                }
            }
        }
    }

    groupMaterials() {
        const groups = new Map([
            ['Solids', []],
            ['Liquids', []],
            ['Gases', []],
            ['Powders', []],
            ['Special', []]
        ]);

        const sortedMaterials = Array.from(this.materials.values())
            .filter(m => m.id !== 0 && m.id !== 99) // Exclude Empty and Eraser
            .sort((a, b) => a.name.localeCompare(b.name));

        for (const material of sortedMaterials) {
            if (material.is_rigid_solid || material.is_stationary) {
                groups.get('Solids').push(material);
            } else if (material.is_liquid) {
                groups.get('Liquids').push(material);
            } else if (material.is_gas) {
                groups.get('Gases').push(material);
            } else if (material.is_powder) {
                groups.get('Powders').push(material);
            } else {
                groups.get('Special').push(material);
            }
        }

        return groups;
    }

    createMaterialButton(material, isEraser) {
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
        
        // Add material type indicators
        const indicators = [];
        if (material.is_stationary) indicators.push('S');
        if (material.is_liquid) indicators.push('L');
        if (material.is_gas) indicators.push('G');
        if (material.is_powder) indicators.push('P');
        
        if (indicators.length > 0) {
            button.classList.add('has-indicators');
            button.dataset.indicators = indicators.join('');
        }
        
        // Create tooltip with material properties
        const tooltip = this.createMaterialTooltip(material);
        button.title = tooltip;
        
        button.addEventListener('click', () => {
            this.setCurrentMaterial(material.id);
        });
        
        return button;
    }

    createMaterialTooltip(material) {
        const lines = [
            `${material.name}`,
            `Density: ${material.density.toFixed(2)}`,
        ];
        
        const types = [];
        if (material.is_stationary) types.push('Stationary');
        if (material.is_rigid_solid) types.push('Rigid Solid');
        if (material.is_liquid) types.push('Liquid');
        if (material.is_gas) types.push('Gas');
        if (material.is_powder) types.push('Powder');
        
        if (types.length > 0) {
            lines.push(`Type: ${types.join(', ')}`);
        }
        
        return lines.join('\\n');
    }

    updateUIText() {
        const brushSize = window.brushManager ? window.brushManager.getBrushSize() : 3;
        this.uiMaterialText.textContent = `Brush: ${this.currentMaterial} (Size: ${brushSize})`;
        
        // Update button selection
        document.querySelectorAll('#palette button').forEach(button => {
            button.classList.toggle('selected', parseInt(button.dataset.materialId) === this.currentMaterialType);
        });
    }
}