//! Pattern definitions for fact matching

use crate::constraint::{Constraint, ConstraintContext};
use crate::error::Result;
use crate::fact::{Fact, FactHandle};
use std::any::TypeId;
use std::fmt::Debug;
use std::marker::PhantomData;

/// A pattern that matches facts in working memory
pub trait Pattern: Debug + Send + Sync {
    /// Get the type ID this pattern matches
    fn type_id(&self) -> TypeId;

    /// Check if a fact matches this pattern
    fn matches(&self, fact: &FactHandle, context: &ConstraintContext) -> Result<bool>;

    /// Get the alias for this pattern
    fn alias(&self) -> &str;

    /// Clone this pattern into a box
    fn clone_box(&self) -> Box<dyn Pattern>;
}

/// An object pattern that matches facts of a specific type with constraints
#[derive(Clone)]
pub struct ObjectPattern<T: Fact> {
    /// Alias for this pattern in rules
    pub alias: String,
    /// Constraints to apply
    pub constraints: Vec<Box<dyn Constraint>>,
    /// Type marker
    _phantom: PhantomData<T>,
}

impl<T: Fact> ObjectPattern<T> {
    /// Create a new object pattern
    pub fn new(alias: impl Into<String>) -> Self {
        Self {
            alias: alias.into(),
            constraints: Vec::new(),
            _phantom: PhantomData,
        }
    }

    /// Add a constraint to this pattern
    pub fn with_constraint(mut self, constraint: Box<dyn Constraint>) -> Self {
        self.constraints.push(constraint);
        self
    }

    /// Add a function constraint
    pub fn with_filter<F>(self, f: F, description: impl Into<String>) -> Self
    where
        F: Fn(&T) -> bool + Send + Sync + 'static,
    {
        use crate::constraint::FunctionConstraint;
        let constraint = FunctionConstraint::new(
            move |fact: &FactHandle, _ctx| {
                fact.downcast_ref::<T>().map(&f).unwrap_or(false)
            },
            description,
        );
        self.with_constraint(Box::new(constraint))
    }
}

impl<T: Fact> Debug for ObjectPattern<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ObjectPattern")
            .field("alias", &self.alias)
            .field("type", &std::any::type_name::<T>())
            .field("constraints", &self.constraints)
            .finish()
    }
}

impl<T: Fact> Pattern for ObjectPattern<T> {
    fn type_id(&self) -> TypeId {
        TypeId::of::<T>()
    }

    fn matches(&self, fact: &FactHandle, context: &ConstraintContext) -> Result<bool> {
        // Check type
        if fact.type_id != self.type_id() {
            return Ok(false);
        }

        // Check all constraints
        for constraint in &self.constraints {
            if !constraint.evaluate(fact, context)? {
                return Ok(false);
            }
        }

        Ok(true)
    }

    fn alias(&self) -> &str {
        &self.alias
    }

    fn clone_box(&self) -> Box<dyn Pattern> {
        Box::new(ObjectPattern::<T> {
            alias: self.alias.clone(),
            constraints: self.constraints.iter().map(|c| c.clone_box()).collect(),
            _phantom: PhantomData,
        })
    }
}

/// A NOT pattern that checks for absence of matching facts
#[derive(Debug, Clone)]
pub struct NotPattern {
    /// The pattern to negate
    pub pattern: Box<dyn Pattern>,
}

impl NotPattern {
    /// Create a new NOT pattern
    pub fn new(pattern: Box<dyn Pattern>) -> Self {
        Self { pattern }
    }
}

impl Pattern for NotPattern {
    fn type_id(&self) -> TypeId {
        self.pattern.type_id()
    }

    fn matches(&self, fact: &FactHandle, context: &ConstraintContext) -> Result<bool> {
        // NOT pattern is handled differently in the Rete network
        // This is a placeholder
        Ok(!self.pattern.matches(fact, context)?)
    }

    fn alias(&self) -> &str {
        self.pattern.alias()
    }

    fn clone_box(&self) -> Box<dyn Pattern> {
        Box::new(self.clone())
    }
}

/// An EXISTS pattern that checks for existence of matching facts
#[derive(Debug, Clone)]
pub struct ExistsPattern {
    /// The pattern to check
    pub pattern: Box<dyn Pattern>,
}

impl ExistsPattern {
    /// Create a new EXISTS pattern
    pub fn new(pattern: Box<dyn Pattern>) -> Self {
        Self { pattern }
    }
}

impl Pattern for ExistsPattern {
    fn type_id(&self) -> TypeId {
        self.pattern.type_id()
    }

    fn matches(&self, fact: &FactHandle, context: &ConstraintContext) -> Result<bool> {
        // EXISTS pattern is handled differently in the Rete network
        // This is a placeholder
        self.pattern.matches(fact, context)
    }

    fn alias(&self) -> &str {
        self.pattern.alias()
    }

    fn clone_box(&self) -> Box<dyn Pattern> {
        Box::new(self.clone())
    }
}

// Implement Clone for Box<dyn Pattern>
impl Clone for Box<dyn Pattern> {
    fn clone(&self) -> Self {
        self.clone_box()
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
    fn test_object_pattern_matching() {
        let pattern = ObjectPattern::<TestFact>::new("test")
            .with_filter(|f| f.value > 40, "value > 40");

        let fact = TestFact { value: 42 };
        let handle = FactHandle::new(fact, 0);
        let context = ConstraintContext::new();

        assert!(pattern.matches(&handle, &context).unwrap());
    }

    #[test]
    fn test_object_pattern_type_check() {
        let pattern = ObjectPattern::<TestFact>::new("test");

        let fact = "not a test fact".to_string();
        let handle = FactHandle::new(fact, 0);
        let context = ConstraintContext::new();

        assert!(!pattern.matches(&handle, &context).unwrap());
    }
}
