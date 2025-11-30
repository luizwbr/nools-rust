use wasm_bindgen::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Set up panic hook for better error messages in WASM
#[cfg(feature = "console_error_panic_hook")]
pub use console_error_panic_hook::set_once as set_panic_hook;

// Use wee_alloc as the global allocator (smaller binary size)
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

/// Initialize the WASM module
#[wasm_bindgen(start)]
pub fn init() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

/// A fact that can be asserted into the working memory
#[wasm_bindgen]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fact {
    id: u64,
    data: String,
}

#[wasm_bindgen]
impl Fact {
    #[wasm_bindgen(constructor)]
    pub fn new(data: String) -> Fact {
        use std::sync::atomic::{AtomicU64, Ordering};
        static NEXT_ID: AtomicU64 = AtomicU64::new(1);
        
        Fact {
            id: NEXT_ID.fetch_add(1, Ordering::SeqCst),
            data,
        }
    }

    #[wasm_bindgen(getter)]
    pub fn id(&self) -> u64 {
        self.id
    }

    #[wasm_bindgen(getter)]
    pub fn data(&self) -> String {
        self.data.clone()
    }

    #[wasm_bindgen(setter)]
    pub fn set_data(&mut self, data: String) {
        self.data = data;
    }
}

/// A rule with a condition and action
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Rule {
    name: String,
    priority: i32,
    condition: String,
    action: String,
}

/// The main Flow container for rules
#[wasm_bindgen]
pub struct Flow {
    name: String,
    rules: Vec<Rule>,
}

#[wasm_bindgen]
impl Flow {
    #[wasm_bindgen(constructor)]
    pub fn new(name: String) -> Flow {
        Flow {
            name,
            rules: Vec::new(),
        }
    }

    /// Add a rule to the flow
    pub fn add_rule(&mut self, name: String, priority: i32) -> RuleBuilder {
        RuleBuilder {
            name,
            priority,
            flow_name: self.name.clone(),
        }
    }

    /// Get the flow name
    #[wasm_bindgen(getter)]
    pub fn name(&self) -> String {
        self.name.clone()
    }

    /// Create a new session from this flow
    pub fn session(&self) -> Session {
        Session {
            flow_name: self.name.clone(),
            rules: self.rules.clone(),
            facts: Vec::new(),
            halted: false,
        }
    }

    /// Get number of rules
    #[wasm_bindgen(getter = ruleCount)]
    pub fn rule_count(&self) -> usize {
        self.rules.len()
    }
}

/// Builder for creating rules
#[wasm_bindgen]
pub struct RuleBuilder {
    name: String,
    priority: i32,
    flow_name: String,
}

#[wasm_bindgen]
impl RuleBuilder {
    pub fn when(&self, condition: String) -> RuleBuilder {
        RuleBuilder {
            name: self.name.clone(),
            priority: self.priority,
            flow_name: self.flow_name.clone(),
        }
    }
}

/// A session for asserting facts and firing rules
#[wasm_bindgen]
pub struct Session {
    flow_name: String,
    rules: Vec<Rule>,
    facts: Vec<Fact>,
    halted: bool,
}

#[wasm_bindgen]
impl Session {
    /// Assert a fact into the working memory
    pub fn assert(&mut self, fact: Fact) {
        self.facts.push(fact);
    }

    /// Retract a fact from the working memory by ID
    pub fn retract(&mut self, fact_id: u64) -> bool {
        if let Some(pos) = self.facts.iter().position(|f| f.id == fact_id) {
            self.facts.remove(pos);
            true
        } else {
            false
        }
    }

    /// Get all facts as JSON
    pub fn get_facts(&self) -> Result<JsValue, JsValue> {
        serde_wasm_bindgen::to_value(&self.facts)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
    }

    /// Get number of facts
    #[wasm_bindgen(getter = factCount)]
    pub fn fact_count(&self) -> usize {
        self.facts.len()
    }

    /// Match and fire all rules
    pub fn match_rules(&mut self) -> Result<u32, JsValue> {
        if self.halted {
            return Ok(0);
        }

        let mut fired = 0;

        // Sort rules by priority (higher first)
        let mut sorted_rules = self.rules.clone();
        sorted_rules.sort_by(|a, b| b.priority.cmp(&a.priority));

        // Simple pattern matching - just fire rules for each fact
        for rule in &sorted_rules {
            for fact in &self.facts {
                // In a real implementation, this would evaluate the condition
                // For now, just fire the rule
                fired += 1;
                
                // Log to console
                web_sys::console::log_1(&format!(
                    "Rule '{}' fired for fact {} (priority: {})",
                    rule.name, fact.id, rule.priority
                ).into());
            }
        }

        Ok(fired)
    }

    /// Halt the session
    pub fn halt(&mut self) {
        self.halted = true;
    }

    /// Check if session is halted
    #[wasm_bindgen(getter)]
    pub fn halted(&self) -> bool {
        self.halted
    }

    /// Dispose the session
    pub fn dispose(&mut self) {
        self.facts.clear();
        self.rules.clear();
        self.halted = true;
    }
}

/// Create a new flow
#[wasm_bindgen]
pub fn flow(name: String) -> Flow {
    Flow::new(name)
}

/// Get the version of the library
#[wasm_bindgen]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fact_creation() {
        let fact = Fact::new("test".to_string());
        assert_eq!(fact.data(), "test");
    }

    #[test]
    fn test_flow_creation() {
        let flow = Flow::new("test".to_string());
        assert_eq!(flow.name(), "test");
        assert_eq!(flow.rule_count(), 0);
    }

    #[test]
    fn test_session() {
        let flow = Flow::new("test".to_string());
        let mut session = flow.session();
        
        let fact = Fact::new("data".to_string());
        session.assert(fact);
        
        assert_eq!(session.fact_count(), 1);
    }
}
