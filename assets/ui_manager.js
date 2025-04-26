// Simple UI Manager to handle status and progress updates
export class UIManager {
    constructor(statusElementId = 'status', progressElementId = 'progress') {
        this.statusDiv = document.getElementById(statusElementId);
        this.progressBar = document.getElementById(progressElementId);
        this.startTime = Date.now();
        this.statusHistory = [];

        if (!this.statusDiv || !this.progressBar) {
            console.error('UI elements (status or progress) not found!');
            // Fallback to console logging if elements are missing
            this.statusDiv = { textContent: '' }; // Dummy object
            this.progressBar = { value: 0 };    // Dummy object
        }
    }

    /**
     * Updates the status text displayed to the user.
     * @param {string} text The text to display.
     */
    setStatus(text) {
        if (this.statusDiv) {
            // this.statusDiv.textContent = text; // Commented out to keep static text
        }
        const timestamp = Date.now() - this.startTime;
        this.statusHistory.push({ time: timestamp, text });
        console.log(`[UI-STATUS] @${timestamp}ms: ${text}`);
    }

    /**
     * Sets the progress bar value.
     * @param {number} value The progress value (0-100).
     */
    setProgress(value) {
        if (this.progressBar) {
            this.progressBar.value = Math.max(0, Math.min(100, value));
        }
        
        if (value % 20 === 0) { // Only log at 0%, 20%, 40%, 60%, 80%, 100%
            const timestamp = Date.now() - this.startTime;
            console.log(`[UI-PROGRESS] @${timestamp}ms: ${value}%`);
        }
    }

    /**
     * Displays an error message and resets progress.
     * @param {string} message The error message to display.
     */
    showError(message) {
        this.setStatus(message);
        this.setProgress(0);
        const timestamp = Date.now() - this.startTime;
        console.error(`[UI-ERROR] @${timestamp}ms: ${message}`);
    }
} 