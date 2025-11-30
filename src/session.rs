//! Session for rule execution

use crate::agenda::Agenda;
use crate::error::Result;
use crate::fact::{Fact, FactHandle, FactId};
use crate::node::{Node, RootNode};
use crate::working_memory::WorkingMemory;
use std::sync::{Arc, RwLock};

/// Session represents an instance of a flow with working memory
pub struct Session {
    /// Name of the flow this session belongs to
    flow_name: String,
    /// Working memory
    working_memory: WorkingMemory,
    /// Agenda for managing activations
    agenda: Agenda,
    /// Root node of the Rete network
    root: Arc<RwLock<RootNode>>,
    /// Whether execution has been halted
    halted: bool,
}

impl Session {
    /// Create a new session
    pub fn new(
        flow_name: String,
        root: Arc<RwLock<RootNode>>,
        strategies: Vec<crate::agenda::ConflictResolution>,
    ) -> Self {
        Self {
            flow_name,
            working_memory: WorkingMemory::new(),
            agenda: Agenda::with_strategies(strategies),
            root,
            halted: false,
        }
    }

    /// Get the flow name
    pub fn flow_name(&self) -> &str {
        &self.flow_name
    }

    /// Assert a fact into working memory
    pub fn assert<T: Fact>(&mut self, fact: T) -> Result<FactId> {
        let handle = self.working_memory.assert(fact)?;
        let fact_id = handle.id;

        // Propagate through Rete network
        let mut root = self.root.write().map_err(|e| {
            crate::error::Error::Execution(format!("Failed to acquire lock: {}", e))
        })?;

        let activations = root.assert_fact(handle)?;

        // Add activations to agenda
        for activation in activations {
            self.agenda.insert(activation)?;
        }

        Ok(fact_id)
    }

    /// Retract a fact from working memory
    pub fn retract(&mut self, fact_id: FactId) -> Result<()> {
        let handle = self.working_memory.retract(fact_id)?;

        // Propagate through Rete network
        let mut root = self.root.write().map_err(|e| {
            crate::error::Error::Execution(format!("Failed to acquire lock: {}", e))
        })?;

        root.retract_fact(handle)?;

        Ok(())
    }

    /// Modify a fact in working memory
    pub fn modify(&mut self, fact_id: FactId) -> Result<()> {
        let handle = self.working_memory.modify(fact_id)?;

        // Propagate through Rete network
        let mut root = self.root.write().map_err(|e| {
            crate::error::Error::Execution(format!("Failed to acquire lock: {}", e))
        })?;

        let activations = root.modify_fact(handle)?;

        // Add activations to agenda
        for activation in activations {
            self.agenda.insert(activation)?;
        }

        Ok(())
    }

    /// Get a fact by ID
    pub fn get_fact(&self, fact_id: FactId) -> Option<Arc<FactHandle>> {
        self.working_memory.get(fact_id)
    }

    /// Get all facts of a specific type
    pub fn get_facts<T: Fact>(&self) -> Vec<Arc<FactHandle>> {
        self.working_memory.get_by_type::<T>()
    }

    /// Set focus to an agenda group
    pub fn focus(&mut self, group: impl Into<String>) -> Result<&mut Self> {
        self.agenda.set_focus(group.into())?;
        Ok(self)
    }

    /// Halt execution
    pub fn halt(&mut self) {
        self.halted = true;
    }

    /// Check if execution has been halted
    pub fn is_halted(&self) -> bool {
        self.halted
    }

    /// Match and fire rules once
    pub async fn match_rules(&mut self) -> Result<usize> {
        let mut fired_count = 0;

        while !self.agenda.is_empty() && !self.halted {
            if let Some(activation) = self.agenda.pop() {
                activation.rule.fire(self, &activation.match_data)?;
                fired_count += 1;
            }
        }

        Ok(fired_count)
    }

    /// Match and fire rules until halt is called
    pub async fn match_until_halt(&mut self) -> Result<usize> {
        let mut fired_count = 0;

        while !self.halted {
            if let Some(activation) = self.agenda.pop() {
                activation.rule.fire(self, &activation.match_data)?;
                fired_count += 1;
            } else {
                // No more activations, wait a bit or break
                // In a real implementation, this might wait for new facts
                break;
            }
        }

        Ok(fired_count)
    }

    /// Dispose of this session
    pub fn dispose(&mut self) {
        self.working_memory.dispose();
        self.agenda.dispose();
    }

    /// Get the number of facts in working memory
    pub fn fact_count(&self) -> usize {
        self.working_memory.len()
    }

    /// Check if working memory is empty
    pub fn is_empty(&self) -> bool {
        self.working_memory.is_empty()
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        self.dispose();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agenda::ConflictResolution;

    #[derive(Debug, Clone)]
    struct TestFact {
        value: i32,
    }

    #[test]
    fn test_session_creation() {
        let root = Arc::new(RwLock::new(RootNode::new()));
        let session = Session::new("test".to_string(), root, vec![ConflictResolution::Salience]);

        assert_eq!(session.flow_name(), "test");
        assert!(session.is_empty());
    }

    #[test]
    fn test_assert_and_retract() {
        let root = Arc::new(RwLock::new(RootNode::new()));
        let mut session = Session::new("test".to_string(), root, vec![ConflictResolution::Salience]);

        let fact = TestFact { value: 42 };
        let id = session.assert(fact).unwrap();

        assert_eq!(session.fact_count(), 1);

        session.retract(id).unwrap();
        assert_eq!(session.fact_count(), 0);
    }

    #[test]
    fn test_get_facts_by_type() {
        let root = Arc::new(RwLock::new(RootNode::new()));
        let mut session = Session::new("test".to_string(), root, vec![ConflictResolution::Salience]);

        session.assert(TestFact { value: 1 }).unwrap();
        session.assert(TestFact { value: 2 }).unwrap();

        let facts = session.get_facts::<TestFact>();
        assert_eq!(facts.len(), 2);
    }
}
