/**
 * Handles communication with the backend server to submit the PoW solution.
 */
export class ApiClient {
    /**
     * Submits the solved challenge details back to the server.
     *
     * @param {string} challenge - The original challenge string.
     * @param {string} nonce_str - The solved nonce as a string.
     * @param {string} timestamp - The original timestamp string.
     * @param {string} difficultyStr - The difficulty level as a string.
     * @returns {Promise<Object>} A promise that resolves with the parsed JSON response if successful.
     * @throws {Error} Throws an error if the fetch fails or the server response is not OK (status >= 400).
     */
    async submitSolution(challenge, nonce_str, timestamp, difficultyStr) {
        console.log("Submitting solution to server...");
        
        // Use the current window location for the fetch URL
        const submitUrl = window.location.href;

        try {
            const response = await fetch(submitUrl, {
                method: "GET", // Or POST, depending on how the server expects verification
                headers: {
                    // Use the constants or ensure header names match the server expectation
                    "X-IronShield-Challenge": challenge,
                    "X-IronShield-Nonce": nonce_str,
                    "X-IronShield-Timestamp": timestamp,
                    "X-IronShield-Difficulty": difficultyStr
                },
                credentials: 'include' // Include cookies in the request
            });

            if (!response.ok) {
                // Throw an error with status text if available, otherwise a generic error
                const errorText = response.statusText || `HTTP error ${response.status}`;
                console.error(`Server verification failed: ${response.status} ${errorText}`);
                throw new Error(`Verification failed (Status: ${response.status})`);
            }

            console.log("Server verification successful.");
            
            // Check if the response is JSON
            const contentType = response.headers.get("content-type");
            if (contentType && contentType.includes("application/json")) {
                // Parse and return the JSON response
                const jsonResponse = await response.json();
                console.log("Received JSON response:", jsonResponse);
                return jsonResponse;
            } else {
                // Fallback to text response for backward compatibility
                console.log("Received text response (not JSON)");
                const textResponse = await response.text();
                return { success: true, message: textResponse };
            }

        } catch (error) {
            console.error("Error submitting solution:", error);
            // Re-throw the error to be caught by the caller
            // If it was a fetch network error, it might not have a specific status
            throw new Error(error.message || "Error sending verification"); 
        }
    }
} 