//! State machine example with agenda groups

use nools::prelude::*;
use nools::pattern::ObjectPattern;

#[derive(Debug, Clone, PartialEq)]
enum StateValue {
    NotRun,
    Running,
    Finished,
}

#[derive(Debug, Clone)]
struct State {
    name: String,
    state: StateValue,
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("State Machine Example with Agenda Groups\n");

    let mut flow = Flow::new("State Machine");

    // Bootstrap rule
    let bootstrap = Rule::new("Bootstrap")
        .when(
            Box::new(
                ObjectPattern::<State>::new("a").with_filter(
                    |s| s.name == "A" && s.state == StateValue::NotRun,
                    "state A not run",
                ),
            ) as Box<dyn Pattern>,
        )
        .then(|_session, match_data| {
            if let Some(handle) = match_data.get("a") {
                if let Some(state) = handle.downcast_ref::<State>() {
                    println!("Bootstrap: {} -> Finished", state.name);
                }
            }
            Ok(())
        })
        .build()?;

    flow.add_rule(bootstrap)?;

    // A to B transition
    let a_to_b = Rule::new("A to B")
        .when(
            Box::new(
                ObjectPattern::<State>::new("a").with_filter(
                    |s| s.name == "A" && s.state == StateValue::Finished,
                    "state A finished",
                ),
            ) as Box<dyn Pattern>,
        )
        .then(|_session, match_data| {
            if let Some(handle) = match_data.get("a") {
                if let Some(state) = handle.downcast_ref::<State>() {
                    println!("Transition: {} -> B", state.name);
                }
            }
            Ok(())
        })
        .build()?;

    flow.add_rule(a_to_b)?;

    // B to C transition with auto-focus
    let b_to_c = Rule::new("B to C")
        .when(
            Box::new(
                ObjectPattern::<State>::new("b").with_filter(
                    |s| s.name == "B" && s.state == StateValue::Finished,
                    "state B finished",
                ),
            ) as Box<dyn Pattern>,
        )
        .then(|session, match_data| {
            if let Some(handle) = match_data.get("b") {
                if let Some(state) = handle.downcast_ref::<State>() {
                    println!("Transition: {} -> C", state.name);
                }
            }
            session.focus("B to D")?;
            Ok(())
        })
        .agenda_group("B to C")
        .auto_focus(true)
        .build()?;

    flow.add_rule(b_to_c)?;

    // B to D transition
    let b_to_d = Rule::new("B to D")
        .when(
            Box::new(
                ObjectPattern::<State>::new("b").with_filter(
                    |s| s.name == "B" && s.state == StateValue::Finished,
                    "state B finished",
                ),
            ) as Box<dyn Pattern>,
        )
        .then(|_session, match_data| {
            if let Some(handle) = match_data.get("b") {
                if let Some(state) = handle.downcast_ref::<State>() {
                    println!("Transition: {} -> D", state.name);
                }
            }
            Ok(())
        })
        .agenda_group("B to D")
        .build()?;

    flow.add_rule(b_to_d)?;

    // Create session
    let mut session = flow.session();

    println!("Initial states:");
    session.assert(State {
        name: "A".to_string(),
        state: StateValue::NotRun,
    })?;
    session.assert(State {
        name: "B".to_string(),
        state: StateValue::Finished,
    })?;
    session.assert(State {
        name: "C".to_string(),
        state: StateValue::NotRun,
    })?;
    session.assert(State {
        name: "D".to_string(),
        state: StateValue::NotRun,
    })?;

    println!("\nExecuting state machine...\n");
    let fired = session.match_rules().await?;
    println!("\nTotal transitions: {}", fired);

    Ok(())
}
