//! Agenda for managing rule activations and conflict resolution

use crate::error::{Error, Result};
use crate::rule::Activation;
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::sync::Arc;

/// Conflict resolution strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConflictResolution {
    /// Order by salience (priority)
    Salience,
    /// Order by activation recency (LIFO)
    ActivationRecency,
    /// Order by fact recency
    FactRecency,
}

/// Wrapper for activations in the priority queue
#[derive(Debug, Clone)]
struct ActivationWrapper {
    activation: Arc<Activation>,
    strategies: Vec<ConflictResolution>,
}

impl ActivationWrapper {
    fn new(activation: Arc<Activation>, strategies: Vec<ConflictResolution>) -> Self {
        Self {
            activation,
            strategies,
        }
    }

    fn compare(&self, other: &Self) -> Ordering {
        for strategy in &self.strategies {
            let ord = match strategy {
                ConflictResolution::Salience => {
                    self.activation.salience().cmp(&other.activation.salience())
                }
                ConflictResolution::ActivationRecency => {
                    self.activation.recency.cmp(&other.activation.recency)
                }
                ConflictResolution::FactRecency => {
                    let self_max = self
                        .activation
                        .match_data
                        .facts
                        .values()
                        .map(|f| f.recency)
                        .max()
                        .unwrap_or(0);
                    let other_max = other
                        .activation
                        .match_data
                        .facts
                        .values()
                        .map(|f| f.recency)
                        .max()
                        .unwrap_or(0);
                    self_max.cmp(&other_max)
                }
            };

            if ord != Ordering::Equal {
                return ord;
            }
        }
        Ordering::Equal
    }
}

impl PartialEq for ActivationWrapper {
    fn eq(&self, other: &Self) -> bool {
        self.compare(other) == Ordering::Equal
    }
}

impl Eq for ActivationWrapper {}

impl PartialOrd for ActivationWrapper {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.compare(other))
    }
}

impl Ord for ActivationWrapper {
    fn cmp(&self, other: &Self) -> Ordering {
        self.compare(other)
    }
}

/// An agenda group containing activations
#[derive(Debug)]
struct AgendaGroup {
    #[allow(dead_code)]
    name: String,
    activations: BinaryHeap<ActivationWrapper>,
    strategies: Vec<ConflictResolution>,
}

impl AgendaGroup {
    fn new(name: String, strategies: Vec<ConflictResolution>) -> Self {
        Self {
            name,
            activations: BinaryHeap::new(),
            strategies,
        }
    }

    fn insert(&mut self, activation: Arc<Activation>) {
        self.activations
            .push(ActivationWrapper::new(activation, self.strategies.clone()));
    }

    fn pop(&mut self) -> Option<Arc<Activation>> {
        self.activations.pop().map(|w| w.activation)
    }

    fn is_empty(&self) -> bool {
        self.activations.is_empty()
    }

    fn clear(&mut self) {
        self.activations.clear();
    }
}

/// The agenda manages rule activations and determines execution order
#[derive(Debug)]
pub struct Agenda {
    /// Agenda groups
    groups: HashMap<String, AgendaGroup>,
    /// Stack of focused agenda groups
    focus_stack: Vec<String>,
    /// Conflict resolution strategies
    strategies: Vec<ConflictResolution>,
    /// Set of rule names that have been registered
    registered_rules: HashSet<String>,
}

impl Agenda {
    /// Create a new agenda with default strategies
    pub fn new() -> Self {
        Self::with_strategies(vec![
            ConflictResolution::Salience,
            ConflictResolution::ActivationRecency,
        ])
    }

    /// Create a new agenda with custom conflict resolution strategies
    pub fn with_strategies(strategies: Vec<ConflictResolution>) -> Self {
        let mut agenda = Self {
            groups: HashMap::new(),
            focus_stack: Vec::new(),
            strategies: strategies.clone(),
            registered_rules: HashSet::new(),
        };

        // Create default "main" group
        agenda.add_agenda_group("main".to_string());
        let _ = agenda.set_focus("main".to_string());

        agenda
    }

    /// Add a new agenda group
    pub fn add_agenda_group(&mut self, name: String) {
        if !self.groups.contains_key(&name) {
            self.groups.insert(
                name.clone(),
                AgendaGroup::new(name, self.strategies.clone()),
            );
        }
    }

