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

```rust
use nools::prelude::*;

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

## Examples

See the `examples/` directory for more detailed examples:
- `fibonacci.rs` - Classic Fibonacci sequence
- `hello_world.rs` - Basic pattern matching
- `state_machine.rs` - Complex state transitions

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

See [`WASM-BUILD-GUIDE.md`](WASM-BUILD-GUIDE.md) for details.

### Option 2: Native Rust

### From crates.io (Rust)

Add to your `Cargo.toml`:

```toml
[dependencies]
nools-rust = "0.1.0"
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

## Publishing

See [PUBLISHING.md](PUBLISHING.md) for detailed instructions on publishing to:
- **crates.io** (Rust package registry)
- **npm** (Node.js package registry)

## Package Names

- **Rust/crates.io**: `nools-rust`
- **npm/Node.js**: `nools-rust`

## License

MIT
