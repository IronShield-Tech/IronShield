import { UIManager } from './ui_manager.js';
import { WorkerPoolManager } from './worker_pool_manager.js';
import { ApiClient } from './api_client.js';

// First, load the WebAssembly module and bindings
// async function loadWasmModule(retryCount = 0) { ... }

// Track performance metrics
const perfMetrics = {
    pageLoadTime: window.pageStartTime || performance.now(),
    challengeStartTime: performance.now(),
    wasmReady: false,
    wasmReadyTime: 0,
    wasmStatus: 'not started',
    wasmThreaded: false,
    solutionTime: 0,
    resourceTimings: []
};

// Collect resource timings
function collectResourceTimings() {
    if (performance && performance.getEntriesByType) {
        const resources = performance.getEntriesByType('resource');
        const wasmResources = resources.filter(r => r.name.includes('wasm'));
        
        // Log timing data for WASM resources
        console.log('[PERF] WASM Resource Timings:');
        wasmResources.forEach(resource => {
            console.log(`[PERF] Resource: ${resource.name.split('/').pop()}`);
            console.log(`[PERF]   - Start time: ${Math.round(resource.startTime)}ms`);
            console.log(`[PERF]   - Response start: ${Math.round(resource.responseStart)}ms`);
            console.log(`[PERF]   - Response end: ${Math.round(resource.responseEnd)}ms`);
            console.log(`[PERF]   - Duration: ${Math.round(resource.duration)}ms`);
            console.log(`[PERF]   - Size: ${resource.transferSize ? Math.round(resource.transferSize / 1024) + 'KB' : 'N/A'}`);
            console.log(`[PERF]   - Was preloaded: ${resource.initiatorType === 'link'}`);
            
            perfMetrics.resourceTimings.push({
                name: resource.name.split('/').pop(),
                startTime: Math.round(resource.startTime),
                responseStart: Math.round(resource.responseStart),
                responseEnd: Math.round(resource.responseEnd),
                duration: Math.round(resource.duration),
                size: resource.transferSize ? Math.round(resource.transferSize / 1024) : null,
                wasPreloaded: resource.initiatorType === 'link'
            });
        });
    }
}

async function solveChallenge() {
    const startTime = performance.now();
    console.log(`[PERF] Challenge solving started at ${Math.round(startTime)}ms since page load`);
    
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
            console.log(`[PERF] Starting worker pool at ${Math.round(performance.now() - startTime)}ms`);
            // Get the current URL's origin to ensure we're loading from the same origin
            const baseUrl = window.location.origin;
            // Pass the original worker script path as fallback with absolute URL - without /assets/ prefix
            const workerPool = new WorkerPoolManager(`${baseUrl}/pow_worker.js`);
            console.log("Using fallback worker path:", `${baseUrl}/pow_worker.js`);

            // Store challenge and difficulty in the worker pool for later use
            workerPool.currentChallenge = challenge;
            workerPool.currentDifficulty = difficulty;
            
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
            
            // Add a callback for WASM status updates
            workerPool.onWasmStatus = (isWasmAvailable, isThreaded) => {
                const wasmStatusTime = performance.now() - startTime;
                perfMetrics.wasmReady = isWasmAvailable;
                perfMetrics.wasmReadyTime = wasmStatusTime;
                perfMetrics.wasmStatus = isWasmAvailable ? 'initialized' : 'failed';
                perfMetrics.wasmThreaded = isThreaded;
                
                console.log(`[PERF] WASM status update at ${Math.round(wasmStatusTime)}ms: ${isWasmAvailable ? 'available' : 'unavailable'}, threaded: ${isThreaded}`);
                
                if (isWasmAvailable) {
                    uiManager.setStatus(`Using WebAssembly${isThreaded ? ' with multi-threading' : ''} for faster computation...`);
                } else {
                    uiManager.setStatus("Using JavaScript implementation (consider enabling WASM for better performance)...");
                }
            };

            // Set initial status before starting the pool
            uiManager.setStatus("Computing hash values (using all available CPU cores)...");
            uiManager.setProgress(25);

            // Get resource timings before solving
            collectResourceTimings();
            
            // Start solving using the worker pool
            const solveStartTime = performance.now();
            console.log(`[PERF] Starting worker pool solve at ${Math.round(solveStartTime - startTime)}ms`);
            solution = await workerPool.solve(challenge, difficulty, timeoutSeconds);
            
            // Calculate solving time
            const solveEndTime = performance.now();
            perfMetrics.solutionTime = solveEndTime - solveStartTime;
            console.log(`[PERF] Solution found after ${Math.round(perfMetrics.solutionTime)}ms`);
            
            // Display WASM usage in final status if it was used
            if (solution.usedWasm) {
                console.log(`[PERF] Solution found using WASM${solution.usedThreadedWasm ? ' with threads' : ''}`);
            }

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
        
        // Get final resource timings
        collectResourceTimings();
        
        // Log performance summary
        console.log(`[PERF] === Performance Summary ===`);
        console.log(`[PERF] Page load to challenge start: ${Math.round(startTime - perfMetrics.pageLoadTime)}ms`);
        console.log(`[PERF] WASM available: ${perfMetrics.wasmReady}, ready at: ${Math.round(perfMetrics.wasmReadyTime)}ms`);
        console.log(`[PERF] WASM threaded: ${perfMetrics.wasmThreaded}`);
        console.log(`[PERF] Solution time: ${Math.round(perfMetrics.solutionTime)}ms`);
        console.log(`[PERF] Total time from page load: ${Math.round(performance.now() - perfMetrics.pageLoadTime)}ms`);
        console.log(`[PERF] Total time from challenge start: ${Math.round(performance.now() - startTime)}ms`);
        
        // Send the solution back to the server using ApiClient
        try {
            // Submit solution but we don't need the response HTML anymore
            await apiClient.submitSolution(
                challenge,
                solution.nonce_str,
                timestamp,
                difficultyStr
            );

            // Instead of rendering the response, redirect the user
            console.log("Challenge verification successful, redirecting to skip.ironshield.cloud...");
            window.location.href = 'https://skip.ironshield.cloud';

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