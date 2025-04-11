// Build script to compile client-side WebAssembly
const { execSync } = require('child_process');
const fs = require('fs');
const path = require('path');

// Ensure the wasm directory exists
const wasmDir = path.join(__dirname, 'wasm');
if (!fs.existsSync(wasmDir)) {
  fs.mkdirSync(wasmDir, { recursive: true });
}

console.log('Building client-side WebAssembly...');

try {
  // Create a temporary directory for the client-side WebAssembly build
  const tempDir = path.join(__dirname, 'temp_wasm_build');
  if (!fs.existsSync(tempDir)) {
    fs.mkdirSync(tempDir, { recursive: true });
  }

  // Create a temporary Cargo.toml for the client build
  const tempCargoToml = path.join(tempDir, 'Cargo.toml');
  fs.writeFileSync(tempCargoToml, `
[package]
name = "pow-wasm"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
wasm-bindgen = { version = "0.2.92", features = ["serde-serialize"] }
sha2 = "0.10"
hex = "0.4"
serde = { version = "1.0", features = ["derive"] }
serde-wasm-bindgen = "0.6.0"
console_error_panic_hook = "0.1.7"
web-sys = { version = "0.3", features = ["console"] }
js-sys = "0.3"
getrandom = { version = "0.2", features = ["js"] }

[profile.release]
lto = true
opt-level = "z"
strip = true
codegen-units = 1
`);

  // Create src directory and copy the pow_client.rs file as lib.rs
  const tempSrcDir = path.join(tempDir, 'src');
  if (!fs.existsSync(tempSrcDir)) {
    fs.mkdirSync(tempSrcDir, { recursive: true });
  }
  
  // Read the pow_client.rs file
  const powClientPath = path.join(__dirname, 'src', 'pow_client.rs');
  const powClientContent = fs.readFileSync(powClientPath, 'utf8');
  
  // Write it to the temporary lib.rs
  fs.writeFileSync(path.join(tempSrcDir, 'lib.rs'), powClientContent);
  
  // Run wasm-pack build in the temporary directory
  console.log('Running wasm-pack build...');
  execSync('wasm-pack build --target web --release', { 
    cwd: tempDir,
    stdio: 'inherit'
  });
  
  // Copy all necessary files from the pkg directory to our wasm directory
  const pkgDir = path.join(tempDir, 'pkg');
  
  // Copy the WebAssembly binary
  const wasmFile = path.join(pkgDir, 'pow_wasm_bg.wasm');
  const targetWasmFile = path.join(wasmDir, 'pow_wasm_bg.wasm');
  fs.copyFileSync(wasmFile, targetWasmFile);
  
  // Copy the JavaScript bindings
  const jsFile = path.join(pkgDir, 'pow_wasm.js');
  const targetJsFile = path.join(wasmDir, 'pow_wasm.js');
  fs.copyFileSync(jsFile, targetJsFile);
  
  // Copy the TypeScript declarations if needed
  // const dtsFile = path.join(pkgDir, 'pow_wasm.d.ts');
  // const targetDtsFile = path.join(wasmDir, 'pow_wasm.d.ts');
  // if (fs.existsSync(dtsFile)) {
  //   fs.copyFileSync(dtsFile, targetDtsFile);
  // }
  
  console.log(`WebAssembly files copied successfully to ${wasmDir}`);
  
  // Clean up
  // fs.rmSync(tempDir, { recursive: true, force: true });
  
} catch (error) {
  console.error('Failed to build WebAssembly:', error);
  process.exit(1);
} 