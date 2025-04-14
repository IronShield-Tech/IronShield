/**
 * Manages a pool of Web Workers to solve the PoW challenge in parallel.
 */
export class WorkerPoolManager {
    /**
     * Creates an instance of WorkerPoolManager.
     * @param {string} workerScriptPath - Path to the worker script.
     * @param {number} [numCores=navigator.hardwareConcurrency || 4] - Number of workers to spawn.
     */
    constructor(workerScriptPath, numCores = navigator.hardwareConcurrency || 4) {
        if (!window.Worker) {
            throw new Error('Web Workers are not supported in this browser.');
        }
        this.workerScriptPath = workerScriptPath;
        this.numCores = numCores;
        this.workers = [];
        this.solutionFound = false;
        this.resolvePromise = null;
        this.rejectPromise = null;
        this.totalAttempts = 0;
        this.startTime = 0;
        this.timeoutHandle = null;

        // Callback for progress updates (can be set externally)
        this.onProgress = null; // e.g., (attempts, hashRate) => { ... }
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
                    const worker = new Worker(this.workerScriptPath);
                    this.workers.push(worker);

                    // Setup message and error handlers (will be implemented next)
                    worker.onmessage = (e) => this._handleWorkerMessage(e, i);
                    worker.onerror = (e) => this._handleWorkerError(e, i, ++failedWorkers);

                    // Start the worker
                    worker.postMessage({
                        challenge: challenge,
                        difficulty: difficulty,
                        workerId: i,
                        startNonce: i, // Start worker i at nonce i
                        nonceStep: this.numCores // Each worker increments by the total number of cores
                    });
                    console.log(`Worker #${i} started.`);

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

    // Placeholder for message handler
    _handleWorkerMessage(event, workerIndex) {
        // console.log(`Received message from worker #${workerIndex}:`, event.data); // Optional: Keep for verbose debugging
        const data = event.data;

        if (this.solutionFound) {
            return; // Ignore messages after solution is found
        }

        switch (data.type) {
            case "success":
                console.log(`Solution found by worker #${data.workerId || workerIndex}`);
                this.solutionFound = true;
                this.terminate(); // Stop all other workers
                if (this.resolvePromise) {
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

    // Placeholder for error handler
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
                this.rejectPromise(new Error("All PoW workers failed"));
            } else {
                console.error("Reject function not available when all workers failed!");
            }
        } else {
             console.log(`Worker #${workerIndex} failed, ${this.numCores - failedWorkerCount} workers remaining.`);
             // We could potentially remove the failed worker from the pool or just let it be silent
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