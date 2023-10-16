//! Tests for numbers.
use scheme_engine::{Env, Handle};

#[test]
fn test_add() {
    let expr = scheme_engine::parse(include_str!("test_number.sch")).expect("parse failed");

    let mut env = Handle::new(Env::new());
    scheme_engine::init_core(&mut env.borrow_mut()).expect("init core");
    scheme_engine::compile(env, &expr).expect("compile failed");
}
