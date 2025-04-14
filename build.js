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
  // Build the WASM module for ironshield-wasm
  console.log('Running wasm-pack build for ironshield-wasm...');
  execSync('cd ironshield-wasm && wasm-pack build --target web --release', { 
    stdio: 'inherit'
  });
  
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
  
  console.log('WebAssembly build completed');
  
} catch (error) {
  console.error('Failed to build WebAssembly:', error);
  process.exit(1);
} 