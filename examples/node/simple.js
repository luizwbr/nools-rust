// Simple example showing basic nools-rust usage
const nools = require('nools-rust');

console.log('=== nools-rust Basic Example ===\n');

// 1. Create a flow
const flow = nools.flow('Shopping Cart Rules');

// 2. Add rules
flow.addRule('FreeShipping', 100);  // High priority
flow.addRule('Discount10', 50);      // Medium priority
flow.addRule('Discount5', 25);       // Lower priority

console.log(`Created flow with ${flow.ruleCount} rules\n`);

// 3. Create a session
const session = flow.session();

// 4. Assert facts (as JSON strings)
const cart1 = new nools.Fact(JSON.stringify({
  cart: 'A123',
  total: 150,
  items: 5
}));
session.assert(cart1);

const cart2 = new nools.Fact(JSON.stringify({
  cart: 'B456',
  total: 45,
  items: 2
}));
session.assert(cart2);

console.log(`Asserted ${session.factCount} facts\n`);

// 5. Match and fire rules
console.log('Firing rules...');
const fired = session.matchRules();
console.log(`\n${fired} rule activations fired\n`);

// 6. Get all facts
const facts = session.getFacts();
console.log('Current facts:', JSON.stringify(facts, null, 2));

// 7. Clean up
session.dispose();
console.log('\n✓ Session completed successfully');
