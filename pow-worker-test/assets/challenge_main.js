import { UIManager } from './ui_manager.js';
import { WorkerPoolManager } from './worker_pool_manager.js';
import { ApiClient } from './api_client.js';

// First, load the WebAssembly module and bindings
// async function loadWasmModule(retryCount = 0) { ... }

async function solveChallenge() {
    const uiManager = new UIManager();
    const apiClient = new ApiClient();

    // Get all parameters from meta tags
    const difficultyMeta = document.querySelector('meta[name="x-ironshield-difficulty"]');
    const timestampMeta = document.querySelector('meta[name="x-ironshield-timestamp"]');
    const challengeMeta = document.querySelector('meta[name="x-ironshield-challenge"]');
    
    // Error handling for missing meta tags
    if (!difficultyMeta || !timestampMeta || !challengeMeta) {
        uiManager.showError("Error: Security parameters missing. Please refresh the page.");
        return;
    }
    
    // Parse the values
    const difficultyStr = difficultyMeta.getAttribute('content');
    const difficulty = parseInt(difficultyStr, 10);
    
    if (isNaN(difficulty) || difficulty <= 0) {
        uiManager.showError("Error: Invalid security parameters. Please refresh the page.");
        return;
    }
    
    const timestamp = timestampMeta.getAttribute('content');
    const challenge = challengeMeta.getAttribute('content');
    
    if (!timestamp || !challenge) {
        uiManager.showError("Error: Missing security parameters. Please refresh the page.");
        return;
    }
    
    console.log(`Using challenge: ${challenge}, difficulty: ${difficulty}, timestamp: ${timestamp}`);
    
    // Update status - Initial preparation
    uiManager.setStatus("Preparing challenge solver...");
    uiManager.setProgress(5); // Small initial progress

    // Define timeout (in seconds)
    const timeoutSeconds = 30;
    let progressInterval = null; // Keep track of the interval for the UI

    try {
        let solution;
        
        // Check if Web Workers are supported
        if (window.Worker) {
            console.log("Using Web Workers for background PoW calculation");
            const workerPool = new WorkerPoolManager('/pow_worker.js'); // Pass worker script path

            // Set up progress reporting from the worker pool to the UI manager
            let lastUIUpdate = 0;
            workerPool.onProgress = (totalAttempts, hashRate) => {
                 // Update UI with total attempts and rate, but limit updates to avoid UI thrashing
                 const now = Date.now();
                 if (now - lastUIUpdate > 100) { // Update UI at most every 100ms
                     uiManager.setStatus(`Computing hash values... (${totalAttempts.toLocaleString()} total attempts, ${hashRate.toLocaleString()} hashes/sec)`);
                     lastUIUpdate = now;
                 }
                 // We can also update the progress bar more smoothly here if desired
                 // e.g., uiManager.setProgress(Math.min(95, some_estimated_completion_percentage));
            };

            // Set initial status before starting the pool
            uiManager.setStatus("Computing hash values (using all available CPU cores)...");
            uiManager.setProgress(25);

            // Start solving using the worker pool
            solution = await workerPool.solve(challenge, difficulty, timeoutSeconds);

        } else {
            // Fallback for browsers without Web Workers
            console.warn("Web Workers not supported. Falling back to main thread calculation (may block UI).");
            uiManager.setStatus("Solving challenge on main thread (may cause temporary freeze)...");
            uiManager.setProgress(10);
            
            // Perform the calculation directly in the main thread (not recommended)
            // Ensure the standalone calculatePowSolution exists or is imported/defined here
            // For now, let's just log a warning and stop, as the worker logic is primary
            // solution = await calculatePowSolution_mainThread(challenge, difficulty); // You would need this function
            console.error("Main thread fallback calculation not implemented yet.");
            uiManager.showError("Error: Web Workers required for this security check.");
            throw new Error("Web Workers are not supported or enabled in this browser.");
        }
        
        // Clear the simple progress update interval if it was running
        if (progressInterval) clearInterval(progressInterval);
        
        // Update UI using uiManager - Challenge Solved!
        uiManager.setStatus(`Challenge solved! (Nonce: ${solution.nonce_str}, Hash: ${solution.hash_prefix}...)`);
        uiManager.setProgress(100);
        
        // Send the solution back to the server using ApiClient
        try {
            const responseHtml = await apiClient.submitSolution(
                challenge,
                solution.nonce_str,
                timestamp,
                difficultyStr
            );
            // Render the response from the server
            document.open();
            document.write(responseHtml);
            document.close();
        } catch (error) {
            // ApiClient throws an error if fetch fails or status is not ok
            console.error("Error submitting solution via ApiClient:", error);
            uiManager.showError(error.message || "Error submitting verification. Please check console.");
        }
        
    } catch (error) {
        console.error("Error solving challenge:", error);
        if (progressInterval) clearInterval(progressInterval); // Clear interval on error too
        // Use uiManager for error reporting
        uiManager.showError("Error during security check: " + error.message);
    }
}

// Start the challenge solving process automatically
document.addEventListener('DOMContentLoaded', solveChallenge); 