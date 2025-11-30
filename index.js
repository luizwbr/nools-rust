// WebAssembly bindings for nools-rust
// This module loads the WASM-compiled nools engine

try {
  // Load the WASM module compiled by wasm-pack
  const wasm = require('./pkg/nools.js');
  module.exports = wasm;
} catch (error) {
  console.error('Failed to load nools-rust WASM module:');
  console.error(error.message);
  console.error('\nPlease build the WASM module first:');
  console.error('  npm run build');
  console.error('\nOr use wasm-pack directly:');
  console.error('  wasm-pack build --target nodejs --out-dir pkg');
  throw error;
}
