// Node.js wrapper for nools-rust WASM module

const path = require('path');
const fs = require('fs');

// Try to load the WASM module
let nools = null;
let loadError = null;

try {
  // Load from pkg directory (built with wasm-pack)
  const pkgPath = path.join(__dirname, 'pkg', 'nools.js');
  
  if (fs.existsSync(pkgPath)) {
    nools = require('./pkg/nools.js');
  } else {
    throw new Error('WASM module not found. Run "npm run build" to compile.');
  }
} catch (e) {
  loadError = e;
}

if (!nools) {
  console.error('❌ Failed to load nools-rust WASM module');
  console.error('');
  console.error('To build the WASM module, run:');
  console.error('  npm run build');
  console.error('');
  console.error('Make sure you have wasm-pack installed:');
  console.error('  cargo install wasm-pack');
  console.error('');
  
  if (loadError) {
    throw loadError;
  }
  throw new Error('Failed to load WASM module');
}

// Export the WASM module
module.exports = nools;
