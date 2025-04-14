/**
 * Manages a pool of Web Workers to solve the PoW challenge in parallel.
 */
export class WorkerPoolManager {
    /**
     * Creates an instance of WorkerPoolManager.
     * @param {string} workerScriptPath - Path to the worker script.
     * @param {number} [numCores=navigator.hardwareConcurrency || 4] - Number of workers to spawn.
     * @param {boolean} [useWasm=true] - Whether to try using WASM when available.
     */
    constructor(workerScriptPath, numCores = navigator.hardwareConcurrency || 4, useWasm = true) {
        if (!window.Worker) {
            throw new Error('Web Workers are not supported in this browser.');
        }
        
        // Get the current URL's origin to ensure we're loading from the same origin
        const baseUrl = window.location.origin;
        
        // Default to the new WASM worker if useWasm is true, otherwise use the JS worker
        // Use path without /assets prefix to match server routes
        this.workerScriptPath = useWasm ? `${baseUrl}/wasm_pow_worker.js` : workerScriptPath;
        console.log(`Using worker path: ${this.workerScriptPath}`);
        this.fallbackWorkerPath = workerScriptPath; // Original JS worker as fallback
        this.numCores = numCores;
        this.workers = [];
        this.solutionFound = false;
        this.resolvePromise = null;
        this.rejectPromise = null;
        this.totalAttempts = 0;
        this.startTime = 0;
        this.timeoutHandle = null;
        this.useWasm = useWasm;
        this.isWasmAvailable = false;
        this.isWasmThreaded = false;
        this.workersInitialized = 0;

        // Callback for progress updates (can be set externally)
        this.onProgress = null; // e.g., (attempts, hashRate) => { ... }
        // Callback for WASM status updates
        this.onWasmStatus = null; // e.g., (isWasmAvailable, isThreaded) => { ... }
    }

    /**
     * Starts the PoW challenge solving process.
     * @param {string} challenge - The challenge string.
     * @param {number} difficulty - The PoW difficulty.
     * @param {number} timeoutSeconds - Timeout duration in seconds.
     * @returns {Promise<object>} A promise that resolves with the solution or rejects on error/timeout.
     */
    solve(challenge, difficulty, timeoutSeconds) {
        console.log(`WorkerPoolManager starting PoW solve with ${this.numCores} workers...`);
        this.solutionFound = false;
        this.totalAttempts = 0;
        this.startTime = Date.now();
        this.workersInitialized = 0;
        let failedWorkers = 0;

        return new Promise((resolve, reject) => {
            this.resolvePromise = resolve;
            this.rejectPromise = reject;

            if (!this.resolvePromise || !this.rejectPromise) {
                console.error("Promise resolve/reject functions not set!");
                return; // Should not happen
            }

            // Create and setup workers
            for (let i = 0; i < this.numCores; i++) {
                try {
                    console.log(`Creating worker #${i} from ${this.workerScriptPath}...`);
                    // Use a local variable to ensure we're creating the worker with the right path
                    const workerPath = this.workerScriptPath;
                    console.log(`Worker #${i} actual path: ${workerPath}`);
                    const worker = new Worker(workerPath);
                    this.workers.push(worker);

                    // Setup message and error handlers
                    worker.onmessage = (e) => this._handleWorkerMessage(e, i);
                    worker.onerror = (e) => this._handleWorkerError(e, i, ++failedWorkers);

                    // Initialize the worker
                    worker.postMessage({
                        type: 'init',
                        workerId: i
                    });
                    console.log(`Worker #${i} initialization started.`);

                } catch (error) {
                    console.error(`Failed to create or start worker #${i}:`, error);
                    // If even one worker fails to start, we probably can't reliably solve
                    this.terminate();
                    this.rejectPromise(new Error(`Failed to initialize workers: ${error.message}`));
                    return; // Stop creating more workers
                }
            }

            // Set up the timeout
            this.timeoutHandle = setTimeout(() => {
                if (!this.solutionFound) {
                    console.warn(`Timeout: PoW calculation took longer than ${timeoutSeconds} seconds.`);
                    this.terminate();
                    this.rejectPromise(new Error("Timeout: PoW calculation took too long"));
                }
            }, timeoutSeconds * 1000);
        });
    }

