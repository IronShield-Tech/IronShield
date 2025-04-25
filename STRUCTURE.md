# IronShield Project Structure

This document outlines the structure of the IronShield project, designed for a gradual migration to WebAssembly.

## Project Components

### 1. Core Library (`ironshield-core`)

The core library contains shared functionality used by both the server-side and client-side implementations:

- Pure Rust implementation of PoW algorithms
- No WASM or platform-specific code
- Contains the core business logic

### 2. WASM Module (`ironshield-wasm`)

The WebAssembly module exposes the core functionalities to the browser:

- Provides JavaScript bindings to the core library
- Handles serialization/deserialization for JS interop
- Compiles to a WebAssembly module using wasm-pack

### 3. Cloudflare Worker (`ironshield-cloudflare`) 

The Cloudflare Worker implementation:

- Uses the core library for server-side verification
- Handles HTTP requests and responses
- Serves the challenge page and assets

## Build Process

The build process is handled by `build.js`, which:

1. Compiles the WASM module using wasm-pack
2. Copies the output files to the assets directory
3. Prepares everything for deployment

## Gradual Migration Plan

1. **Phase 1:** Move shared code to the core library (current)
2. **Phase 2:** Start using the core library in both WASM and Worker code
3. **Phase 3:** Gradually add multithreading support to the WASM implementation
4. **Phase 4:** Add JavaScript fallback for browsers without WASM support

## Development Workflow

1. Make changes to the core library when implementing shared functionality
2. Update the WASM bindings as needed
3. Update the Worker implementation
4. Run `node build.js` to compile and prepare assets
5. Deploy to Cloudflare Workers 