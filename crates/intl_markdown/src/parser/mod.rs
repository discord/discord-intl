use self::{block::parse_block, inline::parse_inline};
use super::{
    block_parser::BlockParser,
    delimiter::{AnyDelimiter, Delimiter},
    lexer::{Lexer, LexerCheckpoint},
    Document,
};
use crate::lexer::{LexContext, LexerState};
use crate::syntax::{
    FromSyntaxElement, SourceText, SyntaxElement, SyntaxKind, SyntaxToken, TextSize, TreeBuilder,
    TreeCheckpoint,
};
use marker::Marker;

mod block;
mod code_span;
mod delimiter;
mod emphasis;
mod icu;
mod inline;
mod link;
mod marker;
mod strikethrough;
mod text;

#[derive(Clone, Copy, Debug, Default)]
pub(super) struct ParserState {}

#[derive(Debug)]
pub(super) struct ParserCheckpoint {
    lexer_checkpoint: LexerCheckpoint,
    builder_checkpoint: TreeCheckpoint,
    delimiter_stack_length: usize,
    state: ParserState,
}

pub struct ParseResult {
    pub source: SourceText,
    pub tree: SyntaxElement,
}

impl ParseResult {
    pub fn to_document(self) -> Document {
        Document::from_syntax_element(self.tree)
    }
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
pub struct ICUMarkdownParser {
    lexer: Lexer,
    source: SourceText,
    builder: TreeBuilder,
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
    previous_token: Option<SyntaxToken>,
}

impl ICUMarkdownParser {
    pub fn new(source: SourceText, include_blocks: bool) -> Self {
        let block_bounds = if include_blocks {
            BlockParser::new(&source).parse_into_block_bounds()
        } else {
            vec![]
        };

        Self {
            lexer: Lexer::new(source.clone(), block_bounds),
            builder: TreeBuilder::new(source.clone()),
            source,
            delimiter_stacks: vec![],
            state: ParserState::default(),
            include_blocks,
            previous_token: None,
        }
    }

    pub fn source(&self) -> &SourceText {
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
        // Eating one starts the parser by reading the first token.
        self.eat();
        self.start_node(SyntaxKind::DOCUMENT);
        if !self.include_blocks {
            parse_inline(self, false);
        } else {
            self.parse_with_blocks();
        }

        self.expect_end_of_file();
        self.finish_node();
    }

    fn parse_with_blocks(&mut self) {
        loop {
            self.skip_whitespace_as_trivia();

            match self.current() {
                SyntaxKind::EOF => break,
                SyntaxKind::BLOCK_START => {
                    let kind = self.eat_block_bound();
                    self.start_node(kind);
                    parse_block(self, kind);
                }
                SyntaxKind::BLOCK_END => {
                    self.eat_block_bound();
                    self.finish_node();
                    self.reset_inline_state();
                }
                SyntaxKind::INLINE_START => {
                    let kind = self.eat_block_bound();
                    self.start_node(kind);
                    parse_inline(self, false);
                }
                SyntaxKind::INLINE_END => {
                    self.eat_block_bound();
                    self.finish_node();
                    self.reset_inline_state();
                }
                _ => unreachable!(
                    "Encountered unexpected kind while parsing at the block level: {:?}.\nTree:---\n{:#?}",
                    self.current(),
                    self.builder,
                ),
            }
        }
    }

