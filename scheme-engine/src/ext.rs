//! Extensions to built-in Rust types.

pub trait StringExt {
    fn first(&self) -> Option<char>;

    fn split_first_char(&self) -> Option<(char, &str)>;
}

impl<T: AsRef<str>> StringExt for T {
    #[inline(always)]
    fn first(&self) -> Option<char> {
        self.as_ref().chars().next()
    }

    #[inline(always)]
    fn split_first_char(&self) -> Option<(char, &str)> {
        let s = self.as_ref();
        s.chars().next().map(|c| (c, &s[c.len_utf8()..]))
    }
}
