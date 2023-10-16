//! Core standard library.

use crate::env::Env;
use crate::error::{Error, Result};
use crate::expr::Expr;

pub fn init_core(env: &mut Env) -> Result<()> {
    env.bind_native_func("+", number_add)?;
    env.bind_native_func("-", number_sub)?;
    env.bind_native_func("*", number_mul)?;

    env.bind_native_func("and", boolean_and)?;
    env.bind_native_func("or", boolean_or)?;

    Ok(())
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

    Ok(Expr::Number(sum))
}

fn number_sub(_env: &mut Env, args: &[Expr]) -> Result<Expr> {
    let mut sum: f64 = 0.0;

    for (index, arg) in args.iter().enumerate() {
        match arg {
            Expr::Number(number) => sum -= number,
            _ => {
                return Err(Error::Reason(format!(
                    "expected argument {index} to be a number, but encountered {arg:?}"
                )))
            }
        }
    }

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
