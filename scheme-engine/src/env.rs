//! Execution environment.
use crate::declare_id;
use crate::error::{Error, Result};
use crate::expr::{Expr, NativeFunc};
use crate::symbol::{SymbolId, SymbolTable};

declare_id!(pub struct ConstantId(u16));
declare_id!(pub struct LocalId(u8));

pub struct Env {
    /// Table of values which do not change during runtime.
    ///
    /// Includes literals like booleans, numbers and strings, but
    /// also defined procedures.
    constants: (),

    /// Table of values which can be mutated during runtime.
    ///
    /// Does not include procedures, but can include closure instances.
    variables: SymbolTable,
    var_values: Vec<Expr>,
}

impl Env {
    /// Create a new empty environment.
    pub fn new() -> Self {
        Env {
            constants: (),

            variables: SymbolTable::new(),
            var_values: Vec::new(),
        }
    }

    pub fn resolve_var(&self, name: &str) -> Option<SymbolId> {
        self.variables.resolve(name)
    }

    pub fn get_var(&self, symbol: SymbolId) -> Option<&Expr> {
        self.var_values.get(symbol.as_usize())
    }

    /// TODO: Store argument arity information so it can be validated on compile or at runtime.
    pub fn bind_native_func(&mut self, name: &str, func: NativeFunc) -> Result<SymbolId> {
        match self.variables.insert_unique(name) {
            Some(symbol) => {
                grow_table(&mut self.var_values, symbol.as_usize());
                self.var_values[symbol.as_usize()] = Expr::NativeFunc(func);
                Ok(symbol)
            }
            None => Err(Error::Reason(format!("variable already bound {name:?}"))),
        }
    }
}

fn grow_table<T: Default>(table: &mut Vec<T>, index: usize) {
    if index >= table.len() {
        table.extend((table.len()..index + 1).map(|_| T::default()));
    }
}