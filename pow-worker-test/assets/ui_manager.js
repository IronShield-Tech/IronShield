// Simple UI Manager to handle status and progress updates
export class UIManager {
    constructor(statusElementId = 'status', progressElementId = 'progress') {
        this.statusDiv = document.getElementById(statusElementId);
        this.progressBar = document.getElementById(progressElementId);

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
            this.statusDiv.textContent = text;
        }
        console.log(`UI Status: ${text}`); // Also log to console for debugging
    }

    /**
     * Sets the progress bar value.
     * @param {number} value The progress value (0-100).
     */
    setProgress(value) {
        if (this.progressBar) {
            this.progressBar.value = Math.max(0, Math.min(100, value));
        }
    }

    /**
     * Displays an error message and resets progress.
     * @param {string} message The error message to display.
     */
    showError(message) {
        this.setStatus(message);
        this.setProgress(0);
        console.error(`UI Error: ${message}`);
    }
} 