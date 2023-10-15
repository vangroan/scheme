use std::rc::Rc;

use crate::{
    error::{Error, Result},
    expr::Expr,
    lexer::Lexer,
    token::{Token, TokenKind},
};

pub fn parse(source: &str) -> Result<Expr> {
    let mut lexer = Lexer::new(source);

    // Position the lexer so the current token points to the first token.
    lexer.next_token();

    parse_expr(&mut lexer)
}

fn parse_expr(lexer: &mut Lexer) -> Result<Expr> {
    println!("parse_expr({:?})", lexer.rest());

    let token = lexer
        .current_token()
        .cloned()
        .ok_or_else(|| Error::Reason(format!("unexpected end")))?;
    lexer.next_token();

    match token.kind {
        TokenKind::LeftParen => parse_sequence(lexer),
        TokenKind::RightParen => Err(Error::Reason("unexpected right parentheses".to_string())),
        TokenKind::QuoteMark => parse_expr(lexer).map(Box::new).map(Expr::Quote),
        _ => {
            let fragment = token.fragment(lexer.source());
            parse_atom(token.clone(), fragment)
        }
    }
}

fn parse_sequence(lexer: &mut Lexer) -> Result<Expr> {
    println!("parse_sequence({:?})", lexer.rest());

    let mut exprs = Vec::new();

    while let Some(token) = lexer.current_token() {
        if token.kind == TokenKind::RightParen {
            lexer.next_token();
            break;
        }

        let expr = parse_expr(lexer)?;
        exprs.push(expr);
    }

    Ok(Expr::List(exprs))
}

fn parse_atom(token: Token, fragment: &str) -> Result<Expr> {
    println!("parse_atom({:?}, {:?})", token, fragment);

    use TokenKind::*;
    debug_assert_eq!(token.kind, Atom);

    let mut chars = fragment.chars();
    let ch = chars
        .next()
        .ok_or_else(|| Error::Reason("expected atom".to_string()))?;

    match ch {
        '0'..='9' => parse_number(token, fragment),
        '#' => match chars.next() {
            Some('t') => Ok(Expr::Bool(true)),
            Some('f') => Ok(Expr::Bool(false)),
            Some('b') => todo!("parse binary number"),
            Some('x') => todo!("parse hexadecimal number"),
            _ => todo!("unexpected character"),
        },
        _ => Err(Error::Reason(format!("unexpected character: {ch:?}"))),
    }
}

fn parse_number(token: Token, fragment: &str) -> Result<Expr> {
    let number = fragment
        .parse::<f64>()
        .map_err(|err| Error::Reason(format!("failed to parse number: {err}")))?;

    Ok(Expr::Number(number))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_numbers() {
        let expr = parse("(1 2 (3 4) (5 (6 7 8)))").expect("parse failed");
        println!("{:#?}", expr);

        let list1 = expr.as_list().unwrap();
        assert_eq!(list1[0], Expr::Number(1.0));
        assert_eq!(list1[1], Expr::Number(2.0));

        let list2 = list1[2].as_list().unwrap();
        assert_eq!(list2[0], Expr::Number(3.0));
        assert_eq!(list2[1], Expr::Number(4.0));

        let list3 = list1[3].as_list().unwrap();
        assert_eq!(list3[0], Expr::Number(5.0));

        let list4 = list3[1].as_list().unwrap();
        assert_eq!(list4[0], Expr::Number(6.0));
        assert_eq!(list4[1], Expr::Number(7.0));
        assert_eq!(list4[2], Expr::Number(8.0));
    }

    #[test]
    fn test_boolean() {
        let expr = parse("(#t #f)").expect("parse failed");

        let list = expr.as_list().unwrap();
        assert_eq!(list[0], Expr::Bool(true));
        assert_eq!(list[1], Expr::Bool(false));
    }
}
