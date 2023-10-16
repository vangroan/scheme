use scheme_engine::Expr;

#[test]
fn test_define() {
    let env = scheme_engine::new_env().unwrap();
    let expr = scheme_engine::parse(include_str!("language/define.scm")).unwrap();
    let closure = scheme_engine::compile(env.clone(), &expr).unwrap();
    scheme_engine::eval(closure).unwrap();

    let symbol_x = env.borrow().resolve_var("x").unwrap();
    let x = env.borrow().get_var(symbol_x).cloned().unwrap();
    assert_eq!(x, Expr::Number(3.0));
}
