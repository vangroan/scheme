//! Token stream.
use std::iter::Peekable;

use crate::{
    error::Error,
    lex::{Lexer, LexerIter},
    token::{Token, TokenKind},
};

pub struct TokenStream<'a> {
    lexer: Peekable<LexerIter<'a>>,
}

impl<'a> TokenStream<'a> {
    pub fn from_lexer(lexer: Lexer<'a>) -> Self {
        Self {
            lexer: lexer.into_iter().peekable(),
        }
    }

    #[inline]
    pub fn next_token(&mut self) -> Option<Token> {
        self.lexer.next()
    }

    /// Return the current token and advance the cursor.
    ///
    /// The consumed token must match the given token type, otherwise
    /// a parsing error is returned.
    ///
    /// # Errors
    ///
    /// Returns a [`TokenError`] if the token kind doesn't match.
    pub fn consume(&mut self, token_kind: TokenKind) -> Result<Token, Error> {
        // We should not consume the token if the types don't match.
        match self.lexer.peek() {
            Some(token) => {
                if token.kind == token_kind {
                    self.lexer.next().ok_or(Error::UnexpectedEOF)
                } else {
                    Err(Error::TokenError {
                        expected: token_kind,
                        actual: token.kind,
                    })
                }
            }
            None => Err(Error::UnexpectedEOF),
        }
    }

    #[inline]
    pub fn at_end(&mut self) -> bool {
        match self.peek_kind() {
            Some(TokenKind::EOF) | None => true,
            Some(_) => false,
        }
    }

    /// Return the next token without advancing the cursor.
    #[inline]
    pub fn peek(&mut self) -> Option<&Token> {
        self.lexer.peek()
    }

    /// Return the current token kind without advancing the cursor.
    #[inline]
    pub fn peek_kind(&mut self) -> Option<TokenKind> {
        self.lexer.peek().map(|token| token.kind)
    }
}
