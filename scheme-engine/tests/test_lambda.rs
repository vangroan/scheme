use scheme_engine::Expr;

/// Test that a lambda call works as expected, and that
/// the lambda's locals and constants don't leak into the
/// outer scope.
#[test]
fn test_lambda_call() {
    let source =
        r"(define add-self (lambda (x) (+ x x))) (add-self 7) (assert (= (add-self 7) 14))";

    let env = scheme_engine::new_env().expect("create core environment");
    let expr = scheme_engine::parse(source, true).expect("parse");
    let closure = scheme_engine::compile(env.clone(), &expr).expect("compile");
    println!("Top-level Closure: {closure:?}");

    let value = scheme_engine::eval(closure).expect("evaluation");
    assert_eq!(value, Expr::Number(14.0));

    assert_eq!(
        env.borrow().resolve_var("x"),
        None,
        "lambda local leaked to global env"
    );
}
