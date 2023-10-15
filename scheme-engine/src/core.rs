//! Core standard library.

use crate::env::Env;
use crate::error::{Error, Result};
use crate::expr::Expr;

pub fn init_core(env: &mut Env) -> Result<()> {
    env.bind_native_func("+", number_add)?;
    env.bind_native_func("-", number_sub)?;
    env.bind_native_func("*", number_mul)?;

    Ok(())
}

// ----------------------------------------------------------------------------
// Number

fn number_add(_env: &Env, args: &[Expr]) -> Result<Expr> {
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

fn number_sub(_env: &Env, args: &[Expr]) -> Result<Expr> {
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

fn number_mul(_env: &Env, args: &[Expr]) -> Result<Expr> {
    let mut sum: f64 = 0.0;

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
