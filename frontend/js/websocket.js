// WebSocket connection management
class WebSocketManager {
    constructor() {
        this.socket = null;
        this.connectionStatus = document.getElementById('connection-status');
        this.messageHandlers = new Map();
    }

    connect() {
        const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
        const wsUrl = `${protocol}//${window.location.host}/ws`;
        
        this.socket = new WebSocket(wsUrl);
        
        this.socket.onopen = () => {
            console.log('Connected to sand engine server');
            this.connectionStatus.textContent = 'Status: Connected';
            this.connectionStatus.className = 'connected';
        };
        
        this.socket.onmessage = (event) => {
            try {
                const message = JSON.parse(event.data);
                this.handleMessage(message);
            } catch (error) {
                console.error('Failed to parse server message:', error);
            }
        };
        
        this.socket.onclose = () => {
            console.log('Disconnected from server');
            this.connectionStatus.textContent = 'Status: Disconnected';
            this.connectionStatus.className = 'disconnected';
            
            // Attempt to reconnect after 3 seconds
            setTimeout(() => this.connect(), 3000);
        };
        
        this.socket.onerror = (error) => {
            console.error('WebSocket error:', error);
            this.connectionStatus.textContent = 'Status: Error';
            this.connectionStatus.className = 'disconnected';
        };
    }

    handleMessage(message) {
        const handler = this.messageHandlers.get(message.type);
        if (handler) {
            handler(message);
        } else {
            console.warn('Unknown message type:', message.type);
        }
    }

    onMessage(type, handler) {
        this.messageHandlers.set(type, handler);
    }

    send(message) {
        if (this.socket && this.socket.readyState === WebSocket.OPEN) {
            this.socket.send(JSON.stringify(message));
        }
    }

    isConnected() {
        return this.socket && this.socket.readyState === WebSocket.OPEN;
    }
}