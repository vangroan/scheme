//! Token Definition

use crate::span::BytePos;

#[derive(Debug, PartialEq, Eq)]
pub enum TokenKind {
    LeftParen,
    RightParen,
    Number,
    Symbol,
    String,
    EOF,
}

pub struct Token {
    pub offset: BytePos,
    pub size: u32,
    pub kind: TokenKind,
}
