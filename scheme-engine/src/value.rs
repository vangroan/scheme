//! Dynamically typed value.
use std::{fmt, rc::Rc};

use smol_str::SmolStr;

pub enum Value {
    Nil,
    Cell(Cell),
    Symbol(SmolStr),
    Number(f64),
    Quote(Box<Value>),
    Sexpr(Vec<Value>),
}

impl Value {
    fn repr(&self) -> String {
        let mut buf = String::new();
        self.repr_recurse(&mut buf);
        buf
    }

    /// Create a string representation of the value.
    fn repr_recurse(&self, buf: &mut String) {
        match self {
            Self::Nil => buf.push_str("'()"),
            Self::Symbol(s) => buf.push_str(s.as_str()),
            Self::Quote(val) => val.repr_recurse(buf),
            _ => todo!(),
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.repr())
    }
}

/// Node in a linked list.
pub struct Cell {
    pub left: Rc<Value>,
    pub right: Rc<Value>,
}
