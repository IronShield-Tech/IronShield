// Build script to compile client-side WebAssembly
const { execSync } = require('child_process');
const fs = require('fs');
const path = require('path');

// Parse command line arguments and environment variables
const args = process.argv.slice(2);
const noParallel = args.includes('--no-parallel') || process.env.NO_PARALLEL === 'true';

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

if (noParallel) {
  console.log('🚫 Building IronShield WebAssembly in no-parallel mode (Mobile Safari compatible)...');
} else {
  console.log('🚀 Building IronShield WebAssembly with parallel features...');
}

try {
  // Determine feature flags based on command line arguments
  const featureFlags = noParallel ? '--no-default-features' : '--features parallel';
  
  if (!noParallel) {
    // Try to build with threading support (parallel mode)
    console.log('Attempting to build WASM with threading support...');
    try {
      // Command for building with threading support and parallel features
      const threadedBuildCmd = `cd ironshield-wasm && RUSTFLAGS="-C target-feature=+atomics,+bulk-memory,+mutable-globals" \
        rustup run nightly wasm-pack build \
        --target web \
        --release \
        ${featureFlags}`;
        
      execSync(threadedBuildCmd, { stdio: 'inherit' });
      console.log('✅ Successfully built WASM with threading and parallel features!');
    } catch (threadError) {
      console.warn('⚠️  Failed to build with threading support:', threadError.message);
      console.log('📱 Falling back to standard build without threading...');
      
      // Build without threading support as fallback
      execSync(`cd ironshield-wasm && wasm-pack build --target web --release --no-default-features`, { 
        stdio: 'inherit'
      });
      console.log('✅ Successfully built WASM without threading support');
    }
  } else {
    // Build without parallel features (no-parallel mode)
    console.log('Building WASM without parallel features for mobile compatibility...');
    execSync(`cd ironshield-wasm && wasm-pack build --target web --release ${featureFlags}`, { 
      stdio: 'inherit'
    });
    console.log('✅ Successfully built WASM in no-parallel mode (Mobile Safari compatible)');
  }
  
  // Copy the WebAssembly binary and JavaScript binding to assets
  const srcWasmFile = path.join(__dirname, 'ironshield-wasm', 'pkg', 'ironshield_wasm_bg.wasm');
  const destWasmFile = path.join(wasmDir, 'ironshield_wasm_bg.wasm');
  
  if (fs.existsSync(srcWasmFile)) {
    fs.copyFileSync(srcWasmFile, destWasmFile);
    console.log(`📁 Copied WASM binary to ${destWasmFile}`);
  } else {
    console.warn(`❌ WASM binary not found at ${srcWasmFile}`);
  }
  
  const srcJsFile = path.join(__dirname, 'ironshield-wasm', 'pkg', 'ironshield_wasm.js');
  const destJsFile = path.join(wasmDir, 'ironshield_wasm.js');
  
  if (fs.existsSync(srcJsFile)) {
    fs.copyFileSync(srcJsFile, destJsFile);
    console.log(`📁 Copied WASM JavaScript bindings to ${destJsFile}`);
  } else {
    console.warn(`❌ WASM JavaScript bindings not found at ${srcJsFile}`);
  }
  
  // Copy additional worker files needed for threading (only in parallel mode)
  if (!noParallel) {
    const threadWorkerFile = path.join(__dirname, 'ironshield-wasm', 'pkg', 'ironshield_wasm_bg.worker.js');
    const destThreadWorkerFile = path.join(wasmDir, 'ironshield_wasm_bg.worker.js');
    
    if (fs.existsSync(threadWorkerFile)) {
      fs.copyFileSync(threadWorkerFile, destThreadWorkerFile);
      console.log(`🧵 Copied WASM thread worker to ${destThreadWorkerFile}`);
    } else {
      console.warn(`⚠️  WASM thread worker not found at ${threadWorkerFile} - threading may not be available`);
    }
  } else {
    console.log('🚫 Skipping thread worker copy (no-parallel mode)');
  }
  
  if (noParallel) {
    console.log('✅ WebAssembly build completed in no-parallel mode (Mobile Safari compatible)');
  } else {
    console.log('✅ WebAssembly build completed with parallel features');
  }
  
} catch (error) {
  console.error('❌ Failed to build WebAssembly:', error);
  process.exit(1);
} 