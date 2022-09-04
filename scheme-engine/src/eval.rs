use std::{collections::HashMap, rc::Rc};

use smol_str::SmolStr;

use crate::{lex::Lexer, parse::parse_value, token_stream::TokenStream, value::Value};

pub fn eval_source(source: &str) {
    let lexer = Lexer::new(source);
    let mut tokens = TokenStream::from_lexer(lexer);

    while !tokens.at_end() {
        let value = parse_value(&mut tokens);
    }
}

pub struct Env {
    pub var: HashMap<SmolStr, Rc<Value>>,
    pub funcs: HashMap<SmolStr, ()>,
}
