//! Hello World example demonstrating basic rule matching

use nools::prelude::*;
use nools::pattern::ObjectPattern;

#[derive(Debug, Clone)]
struct Message {
    text: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Create a new flow
    let mut flow = Flow::new("Hello World");

    // Rule 1: Find messages containing "hello" and append " world"
    let rule1 = Rule::new("Hello")
        .when(
            Box::new(
                ObjectPattern::<Message>::new("m")
                    .with_filter(|m| m.text.contains("hello"), "text contains 'hello'"),
            ) as Box<dyn Pattern>,
        )
        .then(|session, match_data| {
            if let Some(handle) = match_data.get("m") {
                if let Some(msg) = handle.downcast_ref::<Message>() {
                    println!("Rule 'Hello' matched: {}", msg.text);
                    // In a real implementation, we would modify the fact
                    // For now, just print
                }
            }
            Ok(())
        })
        .build()?;

    flow.add_rule(rule1)?;

    // Rule 2: Find messages ending with "world"
    let rule2 = Rule::new("Goodbye")
        .when(
            Box::new(
                ObjectPattern::<Message>::new("m")
                    .with_filter(|m| m.text.ends_with("world"), "text ends with 'world'"),
            ) as Box<dyn Pattern>,
        )
        .then(|_session, match_data| {
            if let Some(handle) = match_data.get("m") {
                if let Some(msg) = handle.downcast_ref::<Message>() {
                    println!("Rule 'Goodbye' matched: {}", msg.text);
                }
            }
            Ok(())
        })
        .priority(5)
        .build()?;

    flow.add_rule(rule2)?;

    // Create a session and assert facts
    let mut session = flow.session();

    println!("Asserting facts...");
    session.assert(Message {
        text: "hello".to_string(),
    })?;
    session.assert(Message {
        text: "hello world".to_string(),
    })?;
    session.assert(Message {
        text: "goodbye".to_string(),
    })?;

    println!("\nMatching rules...");
    let fired = session.match_rules().await?;
    println!("\nTotal rules fired: {}", fired);

    Ok(())
}
