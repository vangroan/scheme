use std::rc::Rc;

use crate::{
    error::{Error, Result},
    token::TokenKind as T,
    token_stream::TokenStream,
    value::{Cell, Value},
};

pub struct Parser {
    stack: Vec<()>,
}

pub fn parse_value(tokens: &mut TokenStream) -> Result<Value> {
    match tokens.next_token() {
        Some(token) => match token.kind {
            T::LeftParen => {
                // Parse linked list.
                debug_assert_eq!(token.kind, T::LeftParen);
                if tokens.peek_kind() == Some(T::RightParen) {
                    Ok(Value::Nil)
                } else {
                    let mut cell = Cell {
                        left: Rc::new(Value::Nil),
                        right: Rc::new(Value::Nil),
                    };

                    todo!("parse cell");

                    // Ok(Value::Cell(cell))
                }
            }
            T::RightParen => {
                todo!()
            }
            _ => todo!("token parsing not implemented yet: {:?}", token.kind),
        },
        None => Err(Error::UnexpectedEOF),
    }
}
