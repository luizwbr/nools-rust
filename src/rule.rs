//! Rule definitions and execution

use crate::constraint::ConstraintContext;
use crate::error::Result;
use crate::fact::FactHandle;
use crate::pattern::Pattern;
use crate::session::Session;
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;

/// Priority type for rules
pub type Priority = i32;

/// Action to execute when a rule fires
pub type RuleAction = Arc<dyn Fn(&mut Session, &Match) -> Result<()> + Send + Sync>;

/// A match of facts that satisfy a rule's patterns
#[derive(Debug, Clone)]
pub struct Match {
    /// Facts matched by alias
    pub facts: HashMap<String, Arc<FactHandle>>,
    /// Constraint context with bindings
    pub context: ConstraintContext,
}

impl Match {
    /// Create a new match
    pub fn new() -> Self {
        Self {
            facts: HashMap::new(),
            context: ConstraintContext::new(),
        }
    }

    /// Get a fact by alias
    pub fn get(&self, alias: &str) -> Option<&Arc<FactHandle>> {
        self.facts.get(alias)
    }

    /// Add a fact to this match
    pub fn insert(&mut self, alias: String, fact: Arc<FactHandle>) {
        self.context.set(alias.clone(), Arc::clone(&fact));
        self.facts.insert(alias, fact);
    }
}

impl Default for Match {
    fn default() -> Self {
        Self::new()
    }
}

/// An activation represents a rule that is ready to fire
#[derive(Debug, Clone)]
pub struct Activation {
    /// The rule to fire
    pub rule: Arc<Rule>,
    /// The matched facts
    pub match_data: Match,
    /// Recency for conflict resolution
    pub recency: u64,
}

impl Activation {
    /// Create a new activation
    pub fn new(rule: Arc<Rule>, match_data: Match, recency: u64) -> Self {
        Self {
            rule,
            match_data,
            recency,
        }
    }

    /// Calculate salience for this activation
    pub fn salience(&self) -> Priority {
        self.rule.priority
    }
}

/// A rule in the rules engine
#[derive(Clone)]
pub struct Rule {
    /// Name of the rule
    pub name: String,
    /// Patterns to match
    pub patterns: Vec<Box<dyn Pattern>>,
    /// Action to execute
    pub action: RuleAction,
    /// Priority/salience
    pub priority: Priority,
    /// Agenda group
    pub agenda_group: String,
    /// Auto-focus on activation
    pub auto_focus: bool,
}

impl Debug for Rule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Rule")
            .field("name", &self.name)
            .field("patterns", &self.patterns)
            .field("priority", &self.priority)
            .field("agenda_group", &self.agenda_group)
            .field("auto_focus", &self.auto_focus)
            .finish()
    }
}

impl Rule {
    /// Create a new rule
    pub fn new(name: impl Into<String>) -> RuleBuilder {
        RuleBuilder {
            name: name.into(),
            patterns: Vec::new(),
            action: None,
            priority: 0,
            agenda_group: "main".to_string(),
            auto_focus: false,
        }
    }

    /// Fire this rule with the given match
    pub fn fire(&self, session: &mut Session, match_data: &Match) -> Result<()> {
        (self.action)(session, match_data)
    }
}

/// Builder for constructing rules
pub struct RuleBuilder {
    name: String,
    patterns: Vec<Box<dyn Pattern>>,
    action: Option<RuleAction>,
    priority: Priority,
    agenda_group: String,
    auto_focus: bool,
}

impl RuleBuilder {
    /// Add a pattern to this rule
    pub fn when(mut self, pattern: Box<dyn Pattern>) -> Self {
        self.patterns.push(pattern);
        self
    }

    /// Set the action for this rule
    pub fn then<F>(mut self, action: F) -> Self
    where
        F: Fn(&mut Session, &Match) -> Result<()> + Send + Sync + 'static,
    {
        self.action = Some(Arc::new(action));
        self
    }

    /// Set the priority/salience
    pub fn priority(mut self, priority: Priority) -> Self {
        self.priority = priority;
        self
    }

    /// Set the agenda group
    pub fn agenda_group(mut self, group: impl Into<String>) -> Self {
        self.agenda_group = group.into();
        self
    }

    /// Set auto-focus
    pub fn auto_focus(mut self, auto_focus: bool) -> Self {
        self.auto_focus = auto_focus;
        self
    }

    /// Build the rule
    pub fn build(self) -> Result<Rule> {
        let action = self
            .action
            .ok_or_else(|| crate::error::Error::Compilation("Rule action not defined".into()))?;

        Ok(Rule {
            name: self.name,
            patterns: self.patterns,
            action,
            priority: self.priority,
            agenda_group: self.agenda_group,
            auto_focus: self.auto_focus,
        })
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
    fn test_rule_builder() {
        let pattern = Box::new(ObjectPattern::<TestFact>::new("test")) as Box<dyn Pattern>;

        let rule = Rule::new("test_rule")
            .when(pattern)
            .then(|_session, _match| Ok(()))
            .priority(10)
            .agenda_group("test")
            .build()
            .unwrap();

        assert_eq!(rule.name, "test_rule");
        assert_eq!(rule.priority, 10);
        assert_eq!(rule.agenda_group, "test");
        assert_eq!(rule.patterns.len(), 1);
    }

    #[test]
    fn test_match_operations() {
        let mut match_data = Match::new();
        let fact = TestFact { value: 42 };
        let handle = Arc::new(crate::fact::FactHandle::new(fact, 0));

        match_data.insert("test".to_string(), Arc::clone(&handle));

        assert!(match_data.get("test").is_some());
        assert_eq!(match_data.get("test").unwrap().id, handle.id);
    }
}
