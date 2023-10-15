//! Tests for numbers.
use scheme_engine::Env;

#[test]
fn test_add() {
    let expr = scheme_engine::parse(include_str!("test_number.sch")).expect("parse failed");

    let mut env = Env::new();
    scheme_engine::init_core(&mut env).expect("init core");
    scheme_engine::compile(env, &expr).expect("compile failed");
}
