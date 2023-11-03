use criterion::{black_box, criterion_group, criterion_main, Criterion};
use scheme_engine::Expr;

fn fibonacci_benchmark(c: &mut Criterion) {
    let source = include_str!("fibonacci.scm");
    let env = scheme_engine::new_env().unwrap();
    let expr = scheme_engine::parse(source, true).unwrap();
    let program = scheme_engine::compile(env.clone(), &expr).unwrap();

    // Run program to define variables.
    scheme_engine::eval(program).expect("evaluating top-level fibonacci program");

    let fibonacci = env
        .borrow()
        .lookup_var("fib")
        .expect("variable 'fib' not found")
        .as_closure()
        .expect("variable is not a closure")
        .clone();
    let args: Vec<Expr> = vec![Expr::Number(20.0)];

    c.bench_function("fib 20", |b| {
        b.iter(|| scheme_engine::call(black_box(fibonacci.clone()), black_box(&args)))
    });
}

criterion_group!(benches, fibonacci_benchmark);
criterion_main!(benches);
