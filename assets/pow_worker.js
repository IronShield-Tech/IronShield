// Simple worker that receives challenge data and performs calculation
// This file is used as a Web Worker to perform PoW calculations in a separate thread
// Use the correct global scope to ensure compatibility across environments
const workerScope = typeof self !== 'undefined' ? self : this;

workerScope.onmessage = async function(e) {
    try {
        // Get challenge data
        const { challenge, difficulty, workerId, startNonce, nonceStep } = e.data;
        console.log("Worker #" + workerId + " received challenge, difficulty:", difficulty);
        
        // We'll calculate the PoW without WebAssembly for reliability
        // This is a pure JavaScript implementation of the PoW algorithm
        console.log("Worker #" + workerId + " starting calculation from nonce " + startNonce + " with step " + nonceStep);
        const startTime = performance.now();
        
        // Send a message back to indicate we're starting the hashing
        if (workerId === 0) {
            workerScope.postMessage({
                type: "hashingStarted"
            });
        }
        
        const solution = await calculatePowSolution(challenge, difficulty, workerId, startNonce, nonceStep);
        
        const endTime = performance.now();
        console.log("Worker #" + workerId + " found solution in " + (endTime - startTime).toFixed(2) + "ms");
        
        // Send the result back to the main thread
        workerScope.postMessage({
            type: "success",
            solution: solution,
            timeTaken: endTime - startTime,
            workerId: workerId
        });
    } catch (error) {
        console.error("Worker #" + e.data.workerId + " error:", error);
        workerScope.postMessage({
            type: "error",
            message: error.message || "Unknown error",
            workerId: e.data.workerId
        });
    }
};

// Pure JavaScript implementation of the PoW algorithm
// Modified to use a specific starting nonce and step value for parallel execution
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
        
        // Just do a single hash - multiple hashes were causing the process to hang
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
            // Report actual attempts, not just a signal
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