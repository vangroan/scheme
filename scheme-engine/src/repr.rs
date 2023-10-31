//! External text representation.

use crate::expr::Keyword;
use crate::{Expr, Pair};
use std::fmt;
use std::fmt::Formatter;
use std::rc::Rc;

pub struct ExprRepr<'a> {
    expr: &'a Expr,
}

impl<'a> ExprRepr<'a> {
    pub(crate) const fn new(expr: &'a Expr) -> Self {
        Self { expr }
    }

    fn fmt_pair(&self, f: &mut Formatter, pair: &Pair) -> fmt::Result {
        if let (head, tail) = pair.split_first() {
            write!(f, "{}", ExprRepr::new(head))?;
            match tail {
                Expr::Nil => {}
                Expr::Pair(inner_pair) => {
                    write!(f, " ")?;
                    self.fmt_pair(f, &*inner_pair.borrow())?;
                }
                _ => write!(f, " . {}", ExprRepr { expr: tail })?,
            }
        }

        Ok(())
    }
}

impl<'a> fmt::Display for ExprRepr<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self.expr {
            Expr::Nil => write!(f, "'()"),
            Expr::Bool(boolean) => {
                if *boolean {
                    write!(f, "#t")
                } else {
                    write!(f, "#f")
                }
            }
            Expr::Number(number) => write!(f, "{number}"),
            Expr::String(string) => write!(f, "{string}"),
            Expr::Ident(name) => write!(f, "{name}"),
            Expr::Keyword(keyword) => match keyword {
                Keyword::Dot => write!(f, "."),
            },
            Expr::Pair(pair) => {
                write!(f, "(")?;
                self.fmt_pair(f, &*pair.borrow())?;
                write!(f, ")")?;
                Ok(())
            }
            Expr::List(list) => {
                write!(f, "(")?;
                for expr in list {
                    let repr = ExprRepr { expr };
                    write!(f, "{repr}")?;
                }
                write!(f, ")")?;
                Ok(())
            }
            Expr::Sequence(expressions) => {
                write!(f, "(")?;
                for expr in expressions {
                    let repr = ExprRepr { expr };
                    write!(f, "{repr}")?;
                }
                write!(f, ")")?;
                Ok(())
            }
            Expr::Procedure(procedure) => {
                write!(f, "<procedure {:?}>", Rc::as_ptr(procedure))
            }
            Expr::Closure(closure) => {
                write!(
                    f,
                    "<procedure {:?}>",
                    Rc::as_ptr(&closure.borrow().procedure_rc())
                )
            }
            Expr::NativeFunc(func) => {
                //  TODO!("keep Rust function name")
                write!(f, "<native-function>")
            }
            _ => todo!("expression type repr not implemented yet"),
        }
    }
}
