// TypeScript example for nools-rust
import { flow, Session } from 'nools-rust';

interface Message {
  text: string;
}

interface Counter {
  count: number;
}

async function main() {
  // Create a flow
  const myFlow = flow('Counter Example');
  
  // Add a rule that increments counter
  myFlow.addRule('increment', {
    priority: 10,
    agendaGroup: 'main'
  }, (facts: { counter: Counter }) => {
    if (facts.counter.count < 10) {
      console.log(`Count: ${facts.counter.count}`);
      facts.counter.count++;
    }
  });
  
  // Add a rule that completes when counter reaches 10
  myFlow.addRule('complete', {
    priority: 5
  }, (facts: { counter: Counter }) => {
    if (facts.counter.count === 10) {
      console.log('Counter reached 10!');
    }
  });
  
  // Create session and run
  const session = myFlow.session();
  
  await session.assert({ count: 0 });
  
  const fired = await session.matchRules();
  console.log(`Rules fired: ${fired}`);
  
  session.dispose();
}

main().catch(console.error);
