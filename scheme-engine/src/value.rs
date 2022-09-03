//! Dynamically typed value.
use std::fmt;

use smol_str::SmolStr;

pub enum Value {
    Nil,
    Symbol(SmolStr),
    Number(f64),
    Quote(Box<Value>),
    Sexpr(Vec<Value>),
}

impl Value {
    /// Create a string representation of the value.
    pub fn repr(&self) -> String {
        todo!()
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        todo!()
    }
}
