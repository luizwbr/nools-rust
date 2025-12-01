# Nools-Rust

[![Crates.io](https://img.shields.io/crates/v/nools-rust.svg)](https://crates.io/crates/nools-rust)
[![npm](https://img.shields.io/npm/v/nools-rust.svg)](https://www.npmjs.com/package/nools-rust)
[![npm downloads](https://img.shields.io/npm/dm/nools-rust.svg)](https://www.npmjs.com/package/nools-rust)
[![Documentation](https://docs.rs/nools-rust/badge.svg)](https://docs.rs/nools-rust)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-2021-orange.svg)](https://www.rust-lang.org)
[![WebAssembly](https://img.shields.io/badge/WebAssembly-enabled-blue.svg)](https://webassembly.org/)

A modern Rust implementation of the Nools rules engine, based on the [Rete algorithm](https://en.wikipedia.org/wiki/Rete_algorithm).

> **High-performance rules engine with type safety and memory safety guaranteed by Rust.**  
> **Now with WebAssembly support for universal compatibility!**

## Features

- **Fast Pattern Matching**: Implements the Rete algorithm for efficient rule evaluation
- **Type-Safe**: Leverages Rust's type system for compile-time safety
- **Async Support**: Built with Tokio for asynchronous rule execution
- **Flexible Constraints**: Support for complex pattern matching and constraints
- **Agenda Groups**: Logical grouping of rules with focus management
- **Conflict Resolution**: Configurable strategies for activation ordering

## Quick Start

### Node.js / npm

```bash
npm install nools-rust
```

```javascript
const nools = require('nools-rust');

// Create a flow
const flow = nools.flow('My Rules');

// Add rules (with priority)
flow.addRule('Rule1', 100);
flow.addRule('Rule2', 50);

// Create a session
const session = flow.session();

// Assert facts (as JSON strings)
const fact = new nools.Fact(JSON.stringify({ type: 'order', total: 150 }));
session.assert(fact);

// Fire rules
const fired = session.matchRules();
console.log(`${fired} rules fired`);

// Clean up
session.dispose();
```

### Rust (Native)

```rust
use nools_rust::prelude::*;

#[derive(Debug, Clone, Fact)]
struct Message {
    text: String,
}

fn main() {
    let mut flow = Flow::new("hello_world");
    
    flow.rule("greet")
        .when(|m: &Message| m.text.contains("hello"))
        .then(|ctx, m: &mut Message| {
            m.text = format!("{} world!", m.text);
            ctx.modify(m);
        });
    
    let mut session = flow.session();
    session.assert(Message {
        text: "hello".to_string(),
    });
    
    session.match_rules().await?;
}
```

## Architecture

- **Flow**: Container for rules and their execution context
- **Session**: Instance of a flow with working memory
- **Rule**: Pattern-action pair with priority and agenda group
- **Working Memory**: Storage for facts with efficient indexing
- **Agenda**: Priority queue for rule activations
- **Rete Network**: Optimized pattern matching network

## API Reference (WebAssembly)

### Main Functions

- **`flow(name: string): Flow`** - Create a new flow
- **`version(): string`** - Get library version
- **`init(): void`** - Initialize WASM module (auto-called)

### Flow Class

- **`new Flow(name: string)`** - Constructor
- **`addRule(name: string, priority: number): RuleBuilder`** - Add a rule
- **`session(): Session`** - Create a session
- **`name: string`** - Get flow name (getter)
- **`ruleCount: number`** - Get number of rules (getter)

### Session Class

- **`assert(fact: Fact): void`** - Assert a fact (consumes the fact)
- **`retract(factId: bigint): boolean`** - Retract a fact by ID
- **`matchRules(): number`** - Fire all matching rules, returns count
- **`halt(): void`** - Stop rule execution
- **`getFacts(): any`** - Get all facts as JSON array
- **`dispose(): void`** - Clean up the session
- **`factCount: number`** - Get number of facts (getter)
- **`halted: boolean`** - Check if halted (getter)

### Fact Class

- **`new Fact(data: string)`** - Create a fact with JSON data
- **`id: bigint`** - Unique fact ID (getter, read before asserting!)
- **`data: string`** - Fact data as JSON string (getter)

### Important Notes

⚠️ **Facts are consumed by `assert()`** - Save the `fact.id` before asserting if you need it later:
```javascript
const fact = new nools.Fact(JSON.stringify({ type: 'order' }));
const factId = fact.id;  // Save ID first!
session.assert(fact);     // fact is now invalid
session.retract(factId);  // Use saved ID
```

## Examples

See the `examples/` directory for detailed examples:
- **`examples/node/simple.js`** - Basic WebAssembly usage
- **`examples/node/hello_world.js`** - Complete API demonstration
- **`examples/fibonacci.rs`** - Classic Fibonacci sequence (Rust)
- **`examples/hello_world.rs`** - Basic pattern matching (Rust)
- **`examples/state_machine.rs`** - Complex state transitions (Rust)

## Performance

This implementation focuses on:
- Zero-copy fact handling where possible
- Minimal allocations using arena patterns
- Efficient indexing with hash maps
- Parallel rule evaluation (where applicable)

## Installation

### Option 1: WebAssembly (Recommended) 🚀

**Universal compatibility - works on all platforms!**

```bash
# Install from npm
npm install nools-rust

# Build from source
npm run build
```

No compiler required! Just Node.js.

### Option 2: Native Rust

### From crates.io (Rust)

Add to your `Cargo.toml`:

```toml
[dependencies]
nools-rust = "0.1.3"
```

### From npm (Node.js)

```bash
npm install nools-rust
```

Then in your JavaScript/TypeScript:

```javascript
const nools = require('nools-rust');

const flow = nools.flow('example');
// Use the API
```

## Package Names

- **Rust/crates.io**: `nools-rust`
- **npm/Node.js**: `nools-rust`

## License

MIT
