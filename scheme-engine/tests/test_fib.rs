#[test]
fn test_fibonacci_sequence() {
    let source = include_str!("fib.scm");
    let env = scheme_engine::new_env().unwrap();
    let expr = scheme_engine::parse(source, true).unwrap();
    let closure = scheme_engine::compile(env.clone(), &expr).unwrap();

    scheme_engine::eval(closure).expect("fibonacci sequence failed");
}
