//! Fact representation and management

use std::any::{Any, TypeId};
use std::fmt::Debug;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

/// Unique identifier for facts in working memory
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct FactId(u64);

impl FactId {
    /// Create a new unique fact ID
    pub fn new() -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(1);
        FactId(COUNTER.fetch_add(1, Ordering::Relaxed))
    }

    /// Get the raw ID value
    pub fn as_u64(&self) -> u64 {
        self.0
    }
}

impl Default for FactId {
    fn default() -> Self {
        Self::new()
    }
}

/// Trait for types that can be used as facts in the rules engine
///
/// # Safety
/// Implementors must ensure that the type can be safely downcast using `Any`
pub trait Fact: Any + Debug + Send + Sync {
    /// Get a reference to this fact as `Any`
    fn as_any(&self) -> &dyn Any;

    /// Get a mutable reference to this fact as `Any`
    fn as_any_mut(&mut self) -> &mut dyn Any;

    /// Clone this fact into a box
    fn clone_fact(&self) -> Box<dyn Fact>;

    /// Get the type name of this fact
    fn type_name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }
}

// Blanket implementation for all types that are Clone + Debug + Send + Sync + 'static
impl<T> Fact for T
where
    T: Clone + Debug + Send + Sync + 'static,
{
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn clone_fact(&self) -> Box<dyn Fact> {
        Box::new(self.clone())
    }
}

/// A wrapper around a fact with metadata
#[derive(Debug, Clone)]
pub struct FactHandle {
    /// Unique identifier for this fact
    pub id: FactId,
    /// The actual fact data
    pub fact: Arc<dyn Fact>,
    /// Type ID for efficient type checking
    pub type_id: TypeId,
    /// Recency counter for conflict resolution
    pub recency: u64,
}

impl FactHandle {
    /// Create a new fact handle
    pub fn new<T: Fact>(fact: T, recency: u64) -> Self {
        Self {
            id: FactId::new(),
            type_id: TypeId::of::<T>(),
            fact: Arc::new(fact),
            recency,
        }
    }

    /// Try to downcast the fact to a specific type
    pub fn downcast_ref<T: Fact>(&self) -> Option<&T> {
        self.fact.as_any().downcast_ref::<T>()
    }

    /// Check if this fact is of a specific type
    pub fn is_type<T: Fact>(&self) -> bool {
        self.type_id == TypeId::of::<T>()
    }

    /// Get the type name of the contained fact
    pub fn type_name(&self) -> &'static str {
        self.fact.type_name()
    }
}

impl PartialEq for FactHandle {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for FactHandle {}

impl std::hash::Hash for FactHandle {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone)]
    struct TestFact {
        value: i32,
    }

    #[test]
    fn test_fact_id_uniqueness() {
        let id1 = FactId::new();
        let id2 = FactId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_fact_handle_creation() {
        let fact = TestFact { value: 42 };
        let handle = FactHandle::new(fact, 0);
        
        assert!(handle.is_type::<TestFact>());
        assert_eq!(handle.downcast_ref::<TestFact>().unwrap().value, 42);
    }

    #[test]
    fn test_fact_handle_type_checking() {
        let fact = TestFact { value: 42 };
        let handle = FactHandle::new(fact, 0);
        
        assert!(handle.is_type::<TestFact>());
        assert!(!handle.is_type::<String>());
    }
}