    /// Consume this parser, interpreting its events into a constructed,
    /// lossless syntax tree. The return value is the root Node of that tree,
    /// a Document.
    pub fn finish(self) -> ParseResult {
        ParseResult {
            source: self.source,
            tree: self.builder.finish(),
        }
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

    pub(super) fn current_token_len(&self) -> TextSize {
        self.lexer.current_byte_span().len() as u32
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

    /// Bump a token into the buffer with the given SyntaxKind associated onto it.
    #[inline]
    pub(super) fn bump_as(&mut self, kind: SyntaxKind, context: LexContext) {
        let token = self.eat_with_context(context, kind);
        self.push_token(token);
    }

    /// Advance the lexer by one token _without_ adding the current token to
    /// the event buffer. The token that was eaten is returned for the caller to
    /// use as needed.
    #[inline]
    pub(super) fn eat_with_context(
        &mut self,
        context: LexContext,
        kind: SyntaxKind,
    ) -> SyntaxToken {
        let token = self.lexer.extract_current_token_as_kind(kind);
        self.lexer.next_token(context);
        token
    }

    /// Advance the lexer by one token without processing anything about the current token.
    /// This method should only be used for trivia and elements that are tracked in other ways to
    /// avoid losing any content from the source text.
    #[inline]
    pub(super) fn skip_with_context(&mut self, context: LexContext) {
        self.lexer.skip_current_token();
        self.lexer.next_token(context);
    }

    #[inline]
    pub(super) fn eat(&mut self) -> SyntaxToken {
        self.eat_with_context(LexContext::Regular, self.current())
    }

    /// Eats the next token from the input as a Trivia token, adds it to the
    /// trivia list, and returns a reference to that trivia for the caller to
    /// inspect.
    ///
    /// NOTE: Trivia cannot be rewound within a token. To reparse trivia and
    /// remove it from the previous token, the parser must be rewound to before
    /// the target token.
    pub(super) fn bump_as_trivia(&mut self, context: LexContext) {
        debug_assert!(
            self.current().is_trivia(),
            "Attempted to eat a token as trivia, but it is not a trivial kind: {:?}",
            self.current()
        );
        self.builder.add_trivia(self.lexer.current_text());
        self.lexer.next_token(context);
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
            builder_checkpoint: self.builder.checkpoint(),
            delimiter_stack_length: self.delimiter_stack_length(),
            state: self.state,
        }
    }

    pub(super) fn rewind(&mut self, checkpoint: ParserCheckpoint) {
        self.lexer.rewind(checkpoint.lexer_checkpoint);
        self.builder.rewind(checkpoint.builder_checkpoint);
        self.delimiter_stack()
            .truncate(checkpoint.delimiter_stack_length);
        self.state = checkpoint.state;
    }

    pub(super) fn start_node(&mut self, kind: SyntaxKind) {
        self.builder.start_node(kind)
    }

    pub(super) fn start_node_at(&mut self, kind: SyntaxKind, checkpoint: TreeCheckpoint) {
        self.builder.start_node_at(kind, checkpoint)
    }

    pub(super) fn finish_node(&mut self) {
        self.builder.finish_node()
    }

    pub(super) fn wrap_with_node(
        &mut self,
        kind: SyntaxKind,
        start: TreeCheckpoint,
        end: TreeCheckpoint,
    ) {
        self.builder.wrap_with_node(kind, start, end);
    }

    pub(super) fn last_element(&self) -> Option<&SyntaxElement> {
        self.builder.last_element()
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

    /// Creates a new Start event in the buffer and returns a Marker pointing to
    /// it that can be used to resolve a Node in the future.
    pub(super) fn mark(&mut self) -> Marker {
        let index = self.builder.checkpoint();
        Marker::new(index)
    }

    /// Push a plain token onto the back of the event stream. If the token is a
    /// TOMBSTONE, it is not pushed.
    pub(super) fn push_token(&mut self, token: SyntaxToken) {
        if token.kind() == SyntaxKind::TOMBSTONE {
            return;
        }
        self.previous_token = Some(token.clone());
        self.builder.push_token(token)
    }

    /// Push an empty syntax element to indicate that an optional node or token is missing.
    ///
    /// All nodes must be represented in the tree, even if they are not present in the source. Using
    /// empty elements to fill the missing gaps preserves the layout of all syntax elements, no
    /// matter if they are valid or not.
    ///
    /// This method will always return [`None`] as a way to indicate to other parts of the parser
    /// that the element was _parsed_ but did not result in any actual parsed content.
    pub(super) fn push_missing(&mut self) -> Option<()> {
        self.builder.push_missing();
        None
    }

    /// Attempt to parse some content from the input, falling back to placing an Empty element in
    /// the tree if the parse fails.
    ///
    /// NOTE: This method will _not_ rewind the parser if parsing fails. Instead, either perform the
    /// rewind inside the block or catch the [`None`] return from this method to control the rewind.
    #[inline(always)]
    pub(super) fn optional<F: FnMut(&mut ICUMarkdownParser) -> Option<()>>(
        &mut self,
        condition: bool,
        mut func: F,
    ) -> Option<()> {
        condition
            .then(|| func(self))
            .unwrap_or_else(|| self.push_missing())
    }

    pub(super) fn tree_index(&mut self) -> u32 {
        self.builder.index()
    }

    /// Skip consecutive whitespace and newline tokens as Trivia. The resulting
    /// list of Trivia is returned for the caller to determine if it should
    /// become leading or trailing trivia for a token.
    pub(super) fn skip_whitespace_as_trivia(&mut self) {
        self.skip_whitespace_as_trivia_with_context(LexContext::Regular)
    }

    pub(super) fn skip_whitespace_as_trivia_with_context(&mut self, context: LexContext) {
        let trivia_start = self.lexer.current_byte_span().start;
        let mut trivia_end = trivia_start;
        let mut consumed = false;
        while self.current().is_trivia() {
            consumed = true;
            trivia_end = self.lexer.position();
            self.skip_with_context(context);
        }
        if consumed {
            self.builder
                .add_trivia(self.lexer.text(trivia_start..trivia_end))
        }
    }

    pub(super) fn set_lexer_state<F: FnMut(&mut LexerState)>(&mut self, mut func: F) {
        func(&mut self.lexer.state_mut())
    }
}

#[cfg(test)]
mod test {
    use super::ICUMarkdownParser;
    use crate::cst::Document;
    use crate::syntax::{FromSyntax, SourceText};

    #[test]
    fn test_debug() {
        let content = r#"```rust
"#;
        let mut parser = ICUMarkdownParser::new(SourceText::from(content), true);
        println!("Blocks: {:?}\n", parser.lexer.block_bounds());

        parser.parse();
        let result = parser.finish();
        println!("Tree:\n-------\n{:#?}\n", result.tree);

        let ast = Document::from_syntax(result.tree.node().clone());
        println!("AST:\n----\n{:#?}\n", ast);
        //
        // let output = format_ast(&ast);
        // println!("Output: {}", output.unwrap());
        //
        // let json = keyless_json::to_string(&ast);
        // println!("JSON: {}", json.unwrap());
    }
}
