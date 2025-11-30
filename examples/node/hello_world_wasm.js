// Example: Hello World with nools-rust WASM
// This version uses the WebAssembly build instead of native bindings

const nools = require('../../index-wasm.js');

console.log('nools-rust version:', nools.version());
console.log('');

// Create a flow
const flow = nools.flow('Hello World WASM');
console.log('Created flow:', flow.name);

// Create a session
const session = flow.session();

// Create and assert facts
const fact1 = new nools.Fact('{"message": "hello world"}');
const fact2 = new nools.Fact('{"message": "goodbye"}');

console.log('');
console.log('Asserting facts...');
session.assert(fact1);
session.assert(fact2);

console.log('Facts in session:', session.factCount);

// Match rules
console.log('');
console.log('Matching rules...');
const fired = session.match_rules();
console.log('Rules fired:', fired);

// Clean up
session.dispose();
console.log('');
console.log('✅ Example complete!');
