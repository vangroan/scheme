//! Unicode character scanner.
use std::str::CharIndices;

use crate::span::BytePos;

pub const EOF_CHAR: char = '\0';

pub struct Cursor<'a> {
    chars: CharIndices<'a>,
    /// Previous character returned by the internal iterator.
    ///
    /// Store the result of the previous iteration so it's
    /// available on demand as the "current" state of the cursor.
    ///
    /// First tuple field is the unicode character position.
    prev: (u32, char),
    /// Original size of source passed in.
    orig_size: u32,
}

impl<'a> Cursor<'a> {
    /// Construct cursor from given source code text.
    pub fn new(source: &'a str) -> Self {
        Cursor {
            chars: source.char_indices(),
            prev: (0, EOF_CHAR),
            orig_size: source.len() as u32,
        }
    }

    /// Byte offset of the current character.
    #[inline]
    pub fn offset(&self) -> BytePos {
        BytePos(self.prev.0)
    }

    /// Current unicode position and character in the iteration.
    ///
    /// If iteration has not started, will return end-of-file character.
    #[inline]
    pub fn current(&self) -> (u32, char) {
        self.prev
    }

    /// Current position of unicode character in the iteration.
    #[inline]
    pub fn pos(&self) -> u32 {
        self.prev.0
    }

    /// Current unicode character in the iteration.
    #[inline]
    pub fn char(&self) -> char {
        self.prev.1
    }

    /// Peek the next character without advancing the cursor.
    #[inline]
    pub fn peek(&self) -> char {
        let mut iter = self.chars.clone();
        iter.next().map(|(_, c)| c).unwrap_or(EOF_CHAR)
    }

    /// Peek two characters ahead without advancing the cursor.
    #[allow(dead_code)]
    pub fn peek2(&self) -> char {
        let mut iter = self.chars.clone();
        iter.next();
        iter.next().map(|(_, c)| c).unwrap_or(EOF_CHAR)
    }

    /// Peek the byte position of the next character.
    pub fn peek_offset(&self) -> BytePos {
        // Byte position of next character is determined by number
        // of bytes taken up by the current character.
        //
        // Because of UTF-8 encoding, there is no easy way
        // to know the size of the current character except
        // advancing the iterator.
        let mut iter = self.chars.clone();
        iter.next()
            .map(|(index, _)| BytePos(index as u32))
            .unwrap_or_else(|| BytePos(self.orig_size))
    }

    /// Original length of source text before cursor moved.
    pub fn original_length(&self) -> u32 {
        self.orig_size
    }

    /// Indicates whether the cursor is at the end of the source.
    pub fn at_end(&self) -> bool {
        // The iterator may be exhausted, there could be a previous
        // character stored in the state.
        //
        // Cursor is only considered at end when last character is
        // overwritten with EOF.
        match self.prev {
            (_, EOF_CHAR) => {
                let mut iter = self.chars.clone();
                iter.next().is_none()
            }
            _ => false,
        }
    }

    /// Advances the cursor to the next character.
    ///
    /// Returns `None` if the cursor is end-of-file.
    pub fn bump(&mut self) -> Option<(BytePos, char)> {
        match self.chars.next() {
            Some((i, c)) => {
                // Convert index to smaller integer so
                // tuple fits into 64-bits.
                let i = i as u32;
                self.prev = (i, c);
                Some((BytePos(i), c))
            }
            None => {
                // Point the internal byte offset to one
                // element after the source text, so calls
                // to `offset` and `current` show that the
                // cursor is exhausted.
                self.prev = (self.orig_size, EOF_CHAR);
                None
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_peek() {
        let mut cursor = Cursor::new("abcd");
        assert_eq!(cursor.peek(), 'a');
        assert_eq!(cursor.peek2(), 'b');

        assert_eq!(cursor.bump(), Some((BytePos(0), 'a')));
        assert_eq!(cursor.bump(), Some((BytePos(1), 'b')));

        assert_eq!(cursor.peek(), 'c');
        assert_eq!(cursor.peek2(), 'd');

        assert_eq!(cursor.bump(), Some((BytePos(2), 'c')));

        assert_eq!(cursor.peek(), 'd');
        assert_eq!(cursor.peek2(), EOF_CHAR);

        assert_eq!(cursor.bump(), Some((BytePos(3), 'd')));

        assert_eq!(cursor.peek(), EOF_CHAR);
        assert_eq!(cursor.peek2(), EOF_CHAR);
    }

    #[test]
    fn test_eof() {
        assert_eq!(Cursor::new("").at_end(), true);
        assert_eq!(Cursor::new("abc").at_end(), false);

        // Exhausted cursor must return EOF
        let mut cursor = Cursor::new("a");
        // Initial state
        assert_eq!(cursor.char(), EOF_CHAR);
        assert_eq!(cursor.offset(), 0);
        cursor.bump();
        assert_eq!(cursor.char(), 'a');
        assert_eq!(cursor.offset(), 0);
        cursor.bump();
        assert_eq!(cursor.char(), EOF_CHAR);
        assert_eq!(cursor.offset(), 1);

        // Test case where string has explicit EOF sentinal.
        let mut cursor = Cursor::new("abc\0");
        assert_eq!(cursor.bump(), Some((BytePos(0), 'a')));
        assert_eq!(cursor.char(), 'a');
        assert_eq!(cursor.offset(), 0);

        assert_eq!(cursor.bump(), Some((BytePos(1), 'b')));
        assert_eq!(cursor.char(), 'b');
        assert_eq!(cursor.offset(), 1);

        assert_eq!(cursor.bump(), Some((BytePos(2), 'c')));
        assert_eq!(cursor.char(), 'c');
        assert_eq!(cursor.offset(), 2);

        assert_eq!(cursor.bump(), Some((BytePos(3), EOF_CHAR)));
        assert_eq!(cursor.char(), EOF_CHAR); // explicit
        assert_eq!(cursor.offset(), 3);

        assert_eq!(cursor.bump(), None);
        assert_eq!(cursor.char(), EOF_CHAR); // implicit
        assert_eq!(cursor.offset(), 4);
    }
}
