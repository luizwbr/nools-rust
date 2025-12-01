// Example usage of nools-rust WASM module from Node.js
const nools = require('nools-rust');

console.log('nools-rust version:', nools.version());
console.log('');

// Create a flow (rules container)
const flow = nools.flow('Hello World Flow');
console.log('Created flow:', flow.name);

// Add rules to the flow
const rule1 = flow.addRule('Rule1', 10);
console.log('Added Rule1 with priority 10');

const rule2 = flow.addRule('Rule2', 5);
console.log('Added Rule2 with priority 5');

console.log('Total rules in flow:', flow.ruleCount);
console.log('');

// Create a session from the flow
const session = flow.session();
console.log('Created session, initial fact count:', session.factCount);

// Assert facts into the session
// Facts are JSON objects stored as strings
// Note: Fact IDs must be saved BEFORE asserting, as assert() consumes the fact
const fact1 = new nools.Fact(JSON.stringify({ 
  type: 'message', 
  text: 'hello' 
}));
const fact1Id = fact1.id;
session.assert(fact1);
console.log('Asserted fact1, ID:', fact1Id);

const fact2 = new nools.Fact(JSON.stringify({ 
  type: 'message', 
  text: 'world' 
}));
const fact2Id = fact2.id;
session.assert(fact2);
console.log('Asserted fact2, ID:', fact2Id);

console.log('Fact count after assertions:', session.factCount);
console.log('');

// Get all facts
const allFacts = session.getFacts();
console.log('All facts in session:', allFacts);
console.log('');

// Match and fire rules
const firedCount = session.matchRules();
console.log('Rules fired:', firedCount);
console.log('Session halted?', session.halted);
console.log('');

// Retract a fact
const retracted = session.retract(fact1Id);
console.log('Retracted fact1?', retracted);
console.log('Fact count after retraction:', session.factCount);
console.log('');

// Clean up
session.dispose();
console.log('Session disposed successfully!');
