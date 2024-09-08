use arcstr::ArcStr;

use crate::{
    lexer::{LexContext, LexerState},
    token::Trivia,
};
use crate::token::TriviaList;

use super::{
    block_parser::BlockParser,
    delimiter::{AnyDelimiter, Delimiter},
    event::{Event, Marker},
    lexer::{Lexer, LexerCheckpoint},
    SyntaxKind,
    SyntaxToken,
    token::TokenFlags, tree_builder::cst::{Document, parser_events_to_cst},
};

use self::{block::parse_block, inline::parse_inline};

mod block;
mod code_span;
mod delimiter;
mod emphasis;
mod icu;
mod inline;
mod link;
mod strikethrough;
mod text;

#[derive(Clone, Copy, Debug, Default)]
pub(super) struct ParserState {}

#[derive(Debug)]
pub(super) struct ParserCheckpoint {
    lexer_checkpoint: LexerCheckpoint,
    buffer_index: usize,
    trivia_index: usize,
    delimiter_stack_length: usize,
    state: ParserState,
}

/// A specialized Markdown parser that understands ICU MessageFormat and
/// ensures that Markdown syntax and ICU interpolations do not overlap, but can
/// be nested in either direction.
///
/// This isn't possible to do with two purely separate parsers because there's
/// no overlap in the syntax, either, so one will always treat the significant
/// characters of the other as insignificant literal text.
///
/// This parser also handles the special syntax that Discord has used in
/// messages as extensions to Markdown, specifically hooks and "unsafe" blocks
/// (`$[content](hookName)` and `!!{something}!!`, respectively).
pub struct ICUMarkdownParser<'source> {
    lexer: Lexer<'source>,
    source: ArcStr,
    buffer: Vec<Event>,
    trivia_list: TriviaList,
    /// A stack of delimiter stacks, storing the hierarchical state of delimiters across different
    /// contexts as needed (i.e., when inside an ICU plural value), where new content will be pushed
    /// onto the stack and _should not_ be treated as part of the containing elements.
    ///
    /// For example:
    ///
    ///     *{count, plural, one {yes*}}*
    ///
    /// The appropriate parsing of this is a single `Emphasis` around the entirety of the
    /// ICU content, so the HTML result would be:
    ///
    ///     <em>yes*</em>
    ///
    /// But since the plural value contains a valid closing delimiter as well (the `*` in `yes*`),
    /// if that delimiter gets put onto the same stack as the outer containing context, it would end
    /// up failing to match, since that inner delimiter is at a different node level than the
    /// container, causing an invalid event buffer order.
    delimiter_stacks: Vec<Vec<AnyDelimiter>>,
    state: ParserState,

    // Configuration
    /// When true, the parser will first analyze the document for Blocks according to the Markdown
    /// spec, then parse each block as inline content. When false, block parsing is skipped and the
    /// entire block is treated as a single segment of inline content.
    include_blocks: bool,
}

impl<'source> ICUMarkdownParser<'source> {
    pub fn new(source: &'source str, include_blocks: bool) -> Self {
        let block_bounds = if include_blocks {
            BlockParser::new(source).parse_into_block_bounds()
        } else {
            vec![]
        };

        let arc = ArcStr::from(source);

        Self {
            lexer: Lexer::new(source, block_bounds),
            source: arc,
            buffer: Vec::with_capacity(source.len() / 2),
            // Pre-allocating some size here should avoid the need to allocate
            // at any point within the parser in _most_ cases, at the expense of
            // extra allocations for simple sources.
            trivia_list: TriviaList::new(),
            delimiter_stacks: vec![],
            state: ParserState::default(),
            include_blocks,
        }
    }

    pub fn source(&self) -> &ArcStr {
        &self.source
    }

    /// Returns a mutable reference to the current top of the stack of delimiter stacks.
    pub fn delimiter_stack(&mut self) -> &mut Vec<AnyDelimiter> {
        self.delimiter_stacks.last_mut().unwrap()
    }

    pub fn push_delimiter_stack(&mut self) {
        self.delimiter_stacks.push(vec![])
    }

    pub fn pop_delimiter_stack(&mut self) -> Vec<AnyDelimiter> {
        self.delimiter_stacks.pop().unwrap()
    }

