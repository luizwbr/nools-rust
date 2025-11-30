//! Constraint evaluation for pattern matching

use crate::error::Result;
use crate::fact::FactHandle;
use std::fmt::Debug;
use std::sync::Arc;

/// A constraint that can be evaluated against facts
pub trait Constraint: Debug + Send + Sync {
    /// Evaluate this constraint against a fact
    fn evaluate(&self, fact: &FactHandle, context: &ConstraintContext) -> Result<bool>;

    /// Clone this constraint into a box
    fn clone_box(&self) -> Box<dyn Constraint>;
}

/// Context for constraint evaluation
#[derive(Debug, Default, Clone)]
pub struct ConstraintContext {
    /// Variables bound during pattern matching
    pub bindings: std::collections::HashMap<String, Arc<FactHandle>>,
}

impl ConstraintContext {
    /// Create a new empty context
    pub fn new() -> Self {
        Self::default()
    }

    /// Get a binding by name
    pub fn get(&self, name: &str) -> Option<&Arc<FactHandle>> {
        self.bindings.get(name)
    }

    /// Set a binding
    pub fn set(&mut self, name: String, fact: Arc<FactHandle>) {
        self.bindings.insert(name, fact);
    }

    /// Clone the context
    pub fn clone_bindings(&self) -> Self {
        Self {
            bindings: self.bindings.clone(),
        }
    }
}

/// A constraint defined by a closure
#[derive(Clone)]
pub struct FunctionConstraint<F> {
    func: Arc<F>,
    description: String,
}

impl<F> FunctionConstraint<F>
where
    F: Fn(&FactHandle, &ConstraintContext) -> bool + Send + Sync + 'static,
{
    /// Create a new function constraint
    pub fn new(func: F, description: impl Into<String>) -> Self {
        Self {
            func: Arc::new(func),
            description: description.into(),
        }
    }
}

impl<F> Debug for FunctionConstraint<F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FunctionConstraint")
            .field("description", &self.description)
            .finish()
    }
}

impl<F> Constraint for FunctionConstraint<F>
where
    F: Fn(&FactHandle, &ConstraintContext) -> bool + Send + Sync + 'static,
{
    fn evaluate(&self, fact: &FactHandle, context: &ConstraintContext) -> Result<bool> {
        Ok((self.func)(fact, context))
    }

    fn clone_box(&self) -> Box<dyn Constraint> {
        Box::new(FunctionConstraint {
            func: self.func.clone(),
            description: self.description.clone(),
        })
    }
}

/// Combines multiple constraints with AND logic
#[derive(Debug, Clone)]
pub struct AndConstraint {
    constraints: Vec<Box<dyn Constraint>>,
}

impl AndConstraint {
    /// Create a new AND constraint
    pub fn new(constraints: Vec<Box<dyn Constraint>>) -> Self {
        Self { constraints }
    }
}

impl Constraint for AndConstraint {
    fn evaluate(&self, fact: &FactHandle, context: &ConstraintContext) -> Result<bool> {
        for constraint in &self.constraints {
            if !constraint.evaluate(fact, context)? {
                return Ok(false);
            }
        }
        Ok(true)
    }

    fn clone_box(&self) -> Box<dyn Constraint> {
        Box::new(Self {
            constraints: self.constraints.iter().map(|c| c.clone_box()).collect(),
        })
    }
}

/// Combines multiple constraints with OR logic
#[derive(Debug, Clone)]
pub struct OrConstraint {
    constraints: Vec<Box<dyn Constraint>>,
}

impl OrConstraint {
    /// Create a new OR constraint
    pub fn new(constraints: Vec<Box<dyn Constraint>>) -> Self {
        Self { constraints }
    }
}

impl Constraint for OrConstraint {
    fn evaluate(&self, fact: &FactHandle, context: &ConstraintContext) -> Result<bool> {
        for constraint in &self.constraints {
            if constraint.evaluate(fact, context)? {
                return Ok(true);
            }
        }
        Ok(false)
    }

    fn clone_box(&self) -> Box<dyn Constraint> {
        Box::new(Self {
            constraints: self.constraints.iter().map(|c| c.clone_box()).collect(),
        })
    }
}

/// Negates a constraint
#[derive(Debug, Clone)]
pub struct NotConstraint {
    constraint: Box<dyn Constraint>,
}

impl NotConstraint {
    /// Create a new NOT constraint
    pub fn new(constraint: Box<dyn Constraint>) -> Self {
        Self { constraint }
    }
}

impl Constraint for NotConstraint {
    fn evaluate(&self, fact: &FactHandle, context: &ConstraintContext) -> Result<bool> {
        Ok(!self.constraint.evaluate(fact, context)?)
    }

    fn clone_box(&self) -> Box<dyn Constraint> {
        Box::new(Self {
            constraint: self.constraint.clone_box(),
        })
    }
}

// Implement Clone for Box<dyn Constraint>
impl Clone for Box<dyn Constraint> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fact::{Fact, FactHandle};

    #[derive(Debug, Clone)]
    struct TestFact {
        value: i32,
    }

    #[test]
    fn test_function_constraint() {
        let fact = TestFact { value: 42 };
        let handle = FactHandle::new(fact, 0);
        let context = ConstraintContext::new();

        let constraint = FunctionConstraint::new(
            |fact: &FactHandle, _ctx| {
                fact.downcast_ref::<TestFact>()
                    .map(|f| f.value > 40)
                    .unwrap_or(false)
            },
            "value > 40",
        );

        assert!(constraint.evaluate(&handle, &context).unwrap());
    }

    #[test]
    fn test_and_constraint() {
        let fact = TestFact { value: 42 };
        let handle = FactHandle::new(fact, 0);
        let context = ConstraintContext::new();

        let c1 = Box::new(FunctionConstraint::new(
            |fact: &FactHandle, _ctx| {
                fact.downcast_ref::<TestFact>()
                    .map(|f| f.value > 40)
                    .unwrap_or(false)
            },
            "value > 40",
        )) as Box<dyn Constraint>;

        let c2 = Box::new(FunctionConstraint::new(
            |fact: &FactHandle, _ctx| {
                fact.downcast_ref::<TestFact>()
                    .map(|f| f.value < 50)
                    .unwrap_or(false)
            },
            "value < 50",
        )) as Box<dyn Constraint>;

        let and = AndConstraint::new(vec![c1, c2]);
        assert!(and.evaluate(&handle, &context).unwrap());
    }
}
