//! Lexical analysis.
use crate::{
    cursor::{Cursor, EOF_CHAR},
    span::BytePos,
    token::{Token, TokenKind},
};

macro_rules! trace {
    ($($arg:tt)+) => {
        println!($($arg)+)
    };
}

pub struct Lexer<'a> {
    cursor: Cursor<'a>,
    /// Original source.
    source: &'a str,
    /// Byte position where the current token starts
    /// in the original source string.
    start_offset: BytePos,
}

impl<'a> Lexer<'a> {
    /// Create a new lexer from the given source code.
    pub fn new(source: &'a str) -> Self {
        let mut cursor = Cursor::new(source);

        // Initial state of the cursor is a non-existant EOF char,
        // but the initial state of the lexer should be a valid
        // token starting character.
        //
        // Prime the cursor for the first iteration.
        cursor.bump();
        let start_offset = cursor.offset();

        Self {
            cursor,
            source,
            start_offset,
        }
    }

    /// Original source passed into the lexer.
    #[inline]
    pub fn source(&self) -> &str {
        self.source
    }

    /// Indicates whether the lexer is at the end of the source.
    ///
    /// Note that source can contain '\0' (end-of-file) characters,
    /// but not be at the actual end. It's thus important to verify
    /// with this function whenever a [`TokenKind::EOF`] is encountered.
    pub fn at_end(&self) -> bool {
        self.cursor.at_end()
    }

    /// Primes the lexer to consume the next token.
    fn start_token(&mut self) {
        self.start_offset = self.cursor.offset();
    }

    fn make_token(&mut self, kind: TokenKind) -> Token {
        let start = self.start_offset.0 as u32;
        let end = self.cursor.peek_offset().0;

        // start and end can be equal, and a token can have 0 size.
        debug_assert!(end >= start);
        let size = end - start;

        // After this token is built, the lexer's internal state
        // is no longer dedicated to this iteration, but to preparing
        // for the next iteration.
        let token = Token {
            offset: self.start_offset,
            size,
            kind,
        };

        // Position the cursor to the starting character for the
        // next token, so the lexer's internal state is primed
        // for the next iteration.
        self.cursor.bump();

        token
    }

    /// Scan the source characters and construct the next token.
    pub fn next_token(&mut self) -> Token {
        // Shorter name for more readable match body.
        use TokenKind as T;

        trace!("before -> {:?}", self.cursor.current());
        // Discard whitespace
        while rules::is_whitespace(self.cursor.char()) {
            trace!("skip whitespace -> {:?}", self.cursor.current());
            self.cursor.bump();
        }

        // Invariant: The cursor must be pointing to the start of the
        //            remainder of the source to be consumed next.
        self.start_token();

        trace!("current -> {:?}", self.cursor.current());
        match self.cursor.char() {
            '(' => self.make_token(T::LeftParen),
            ')' => self.make_token(T::RightParen),
            '0'..='9' => self.consume_number(),
            '"' => {
                todo!("string")
            }
            '\'' => self.make_token(T::QuoteMark),
            EOF_CHAR => {
                // Source may contain a \0 character but not
                // actually be at the end of the stream.
                self.make_token(TokenKind::EOF)
            }
            c if rules::is_symbol(c) => self.consume_symbol(),
            _ => {
                // The source stream has run out, so we signal
                // the caller by emitting an end-of-file token that
                // doesn't exist in the text.
                //
                // The token's span thus points to the element
                // beyond the end of the collection, and has 0 length.
                todo!();
            }
        }
    }
}

/// Methods for consuming token types.
impl<'a> Lexer<'a> {
    fn consume_number(&mut self) -> Token {
        trace!("consume_number {:?}", self.cursor.current());
        debug_assert!(rules::is_symbol(self.cursor.char()));

        while rules::is_number(self.cursor.peek()) {
            self.cursor.bump();
        }

        self.make_token(TokenKind::Number)
    }

    fn consume_symbol(&mut self) -> Token {
        trace!("consume_symbol {:?}", self.cursor.current());
        debug_assert!(rules::is_symbol(self.cursor.char()));

        while rules::is_symbol(self.cursor.peek()) {
            self.cursor.bump();
        }

        self.make_token(TokenKind::Symbol)
    }
}

/// Functions for testing characters.
mod rules {
    /// Test whether the character is considered whitespace
    /// that should be ignored by the parser later.
    #[inline(always)]
    pub fn is_whitespace(c: char) -> bool {
        matches!(
            c,
            '\u{0020}' // space
            | '\u{0009}' // tab
            | '\u{000A}' // linefeed
            | '\u{000D}' // carriage return
            | '\u{00A0}' // no-break space
            | '\u{FEFF}' // zero width no-break space
        )
    }

    #[inline(always)]
    pub fn is_number(c: char) -> bool {
        matches!(c, '0'..='9')
    }

    #[inline(always)]
    pub fn is_symbol(c: char) -> bool {
        matches!(c, 'a'..='z' | 'A'..='Z' | '_')
    }
}

impl<'a> IntoIterator for Lexer<'a> {
    type Item = Token;
    type IntoIter = LexerIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        LexerIter {
            lexer: self,
            done: false,
        }
    }
}

/// Convenience iterator that wraps the lexer.
pub struct LexerIter<'a> {
    // Track end so an EOF token is emitted once.
    done: bool,
    lexer: Lexer<'a>,
}

impl<'a> Iterator for LexerIter<'a> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        if self.lexer.at_end() {
            if self.done {
                None
            } else {
                self.done = true;
                Some(self.lexer.next_token())
            }
        } else {
            Some(self.lexer.next_token())
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_rules() {
        assert!(!rules::is_symbol('('));
        assert!(!rules::is_symbol(')'));
        assert!(rules::is_symbol('c'));
        assert!(rules::is_symbol('a'));
    }

    #[test]
    fn test_lexer_pos() {
        let mut lexer = Lexer::new("(a b c)");
        assert_eq!(lexer.cursor.current(), (0, '('));
        assert_eq!(lexer.next_token().kind, TokenKind::LeftParen);
        assert_eq!(lexer.cursor.current(), (1, 'a')); //  rest: "a b c)"
        assert_eq!(lexer.next_token().kind, TokenKind::Symbol);
        assert_eq!(lexer.cursor.current(), (2, ' ')); // rest: " b c)"
        assert_eq!(lexer.next_token().kind, TokenKind::Symbol); // b
        assert_eq!(lexer.cursor.current(), (4, ' ')); // rest: " c)"
        assert_eq!(lexer.next_token().kind, TokenKind::Symbol); // c
        assert_eq!(lexer.cursor.current(), (6, ')')); // rest: ")"
        assert_eq!(lexer.next_token().kind, TokenKind::RightParen);
    }

    #[test]
    fn test_lexer() {}
}