    /// Start the parser and return an event iterator for the caller to consume.
    ///
    /// This method will first parse the content into blocks, and then each
    /// block's content will be parsed as inline syntax.
    pub fn parse(&mut self) {
        if !self.include_blocks {
            self.parse_inline_only();
            return;
        }

        // Eating one starts the parser by reading the first token.
        self.eat();
        self.push_event(Event::Start(SyntaxKind::DOCUMENT));

        loop {
            self.skip_whitespace_as_trivia();

            match self.current() {
                SyntaxKind::EOF => break,
                SyntaxKind::BLOCK_START => {
                    let kind = self.eat_block_bound();
                    self.push_event(Event::Start(kind));
                    parse_block(self, kind);
                }
                SyntaxKind::BLOCK_END => {
                    let kind = self.eat_block_bound();
                    self.push_event(Event::Finish(kind));
                    self.reset_inline_state();
                }
                SyntaxKind::INLINE_START => {
                    let kind = self.eat_block_bound();
                    self.push_event(Event::Start(kind));
                    parse_inline(self, false);
                }
                SyntaxKind::INLINE_END => {
                    let kind = self.eat_block_bound();
                    self.push_event(Event::Finish(kind));
                    self.reset_inline_state();
                }
                _ => unreachable!(
                    "Encountered unexpected kind while parsing at the block level {:?}",
                    self.current()
                ),
            }
        }
        self.expect_end_of_file();
        self.push_event(Event::Finish(SyntaxKind::DOCUMENT));
    }

    /// Parse the entire content as a single inline segment. This skips block
    /// parsing entirely, meaning any block-like syntax like headers, lists,
    /// and link references will be treated as insignificant and/or interpreted
    /// as inline syntax instead, even if there are multiple newlines separating
    /// pieces of the text.
    pub fn parse_inline_only(&mut self) {
        self.eat();
        self.push_event(Event::Start(SyntaxKind::DOCUMENT));
        parse_inline(self, false);
        self.expect_end_of_file();
        self.push_event(Event::Finish(SyntaxKind::DOCUMENT));
    }

    /// Consume this parser, interpreting its events into a constructed,
    /// lossless syntax tree. The return value is the root Node of that tree,
    /// a Document.
    pub fn into_cst(self) -> Document {
        let arc = ArcStr::clone(self.source());
        parser_events_to_cst(self.buffer, arc, self.trivia_list)
    }

    // Options API
    //
    // The following methods provide an interface for consumers to read the
    // applied configuration of the parser.

    pub fn are_blocks_included(&self) -> bool {
        self.include_blocks
    }

    // Internal API
    //
    // All of the following are the interface for parsing functions to use for
    // querying and mutating the parse state.

    /// Reset all of the state related to parsing inline elements. This should
    /// be done any time a block boundary is passed, like starting a new
    /// paragraph or passing a thematic break, since inline elements are not
    /// allowed to span across those boundaries.
    pub(super) fn reset_inline_state(&mut self) {
        self.state = ParserState::default();
    }

    pub(super) fn current(&self) -> SyntaxKind {
        self.lexer.current_kind()
    }

    pub(super) fn current_flags(&self) -> TokenFlags {
        self.lexer.current_flags()
    }

    /// Advances by 1 if the current token matches the given kind and returns
    /// that token. Otherwise, returns None indicating no bump was made.
    #[inline]
    #[must_use = "The result of `expect` is a None if the current token does not match, which should be propagated or handled."]
    pub(super) fn expect(&mut self, kind: SyntaxKind) -> Option<SyntaxKind> {
        self.expect_with_context(kind, LexContext::Regular)
    }

    /// Advances by 1 if the current token matches the given kind and returns
    /// that token. Otherwise, returns an unexpected token error. The following
    /// token will be lexed using the given context.
    #[must_use = "The result of `expect` is a None if the current token does not match, which should be propagated or handled."]
    pub(super) fn expect_with_context(
        &mut self,
        kind: SyntaxKind,
        context: LexContext,
    ) -> Option<SyntaxKind> {
        if self.current() != kind {
            return None;
        }

        self.bump_with_context(context);
        Some(kind)
    }

