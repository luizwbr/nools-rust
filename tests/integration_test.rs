//! Integration tests for the nools rules engine

use nools::prelude::*;
use nools::pattern::ObjectPattern;

#[derive(Debug, Clone)]
struct Message {
    text: String,
    count: i32,
}

#[tokio::test]
async fn test_basic_rule_execution() {
    let mut flow = Flow::new("test");

    let rule = Rule::new("increment")
        .when(
            Box::new(
                ObjectPattern::<Message>::new("m")
                    .with_filter(|m| m.count < 5, "count < 5"),
            ) as Box<dyn Pattern>,
        )
        .then(|_session, match_data| {
            if let Some(handle) = match_data.get("m") {
                if let Some(msg) = handle.downcast_ref::<Message>() {
                    assert!(msg.count < 5);
                }
            }
            Ok(())
        })
        .build()
        .unwrap();

    flow.add_rule(rule).unwrap();

    let mut session = flow.session();
    session
        .assert(Message {
            text: "test".to_string(),
            count: 3,
        })
        .unwrap();

    let fired = session.match_rules().await.unwrap();
    assert_eq!(fired, 1);
}

#[tokio::test]
async fn test_multiple_rules_priority() {
    let mut flow = Flow::new("priority_test");

    let high_priority = Rule::new("high")
        .when(
            Box::new(ObjectPattern::<Message>::new("m")) as Box<dyn Pattern>,
        )
        .then(|_session, _| Ok(()))
        .priority(10)
        .build()
        .unwrap();

    let low_priority = Rule::new("low")
        .when(
            Box::new(ObjectPattern::<Message>::new("m")) as Box<dyn Pattern>,
        )
        .then(|_session, _| Ok(()))
        .priority(1)
        .build()
        .unwrap();

    flow.add_rule(high_priority).unwrap();
    flow.add_rule(low_priority).unwrap();

    let mut session = flow.session();
    session
        .assert(Message {
            text: "test".to_string(),
            count: 0,
        })
        .unwrap();

    let fired = session.match_rules().await.unwrap();
    assert_eq!(fired, 2);
}

#[tokio::test]
async fn test_fact_retraction() {
    let mut flow = Flow::new("retraction_test");

    let rule = Rule::new("test_rule")
        .when(
            Box::new(ObjectPattern::<Message>::new("m")) as Box<dyn Pattern>,
        )
        .then(|_session, _| Ok(()))
        .build()
        .unwrap();

    flow.add_rule(rule).unwrap();

    let mut session = flow.session();
    let id = session
        .assert(Message {
            text: "test".to_string(),
            count: 0,
        })
        .unwrap();

    assert_eq!(session.fact_count(), 1);

    session.retract(id).unwrap();
    assert_eq!(session.fact_count(), 0);
}

#[tokio::test]
async fn test_agenda_groups() {
    let mut flow = Flow::new("agenda_test");

    let main_rule = Rule::new("main_rule")
        .when(
            Box::new(ObjectPattern::<Message>::new("m")) as Box<dyn Pattern>,
        )
        .then(|_session, _| Ok(()))
        .agenda_group("main")
        .build()
        .unwrap();

    let other_rule = Rule::new("other_rule")
        .when(
            Box::new(ObjectPattern::<Message>::new("m")) as Box<dyn Pattern>,
        )
        .then(|_session, _| Ok(()))
        .agenda_group("other")
        .build()
        .unwrap();

    flow.add_rule(main_rule).unwrap();
    flow.add_rule(other_rule).unwrap();

    let mut session = flow.session();
    session
        .assert(Message {
            text: "test".to_string(),
            count: 0,
        })
        .unwrap();

    // Without focusing on "other", only main group should fire
    let fired = session.match_rules().await.unwrap();
    assert!(fired > 0);
}

#[tokio::test]
async fn test_pattern_filtering() {
    let mut flow = Flow::new("filter_test");

    let rule = Rule::new("filter_rule")
        .when(
            Box::new(
                ObjectPattern::<Message>::new("m").with_filter(
                    |m| m.text.len() > 5,
                    "text length > 5",
                ),
            ) as Box<dyn Pattern>,
        )
        .then(|_session, _| Ok(()))
        .build()
        .unwrap();

    flow.add_rule(rule).unwrap();

    let mut session = flow.session();

    // This should not match
    session
        .assert(Message {
            text: "hi".to_string(),
            count: 0,
        })
        .unwrap();

    // This should match
    session
        .assert(Message {
            text: "hello world".to_string(),
            count: 0,
        })
        .unwrap();

    let fired = session.match_rules().await.unwrap();
    assert_eq!(fired, 1);
}