    _handleWorkerMessage(event, workerIndex) {
        // console.log(`Received message from worker #${workerIndex}:`, event.data); // Optional: Keep for verbose debugging
        const data = event.data;

        if (this.solutionFound) {
            return; // Ignore messages after solution is found
        }

        switch (data.type) {
            case "init_complete":
                console.log(`Worker #${data.workerId || workerIndex} initialization complete`);
                this.workersInitialized++;
                
                // Store WASM availability status from this worker
                if (data.useWasm) {
                    this.isWasmAvailable = true;
                    this.isWasmThreaded = data.wasmThreaded;
                    
                    // Call the WASM status callback if provided
                    if (this.onWasmStatus) {
                        try {
                            this.onWasmStatus(true, data.wasmThreaded);
                        } catch (e) {
                            console.error("Error in onWasmStatus callback:", e);
                        }
                    }
                    
                    console.log(`Worker #${data.workerId} using WASM${data.wasmThreaded ? ' with threads' : ''}`);
                } else if (data.error) {
                    console.warn(`Worker #${data.workerId} couldn't use WASM: ${data.error}`);
                }
                
                // Once all workers are initialized, start solving
                if (this.workersInitialized === this.numCores) {
                    // Now send the actual solve command to all workers
                    for (let i = 0; i < this.workers.length; i++) {
                        this.workers[i].postMessage({
                            type: 'solve',
                            challenge: this.currentChallenge,
                            difficulty: this.currentDifficulty,
                            workerId: i,
                            startNonce: i, // Start worker i at nonce i
                            nonceStep: this.numCores // Each worker increments by the total number of cores
                        });
                    }
                }
                break;

            case "success":
                console.log(`Solution found by worker #${data.workerId || workerIndex}`);
                this.solutionFound = true;
                this.terminate(); // Stop all other workers
                if (this.resolvePromise) {
                    // Note if WASM was used in the solution for metrics
                    data.solution.usedWasm = data.useWasm;
                    data.solution.usedThreadedWasm = data.wasmThreaded;
                    this.resolvePromise(data.solution);
                } else {
                    console.error("Resolve function not available when solution found!");
                }
                break;

            case "progress":
                this.totalAttempts += data.attempts; // Add attempts reported by the worker
                // Optionally, update external progress callback
                if (this.onProgress) {
                    const elapsedMs = Date.now() - this.startTime;
                    const hashRate = elapsedMs > 0 ? Math.round(this.totalAttempts / elapsedMs * 1000) : 0;
                    // Provide total attempts and current hash rate
                    try {
                        this.onProgress(this.totalAttempts, hashRate);
                    } catch (e) {
                        console.error("Error in onProgress callback:", e);
                    }
                }
                break;
            
            case "finalProgress":
                // This message mainly exists for logging within the worker/main thread
                // We could potentially update a final accurate attempt count here if needed
                // console.log(`Worker #${data.workerId} final attempt count: ${data.attempts.toLocaleString()}`);
                break;

            case "hashingStarted":
                 // console.log(`Worker #${data.workerId || workerIndex} reported hashing started.`);
                 // Could potentially trigger initial UI update via onProgress or a dedicated callback
                 break;

            default:
                console.warn(`Unknown message type from worker #${workerIndex}:`, data.type);
        }
    }

    _handleWorkerError(event, workerIndex, failedWorkerCount) {
        console.error(`Error from worker #${workerIndex}:`, event.message || event);
        
        // Prevent default error handling which might stop the script
        event.preventDefault(); 

        if (this.solutionFound) {
            return; // Ignore errors after solution is found
        }

        // Check if all workers have failed
        if (failedWorkerCount >= this.numCores) {
            console.error("All workers have failed.");
            this.terminate();
            if (this.rejectPromise) {
                // Provide more context for debugging
                this.rejectPromise(new Error(`All PoW workers failed. Last error: ${event.message || 'Unknown error'}`));
            } else {
                console.error("Reject function not available when all workers failed!");
            }
        } else {
             console.log(`Worker #${workerIndex} failed, ${this.numCores - failedWorkerCount} workers remaining.`);
             // If over half of the workers have failed, try increasing error visibility
             if (failedWorkerCount > this.numCores / 2) {
                 console.warn(`Warning: ${failedWorkerCount} out of ${this.numCores} workers have failed.`);
             }
        }
    }

    /**
     * Terminates all workers in the pool.
     */
    terminate() {
        console.log("Terminating all workers.");
        this.workers.forEach(worker => worker.terminate());
        this.workers = [];
        if (this.timeoutHandle) {
            clearTimeout(this.timeoutHandle);
            this.timeoutHandle = null;
        }
    }
} 