// Worker that uses WebAssembly for PoW calculations with a fallback to JS implementation
// This is used as a Web Worker to perform PoW calculations in a separate thread
const workerScope = typeof self !== 'undefined' ? self : this;

// Flag to track if WASM is available and loaded
let wasmModule = null;
let useWasm = false;
let wasmThreaded = false;
let fallbackJsReady = false;

// Get base URL for imports
const baseUrl = self.location.origin;

// Implement a basic SHA-256 proof of work solution directly in this file
// This avoids the need to import from pow_worker.js and prevents redeclaration errors
async function calculatePowSolution(challenge, difficulty, workerId, startNonce, nonceStep) {
    const targetPrefix = "0".repeat(difficulty);
    let nonce = startNonce;
    let hash = "";
    let attempts = 0;
    let lastReportedAttempts = 0;
    
    while (true) {
        // Create the data to hash: challenge:nonce
        const dataToHash = challenge + ":" + nonce;
        
        // Calculate SHA-256 hash using the Web Crypto API
        const encoder = new TextEncoder();
        const data = encoder.encode(dataToHash);
        
        // Just do a single hash
        const hashBuffer = await crypto.subtle.digest('SHA-256', data);
        
        // Convert to hex string
        const hashArray = Array.from(new Uint8Array(hashBuffer));
        hash = hashArray.map(b => b.toString(16).padStart(2, '0')).join('');
        
        // Check if the hash meets the difficulty requirement
        if (hash.startsWith(targetPrefix)) {
            // Found a solution!
            // Send final progress report before returning
            workerScope.postMessage({
                type: "finalProgress",
                attempts: attempts,
                workerId: workerId
            });
            
            return {
                nonce_str: nonce.toString(),
                hash: hash,
                hash_prefix: hash.substring(0, 10),
                attempts: attempts
            };
        }
        
        // Move to next nonce according to our step pattern (allows parallel execution)
        nonce += nonceStep;
        attempts++;
        
        // Yield to prevent blocking the thread completely
        if (attempts % 1000 === 0) {
            // Report actual attempts
            workerScope.postMessage({
                type: "progress",
                attempts: attempts - lastReportedAttempts, // Report only new attempts since last time
                totalAttempts: attempts,
                nonce: nonce,
                workerId: workerId
            });
            lastReportedAttempts = attempts;
            
            // Brief yield to allow message processing
            await new Promise(resolve => setTimeout(resolve, 0));
        }
        
        // Safety limit - each worker checks a different range
        if (attempts > 1000000) {
            throw new Error("Worker #" + workerId + " could not find solution within attempt limit");
        }
    }
}

// No need to try importing from elsewhere - we have the implementation directly in this file
fallbackJsReady = true;
console.log("Using built-in JS fallback implementation");

// Initialize worker, try to load WASM module
workerScope.onmessage = async function(e) {
    try {
        // First message should be initialization
        if (e.data.type === 'init') {
            console.log(`Worker #${e.data.workerId} initializing...`);
            try {
                // Try to load the WASM module
                await loadWasmModule();
                // Success! WASM is ready
                console.log(`Worker #${e.data.workerId} loaded WASM module successfully`);
                
                // Report back that initialization is complete
                workerScope.postMessage({
                    type: 'init_complete',
                    workerId: e.data.workerId,
                    useWasm: useWasm,
                    wasmThreaded: wasmThreaded
                });
            } catch (error) {
                console.warn(`Worker #${e.data.workerId} failed to load WASM:`, error);
                console.log(`Worker #${e.data.workerId} falling back to JS implementation`);
                // Report that we'll use JS fallback
                workerScope.postMessage({
                    type: 'init_complete',
                    workerId: e.data.workerId,
                    useWasm: false,
                    wasmThreaded: false,
                    error: error.message
                });
            }
            return;
        }
        
        // Handle the actual PoW challenge
        if (e.data.type === 'solve') {
            const { challenge, difficulty, workerId, startNonce, nonceStep } = e.data;
            console.log(`Worker #${workerId} received challenge, difficulty: ${difficulty}`);
            
            workerScope.postMessage({
                type: "hashingStarted",
                workerId
            });
            
            const startTime = performance.now();
            let solution;
            
            // Use WASM if available, otherwise use JS implementation
            if (useWasm) {
                console.log(`Worker #${workerId} using WASM implementation`);
                try {
                    if (wasmThreaded && wasmModule.are_threads_supported()) {
                        // Use the threaded version if supported
                        solution = await solveWithThreadedWasm(challenge, difficulty, workerId);
                    } else {
                        // Use the non-threaded version
                        solution = await solveWithWasm(challenge, difficulty, workerId);
                    }
                } catch (wasmError) {
                    console.warn(`Worker #${workerId} WASM execution failed:`, wasmError);
                    console.log(`Worker #${workerId} falling back to JS implementation`);
                    // Fall back to JS implementation if WASM fails
                    try {
                        solution = await calculatePowSolution(challenge, difficulty, workerId, startNonce, nonceStep);
                    } catch (jsError) {
                        console.error(`Worker #${workerId} JS fallback also failed:`, jsError);
                        throw new Error("All solution methods failed");
                    }
                }
            } else {
                console.log(`Worker #${workerId} using JS implementation`);
                // Use the JS implementation directly
                try {
                    solution = await calculatePowSolution(challenge, difficulty, workerId, startNonce, nonceStep);
                } catch (jsError) {
                    console.error(`Worker #${workerId} JS implementation failed:`, jsError);
                    throw new Error("JS implementation failed to solve challenge");
                }
            }
            
            const endTime = performance.now();
            console.log(`Worker #${workerId} found solution in ${(endTime - startTime).toFixed(2)}ms`);
            
            // Send the result back to the main thread
            workerScope.postMessage({
                type: "success",
                solution: solution,
                timeTaken: endTime - startTime,
                workerId: workerId,
                useWasm: useWasm,
                wasmThreaded: wasmThreaded && wasmModule?.are_threads_supported()
            });
        }
    } catch (error) {
        console.error(`Worker error:`, error);
        workerScope.postMessage({
            type: "error",
            message: error.message || "Unknown error",
            workerId: e.data.workerId
        });
    }
};

