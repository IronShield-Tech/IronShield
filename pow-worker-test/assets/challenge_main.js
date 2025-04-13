// First, load the WebAssembly module and bindings
// async function loadWasmModule(retryCount = 0) { ... }

async function solveChallenge() {
    // Get all parameters from meta tags
    const difficultyMeta = document.querySelector('meta[name="x-ironshield-difficulty"]');
    const timestampMeta = document.querySelector('meta[name="x-ironshield-timestamp"]');
    const challengeMeta = document.querySelector('meta[name="x-ironshield-challenge"]');
    
    const statusDiv = document.getElementById("status");
    const progressBar = document.getElementById("progress");
    
    // Error handling for missing meta tags
    if (!difficultyMeta || !timestampMeta || !challengeMeta) {
        statusDiv.textContent = "Error: Security parameters missing. Please refresh the page.";
        progressBar.value = 0;
        return;
    }
    
    // Parse the values
    const difficultyStr = difficultyMeta.getAttribute('content');
    const difficulty = parseInt(difficultyStr, 10);
    
    if (isNaN(difficulty) || difficulty <= 0) {
        statusDiv.textContent = "Error: Invalid security parameters. Please refresh the page.";
        progressBar.value = 0;
        return;
    }
    
    const timestamp = timestampMeta.getAttribute('content');
    const challenge = challengeMeta.getAttribute('content');
    
    if (!timestamp || !challenge) {
        statusDiv.textContent = "Error: Missing security parameters. Please refresh the page.";
        progressBar.value = 0;
        return;
    }
    
    console.log(`Using challenge: ${challenge}, difficulty: ${difficulty}, timestamp: ${timestamp}`);
    
    // Update status - no need to load WASM anymore
    statusDiv.textContent = "Preparing challenge solver...";
    progressBar.value = 5; // Small initial progress

    try {
        // Update status
        statusDiv.textContent = "Solving challenge...";
        progressBar.value = 10;
        
        // Set up an update interval for the progress bar to show activity
        const startTime = Date.now();
        const progressInterval = setInterval(() => {
            const elapsedMs = Date.now() - startTime;
            // Slowly increment progress, max 95% (save 5% for completion)
            progressBar.value = Math.min(95, (elapsedMs / 10000) * 100);
        }, 100);
        
        // Define timeout (in seconds)
        const timeoutSeconds = 30;
        
        // Get the base URL and other necessary URLs
        const baseUrl = window.location.origin;
        
        let solution;
        
        // Check if Web Workers are supported
        if (window.Worker) {
            console.log("Using Web Workers for background PoW calculation");
            
            // Run the PoW calculation in multiple background Workers to utilize all CPU cores
            solution = await new Promise((resolve, reject) => {
                try {
                    // Determine the number of CPU cores (logical processors)
                    const numCores = navigator.hardwareConcurrency || 4; // Default to 4 if not available
                    console.log(`Detected ${numCores} CPU cores, attempting to launch workers from /pow_worker.js`);
                    
                    // Create an array to hold all our workers
                    const workers = [];
                    
                    // Track if we've found a solution
                    let solutionFound = false;
                    
                    // Create a promise that will be rejected if all workers fail
                    let failedWorkers = 0;
                    
                    // Track total attempts across all workers
                    let totalAttempts = 0;
                    let lastUIUpdate = Date.now();
                    
                    // Track per-worker stats
                    const workerStats = Array(numCores).fill(0);
                    const startTimestamp = Date.now();
                    
                    // Update status to show hashing has started immediately
                    statusDiv.textContent = "Computing hash values (using all available CPU cores)...";
                    progressBar.value = 25;
                    
                    // Create and start multiple workers
                    for (let i = 0; i < numCores; i++) {
                        console.log(`Creating worker #${i}...`);
                        const worker = new Worker('/pow_worker.js');
                        console.log(`Worker #${i} created successfully.`);
                        workers.push(worker);
                        
                        // Handle messages from each worker
                        worker.onmessage = function(e) {
                            console.log(`Received message from worker #${e.data?.workerId ?? i}:`, e.data);
                            if (e.data.type === "success") {
                                if (!solutionFound) {
                                    const elapsedSeconds = ((Date.now() - startTimestamp) / 1000).toFixed(2);
                                    const hashRate = Math.round(totalAttempts / (Date.now() - startTimestamp) * 1000);
                                    
                                    console.log(`Solution found by worker #${e.data.workerId} in ${elapsedSeconds}s:`, e.data.solution);
                                    console.log(`Total attempts: ${totalAttempts.toLocaleString()} (${hashRate.toLocaleString()} hashes/sec)`);
                                    console.log(`Per-worker attempts: ${workerStats.map(a => a.toLocaleString()).join(', ')}`);
                                    
                                    solutionFound = true;
                                    
                                    // Update UI with final stats
                                    statusDiv.textContent = `Computing hash values... (${totalAttempts.toLocaleString()} attempts, ${hashRate.toLocaleString()} hashes/sec)`;
                                    
                                    // Terminate all workers
                                    workers.forEach(w => w.terminate());
                                    
                                    // Log total attempts that were needed
                                    console.log(`Total attempts across all workers: ${totalAttempts.toLocaleString()}`);
                                    
                                    // Resolve the promise with the solution
                                    resolve(e.data.solution);
                                }
                            } else if (e.data.type === "error") {
                                console.error(`Worker #${e.data.workerId} error:`, e.data.message);
                                failedWorkers++;
                                
                                // If all workers have failed, reject with an error
                                if (failedWorkers >= numCores) {
                                    workers.forEach(w => w.terminate());
                                    console.error("All workers failed to find a solution.");
                                    reject(new Error("All workers failed to find a solution"));
                                }
                            } else if (e.data.type === "progress") {
                                // Add these attempts to our total count - use the exact number reported
                                totalAttempts += e.data.attempts;
                                
                                // Update per-worker stats
                                workerStats[e.data.workerId] = e.data.totalAttempts;
                                
                                // Calculate hash rate
                                const elapsedMs = Date.now() - startTimestamp;
                                if (elapsedMs > 0) {
                                    const hashRate = Math.round(totalAttempts / elapsedMs * 1000);
                                    
                                    // Update UI with total attempts and rate, but limit updates to avoid UI thrashing
                                    const now = Date.now();
                                    if (now - lastUIUpdate > 100) { // Update UI at most every 100ms
                                        statusDiv.textContent = `Computing hash values... (${totalAttempts.toLocaleString()} total attempts, ${hashRate.toLocaleString()} hashes/sec)`;
                                        lastUIUpdate = now;
                                    }
                                }
                            } else if (e.data.type === "finalProgress") {
                                // Update final count from the worker that found the solution
                                workerStats[e.data.workerId] = e.data.attempts;
                                console.log(`Worker #${e.data.workerId} final attempt count: ${e.data.attempts.toLocaleString()}`);
                            }
                        };
                        
                        // Handle errors
                        worker.onerror = function(e) {
                            console.error(`Worker #${i} encountered an error:`, e);
                            failedWorkers++;
                            
                            // If all workers have failed, reject with an error
                            if (failedWorkers >= numCores) {
                                workers.forEach(w => w.terminate());
                                console.error("All workers failed to find a solution due to errors.");
                                reject(new Error("All workers failed to find a solution"));
                            }
                        };
                        
                        // Start each worker with a different nonce range
                        // Worker 0 checks nonces 0, numCores, 2*numCores, ...
                        // Worker 1 checks nonces 1, numCores+1, 2*numCores+1, ...
                        // And so on, ensuring no overlap and full CPU utilization
                        worker.postMessage({
                            challenge: challenge,
                            difficulty: difficulty,
                            workerId: i,
                            startNonce: i,
                            nonceStep: numCores
                        });
                    }
                    
                    // Set up a timeout to terminate all workers if they take too long
                    setTimeout(() => {
                        if (!solutionFound) {
                            workers.forEach(w => w.terminate());
                            console.warn("Timeout: PoW calculation took too long. Terminating workers.");
                            reject(new Error("Timeout: PoW calculation took too long"));
                        }
                    }, timeoutSeconds * 1000);
                    
                } catch (error) {
                    console.error("Error setting up Web Workers:", error);
                    reject(error);
                }
            });
        } else {
            // Fallback for browsers without Web Workers
            console.warn("Web Workers not supported. Falling back to main thread calculation (may block UI).");
            statusDiv.textContent = "Solving challenge on main thread (may cause temporary freeze)...";
            progressBar.value = 10;
            
            // Perform the calculation directly in the main thread (not recommended)
            // Ensure the standalone calculatePowSolution exists or is imported/defined here
            // For now, let's just log a warning and stop, as the worker logic is primary
            // solution = await calculatePowSolution_mainThread(challenge, difficulty); // You would need this function
            console.error("Main thread fallback calculation not implemented yet.");
            statusDiv.textContent = "Error: Web Workers required for this security check.";
            throw new Error("Web Workers are not supported or enabled in this browser.");
        }
        
        // Clear the progress update interval
        clearInterval(progressInterval);
        
        // Update UI
        statusDiv.textContent = `Challenge solved! (Nonce: ${solution.nonce_str}, Hash: ${solution.hash_prefix}...)`;
        progressBar.value = 100;
        
        // Send the solution back to the server
        fetch(window.location.href, {
            method: "GET",
            headers: {
                "X-IronShield-Challenge": challenge,
                "X-IronShield-Nonce": solution.nonce_str,
                "X-IronShield-Timestamp": timestamp,
                "X-IronShield-Difficulty": difficultyStr
            }
        })
        .then(response => {
            if (response.ok) {
                return response.text().then(html => {
                    document.open();
                    document.write(html);
                    document.close();
                });
            } else {
                statusDiv.textContent = `Verification failed (Status: ${response.status}). Please try refreshing.`;
                progressBar.value = 0;
            }
        })
        .catch(error => {
            console.error("Error sending verification:", error);
            statusDiv.textContent = "Error sending verification. Please check console.";
            progressBar.value = 0;
        });
        
    } catch (error) {
        console.error("Error solving challenge:", error);
        statusDiv.textContent = "Error during security check: " + error.message;
        progressBar.value = 0;
        
        // Clear interval in case of error
        // Find the interval variable if it's declared elsewhere or adjust scope
        // clearInterval(progressInterval); // Make sure progressInterval is accessible here if needed
    }
}

// Start the challenge solving process automatically
document.addEventListener('DOMContentLoaded', solveChallenge); 