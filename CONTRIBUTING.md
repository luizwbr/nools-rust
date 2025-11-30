# Contributing to Nools-RS

Thank you for your interest in contributing to Nools-RS!

## Development Setup

1. Install Rust (latest stable): https://rustup.rs/
2. Clone the repository
3. Run tests: `cargo test`
4. Run examples: `cargo run --example hello_world`

## Code Style

- Follow Rust standard formatting: `cargo fmt`
- Run clippy for lints: `cargo clippy`
- Write tests for new features
- Document public APIs with doc comments

## Pull Request Process

1. Ensure all tests pass
2. Update documentation as needed
3. Add examples for new features
4. Write clear commit messages

## Architecture

### Core Components

- **Fact**: Type-safe fact representation with Any trait
- **Pattern**: Declarative pattern matching with constraints
- **Rule**: Pattern-action pairs with priority
- **Working Memory**: Efficient fact storage and indexing
- **Agenda**: Priority queue for rule activations
- **Rete Network**: Alpha and beta nodes for pattern matching
- **Flow**: Container for rules
- **Session**: Runtime instance with working memory

### Design Principles

1. **Type Safety**: Leverage Rust's type system
2. **Zero-Copy**: Minimize allocations using Arc and references
3. **Concurrency**: Use async/await and thread-safe structures
4. **Ergonomics**: Builder patterns and method chaining
5. **Performance**: Efficient indexing and lazy evaluation

## Testing

- Unit tests in each module
- Integration tests in `tests/`
- Benchmarks in `benches/`
- Examples in `examples/`

Run all tests:
```bash
cargo test --all-features
```

Run benchmarks:
```bash
cargo bench
```

## Questions?

Open an issue or start a discussion on GitHub.
