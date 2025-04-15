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
    // Add timing info
    const loadStart = performance.now();
    let streamingStartTime, fetchEndTime, compileStartTime, compileEndTime;
    let wasmInitStartTime, wasmInitEndTime;
    
    try {
        console.log(`[WASM-STREAM] Starting WASM module load at ${Math.round(loadStart)}ms since worker start`);
        
        // Determine the URLs for both WASM and JS files
        const jsUrl = `${baseUrl}/ironshield_wasm.js`;
        const wasmUrl = `${baseUrl}/ironshield_wasm_bg.wasm`;
        
        const jsUrlAlternative = `${baseUrl}/assets/wasm/ironshield_wasm.js`;
        const wasmUrlAlternative = `${baseUrl}/assets/wasm/ironshield_wasm_bg.wasm`;
        
        // First, try to fetch the JS binding file to determine which URL pattern works
        console.log(`[WASM-STREAM] Testing JS bindings path at ${jsUrl}`);
        
        let jsBindingUrl;
        let wasmBinaryUrl;
        
        try {
            // Check if the direct path is available by doing a HEAD request
            const headStartTime = performance.now();
            const testResponse = await fetch(jsUrl, { method: 'HEAD' });
            console.log(`[WASM-STREAM] HEAD request completed in ${Math.round(performance.now() - headStartTime)}ms`);
            
            if (testResponse.ok) {
                jsBindingUrl = jsUrl;
                wasmBinaryUrl = wasmUrl;
                console.log(`[WASM-STREAM] Using direct URLs for WASM files`);
            } else {
                throw new Error("Direct path not available");
            }
        } catch (e) {
            console.log(`[WASM-STREAM] Direct path not available: ${e.message}, trying alternative paths`);
            jsBindingUrl = jsUrlAlternative;
            wasmBinaryUrl = wasmUrlAlternative;
            console.log(`[WASM-STREAM] Using alternative URLs for WASM files`);
        }
        
        // Start loading the JS bindings
        const jsStartTime = performance.now();
        console.log(`[WASM-STREAM] Fetching JS bindings from ${jsBindingUrl}`);
        const jsPromise = import(jsBindingUrl);
        
        // Immediately start streaming the WASM binary in parallel
        streamingStartTime = performance.now();
        console.log(`[WASM-STREAM] Streaming WASM binary from ${wasmBinaryUrl} at ${Math.round(streamingStartTime - loadStart)}ms`);
        
        // Create a more detailed fetch with timing info
        const wasmFetch = fetch(wasmBinaryUrl)
            .then(response => {
                fetchEndTime = performance.now();
                console.log(`[WASM-STREAM] WASM fetch headers received after ${Math.round(fetchEndTime - streamingStartTime)}ms`);
                console.log(`[WASM-STREAM] Response type: ${response.type}, status: ${response.status}`);
                
                // Check for streaming-related headers
                const headers = {};
                response.headers.forEach((value, key) => {
                    headers[key] = value;
                    if (['content-type', 'content-length', 'content-encoding', 'accept-ranges'].includes(key.toLowerCase())) {
                        console.log(`[WASM-STREAM] ${key}: ${value}`);
                    }
                });
                
                return response;
            })
            .catch(err => {
                console.error(`[WASM-STREAM] WASM fetch failed: ${err.message}`);
                throw err;
            });
        
        // Wait for JS module to load
        console.log(`[WASM-STREAM] Waiting for JS bindings to load...`);
        const jsLoadStartTime = performance.now();
        const wasm = await jsPromise;
        const jsLoadEndTime = performance.now();
        console.log(`[WASM-STREAM] JS bindings loaded after ${Math.round(jsLoadEndTime - jsLoadStartTime)}ms`);
        
        wasmModule = wasm;
        
        // Now initialize the WASM module with streaming instantiation
        console.log(`[WASM-STREAM] Starting WASM module initialization...`);
        wasmInitStartTime = performance.now();
        
        // Call the default function, which should be able to use our streaming fetch
        // if the browser supports WASM streaming instantiation
        await wasm.default();
        
        wasmInitEndTime = performance.now();
        const initDuration = Math.round(wasmInitEndTime - wasmInitStartTime);
        console.log(`[WASM-STREAM] WASM module initialized in ${initDuration}ms`);
        
        // Check if threads are supported
        wasmThreaded = wasm.are_threads_supported();
        useWasm = true;
        console.log(`[WASM-STREAM] WASM threading support: ${wasmThreaded}`);
        
        // If threads are supported, initialize the thread pool
        if (wasmThreaded) {
            // Use navigator.hardwareConcurrency but cap at a reasonable number
            const maxThreads = 4;
            const numThreads = Math.min(navigator.hardwareConcurrency || 2, maxThreads);
            
            // Initialize the thread pool
            const threadStartTime = performance.now();
            console.log(`[WASM-STREAM] Initializing thread pool with ${numThreads} threads...`);
            await wasm.init_threads(numThreads);
            const threadEndTime = performance.now();
            console.log(`[WASM-STREAM] Thread pool initialized in ${Math.round(threadEndTime - threadStartTime)}ms`);
        }
        
        const totalTime = performance.now() - loadStart;
        console.log(`[WASM-STREAM] Total WASM load process completed in ${Math.round(totalTime)}ms`);
        
        // Log summary of timings
        console.log(`[WASM-STREAM] Timing Summary:
            - Total load time: ${Math.round(totalTime)}ms
            - JS fetch/eval time: ${Math.round(jsLoadEndTime - jsLoadStartTime)}ms
            - WASM fetch header time: ${fetchEndTime ? Math.round(fetchEndTime - streamingStartTime) : 'N/A'}ms
            - WASM initialization time: ${Math.round(wasmInitEndTime - wasmInitStartTime)}ms
            - Thread initialization: ${wasmThreaded ? Math.round(threadEndTime - threadStartTime) : 'N/A'}ms`);
        
        return true;
    } catch (error) {
        const errorTime = performance.now();
        console.warn(`[WASM-STREAM] Failed to load WASM module after ${Math.round(errorTime - loadStart)}ms:`, error);
        useWasm = false;
        wasmThreaded = false;
        throw error;
    }
}

// Function to solve the PoW challenge using the threaded WASM implementation
async function solveWithThreadedWasm(challenge, difficulty, workerId) {
    try {
        // Use the same thread count calculation as in loadWasmModule
        const maxThreads = 4;
        const numThreads = Math.min(navigator.hardwareConcurrency || 2, maxThreads);
        
        console.log(`Worker #${workerId} using threaded WASM with ${numThreads} threads`);
        
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