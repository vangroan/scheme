//! Location in source code.

/// Absolute byte position of a character in source code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BytePos(pub(crate) u32);

impl BytePos {
    pub fn from_u32(index: u32) -> BytePos {
        BytePos(index)
    }

    pub fn to_u32(self) -> u32 {
        self.0
    }
}

impl std::cmp::PartialEq<u32> for BytePos {
    fn eq(&self, other: &u32) -> bool {
        self.0 == *other
    }
}

impl std::fmt::Display for BytePos {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.0, f)
    }
}

pub struct Pos {
    pub offset: BytePos,
    pub column: u16,
    pub line: u16,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_span_type_sizes() {
        assert_eq!(std::mem::size_of::<BytePos>(), 4);
        assert_eq!(std::mem::size_of::<Pos>(), 8);
    }
}