    #[must_use = "The result of `expect` is a None if the current token does not match, which should be propagated or handled."]
    pub(super) fn expect_block_bound(&mut self, kind: SyntaxKind) -> Option<SyntaxKind> {
        if !self.at_block_bound(kind) {
            return None;
        }

        self.eat_block_bound();
        Some(kind)
    }

    /// Returns true if the current block boundary matches the given kind.
    #[inline(always)]
    pub(super) fn at_block_bound(&mut self, kind: SyntaxKind) -> bool {
        self.lexer.current_block_kind() == kind
    }

    /// Assert that the parser has reached the end of the input, and consume
    /// that final token to pick up any trailing trivia.
    pub(super) fn expect_end_of_file(&mut self) {
        // At the end of parsing, the lexer must be at the end of the input.
        assert!(self.at(SyntaxKind::EOF));
        // Add the EOF token to the input so that trailing trivia on the
        // document are picked up.
        self.bump();
    }

    /// Advances the lexer by one token, adding the current token to the end of
    /// the event buffer as a Token event.
    #[inline]
    pub(super) fn bump(&mut self) {
        self.bump_as(self.current(), LexContext::Regular);
    }

    /// Advances the lexer by one token, adding the current token to the end of
    /// the event buffer as a Token event.
    #[inline]
    pub(super) fn bump_with_context(&mut self, context: LexContext) {
        self.bump_as(self.current(), context);
    }

    /// Bump a token into the buffer with the given SyntaxKind associated to it.
    /// Bumped tokens are always inline events. Use `push_block` to create a new
    /// block event.
    #[inline]
    pub(super) fn bump_as(&mut self, kind: SyntaxKind, context: LexContext) {
        let token = self.eat_with_context(context);
        self.push_token(token.with_kind(kind));
    }

    /// Advance the lexer by one token _without_ adding the current token to
    /// the event buffer. The token that was eaten is returned for the caller to
    /// use as needed.
    #[inline]
    pub(super) fn eat_with_context(&mut self, context: LexContext) -> SyntaxToken {
        let token = self.lexer.extract_current_token();
        self.lexer.next_token(context);
        token
    }

    #[inline]
    pub(super) fn eat(&mut self) -> SyntaxToken {
        self.eat_with_context(LexContext::Regular)
    }

    fn extract_as_trivia(&mut self) -> Trivia {
        let token = self.lexer.extract_current_token();
        Trivia::new(
            token.kind(),
            self.source().substr(token.span()),
            // This is...arbitrary, but the lexer enforces that the only time
            // LEADING_WHITESPACE is created is when there is non-whitespace
            // content on the line as well, and the leading whitespace will
            // always consume up until the first significant character, meaning
            // it is the only kind of token that can become leading trivia.
            //
            // However, if this trivia is at the very start of the input, then
            // it can't be trailing, so it gets forced as leading trivia, too.
            token.span_start() > 0 && token.kind() != SyntaxKind::LEADING_WHITESPACE,
        )
    }

    /// Eats the next token from the input as a Trivia token, adds it to the
    /// trivia list, and returns a reference to that trivia for the caller to
    /// inspect.
    pub(super) fn bump_as_trivia(&mut self) -> &Trivia {
        debug_assert!(
            self.current().is_trivia(),
            "Attempted to eat a token as trivia, but it is not a trivial kind: {:?}",
            self.current()
        );
        let trivia = self.extract_as_trivia();
        self.trivia_list.push(trivia);
        self.lexer.next_token(LexContext::Regular);
        self.trivia_list.last().unwrap()
    }

    /// Eats the current token from the stream, consuming the kind stored on
    /// the current block bound and returning that instead of the actual token
    /// that was eaten.
    pub(super) fn eat_block_bound(&mut self) -> SyntaxKind {
        let bound_kind = self.lexer.current_block_kind();
        self.lexer.advance_block_bound();
        self.lexer.next_token(LexContext::Regular);
        bound_kind
    }

    pub(super) fn relex_with_context(&mut self, context: LexContext) -> SyntaxKind {
        self.lexer.relex_with_context(context)
    }

    /// Returns true if the lexer is currently at a token of the given kind.
    #[inline]
    pub(super) fn at(&self, kind: SyntaxKind) -> bool {
        self.current() == kind
    }

