use criterion::{black_box, criterion_group, criterion_main, Criterion};
use nools::prelude::*;
use nools::pattern::ObjectPattern;

#[derive(Debug, Clone)]
struct Fibonacci {
    sequence: u32,
    value: i64,
}

fn create_fibonacci_flow() -> Flow {
    let mut flow = Flow::new("Fibonacci Benchmark");

    let bootstrap = Rule::new("Bootstrap")
        .when(
            Box::new(
                ObjectPattern::<Fibonacci>::new("f").with_filter(
                    |f| f.value == -1 && (f.sequence == 1 || f.sequence == 2),
                    "bootstrap",
                ),
            ) as Box<dyn Pattern>,
        )
        .then(|_session, _match_data| Ok(()))
        .build()
        .unwrap();

    flow.add_rule(bootstrap).unwrap();
    flow
}

fn benchmark_fibonacci(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("fibonacci_10", |b| {
        b.iter(|| {
            rt.block_on(async {
                let flow = create_fibonacci_flow();
                let mut session = flow.session();

                for i in 1..=black_box(10) {
                    session
                        .assert(Fibonacci {
                            sequence: i,
                            value: if i <= 2 { 1 } else { -1 },
                        })
                        .unwrap();
                }

                session.match_rules().await.unwrap();
            })
        })
    });

    c.bench_function("fibonacci_20", |b| {
        b.iter(|| {
            rt.block_on(async {
                let flow = create_fibonacci_flow();
                let mut session = flow.session();

                for i in 1..=black_box(20) {
                    session
                        .assert(Fibonacci {
                            sequence: i,
                            value: if i <= 2 { 1 } else { -1 },
                        })
                        .unwrap();
                }

                session.match_rules().await.unwrap();
            })
        })
    });
}

criterion_group!(benches, benchmark_fibonacci);
criterion_main!(benches);
