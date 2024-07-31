use crate::{
    event::{Event, EventBuffer},
    token::Token,
    SyntaxKind,
};

pub mod cst;

/// General trait allowing callers to access the first and last tokens of any kind of node, even if
/// the exact token isn't referenced until multiple levels down.
pub(crate) trait TokenSpan {
    fn first_token(&self) -> Option<&Token>;
    fn last_token(&self) -> Option<&Token>;
}

/// General trait for constructing a value by reading events from the event
/// buffer. Every Node type implements this trait, and can be derived using
/// `#[derive(ReadFromEventBuf)]`.
pub(crate) trait ReadFromEventBuf {
    const KIND: SyntaxKind = SyntaxKind::TOMBSTONE;
    const IS_TOKEN: bool = Self::KIND.is_token();

    fn matches_kind(kind: SyntaxKind) -> bool {
        Self::KIND == kind
    }

    fn read_from<I: Iterator<Item = Event>>(buf: &mut EventBuffer<I>) -> Self;

    /// Like `read_from`, but allows the node to be missing from the
    /// buffer, in which case this method returns None.
    fn read_optional_from<I: Iterator<Item = Event>>(buf: &mut EventBuffer<I>) -> Option<Self>
    where
        Self: Sized,
    {
        if buf
            .peek()
            .is_some_and(|event| Self::matches_kind(event.kind()))
        {
            Some(Self::read_from(buf))
        } else {
            None
        }
    }
}

impl<T: ReadFromEventBuf> ReadFromEventBuf for Option<T> {
    const KIND: SyntaxKind = T::KIND;
    const IS_TOKEN: bool = T::IS_TOKEN;

    #[inline(always)]
    fn read_from<I: Iterator<Item = Event>>(buf: &mut EventBuffer<I>) -> Self {
        if T::IS_TOKEN {
            if matches!(buf.peek(), Some(Event::Token(_))) {
                Some(T::read_from(buf))
            } else {
                None
            }
        } else {
            T::read_optional_from(buf)
        }
    }
}

impl ReadFromEventBuf for Token {
    const IS_TOKEN: bool = true;

    #[inline(always)]
    fn read_from<I: Iterator<Item = Event>>(buf: &mut EventBuffer<I>) -> Self {
        buf.next_as_token()
    }
}

impl<T: ReadFromEventBuf> ReadFromEventBuf for Vec<T> {
    #[inline(always)]
    fn read_from<I: Iterator<Item = Event>>(buf: &mut EventBuffer<I>) -> Self {
        // Special-casing here ensures that flat lists of tokens can be read
        // iteratively and still stop at the next node boundary (either a Start
        // _or_ a Finish).
        if T::IS_TOKEN {
            let mut children = vec![];
            while matches!(buf.peek(), Some(Event::Token(_))) {
                children.push(T::read_from(buf));
            }
            children
        } else {
            let mut children = vec![];
            while !matches!(buf.peek(), None | Some(Event::Finish(_))) {
                children.push(T::read_from(buf));
            }
            children
        }
    }
}

impl<T: TokenSpan> TokenSpan for Vec<T> {
    fn first_token(&self) -> Option<&Token> {
        self.first().and_then(TokenSpan::first_token)
    }

    fn last_token(&self) -> Option<&Token> {
        self.last().and_then(TokenSpan::last_token)
    }
}

impl<T: TokenSpan> TokenSpan for Option<T> {
    fn first_token(&self) -> Option<&Token> {
        self.as_ref().and_then(TokenSpan::first_token)
    }

    fn last_token(&self) -> Option<&Token> {
        self.as_ref().and_then(TokenSpan::last_token)
    }
}