    /// Get the currently focused agenda group
    pub fn get_focused(&self) -> Option<&str> {
        self.focus_stack.last().map(|s| s.as_str())
    }

    /// Set focus to an agenda group
    pub fn set_focus(&mut self, name: String) -> Result<()> {
        if !self.groups.contains_key(&name) {
            return Err(Error::AgendaGroupNotFound(name));
        }

        if self.get_focused() != Some(&name) {
            self.focus_stack.push(name);
        }

        Ok(())
    }

    /// Register a rule name
    pub fn register_rule(&mut self, rule_name: String, agenda_group: Option<String>) {
        self.registered_rules.insert(rule_name);
        if let Some(group) = agenda_group {
            self.add_agenda_group(group);
        }
    }

    /// Insert an activation into the appropriate agenda group
    pub fn insert(&mut self, activation: Arc<Activation>) -> Result<()> {
        let group_name = &activation.rule.agenda_group;

        if !self.groups.contains_key(group_name) {
            self.add_agenda_group(group_name.clone());
        }

        let group = self
            .groups
            .get_mut(group_name)
            .ok_or_else(|| Error::AgendaGroupNotFound(group_name.clone()))?;

        group.insert(activation.clone());

        // Auto-focus if needed
        if activation.rule.auto_focus {
            self.set_focus(group_name.clone())?;
        }

        Ok(())
    }

    /// Pop the next activation from the focused agenda group
    pub fn pop(&mut self) -> Option<Arc<Activation>> {
        // Try focused groups from top of stack
        while let Some(focused) = self.focus_stack.last().cloned() {
            if let Some(group) = self.groups.get_mut(&focused) {
                if let Some(activation) = group.pop() {
                    return Some(activation);
                }
            }

            // Pop empty group unless it's "main"
            if focused != "main" {
                self.focus_stack.pop();
            } else {
                break;
            }
        }

        None
    }

    /// Check if the agenda is empty
    pub fn is_empty(&self) -> bool {
        // Check if focused groups have any activations
        for focused in self.focus_stack.iter().rev() {
            if let Some(group) = self.groups.get(focused) {
                if !group.is_empty() {
                    return false;
                }
            }
        }
        true
    }

    /// Clear all activations from all groups
    pub fn clear(&mut self) {
        for group in self.groups.values_mut() {
            group.clear();
        }
    }

    /// Dispose of the agenda
    pub fn dispose(&mut self) {
        self.clear();
        self.groups.clear();
        self.focus_stack.clear();
        self.registered_rules.clear();
    }
}

impl Default for Agenda {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rule::{Match, Rule};

    fn create_test_activation(name: &str, priority: Priority, recency: u64) -> Arc<Activation> {
        let rule = Arc::new(
            Rule::new(name)
                .then(|_, _| Ok(()))
                .priority(priority)
                .build()
                .unwrap(),
        );

        Arc::new(Activation::new(rule, Match::new(), recency))
    }

    #[test]
    fn test_agenda_creation() {
        let agenda = Agenda::new();
        assert_eq!(agenda.get_focused(), Some("main"));
    }

    #[test]
    fn test_agenda_group_management() {
        let mut agenda = Agenda::new();
        agenda.add_agenda_group("test".to_string());
        assert!(agenda.groups.contains_key("test"));
    }

    #[test]
    fn test_activation_insertion_and_pop() {
        let mut agenda = Agenda::new();
        let act1 = create_test_activation("rule1", 10, 1);
        let act2 = create_test_activation("rule2", 5, 2);

        agenda.insert(act1).unwrap();
        agenda.insert(act2).unwrap();

        // Should pop the higher priority one first
        let popped = agenda.pop().unwrap();
        assert_eq!(popped.rule.name, "rule1");
    }

    #[test]
    fn test_conflict_resolution_by_salience() {
        let mut agenda = Agenda::with_strategies(vec![ConflictResolution::Salience]);

        let act1 = create_test_activation("low", 1, 3);
        let act2 = create_test_activation("high", 10, 1);

        agenda.insert(act1).unwrap();
        agenda.insert(act2).unwrap();

        let first = agenda.pop().unwrap();
        assert_eq!(first.rule.name, "high");
    }

    #[test]
    fn test_focus_management() {
        let mut agenda = Agenda::new();
        agenda.add_agenda_group("group1".to_string());
        agenda.set_focus("group1".to_string()).unwrap();

        assert_eq!(agenda.get_focused(), Some("group1"));
    }
}
