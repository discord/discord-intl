use std::fmt::Write;
use std::rc::Rc;

use arcstr::ArcStr;

use crate::token::{Token, TriviaList, TriviaPointer};

use super::{ICUMarkdownParser, SyntaxKind, SyntaxToken};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Marker {
    event_index: usize,
}

impl Marker {
    pub(crate) fn new(event_index: usize) -> Self {
        Self { event_index }
    }

    pub(crate) fn event_index(&self) -> usize {
        self.event_index
    }

    pub(crate) fn span_to(self, close: Marker) -> MarkerSpan {
        MarkerSpan::from_markers(self, close)
    }

    pub(crate) fn complete(self, p: &mut ICUMarkdownParser, kind: SyntaxKind) -> Option<()> {
        match p.get_event_mut(self.event_index) {
            Some(Event::Start(ref mut slot)) => *slot = kind,
            _ => unreachable!(),
        }

        p.push_event(Event::Finish(kind));
        Some(())
    }

    pub(crate) fn complete_as_start(self, p: &mut ICUMarkdownParser, kind: SyntaxKind) {
        match p.get_event_mut(self.event_index) {
            Some(event) => *event = Event::Start(kind),
            found => unreachable!("complete_as_start requires a Start or Finish event to be at the given index ({}), but found {:?} instead", self.event_index, found),
        }
    }

    pub(crate) fn complete_as_finish(self, p: &mut ICUMarkdownParser, kind: SyntaxKind) {
        match p.get_event_mut(self.event_index) {
            Some(Event::Token(_)) => unreachable!(),
            Some(event) => *event = Event::Finish(kind),
            found => unreachable!("complete_as_finish requires a Start or Finish event to be at the given index ({}), but found {:?} instead", self.event_index, found),
        }
    }
}

/// An expanded Marker representing two points, a beginning and an end, that
/// can be completed as a matching pair in a single go.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct MarkerSpan(Marker, Marker);

impl MarkerSpan {
    pub(crate) fn from_markers(open: Marker, close: Marker) -> Self {
        Self(open, close)
    }

    pub(crate) fn new(open_index: usize, close_index: usize) -> Self {
        Self(Marker::new(open_index), Marker::new(close_index))
    }

    #[inline(always)]
    pub(crate) fn complete(self, p: &mut ICUMarkdownParser, kind: SyntaxKind) {
        self.0.complete_as_start(p, kind);
        self.1.complete_as_finish(p, kind);
    }
}

#[derive(Clone, Debug)]
pub(crate) enum Event {
    Start(SyntaxKind),
    Finish(SyntaxKind),
    Token(SyntaxToken),
}

impl Event {
    pub(crate) fn tombstone() -> Self {
        Event::Start(SyntaxKind::TOMBSTONE)
    }

    pub(crate) fn kind(&self) -> SyntaxKind {
        match self {
            Event::Start(kind) => *kind,
            Event::Finish(kind) => *kind,
            Event::Token(token) => token.kind(),
        }
    }
}

impl std::fmt::Display for Event {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Event::Start(kind) => f.write_fmt(format_args!("Start: {:?}", kind)),
            Event::Finish(kind) => f.write_fmt(format_args!("Finish: {:?}", kind)),
            Event::Token(token) => f.write_fmt(format_args!("{:?}", token.kind())),
        }
    }
}

pub(crate) struct EventBuffer<I> {
    buffer: I,
    source: ArcStr,
    trivia_list: Rc<TriviaList>,
    trivia_cursor: usize,
    peeked: Option<Option<Event>>,
}

