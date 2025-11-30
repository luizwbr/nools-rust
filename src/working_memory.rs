//! Working memory for storing and indexing facts

use crate::error::{Error, Result};
use crate::fact::{Fact, FactHandle, FactId};
use std::any::TypeId;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::collections::HashMap;
use std::cell::RefCell;

/// Working memory stores all facts currently in the system
/// (WASM-compatible version using RefCell instead of DashMap)
#[derive(Debug)]
pub struct WorkingMemory {
    /// All facts indexed by their ID
    facts: RefCell<HashMap<FactId, Arc<FactHandle>>>,
    /// Facts indexed by type for efficient type-based queries
    facts_by_type: RefCell<HashMap<TypeId, Vec<Arc<FactHandle>>>>,
    /// Recency counter for conflict resolution
    recency: AtomicU64,
}

impl WorkingMemory {
    /// Create a new working memory
    pub fn new() -> Self {
        Self {
            facts: RefCell::new(HashMap::new()),
            facts_by_type: RefCell::new(HashMap::new()),
            recency: AtomicU64::new(0),
        }
    }

    /// Assert a new fact into working memory
    pub fn assert<T: Fact>(&self, fact: T) -> Result<Arc<FactHandle>> {
        let recency = self.recency.fetch_add(1, Ordering::SeqCst);
        let handle = Arc::new(FactHandle::new(fact, recency));
        let type_id = handle.type_id;
        let id = handle.id;

        // Store in main index
        self.facts.borrow_mut().insert(id, Arc::clone(&handle));

        // Store in type index
        self.facts_by_type
            .borrow_mut()
            .entry(type_id)
            .or_insert_with(Vec::new)
            .push(Arc::clone(&handle));

        Ok(handle)
    }

    /// Retract a fact from working memory
    pub fn retract(&self, fact_id: FactId) -> Result<Arc<FactHandle>> {
        let handle = self
            .facts
            .borrow_mut()
            .remove(&fact_id)
            .ok_or_else(|| Error::FactNotFound(format!("{:?}", fact_id)))?;

        // Remove from type index
        if let Some(facts) = self.facts_by_type.borrow_mut().get_mut(&handle.type_id) {
            facts.retain(|f| f.id != fact_id);
        }

        Ok(handle)
    }

    /// Modify a fact (retract and re-assert with updated recency)
    pub fn modify(&self, fact_id: FactId) -> Result<Arc<FactHandle>> {
        let old_handle = self.retract(fact_id)?;
        let recency = self.recency.fetch_add(1, Ordering::SeqCst);

        let new_handle = Arc::new(FactHandle {
            id: old_handle.id,
            fact: Arc::clone(&old_handle.fact),
            type_id: old_handle.type_id,
            recency,
        });

        // Re-insert with updated recency
        self.facts.borrow_mut().insert(fact_id, Arc::clone(&new_handle));

        self.facts_by_type
            .borrow_mut()
            .entry(new_handle.type_id)
            .or_insert_with(Vec::new)
            .push(Arc::clone(&new_handle));

        Ok(new_handle)
    }

    /// Get a fact by ID
    pub fn get(&self, fact_id: FactId) -> Option<Arc<FactHandle>> {
        self.facts.borrow().get(&fact_id).map(|h| Arc::clone(h))
    }

    /// Get all facts of a specific type
    pub fn get_by_type<T: Fact>(&self) -> Vec<Arc<FactHandle>> {
        let type_id = TypeId::of::<T>();
        self.facts_by_type
            .borrow()
            .get(&type_id)
            .map(|facts| facts.clone())
            .unwrap_or_default()
    }

    /// Get all facts
    pub fn get_all(&self) -> Vec<Arc<FactHandle>> {
        self.facts.borrow().values().map(|h| Arc::clone(h)).collect()
    }

    /// Get the number of facts in memory
    pub fn len(&self) -> usize {
        self.facts.borrow().len()
    }

    /// Check if working memory is empty
    pub fn is_empty(&self) -> bool {
        self.facts.borrow().is_empty()
    }

    /// Clear all facts from memory
    pub fn clear(&self) {
        self.facts.borrow_mut().clear();
        self.facts_by_type.borrow_mut().clear();
    }

    /// Dispose of working memory
    pub fn dispose(&self) {
        self.clear();
    }
}

impl Default for WorkingMemory {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone)]
    struct TestFact {
        value: i32,
    }

    #[derive(Debug, Clone)]
    struct OtherFact {
        name: String,
    }

    #[test]
    fn test_assert_and_get() {
        let wm = WorkingMemory::new();
        let fact = TestFact { value: 42 };
        let handle = wm.assert(fact).unwrap();

        let retrieved = wm.get(handle.id).unwrap();
        assert_eq!(retrieved.id, handle.id);
    }

    #[test]
    fn test_get_by_type() {
        let wm = WorkingMemory::new();
        wm.assert(TestFact { value: 1 }).unwrap();
        wm.assert(TestFact { value: 2 }).unwrap();
        wm.assert(OtherFact {
            name: "test".into(),
        })
        .unwrap();

        let test_facts = wm.get_by_type::<TestFact>();
        assert_eq!(test_facts.len(), 2);

        let other_facts = wm.get_by_type::<OtherFact>();
        assert_eq!(other_facts.len(), 1);
    }

    #[test]
    fn test_retract() {
        let wm = WorkingMemory::new();
        let handle = wm.assert(TestFact { value: 42 }).unwrap();
        let id = handle.id;

        assert_eq!(wm.len(), 1);

        wm.retract(id).unwrap();
        assert_eq!(wm.len(), 0);
        assert!(wm.get(id).is_none());
    }

    #[test]
    fn test_modify() {
        let wm = WorkingMemory::new();
        let handle = wm.assert(TestFact { value: 42 }).unwrap();
        let id = handle.id;
        let old_recency = handle.recency;

        let new_handle = wm.modify(id).unwrap();
        assert_eq!(new_handle.id, id);
        assert!(new_handle.recency > old_recency);
    }

    #[test]
    fn test_clear() {
        let wm = WorkingMemory::new();
        wm.assert(TestFact { value: 1 }).unwrap();
        wm.assert(TestFact { value: 2 }).unwrap();

        assert_eq!(wm.len(), 2);
        wm.clear();
        assert_eq!(wm.len(), 0);
    }
}
