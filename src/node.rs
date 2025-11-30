//! Rete network node implementations

use crate::constraint::ConstraintContext;
use crate::error::Result;
use crate::fact::FactHandle;
use crate::pattern::Pattern;
use crate::rule::{Activation, Match};
use std::sync::Arc;

/// Base trait for nodes in the Rete network
pub trait Node: Send + Sync {
    /// Process a fact assertion
    fn assert_fact(&mut self, fact: Arc<FactHandle>) -> Result<Vec<Arc<Activation>>>;

    /// Process a fact retraction
    fn retract_fact(&mut self, fact: Arc<FactHandle>) -> Result<Vec<Arc<Activation>>>;

    /// Process a fact modification
    fn modify_fact(&mut self, fact: Arc<FactHandle>) -> Result<Vec<Arc<Activation>>> {
        // Default: retract then assert
        let mut results = self.retract_fact(Arc::clone(&fact))?;
        results.extend(self.assert_fact(fact)?);
        Ok(results)
    }
}

/// Root node of the Rete network
pub struct RootNode {
    /// Child nodes (type nodes)
    children: Vec<Box<dyn Node>>,
}

impl RootNode {
    /// Create a new root node
    pub fn new() -> Self {
        Self {
            children: Vec::new(),
        }
    }

    /// Add a child node
    pub fn add_child(&mut self, child: Box<dyn Node>) {
        self.children.push(child);
    }
}

impl Default for RootNode {
    fn default() -> Self {
        Self::new()
    }
}

impl Node for RootNode {
    fn assert_fact(&mut self, fact: Arc<FactHandle>) -> Result<Vec<Arc<Activation>>> {
        let mut activations = Vec::new();
        for child in &mut self.children {
            activations.extend(child.assert_fact(Arc::clone(&fact))?);
        }
        Ok(activations)
    }

    fn retract_fact(&mut self, fact: Arc<FactHandle>) -> Result<Vec<Arc<Activation>>> {
        let mut activations = Vec::new();
        for child in &mut self.children {
            activations.extend(child.retract_fact(Arc::clone(&fact))?);
        }
        Ok(activations)
    }
}

/// Alpha node for pattern matching
pub struct AlphaNode {
    /// Pattern to match
    pattern: Box<dyn Pattern>,
    /// Child nodes
    children: Vec<Box<dyn Node>>,
    /// Matched facts
    memory: Vec<Arc<FactHandle>>,
}

impl AlphaNode {
    /// Create a new alpha node
    pub fn new(pattern: Box<dyn Pattern>) -> Self {
        Self {
            pattern,
            children: Vec::new(),
            memory: Vec::new(),
        }
    }

    /// Add a child node
    pub fn add_child(&mut self, child: Box<dyn Node>) {
        self.children.push(child);
    }
}

impl std::fmt::Debug for AlphaNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AlphaNode")
            .field("pattern", &self.pattern)
            .field("memory_size", &self.memory.len())
            .finish()
    }
}

impl Node for AlphaNode {
    fn assert_fact(&mut self, fact: Arc<FactHandle>) -> Result<Vec<Arc<Activation>>> {
        let context = ConstraintContext::new();

        if self.pattern.matches(&fact, &context)? {
            self.memory.push(Arc::clone(&fact));

            let mut activations = Vec::new();
            for child in &mut self.children {
                activations.extend(child.assert_fact(Arc::clone(&fact))?);
            }
            return Ok(activations);
        }

        Ok(Vec::new())
    }

    fn retract_fact(&mut self, fact: Arc<FactHandle>) -> Result<Vec<Arc<Activation>>> {
        self.memory.retain(|f| f.id != fact.id);

        let mut activations = Vec::new();
        for child in &mut self.children {
            activations.extend(child.retract_fact(Arc::clone(&fact))?);
        }
        Ok(activations)
    }
}

/// Terminal node that creates activations
pub struct TerminalNode {
    /// The rule this terminal represents
    rule: Arc<crate::rule::Rule>,
    /// Counter for activation recency
    recency: std::sync::atomic::AtomicU64,
}

impl TerminalNode {
    /// Create a new terminal node
    pub fn new(rule: Arc<crate::rule::Rule>) -> Self {
        Self {
            rule,
            recency: std::sync::atomic::AtomicU64::new(0),
        }
    }
}

impl std::fmt::Debug for TerminalNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TerminalNode")
            .field("rule", &self.rule.name)
            .finish()
    }
}

impl Node for TerminalNode {
    fn assert_fact(&mut self, fact: Arc<FactHandle>) -> Result<Vec<Arc<Activation>>> {
        let recency = self
            .recency
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);

        let mut match_data = Match::new();
        // For simple rules with one pattern, use the first pattern's alias
        if let Some(pattern) = self.rule.patterns.first() {
            match_data.insert(pattern.alias().to_string(), fact);
        }

        let activation = Arc::new(Activation::new(
            Arc::clone(&self.rule),
            match_data,
            recency,
        ));

        Ok(vec![activation])
    }

    fn retract_fact(&mut self, _fact: Arc<FactHandle>) -> Result<Vec<Arc<Activation>>> {
        // Retractions don't create activations in terminal nodes
        Ok(Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fact::Fact;
    use crate::pattern::ObjectPattern;
    use crate::rule::Rule;

    #[derive(Debug, Clone)]
    struct TestFact {
        value: i32,
    }

    #[test]
    fn test_alpha_node_matching() {
        let pattern = Box::new(
            ObjectPattern::<TestFact>::new("test").with_filter(|f| f.value > 40, "value > 40"),
        ) as Box<dyn Pattern>;

        let mut node = AlphaNode::new(pattern);

        let fact1 = FactHandle::new(TestFact { value: 42 }, 0);
        let fact2 = FactHandle::new(TestFact { value: 30 }, 1);

        node.assert_fact(Arc::new(fact1)).unwrap();
        assert_eq!(node.memory.len(), 1);

        node.assert_fact(Arc::new(fact2)).unwrap();
        assert_eq!(node.memory.len(), 1); // Should still be 1
    }

    #[test]
    fn test_terminal_node_activation() {
        let rule = Arc::new(
            Rule::new("test_rule")
                .when(Box::new(ObjectPattern::<TestFact>::new("test")) as Box<dyn Pattern>)
                .then(|_, _| Ok(()))
                .build()
                .unwrap(),
        );

        let mut node = TerminalNode::new(rule);

        let fact = FactHandle::new(TestFact { value: 42 }, 0);
        let activations = node.assert_fact(Arc::new(fact)).unwrap();

        assert_eq!(activations.len(), 1);
        assert_eq!(activations[0].rule.name, "test_rule");
    }
}