// Function to load the WebAssembly module
async function loadWasmModule() {
    try {
        // Try to import the WASM module using absolute path
        console.log(`Attempting to load WASM module from ${baseUrl}/ironshield_wasm.js`);
        
        // Support both potential paths
        let wasm;
        try {
            wasm = await import(`${baseUrl}/ironshield_wasm.js`);
            console.log("Successfully loaded WASM from direct path");
        } catch (e) {
            console.log("Failed to load WASM from direct path, trying with /assets/ prefix", e);
            wasm = await import(`${baseUrl}/assets/wasm/ironshield_wasm.js`);
            console.log("Successfully loaded WASM from /assets/wasm/ path");
        }
        
        wasmModule = wasm;
        
        // Initialize the WASM module
        console.log("Initializing WASM module");
        await wasm.default();
        console.log("WASM module initialized");
        
        // Check if threads are supported
        wasmThreaded = wasm.are_threads_supported();
        useWasm = true;
        
        // If threads are supported, initialize the thread pool
        if (wasmThreaded) {
            // Get the number of CPU cores to use (hardcoded to 4 for now)
            const numThreads = 4;
            // Initialize the thread pool
            await wasm.init_threads(numThreads);
            console.log(`Initialized WASM thread pool with ${numThreads} threads`);
        }
        
        return true;
    } catch (error) {
        console.warn('Failed to load WASM module:', error);
        useWasm = false;
        wasmThreaded = false;
        throw error;
    }
}

// Function to solve the PoW challenge using the threaded WASM implementation
async function solveWithThreadedWasm(challenge, difficulty, workerId) {
    try {
        // Number of threads to use (should match init_threads call)
        const numThreads = 4;
        
        // Use the parallel WASM implementation
        const result = wasmModule.solve_pow_challenge_parallel(challenge, difficulty, numThreads);
        
        // Convert the WASM result to a JS object
        return {
            nonce_str: result.nonce_str,
            hash: result.hash,
            hash_prefix: result.hash_prefix,
            attempts: 0 // We don't track attempts in WASM (could be added)
        };
    } catch (error) {
        console.error(`Worker #${workerId} threaded WASM error:`, error);
        throw error;
    }
}

// Function to solve the PoW challenge using the non-threaded WASM implementation
async function solveWithWasm(challenge, difficulty, workerId) {
    try {
        // Use the non-threaded WASM implementation
        const result = wasmModule.solve_pow_challenge(challenge, difficulty);
        
        // Convert the WASM result to a JS object
        return {
            nonce_str: result.nonce_str,
            hash: result.hash,
            hash_prefix: result.hash_prefix,
            attempts: 0 // We don't track attempts in WASM (could be added)
        };
    } catch (error) {
        console.error(`Worker #${workerId} WASM error:`, error);
        throw error;
    }
} 