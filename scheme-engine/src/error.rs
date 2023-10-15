use crate::token::TokenKind;

pub type Result<T> = std::result::Result<T, self::Error>;

#[derive(Debug)]
pub enum Error {
    Reason(String),
    TokenError {
        expected: TokenKind,
        actual: TokenKind,
    },
    UnexpectedEOF,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Reason(message) => write!(f, "{}", message),
            Self::TokenError { expected, actual } => {
                write!(f, "token error: expected {:?} found {:?}", expected, actual)
            }
            Self::UnexpectedEOF => write!(f, "unexpected end-of-file"),
        }
    }
}
