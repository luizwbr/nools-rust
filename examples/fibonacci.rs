//! Fibonacci sequence calculation using rules

use nools::prelude::*;
use nools::pattern::ObjectPattern;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
struct Fibonacci {
    sequence: u32,
    value: i64,
}

#[derive(Debug, Clone)]
struct Result {
    value: i64,
}

#[tokio::main]
async fn main() -> nools::error::Result<()> {
    println!("Fibonacci Rules Engine Example\n");

    // Calculate Fibonacci for sequence 10
    calculate_fibonacci(10).await?;

    Ok(())
}

async fn calculate_fibonacci(n: u32) -> nools::error::Result<()> {
    let mut flow = Flow::new("Fibonacci");
    let result = Arc::new(Mutex::new(Result { value: -1 }));
    let result_clone = Arc::clone(&result);

    // Bootstrap rule: Set value for sequence 1 and 2
    let bootstrap = Rule::new("Bootstrap")
        .when(
            Box::new(
                ObjectPattern::<Fibonacci>::new("f").with_filter(
                    |f| f.value == -1 && (f.sequence == 1 || f.sequence == 2),
                    "bootstrap condition",
                ),
            ) as Box<dyn Pattern>,
        )
        .then(move |_session, match_data| {
            if let Some(handle) = match_data.get("f") {
                if let Some(fib) = handle.downcast_ref::<Fibonacci>() {
                    println!("Bootstrap: sequence {} = 1", fib.sequence);
                    // In a full implementation, we would modify the fact
                }
            }
            Ok(())
        })
        .build()?;

    flow.add_rule(bootstrap)?;

    // Calculate rule: f3 = f1 + f2
    let calculate = Rule::new("Calculate")
        .when(
            Box::new(
                ObjectPattern::<Fibonacci>::new("f").with_filter(
                    |f| f.value != -1,
                    "has value",
                ),
            ) as Box<dyn Pattern>,
        )
        .then(move |_session, match_data| {
            if let Some(handle) = match_data.get("f") {
                if let Some(fib) = handle.downcast_ref::<Fibonacci>() {
                    println!("Calculate: sequence {} = {}", fib.sequence, fib.value);
                    let mut res = result_clone.lock().unwrap();
                    res.value = fib.value;
                }
            }
            Ok(())
        })
        .priority(10)
        .build()?;

    flow.add_rule(calculate)?;

    // Create session and assert initial facts
    let mut session = flow.session();

    // Assert Fibonacci numbers from 1 to n
    for i in 1..=n {
        session.assert(Fibonacci {
            sequence: i,
            value: if i <= 2 { 1 } else { -1 },
        })?;
    }

    // Match rules
    println!("\nMatching rules for Fibonacci({})...\n", n);
    let fired = session.match_rules().await?;

    println!("\nRules fired: {}", fired);
    let final_result = result.lock().unwrap();
    println!("Result: {}", final_result.value);

    Ok(())
}
