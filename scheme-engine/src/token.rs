//! Token Definition

use crate::span::BytePos;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum TokenKind {
    LeftParen,
    RightParen,
    Number,
    Symbol,
    String,
    QuoteMark,
    EOF,
}

pub struct Token {
    pub offset: BytePos,
    pub size: u32,
    pub kind: TokenKind,
}
