//! Nools-Rust: A Rete-based rules engine
//!
//! This crate provides a fast, type-safe rules engine implementation based on the Rete algorithm.
//! It compiles to WebAssembly for use in Node.js and browsers.

#[cfg(target_arch = "wasm32")]
mod wasm;

#[cfg(target_arch = "wasm32")]
pub use wasm::*;

#[cfg(not(target_arch = "wasm32"))]
pub mod agenda;
#[cfg(not(target_arch = "wasm32"))]
pub mod constraint;
#[cfg(not(target_arch = "wasm32"))]
pub mod error;
#[cfg(not(target_arch = "wasm32"))]
pub mod fact;
#[cfg(not(target_arch = "wasm32"))]
pub mod flow;
#[cfg(not(target_arch = "wasm32"))]
pub mod node;
#[cfg(not(target_arch = "wasm32"))]
pub mod pattern;
#[cfg(not(target_arch = "wasm32"))]
pub mod rule;
#[cfg(not(target_arch = "wasm32"))]
pub mod session;
#[cfg(not(target_arch = "wasm32"))]
pub mod working_memory;

/// Commonly used types and traits
#[cfg(not(target_arch = "wasm32"))]
pub mod prelude {
    pub use crate::error::{Error, Result};
    pub use crate::fact::{Fact, FactId};
    pub use crate::flow::Flow;
    pub use crate::pattern::Pattern;
    pub use crate::rule::{Rule, RuleBuilder};
    pub use crate::session::Session;
}

#[cfg(not(target_arch = "wasm32"))]
pub use prelude::*;