    pub(super) fn checkpoint(&self) -> ParserCheckpoint {
        ParserCheckpoint {
            lexer_checkpoint: self.lexer.checkpoint(),
            buffer_index: self.buffer_index(),
            trivia_index: self.trivia_list.len(),
            delimiter_stack_length: self.delimiter_stack_length(),
            state: self.state,
        }
    }

    pub(super) fn rewind(&mut self, checkpoint: ParserCheckpoint) {
        self.lexer.rewind(checkpoint.lexer_checkpoint);
        self.buffer.truncate(checkpoint.buffer_index);
        self.trivia_list.truncate(checkpoint.trivia_index);
        self.delimiter_stack()
            .truncate(checkpoint.delimiter_stack_length);
        self.state = checkpoint.state;
    }

    pub(super) fn push_delimiter(&mut self, delimiter: AnyDelimiter) {
        self.delimiter_stack().push(delimiter);
    }

    pub(super) fn delimiter_stack_length(&self) -> usize {
        self.delimiter_stacks.last().unwrap().len()
    }

    pub(super) fn deactivate_delimiter(&mut self, delimiter_index: usize) {
        let delimiter = &mut self.delimiter_stack()[delimiter_index];
        delimiter.deactivate();
    }

    pub(super) fn buffer_index(&self) -> usize {
        self.buffer.len()
    }

    /// Push a plain token onto the back of the event stream. If the token is a
    /// TOMBSTONE, it is not pushed.
    pub(super) fn push_token(&mut self, token: SyntaxToken) {
        if token.kind() == SyntaxKind::TOMBSTONE {
            return;
        }

        self.push_event(Event::Token(token));
    }

    pub(super) fn push_event(&mut self, event: Event) {
        self.buffer.push(event);
    }

    /// Creates a new Start event in the buffer and returns a Marker pointing to
    /// it that can be used to resolve a Node in the future.
    pub(super) fn mark(&mut self) -> Marker {
        let index = self.buffer.len();
        self.buffer.push(Event::tombstone());
        Marker::new(index)
    }

    pub(super) fn get_event_mut(&mut self, index: usize) -> Option<&mut Event> {
        self.buffer.get_mut(index)
    }

    pub(super) fn get_last_event(&self) -> Option<&Event> {
        self.buffer.last()
    }

    /// Skips consecutive whitespace and newline tokens as Trivia. The resulting
    /// list of Trivia is returned for the caller to determine if it should
    /// become leading or trailing trivia for a token.
    pub(super) fn skip_whitespace_as_trivia(&mut self) {
        self.skip_whitespace_as_trivia_with_context(LexContext::Regular)
    }

    pub(super) fn skip_whitespace_as_trivia_with_context(&mut self, context: LexContext) {
        while self.current().is_trivia() {
            let trivia = self.extract_as_trivia();
            self.trivia_list.push(trivia);
            self.lexer.next_token(context);
        }
    }

    pub(super) fn set_lexer_state<F: FnMut(&mut LexerState)>(&mut self, mut func: F) {
        func(&mut self.lexer.state_mut())
    }
}

#[cfg(test)]
mod test {
    use crate::{format_ast, process_cst_to_ast};
    use crate::event::DebugEventBuffer;

    use super::ICUMarkdownParser;

    #[test]
    fn test_debug() {
        let content = "this is a thing with multiple \n  works  ";
        let mut parser = ICUMarkdownParser::new(content, true);
        let source = parser.source.clone();
        println!("Blocks: {:?}\n", parser.lexer.block_bounds());

        parser.parse();
        println!("Trivia: {:#?}\n", parser.trivia_list);

        println!(
            "Events:\n-------\n{:#?}\n",
            DebugEventBuffer(
                parser.buffer.clone(),
                parser.trivia_list.clone().into(),
                parser.source()
            )
        );

        let cst = parser.into_cst();
        println!("CST:\n----\n{:#?}\n", cst);

        let ast = process_cst_to_ast(source, &cst);
        println!("AST:\n----\n{:#?}\n", ast);

        let output = format_ast(&ast);
        println!("Output: {:?}", output.unwrap());
    }
}
