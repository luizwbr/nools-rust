// Example usage of nools-rust from Node.js
// Note: This is a conceptual example - actual implementation will be available after native bindings are fully built

const nools = require('nools-rust');

// Define a simple message type
class Message {
  constructor(text) {
    this.text = text;
  }
}

// Create a flow
const flow = nools.flow('Hello World');

// Add a rule
flow.addRule('Hello', {
  priority: 10
}, function(facts) {
  if (facts.message && facts.message.text.includes('hello')) {
    console.log('Matched:', facts.message.text);
    facts.message.text += ' world';
  }
});

flow.addRule('Goodbye', {
  priority: 5
}, function(facts) {
  if (facts.message && facts.message.text.includes('world')) {
    console.log('Final:', facts.message.text);
  }
});

// Create a session and assert facts
async function run() {
  const session = flow.session();
  
  await session.assert(new Message('hello'));
  await session.assert(new Message('goodbye'));
  
  // Match all rules
  const fired = await session.matchRules();
  console.log(`Total rules fired: ${fired}`);
  
  // Clean up
  session.dispose();
}

// Run the example
run().catch(console.error);

// Expected output:
// Matched: hello
// Final: hello world
// Total rules fired: 2
