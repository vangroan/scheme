//! Core standard library.

use crate::env::Env;
use crate::error::{Error, Result};
use crate::expr::Expr;

pub fn init_core(env: &mut Env) -> Result<()> {
    env.bind_native_func("assert", ext_assert)?;

    env.bind_native_func("+", number_add)?;
    env.bind_native_func("-", number_sub)?;
    env.bind_native_func("*", number_mul)?;

    env.bind_native_func("=", boolean_eq)?;
    env.bind_native_func("not", boolean_not)?;
    env.bind_native_func("and", boolean_and)?;
    env.bind_native_func("or", boolean_or)?;

    Ok(())
}

/// There is no assert in Scheme. This is our own extension to assist with unit testing.
///
/// This must move to a library once they're implemented.
///
/// ```scheme
/// (assert <expr> <message>?)
/// ```
fn ext_assert(_env: &mut Env, args: &[Expr]) -> Result<Expr> {
    let expr = args
        .get(0)
        .ok_or_else(|| Error::Reason("expected assertion expression".to_string()))?;
    let msg = args.get(1); // optional

    if let Expr::Bool(false) = expr {
        match msg {
            Some(Expr::String(message)) => {
                Err(Error::Reason(format!("assertion error: {message}")))
            }
            Some(_) => {
                // TODO: to_string solution that's cogent with Scheme's specification.
                Err(Error::Reason("invalid assertion message type".to_string()))
            }
            None => Err(Error::Reason("assertion failed".to_string())),
        }
    } else {
        Ok(expr.clone())
    }
}

// ----------------------------------------------------------------------------
// Number

fn number_add(_env: &mut Env, args: &[Expr]) -> Result<Expr> {
    let mut sum: f64 = 0.0;

    for (index, arg) in args.iter().enumerate() {
        match arg {
            Expr::Number(number) => sum += number,
            _ => {
                return Err(Error::Reason(format!(
                    "expected argument {index} to be a number, but encountered {arg:?}"
                )))
            }
        }
    }

    println!("number_add -> {sum}");
    Ok(Expr::Number(sum))
}

fn number_sub(_env: &mut Env, args: &[Expr]) -> Result<Expr> {
    let mut sum: f64 = args
        .get(0)
        .ok_or_else(|| Error::Reason("wrong number of arguments passed to procedure".to_string()))?
        .as_number()
        .ok_or_else(|| {
            Error::Reason(format!(
                "expected first argument to be a number, but encountered {:?}",
                &args[0]
            ))
        })?;

    let rest = &args[1..];

    for (index, arg) in rest.iter().enumerate() {
        println!("arg [{index}] {arg:?}; sum -> {sum}");
        match arg {
            Expr::Number(number) => sum -= number,
            _ => {
                return Err(Error::Reason(format!(
                    "expected argument {index} to be a number, but encountered {arg:?}"
                )))
            }
        }
    }

    println!("number_sub -> {sum}");
    Ok(Expr::Number(sum))
}

fn number_mul(_env: &mut Env, args: &[Expr]) -> Result<Expr> {
    let mut sum: f64 = 1.0;

    for (index, arg) in args.iter().enumerate() {
        match arg {
            Expr::Number(number) => sum *= number,
            _ => {
                return Err(Error::Reason(format!(
                    "expected argument {index} to be a number, but encountered {arg:?}"
                )))
            }
        }
    }

    Ok(Expr::Number(sum))
}

// ----------------------------------------------------------------------------
// Boolean

// FIXME: This should be `number_eq` and type check for numbers.
fn boolean_eq(_env: &mut Env, args: &[Expr]) -> Result<Expr> {
    for ab in args.windows(2) {
        match ab {
            [] | [_] => break,
            [a, b, ..] => {
                if a != b {
                    return Ok(Expr::Bool(false));
                }
            }
        }
    }

    Ok(Expr::Bool(true))
}

fn boolean_not(_env: &mut Env, args: &[Expr]) -> Result<Expr> {
    if args.len() > 1 {
        Err(Error::Reason(
            "wrong number of arguments passed to procedure".to_string(),
        ))
    } else {
        let arg0 = &args[0];
        if let Expr::Bool(false) = arg0 {
            Ok(Expr::Bool(true))
        } else {
            Ok(Expr::Bool(false))
        }
    }
}

fn boolean_and(_env: &mut Env, args: &[Expr]) -> Result<Expr> {
    // Default return value if procedure has no arguments.
    let mut expr = &Expr::Bool(true);

    for (index, arg) in args.iter().enumerate() {
        if let Expr::Bool(false) = arg {
            // If any #f is encountered, return early.
            return Ok(Expr::Bool(false));
        } else {
            // Storing reference to argument to avoid cloning on each iteration.
            expr = arg;
        }
    }

    Ok(expr.clone())
}

fn boolean_or(_env: &mut Env, args: &[Expr]) -> Result<Expr> {
    todo!()
}
