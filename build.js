// Build script to compile client-side WebAssembly
const { execSync } = require('child_process');
const fs = require('fs');
const path = require('path');

// Ensure the assets directory exists
const assetsDir = path.join(__dirname, 'assets');
if (!fs.existsSync(assetsDir)) {
  fs.mkdirSync(assetsDir, { recursive: true });
}

// Ensure the wasm directory exists (for the output)
const wasmDir = path.join(assetsDir, 'wasm');
if (!fs.existsSync(wasmDir)) {
  fs.mkdirSync(wasmDir, { recursive: true });
}

console.log('Building IronShield WebAssembly...');

try {
  // First, try to build with threading support
  console.log('Attempting to build WASM with threading support...');
  try {
    // Command for building with threading support
    const threadedBuildCmd = `cd ironshield-wasm && RUSTFLAGS="-C target-feature=+atomics,+bulk-memory,+mutable-globals" \
      rustup run nightly wasm-pack build \
      --target web \
      --release`;
      
    execSync(threadedBuildCmd, { stdio: 'inherit' });
    console.log('Successfully built WASM with threading support!');
  } catch (threadError) {
    console.warn('Failed to build with threading support:', threadError.message);
    console.log('Falling back to standard build without threading...');
    
    // Build without threading support as fallback
    execSync('cd ironshield-wasm && wasm-pack build --target web --release', { 
      stdio: 'inherit'
    });
    console.log('Successfully built WASM without threading support');
  }
  
  // Copy the WebAssembly binary and JavaScript binding to assets
  const srcWasmFile = path.join(__dirname, 'ironshield-wasm', 'pkg', 'ironshield_wasm_bg.wasm');
  const destWasmFile = path.join(wasmDir, 'ironshield_wasm_bg.wasm');
  
  if (fs.existsSync(srcWasmFile)) {
    fs.copyFileSync(srcWasmFile, destWasmFile);
    console.log(`Copied WASM binary to ${destWasmFile}`);
  } else {
    console.warn(`WASM binary not found at ${srcWasmFile}`);
  }
  
  const srcJsFile = path.join(__dirname, 'ironshield-wasm', 'pkg', 'ironshield_wasm.js');
  const destJsFile = path.join(wasmDir, 'ironshield_wasm.js');
  
  if (fs.existsSync(srcJsFile)) {
    fs.copyFileSync(srcJsFile, destJsFile);
    console.log(`Copied WASM JavaScript bindings to ${destJsFile}`);
  } else {
    console.warn(`WASM JavaScript bindings not found at ${srcJsFile}`);
  }
  
  // Copy additional worker files needed for threading
  const threadWorkerFile = path.join(__dirname, 'ironshield-wasm', 'pkg', 'ironshield_wasm_bg.worker.js');
  const destThreadWorkerFile = path.join(wasmDir, 'ironshield_wasm_bg.worker.js');
  
  if (fs.existsSync(threadWorkerFile)) {
    fs.copyFileSync(threadWorkerFile, destThreadWorkerFile);
    console.log(`Copied WASM thread worker to ${destThreadWorkerFile}`);
  } else {
    console.warn(`WASM thread worker not found at ${threadWorkerFile} - threading may not be available`);
  }
  
  console.log('WebAssembly build completed');
  
} catch (error) {
  console.error('Failed to build WebAssembly:', error);
  process.exit(1);
} 