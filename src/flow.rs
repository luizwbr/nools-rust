//! Flow container for rules and their execution

use crate::agenda::ConflictResolution;
use crate::error::{Error, Result};
use crate::node::{AlphaNode, RootNode, TerminalNode};
use crate::rule::Rule;
use crate::session::Session;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Flow represents a container for rules
pub struct Flow {
    /// Name of this flow
    name: String,
    /// Rules in this flow
    rules: HashMap<String, Arc<Rule>>,
    /// Root node of the Rete network
    root: Arc<RwLock<RootNode>>,
    /// Conflict resolution strategies
    strategies: Vec<ConflictResolution>,
}

impl Flow {
    /// Create a new flow with the given name
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            rules: HashMap::new(),
            root: Arc::new(RwLock::new(RootNode::new())),
            strategies: vec![
                ConflictResolution::Salience,
                ConflictResolution::ActivationRecency,
            ],
        }
    }

    /// Get the name of this flow
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Set conflict resolution strategies
    pub fn with_strategies(mut self, strategies: Vec<ConflictResolution>) -> Self {
        self.strategies = strategies;
        self
    }

    /// Add a rule to this flow
    pub fn add_rule(&mut self, rule: Rule) -> Result<()> {
        let rule_name = rule.name.clone();
        if self.rules.contains_key(&rule_name) {
            return Err(Error::Compilation(format!(
                "Rule '{}' already exists",
                rule_name
            )));
        }

        let rule_arc = Arc::new(rule);

        // Build Rete network for this rule
        self.build_network_for_rule(Arc::clone(&rule_arc))?;

        self.rules.insert(rule_name, rule_arc);
        Ok(())
    }

    /// Build Rete network nodes for a rule
    fn build_network_for_rule(&mut self, rule: Arc<Rule>) -> Result<()> {
        let mut root = self.root.write().map_err(|e| {
            Error::Compilation(format!("Failed to acquire lock on root node: {}", e))
        })?;

        // For now, create a simple alpha node -> terminal node chain
        // In a full implementation, this would build a complete Rete network with joins
        for pattern in &rule.patterns {
            let mut alpha = AlphaNode::new(pattern.clone_box());
            let terminal = TerminalNode::new(Arc::clone(&rule));
            alpha.add_child(Box::new(terminal));
            root.add_child(Box::new(alpha));
        }

        Ok(())
    }

    /// Get a rule by name
    pub fn get_rule(&self, name: &str) -> Option<Arc<Rule>> {
        self.rules.get(name).map(Arc::clone)
    }

    /// Check if a rule exists
    pub fn has_rule(&self, name: &str) -> bool {
        self.rules.contains_key(name)
    }

    /// Get all rule names
    pub fn rule_names(&self) -> Vec<String> {
        self.rules.keys().cloned().collect()
    }

    /// Create a new session from this flow
    pub fn session(&self) -> Session {
        Session::new(
            self.name.clone(),
            Arc::clone(&self.root),
            self.strategies.clone(),
        )
    }

    /// Create a fluent rule builder
    pub fn rule(&mut self, name: impl Into<String>) -> FlowRuleBuilder<'_> {
        FlowRuleBuilder {
            flow: self,
            builder: Rule::new(name),
        }
    }
}

/// Builder for adding rules to a flow
pub struct FlowRuleBuilder<'a> {
    flow: &'a mut Flow,
    builder: crate::rule::RuleBuilder,
}

impl<'a> FlowRuleBuilder<'a> {
    /// Add a pattern
    pub fn when(mut self, pattern: Box<dyn crate::pattern::Pattern>) -> Self {
        self.builder = self.builder.when(pattern);
        self
    }

    /// Set the action
    pub fn then<F>(mut self, action: F) -> Result<()>
    where
        F: Fn(&mut Session, &crate::rule::Match) -> Result<()> + Send + Sync + 'static,
    {
        self.builder = self.builder.then(action);
        let rule = self.builder.build()?;
        self.flow.add_rule(rule)
    }

    /// Set priority
    pub fn priority(mut self, priority: i32) -> Self {
        self.builder = self.builder.priority(priority);
        self
    }

    /// Set agenda group
    pub fn agenda_group(mut self, group: impl Into<String>) -> Self {
        self.builder = self.builder.agenda_group(group);
        self
    }

    /// Set auto focus
    pub fn auto_focus(mut self, auto_focus: bool) -> Self {
        self.builder = self.builder.auto_focus(auto_focus);
        self
    }
}

impl std::fmt::Debug for Flow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Flow")
            .field("name", &self.name)
            .field("rules", &self.rules.keys())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fact::Fact;
    use crate::pattern::ObjectPattern;

    #[derive(Debug, Clone)]
    struct TestFact {
        value: i32,
    }

    #[test]
    fn test_flow_creation() {
        let flow = Flow::new("test_flow");
        assert_eq!(flow.name(), "test_flow");
        assert_eq!(flow.rule_names().len(), 0);
    }

    #[test]
    fn test_add_rule() {
        let mut flow = Flow::new("test");
        let rule = Rule::new("test_rule")
            .when(Box::new(ObjectPattern::<TestFact>::new("t")) as Box<dyn crate::pattern::Pattern>)
            .then(|_, _| Ok(()))
            .build()
            .unwrap();

        flow.add_rule(rule).unwrap();
        assert!(flow.has_rule("test_rule"));
    }

    #[test]
    fn test_duplicate_rule_error() {
        let mut flow = Flow::new("test");
        let rule1 = Rule::new("test_rule")
            .when(Box::new(ObjectPattern::<TestFact>::new("t")) as Box<dyn crate::pattern::Pattern>)
            .then(|_, _| Ok(()))
            .build()
            .unwrap();

        let rule2 = Rule::new("test_rule")
            .when(Box::new(ObjectPattern::<TestFact>::new("t")) as Box<dyn crate::pattern::Pattern>)
            .then(|_, _| Ok(()))
            .build()
            .unwrap();

        flow.add_rule(rule1).unwrap();
        assert!(flow.add_rule(rule2).is_err());
    }
}
