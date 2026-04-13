use std::ops::Range;

use std::{iter::Peekable, str::Chars};

#[derive(Debug, Clone)]
pub struct Source<'a> {
    pub file_name: String,
    pub input: &'a str,
}

impl<'a> Source<'a> {
    pub fn new(file_name: &str, input: &'a str) -> Self {
        Self {
            file_name: file_name.into(),
            input,
        }
    }

    #[inline]
    pub fn peekable_chars(&self) -> Peekable<Chars<'a>> {
        self.input.chars().peekable()
    }
}

#[derive(Debug, Clone)]
pub struct Cursor<'a> {
    pub input: &'a str,
    pub chars: Peekable<Chars<'a>>,
    pub pos: usize,
}

impl<'a> Cursor<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            input,
            chars: input.chars().peekable(),
            pos: 0,
        }
    }

    pub fn peekable_chars(&self) -> &Peekable<Chars<'a>> {
        &self.chars
    }
}

pub type Span = Range<usize>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Spanned<T> {
    pub node: T,
    pub span: Span,
}

impl<T> Spanned<T> {
    pub fn new(node: T, span: Span) -> Self {
        Self { node, span }
    }

    pub fn map<U>(self, f: impl FnOnce(T) -> U) -> Spanned<U> {
        Spanned {
            node: f(self.node),
            span: self.span,
        }
    }
}