impl<I> EventBuffer<I>
where
    I: Iterator<Item = Event>,
{
    pub(crate) fn new(buffer: I, source: ArcStr, trivia_list: TriviaList) -> Self {
        Self {
            buffer,
            source,
            trivia_list: Rc::new(trivia_list),
            trivia_cursor: 0,
            peeked: None,
        }
    }

    pub(crate) fn next(&mut self) -> Option<Event> {
        match self.peeked.take() {
            Some(value) => value,
            None => self.buffer.next(),
        }
    }

    pub(crate) fn peek(&mut self) -> Option<&Event> {
        self.peeked
            .get_or_insert_with(|| self.buffer.next())
            .as_ref()
    }

    /// Consumes the next event from the buffer, asserts that it has a token,
    /// and returns that token.
    ///
    /// This method also consumes leading and trailing trivia.
    ///
    /// ## Panics
    ///
    /// Panics if the event does not contain a token (i.e., is a block event).
    pub(crate) fn next_as_token(&mut self) -> Token {
        match self.next() {
            Some(Event::Token(syntax_token)) => {
                let text = self.source.substr(syntax_token.span());
                let trivia_pointer = TriviaPointer::from_token(
                    &syntax_token,
                    &mut self.trivia_list,
                    &mut self.trivia_cursor,
                );
                Token::from_syntax(syntax_token, text, trivia_pointer)
            }
            found => panic!(
                "Attempted to read next event as a token, but got {:?} instead",
                found
            ),
        }
    }

    /// Consumes the next event from the buffer, asserts that it is a Start
    /// event for the given kind, and returns that event.
    ///
    /// ## Panics
    ///
    /// Panics if the event is not a matching Start event.
    pub(crate) fn next_as_start(&mut self) -> Event {
        match self.next() {
            Some(event @ Event::Start(_)) => event,
            found => panic!(
                "Attempted to read next event as a Start event, but got {:?} instead",
                found
            ),
        }
    }

    /// Consumes the next event from the buffer, asserts that it is a Finish
    /// event for the given kind, and returns that event.
    ///
    /// ## Panics
    ///
    /// Panics if the event is not a matching Finish event.
    pub(crate) fn next_as_finish(&mut self, expected_kind: SyntaxKind) -> Event {
        match self.next() {
            Some(event @ Event::Finish(_)) if event.kind() == expected_kind => event,
            found => panic!(
                "Attempted to read next event as a Finish event for {:?}, but got {:?} instead",
                expected_kind, found
            ),
        }
    }
}

#[allow(unused)]
pub(crate) struct DebugEventBuffer<'source>(pub Vec<Event>, pub TriviaList, pub &'source str);

#[allow(unused)]
impl DebugEventBuffer<'_> {
    pub fn events(&self) -> &Vec<Event> {
        &self.0
    }

    pub fn trivia(&self) -> &TriviaList {
        &self.1
    }

    pub fn source(&self) -> &str {
        &self.2
    }
}

impl std::fmt::Debug for DebugEventBuffer<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if !f.alternate() {
            return self.0.fmt(f);
        }

        let mut indent_level = 0;

        let mut trivia_list = Rc::new(self.trivia().clone());
        let mut trivia_cursor = 0;

        for event in self.events() {
            match event {
                Event::Start(SyntaxKind::TOMBSTONE) | Event::Finish(SyntaxKind::TOMBSTONE) => {
                    f.write_fmt(format_args!(
                        "{:indent$}<tombstone>\n",
                        "",
                        indent = indent_level * 2
                    ))?;
                }
                Event::Start(kind) => {
                    f.write_fmt(format_args!(
                        "{:indent$}{:?} start\n",
                        "",
                        kind,
                        indent = indent_level * 2
                    ))?;
                    indent_level += 1;
                }
                Event::Finish(kind) => {
                    indent_level -= 1;
                    f.write_fmt(format_args!(
                        "{:indent$}finish {:?}\n",
                        "",
                        kind,
                        indent = indent_level * 2
                    ))?;
                }
                Event::Token(token) => {
                    let trivia_pointer =
                        TriviaPointer::from_token(&token, &mut trivia_list, &mut trivia_cursor);

                    f.write_fmt(format_args!(
                        "{:indent$}{:?}@{:?}\"{}\" {:?} {:?}",
                        "",
                        token.kind(),
                        token.span(),
                        self.source()[token.span()].escape_debug(),
                        trivia_pointer.leading_trivia(),
                        trivia_pointer.trailing_trivia(),
                        indent = indent_level * 2
                    ))?;

                    if !token.flags().is_empty() {
                        f.write_fmt(format_args!(" {:?}", token.flags()))?;
                    }

                    f.write_char('\n')?;
                }
            }
        }

        Ok(())
    }
}
