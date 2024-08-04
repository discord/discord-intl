use unicode_properties::{GeneralCategoryGroup, UnicodeGeneralCategory};

use crate::syntax::byte_is_significant;

use super::{
    block_parser::BlockBound,
    syntax::SyntaxKind,
    token::{SyntaxToken, TokenFlags},
};

/// A dedicated struct for storing ephemeral state that influences the lexer's
/// decision making.
#[derive(Clone, Copy, Debug, Default)]
pub(super) struct LexerState {
    pub indent_depth: u32,
    /// Index into the block_bounds Vec indicating how far the lexer has
    /// progressed through it so far.
    pub block_bound_index: usize,
    pub last_was_whitespace: bool,
    pub last_was_punctuation: bool,
    pub last_was_newline: bool,
    /// True if the last token was entirely an escaped token, which has an
    /// effect on whether the next token is considered punctuation or not when
    /// computing delimiters.
    pub last_token_was_escape: bool,
    /// True if the lexer has only encountered whitespace tokens since the last
    /// newline.
    pub is_after_newline: bool,
}

impl LexerState {
    /// Creates a new state instance that is accurate for using at the start of
    /// a new input. Some properties default to true specifically because they
    /// occur at the start of the input. Use `LexerState::default()` to get a
    /// true default state.
    pub fn new() -> Self {
        Self {
            indent_depth: 0,
            block_bound_index: 0,
            // The beginning of input counts as whitespace and a newline.
            last_was_newline: true,
            last_was_whitespace: true,
            last_was_punctuation: false,
            last_token_was_escape: false,
            is_after_newline: true,
        }
    }

    /// Reset the state values that have important values when lexing the start
    /// of the input.
    pub fn set_initial_conditions(&mut self) {
        self.last_was_whitespace = true;
        self.last_was_newline = true;
        self.last_was_punctuation = false;
        self.last_token_was_escape = false;
        self.is_after_newline = true;
    }
}

#[derive(Clone, Copy, Debug)]
pub(super) struct LexerCheckpoint {
    position: usize,
    last_position: usize,
    current_kind: SyntaxKind,
    current_flags: TokenFlags,
    state: LexerState,
}

#[derive(Clone, Copy, Debug)]
pub enum LexContext {
    /// Normal lexing, where all tokens are treated as they are intuitively, with no context-
    /// sensitive components. This context is the default where Markdown syntax is parsed.
    Regular,
    /// Code blocks treat enitre lines as single tokens, with no semantics inside of them.
    CodeBlock,
    /// Autolinks only allow email address or URI tokens.
    Autolink,
    /// ICU semantic blocks (e.g., within `{}` segments) ignore Markdown syntax and only lex out
    /// ICU MessageFormat syntax
    Icu,
    /// ICU Style arguments are effectively treated as a single plain text string. The additional
    /// lexing context here lets the lexer just read until the definite end of the argument without
    /// risk of interpreting keywords or other tokens (e.g., a style arg value like
    /// `+! short currency/GBP` could potentially be parsed as a number of keyword and punctuation
    /// tokens, but should be treated as a single string of text in this position).
    IcuStyle,
}

pub struct Lexer<'source> {
    text: &'source str,
    block_bounds: Vec<BlockBound>,
    current_kind: SyntaxKind,
    /// Current byte offset into the text.
    position: usize,
    last_position: usize,
    current_flags: TokenFlags,
    state: LexerState,
}

impl<'source> Lexer<'source> {
    pub fn new(text: &'source str, block_bounds: Vec<BlockBound>) -> Self {
        Self {
            text,
            block_bounds: block_bounds.into(),
            current_kind: SyntaxKind::TOMBSTONE,
            position: 0,
            last_position: 0,
            current_flags: TokenFlags::default(),
            state: LexerState::new(),
        }
    }

    #[allow(unused)]
    pub fn block_bounds(&self) -> &Vec<BlockBound> {
        &self.block_bounds
    }

    pub fn state_mut(&mut self) -> &mut LexerState {
        &mut self.state
    }

    /// Rewind the lexer to the start of the currently-lexed token and
    /// reinterpret it with the given context.
    pub fn relex_with_context(&mut self, context: LexContext) -> SyntaxKind {
        // Moving to one position before the current character lets us regain
        // the ephemeral lexer state (like `last_was_newline`) by just using
        // `self.advance()` again, rather than having to store that information
        // on every advance just in case a relex happens.
        //
        // But if this is the first byte of the input, then we can just assume
        // a truly-default state instead.
        self.position = self.last_position;
        self.get_state_from_previous_character();
        self.current_flags = TokenFlags::default();
        self.next_token(context)
    }

    /// Lex the next token from the source text. If the end of the file has
    /// been reached, EOF will be returned, and this will be true for every
    /// call to this method after the first time EOF is returned.
    pub fn next_token(&mut self, context: LexContext) -> SyntaxKind {
        // Block endings are always present
        if self.is_at_block_bound() {
            return self.consume_block_bound();
        }

        if self.is_eof() {
            self.current_kind = SyntaxKind::EOF;
            return self.current_kind;
        }

        self.current_kind = match context {
            LexContext::Regular => self.next_regular_token(),
            LexContext::CodeBlock => self.next_code_block_token(),
            LexContext::Autolink => self.next_autolink_token(),
            LexContext::Icu => self.next_icu_token(),
            LexContext::IcuStyle => self.next_icu_style_token(),
        };

        self.current_kind
    }

    fn next_regular_token(&mut self) -> SyntaxKind {
        match self.current() {
            b'\0' => self.consume_byte(SyntaxKind::EOF),
            b'\r' | b'\n' => self.consume_line_ending(),
            b'\\' => self.consume_escaped(),
            c if c.is_ascii_whitespace() => self.consume_whitespace(LexContext::Regular),

            b'[' => self.consume_byte(SyntaxKind::LSQUARE),
            b']' => self.consume_byte(SyntaxKind::RSQUARE),
            b'(' => self.consume_byte(SyntaxKind::LPAREN),
            b')' => self.consume_byte(SyntaxKind::RPAREN),
            b'<' => self.consume_byte(SyntaxKind::LANGLE),
            b'>' => self.consume_byte(SyntaxKind::RANGLE),
            b'{' => self.consume_byte(SyntaxKind::LCURLY),
            b'}' => self.consume_maybe_icu_unsafe_rcurly(),
            b'*' | b'_' | b'~' => self.consume_delimiter(),
            b'`' => self.consume_byte(SyntaxKind::BACKTICK),
            b'$' => self.consume_byte(SyntaxKind::DOLLAR),
            b'!' => self.consume_maybe_icu_unsafe_lcurly(),
            b'=' => self.consume_byte(SyntaxKind::EQUAL),
            b'-' => self.consume_byte(SyntaxKind::MINUS),
            b'#' => self.consume_byte(SyntaxKind::HASH),
            b':' => self.consume_byte(SyntaxKind::COLON),
            b'\'' => self.consume_byte(SyntaxKind::QUOTE),
            b'"' => self.consume_byte(SyntaxKind::DOUBLE_QUOTE),
            b'&' => self.consume_char_reference(),
            _ => self.consume_plain_text(),
        }
    }

    fn next_code_block_token(&mut self) -> SyntaxKind {
        match self.current() {
            // Consecutive newlines immediately become blank lines
            b'\n' if self.state.last_was_newline => {
                self.advance();
                return SyntaxKind::BLANK_LINE;
            }
            // Any other whitespaces character after a newline becomes leading
            // whitespace, if the configured `indent_depth` is more than 0. If
            // it is 0, then there cannot be any skipped leading whitespace.
            c if self.state.last_was_newline
                && c.is_ascii_whitespace()
                && self.state.indent_depth > 0 =>
            {
                self.consume_leading_whitespace()
            }
            b'\0' => self.consume_byte(SyntaxKind::EOF),
            _ => self.consume_verbatim_line(),
        }
    }

    //#region Whitespace / Trivia

    /// Consume a single line ending, which can either be a newline, a line
    /// feed, or a line feed followed by a newline.
    fn consume_line_ending(&mut self) -> SyntaxKind {
        self.advance_if(b'\r');
        self.advance_if(b'\n');

        SyntaxKind::LINE_ENDING
    }

    /// Consume any number of contiguous ascii whitespace characters _other_ than
    /// newlines.
    fn consume_whitespace(&mut self, context: LexContext) -> SyntaxKind {
        let started_on_newline = self.state.last_was_newline;
        // Only allow regular whitespace to become leading whitespace in the
        // regular lexing context. In other contexts, leading whitespace can have
        // a semantic meaning.
        let default_kind = if started_on_newline && matches!(context, LexContext::Regular) {
            SyntaxKind::LEADING_WHITESPACE
        } else {
            SyntaxKind::WHITESPACE
        };

        while self.current().is_ascii_whitespace()
            && !(self.is_at_block_bound() || self.current() == b'\n')
        {
            self.advance();
            if self.is_eof() {
                return default_kind;
            }
        }
        // ICU doesn't care about any particular kind of whitespace.
        if matches!(context, LexContext::Icu) {
            return default_kind;
        }

        if self.is_at_block_bound() {
            return default_kind;
        }

        if started_on_newline && self.current() == b'\n' {
            self.advance();
            SyntaxKind::BLANK_LINE
        } else if self.position - self.last_position >= 2 && self.current() == b'\n' {
            self.advance();
            SyntaxKind::HARD_LINE_ENDING
        } else {
            default_kind
        }
    }

    /// Consume ASCII whitespace characters from the start of a new line, up to
    /// the optional configured depth that the lexer is currently at. For
    /// example, in an indented code block, the first 4 effective spaces of the
    /// line are LEADING_WHITESPACE, and are ignored in the actual content of
    /// the block.
    ///
    /// This method assumes that the caller has already checked that the lexer
    /// is at the start of a new line.
    fn consume_leading_whitespace(&mut self) -> SyntaxKind {
        let mut current_depth = 0;

        while current_depth < self.state.indent_depth {
            if self.is_eof() {
                break;
            }

            match self.current() {
                // Reaching the end of the file or line means this is an
                // entirely blank line.
                b'\n' => break,
                // ASCII whitespace contributes 1 to the current depth.
                b' ' => {
                    self.advance();
                    current_depth += 1;
                }
                // Tabs are _stopped_ at 4 spaces, meaning they add up to the
                // next increment of 4 spaces to the current depth.
                b'\t' => {
                    self.advance();
                    current_depth += 4 - (current_depth % 4);
                }
                // Any other character means the leading whitespace is done.
                _ => break,
            }
        }

        SyntaxKind::LEADING_WHITESPACE
    }
    //#endregion

    //#region Autolinks

    /// Try to consume a single ABSOLUTE_URI or EMAIL_ADDRESS token. If the
    /// token is not a valid uri, it is returned as plain TEXT instead.
    fn next_autolink_token(&mut self) -> SyntaxKind {
        let checkpoint = self.checkpoint();
        self.maybe_consume_absolute_uri()
            .or_else(|| {
                self.rewind(checkpoint);
                self.maybe_consume_email_address()
            })
            .unwrap_or_else(|| {
                self.rewind(checkpoint);
                self.consume_plain_text()
            })
    }

    fn maybe_consume_absolute_uri(&mut self) -> Option<SyntaxKind> {
        // First, collect the scheme:
        // "...any sequence of 2–32 characters beginning with an ASCII letter
        // and followed by any combination of ASCII letters, digits, or the
        // symbols plus (“+”), period (“.”), or hyphen (“-”).".
        // First char must be a letter.
        if !self.current().is_ascii_alphabetic() {
            self.advance();
            return None;
        }

        let mut scheme_length = 0;
        while scheme_length < 32 {
            if self.is_eof() {
                break;
            }

            match self.current() {
                b'+' | b'.' | b'-' => scheme_length += 1,
                c if c.is_ascii_alphanumeric() => scheme_length += 1,
                _ => break,
            }
            self.advance();
        }
        // The length must be at least 2. The loop won't accept more than
        // 32, so that bound is already checked.
        if scheme_length < 2 {
            return None;
        }

        // The scheme must be followed by a colon
        if !self.advance_if(b':') {
            return None;
        }

        // Then the URL can continue with whatever other than control
        // characters.
        loop {
            if self.is_eof() {
                break;
            }

            match self.current() {
                c if c.is_ascii_control() => break,
                b' ' | b'<' | b'>' => break,
                _ => self.advance(),
            }
        }

        Some(SyntaxKind::ABSOLUTE_URI)
    }

    fn maybe_consume_email_address(&mut self) -> Option<SyntaxKind> {
        // This implementation is an unrolling of the non-normative HTML5
        // email regex:
        // [a-zA-Z0-9.!#$%&'*+/=?^_`{|}~-]+@[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?(?:\.[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)*
        loop {
            if self.is_eof() {
                return None;
            }

            match self.current() {
                // a-zA-Z0-9
                c if c.is_ascii_alphanumeric() => self.advance(),
                // allowed punctuation
                b'.' | b'!' | b'#' | b'$' | b'%' | b'&' | b'\'' | b'*' | b'+' | b'/' | b'='
                | b'?' | b'^' | b'_' | b'`' | b'{' | b'|' | b'}' | b'~' | b'-' => self.advance(),
                // @ breaks the loop to the next section.
                b'@' => break,
                _ => return None,
            }
        }

        // Storage-less way of checking if there was at least one character in
        // the first loop.
        if self.position == self.last_position {
            return None;
        }

        // Now consume the @
        if !self.advance_if(b'@') {
            return None;
        }

        // After the @ are any number of domain parts. Each of these is up to
        // 61 alphanumeric characters, or a `-` in any other than the first and
        // last positions.
        // [a-zA-Z0-9-]{0,61}[a-zA-Z0-9]
        //
        // Any number of domain parts can be chained, so long as they contain
        // at least one character, but there must be at least one.
        loop {
            if self.is_eof() {
                break;
            }

            // First must be alphanumeric.
            if self.current().is_ascii_alphanumeric() {
                self.advance();
            } else {
                return None;
            }

            // Then up to 60 alphanumerics or `-`s, where the last cannot
            // be a `-`.
            let mut last_was_dash = false;
            for _ in 0..60 {
                if self.is_eof() {
                    break;
                }

                last_was_dash = match self.current() {
                    c if c.is_ascii_alphanumeric() => {
                        self.advance();
                        false
                    }
                    b'-' => {
                        self.advance();
                        true
                    }
                    // . indicates a break to the next domain part.
                    b'.' => break,
                    // Anything else is invalid
                    _ => break,
                }
            }

            // The last character in each part can't be a dash.
            if last_was_dash {
                return None;
            }

            // Domain parts can be chained with `.`s. Putting this at the end
            // ensures that the first one is matched, and the next part will
            // have at least one character.
            if !self.advance_if(b'.') {
                break;
            }
        }

        Some(SyntaxKind::EMAIL_ADDRESS)
    }
    //#endregion

    //#region Block Bounds

    /// Returns true if the current position in the input matches the position
    /// of the next block bound in the stack.
    #[inline]
    fn is_at_block_bound(&self) -> bool {
        match self.block_bounds.get(self.state.block_bound_index) {
            Some(bound) => self.position == *bound.position(),
            None => false,
        }
    }

    /// Assuming the lexer is currently at the postion matching the first block
    /// bound in the stack, checked before calling this method, this method
    /// consumes that block bound and returns a matching SyntaxKind for it,
    /// representing a zero-width internal token that the parser uses to branch
    /// its parsing appropriately.
    fn consume_block_bound(&mut self) -> SyntaxKind {
        self.current_kind = match self.block_bounds.get(self.state.block_bound_index) {
            Some(BlockBound::Start(_, _)) => SyntaxKind::BLOCK_START,
            Some(BlockBound::End(_, _)) => SyntaxKind::BLOCK_END,
            Some(BlockBound::InlineStart(_, _)) => SyntaxKind::INLINE_START,
            Some(BlockBound::InlineEnd(_, _)) => SyntaxKind::INLINE_END,
            _ => unreachable!(),
        };

        return self.current_kind;
    }

    /// Return the current block boundary that the lexer is in. This method
    /// checks that the lexer is currently positioned at a block boundary
    /// before returning.
    pub fn current_block_kind(&self) -> SyntaxKind {
        debug_assert!(
            self.is_at_block_bound() && self.state.block_bound_index < self.block_bounds.len(),
            "Attempted to read current_block_kind when not at a block boundary. current position: {}, next bound: {:?}",
            self.position,
            self.block_bounds.get(self.state.block_bound_index)
        );

        self.block_bounds
            .get(self.state.block_bound_index)
            .unwrap()
            .kind()
    }
    //#endregion

    //#region Markdown Elements

    /// Consume an escaped character, either returning SyntaxKind::ESCAPED for
    /// valid escape sequences, or `text` if the next character would not create
    /// a valid escape.
    fn consume_escaped(&mut self) -> SyntaxKind {
        self.state.last_token_was_escape = true;
        self.current_flags.insert(TokenFlags::IS_ESCAPED);
        self.advance();

        if self.is_eof() {
            return SyntaxKind::TEXT;
        }

        match self.current() {
            // Escaped backticks are treated specially when discovering code
            // spans. The escape won't actually get interpreted when it's
            // _inside_ of the span, but will if it's anywhere else.
            b'`' => self.consume_byte(SyntaxKind::BACKTICK),
            // "Any ASCII punctuation character may be backslash-escaped"
            b if b.is_ascii_punctuation() => self.consume_byte(SyntaxKind::ESCAPED),
            // "A backslash at the end of the line is a hard line break"
            // But if there is a block bound between the slash and the end of
            // the line, then it can't be combined into a single token.
            b'\n' if !self.is_at_block_bound() => self.consume_byte(SyntaxKind::BACKSLASH_BREAK),
            _ => {
                // Markdown only allows the above characters to be escaped,
                // everything else is treated as a literal backslash.
                self.state.last_token_was_escape = false;
                self.current_flags.remove(TokenFlags::IS_ESCAPED);
                SyntaxKind::TEXT
            }
        }
    }

    /// Consume any single delimiter character (one of `*`, `_`, or `~`). The
    /// surrounding context is also examined to determine whether this is a
    /// flanking delimiter (one that can be used to open or close an emphasis
    /// block).
    ///
    /// The parser is responsible for merging these tokens into delimiter runs
    /// and collating the bounds for whether the run can open and/or close.
    fn consume_delimiter(&mut self) -> SyntaxKind {
        // Consume all the same consecutive characters.
        let value = self.current();

        let kind = match value {
            b'*' => SyntaxKind::STAR,
            b'_' => SyntaxKind::UNDER,
            b'~' => SyntaxKind::TILDE,
            _ => unreachable!("Consumed a delimiter run of a non-runnable value {}", value),
        };

        // Then examine the character that follows the run.
        let next = self.peek_char();
        let next_is_whitespace: bool = next.map_or(true, |c| c.is_whitespace());
        let next_is_punctuation = next.is_some_and(|c| {
            matches!(
                c.general_category_group(),
                GeneralCategoryGroup::Punctuation | GeneralCategoryGroup::Symbol
            )
        });
        let next_is_escaped = matches!(next, Some('\\'));

        let mut flags = TokenFlags::default();
        if self.state.last_was_whitespace {
            flags.insert(TokenFlags::HAS_PRECEDING_WHITESPACE);
        }
        if self.state.last_was_punctuation && !self.state.last_token_was_escape {
            flags.insert(TokenFlags::HAS_PRECEDING_PUNCTUATION);
        }
        if next_is_whitespace {
            flags.insert(TokenFlags::HAS_FOLLOWING_WHITESPACE);
        }
        if next_is_punctuation && !next_is_escaped {
            flags.insert(TokenFlags::HAS_FOLLOWING_PUNCTUATION);
        }

        self.advance();

        // Add all the determined flags to the current flag set.
        self.current_flags.insert(flags);

        kind
    }

    /// Attempts to consume the input as a valid html entity or a numeric
    /// character reference (either decimal or hexadecimal).
    fn consume_char_reference(&mut self) -> SyntaxKind {
        // Consume the leading ampersand
        self.advance();

        // Checkpoint just after the ampersand to be able to rewind if the
        // following char don't yield an entity or character reference.
        let checkpoint = self.checkpoint();

        let is_decimal = self.advance_if(b'#');
        let is_hexadecimal = is_decimal && (self.advance_if(b'X') || self.advance_if(b'x'));

        // Then consume alphanumeric characters until a semicolon is reached, or
        // break if there's any other character.
        if is_hexadecimal {
            self.consume_hex_char_reference(checkpoint)
        } else if is_decimal {
            self.consume_decimal_char_reference(checkpoint)
        } else {
            self.consume_html_entity_reference(checkpoint)
        }
    }

    /// Consumes the remainder of a decimal numeric character reference, from
    /// immediately after the `#` through the ending semicolon. If the reference
    /// is invalid, this method will rewind the lexer to `checkpoint` and return
    /// AMPER for the kind instead.
    fn consume_decimal_char_reference(&mut self, checkpoint: LexerCheckpoint) -> SyntaxKind {
        let mut length = 0;
        loop {
            if self.is_eof() {
                self.rewind(checkpoint);
                return SyntaxKind::AMPER;
            }

            let current = self.current();
            if current == b';' && length > 0 {
                self.advance();
                return SyntaxKind::DEC_CHAR_REF;
            }

            // Decimal references can only contain up to 7 arabic digits.
            if !current.is_ascii_digit() || length >= 7 {
                self.rewind(checkpoint);
                return SyntaxKind::AMPER;
            }

            self.advance();
            length += 1;
        }
    }

    /// Consumes the remainder of a hexadecimal numeric character reference,
    /// from immediately after the `x` through the ending semicolon. If the
    /// reference is invalid, this method will rewind the lexer to `checkpoint`
    /// and return AMPER for the kind instead
    fn consume_hex_char_reference(&mut self, checkpoint: LexerCheckpoint) -> SyntaxKind {
        let mut length = 0;
        loop {
            if self.is_eof() {
                self.rewind(checkpoint);
                return SyntaxKind::AMPER;
            }

            let current = self.current();
            if current == b';' && length > 0 {
                self.advance();
                return SyntaxKind::HEX_CHAR_REF;
            }

            // Hex references can only contain up to 6 hex digits.
            if !current.is_ascii_hexdigit() || length >= 6 {
                self.rewind(checkpoint);
                return SyntaxKind::AMPER;
            }

            self.advance();
            length += 1;
        }
    }

    /// Consumes the remainder of an html entity reference, from immediately
    /// after the `&` through the ending semicolon. If the reference is invalid,
    /// this method will rewind the lexer to `checkpoint` and return AMPER for
    /// the kind. Note that this does _not_ check if the reference is an actual
    /// known HTML entity, only if it matches the appropriate syntax.
    fn consume_html_entity_reference(&mut self, checkpoint: LexerCheckpoint) -> SyntaxKind {
        let mut has_content = false;
        while self.current().is_ascii_alphanumeric() {
            has_content = true;
            self.advance();
            if self.is_eof() {
                self.rewind(checkpoint);
                return SyntaxKind::AMPER;
            }
        }

        if self.current() == b';' && has_content {
            self.advance();
            SyntaxKind::HTML_ENTITY
        } else {
            self.rewind(checkpoint);
            SyntaxKind::AMPER
        }
    }

    /// Consumes the input stream as literal text until a significant character
    /// is encountered. This is written to handle underscores within words as
    /// plain text, like "this_is_a_variable_name", rather than as a series of
    /// potential underscore segments.
    fn consume_plain_text(&mut self) -> SyntaxKind {
        loop {
            if self.is_eof() || self.is_at_block_bound() {
                break;
            }

            let current = self.current();

            if byte_is_significant(current) || current.is_ascii_whitespace() {
                break;
            }

            // ICU uses single quote characters as escapes for the control
            // characters. There are a few characters that can be escaped that
            // we don't actually care about, like `'#`, since that doesn't have
            // an effect on the markdown parsing anyway. All that we care about
            // is the brace characters that enter and exit ICU contexts so that
            // we can track literal state.
            if current == b'\'' && matches!(self.peek(), Some(b'{' | b'}')) {
                // Skip past these chars and continue the loop.
                self.advance_n_bytes(2);
                continue;
            }

            self.advance();
        }

        SyntaxKind::TEXT
    }

    /// Consume all the remaining characters on the current line as a single
    /// VERBATIM_LINE token.
    fn consume_verbatim_line(&mut self) -> SyntaxKind {
        loop {
            if self.is_eof() {
                break;
            }
            if self.current() == b'\n' {
                self.advance();
                break;
            }
            self.advance();
        }

        SyntaxKind::VERBATIM_LINE
    }
    //#endregion

    //#region ICU Elements

    fn next_icu_token(&mut self) -> SyntaxKind {
        match self.current() {
            b'\r' | b'\n' => self.consume_line_ending(),
            b'{' => self.consume_byte(SyntaxKind::LCURLY),
            b',' => self.consume_byte(SyntaxKind::COMMA),
            b':' if matches!(self.peek(), Some(b':')) => {
                self.advance_n_bytes(2);
                self.consume_byte(SyntaxKind::ICU_DOUBLE_COLON)
            }
            b'=' => self.consume_icu_plural_exact(),
            b'}' => self.consume_maybe_icu_unsafe_rcurly(),
            // Whitespace is insignificant when inside an ICU block.
            c if c.is_ascii_whitespace() => self.consume_whitespace(LexContext::Icu),
            _ => self.consume_icu_keyword_or_ident(),
        }
    }

    /// Consume the entirety of the style argument for a number, date, or time variable as a single
    /// ICU_STYLE_ARGUMENT token. If the lexer is currently at a closing curly brace `}` when this
    /// function is called, it will be returned as an RCURLY immediately.
    fn next_icu_style_token(&mut self) -> SyntaxKind {
        if self.current() == b'}' {
            return SyntaxKind::RCURLY;
        }

        let mut open_brace_count = 0;
        loop {
            match self.current() {
                // Apostrophes count as quoting characters in ICU syntax, so anything within them
                // will be treated as a string until the second apostrophe closes it, even opening
                // and closing braces.
                // NOTE: This does _not_ deal with "escaped escapes", but that's fine for now.
                b'\'' => {
                    while self.current() != b'\'' {
                        self.advance()
                    }
                }
                b'}' if open_brace_count == 0 => break,
                b'}' => {
                    open_brace_count -= 1;
                    self.advance();
                }
                b'{' => {
                    open_brace_count += 1;
                    self.advance();
                }
                _ => self.advance(),
            }
        }

        SyntaxKind::ICU_STYLE_TEXT
    }

    fn consume_icu_keyword_or_ident(&mut self) -> SyntaxKind {
        self.consume_icu_ident();
        let ident = &self.text[self.current_byte_span()];
        match ident {
            "plural" => SyntaxKind::ICU_PLURAL_KW,
            "select" => SyntaxKind::ICU_SELECT_KW,
            "selectordinal" => SyntaxKind::ICU_SELECTORDINAL_KW,
            "date" => SyntaxKind::ICU_DATE_KW,
            "time" => SyntaxKind::ICU_TIME_KW,
            "number" => SyntaxKind::ICU_NUMBER_KW,
            _ => SyntaxKind::ICU_IDENT,
        }
    }

    fn consume_icu_plural_exact(&mut self) -> SyntaxKind {
        // An exact value must be an = followed immediately by at least one digit.
        if !self.advance_if(b'=') {
            return SyntaxKind::EQUAL;
        }

        if !self.current().is_ascii_digit() {
            return SyntaxKind::EQUAL;
        }

        while self.current().is_ascii_digit() {
            self.advance();
        }

        SyntaxKind::ICU_PLURAL_EXACT
    }

    // We make a strict assertion that ICU identifiers are ASCII alphanumeric, since there's no
    // need for them to ever be anything else, and using unicode _almost definitely_ means the
    // translation has incorrectly translated the field name accidentally.
    fn consume_icu_ident(&mut self) -> SyntaxKind {
        // Idents must start with an alphabetic character or an underscore.
        if self.current() == b'_' || self.current().is_ascii_alphabetic() {
            self.advance();
        } else {
            self.advance();
            return SyntaxKind::TEXT;
        }

        loop {
            match self.current() {
                c if c.is_ascii_alphanumeric() => self.advance(),
                _ => break,
            }
        }

        SyntaxKind::ICU_IDENT
    }

    fn consume_maybe_icu_unsafe_lcurly(&mut self) -> SyntaxKind {
        if self.current() == b'!'
            && matches!(self.peek_at(1), Some(b'!'))
            && matches!(self.peek_at(2), Some(b'{'))
        {
            self.advance_n_bytes(3);
            SyntaxKind::UNSAFE_LCURLY
        } else {
            self.consume_byte(SyntaxKind::EXCLAIM)
        }
    }
    fn consume_maybe_icu_unsafe_rcurly(&mut self) -> SyntaxKind {
        if self.current() == b'}'
            && matches!(self.peek_at(1), Some(b'!'))
            && matches!(self.peek_at(2), Some(b'!'))
        {
            self.advance_n_bytes(3);
            SyntaxKind::UNSAFE_RCURLY
        } else {
            self.consume_byte(SyntaxKind::RCURLY)
        }
    }
    //#endregion

    //#region Internal API (current, advance, etc.)

    /// Advance `n` positions through the source text, then consumes a token
    /// using the current position state after advancing.
    fn consume_byte(&mut self, kind: SyntaxKind) -> SyntaxKind {
        self.advance();
        kind
    }

    pub fn current_kind(&self) -> SyntaxKind {
        self.current_kind
    }

    /// Returns the first byte of the character at the current position.
    fn current(&self) -> u8 {
        debug_assert!(
            self.text.is_char_boundary(self.position),
            "current parser position is not a ut8 char boundary"
        );
        self.text.as_bytes()[self.position]
    }

    fn current_char(&self) -> char {
        self.text[self.position..].chars().next().unwrap()
    }

    /// Returns the flags that are applied for the current token.
    pub fn current_flags(&self) -> TokenFlags {
        self.current_flags
    }

    /// Returns the next character in the source text after the current one.
    fn peek(&self) -> Option<&u8> {
        self.peek_at(1)
    }

    /// Peeks an entire unicode char from the source.
    fn peek_char(&self) -> Option<char> {
        self.text[self.position..].chars().nth(1)
    }

    /// Returns the character `offset` characters after the current one from
    /// the source text.
    fn peek_at(&self, offset: usize) -> Option<&u8> {
        self.text.as_bytes().get(self.position + offset)
    }

    /// Returns true if the current byte position is at or past the end of the
    /// source text.
    fn is_eof(&self) -> bool {
        self.position >= self.text.len()
    }

    pub(super) fn checkpoint(&self) -> LexerCheckpoint {
        LexerCheckpoint {
            position: self.position,
            last_position: self.last_position,
            current_kind: self.current_kind,
            current_flags: self.current_flags,
            state: self.state,
        }
    }

    pub(super) fn rewind(&mut self, checkpoint: LexerCheckpoint) {
        self.position = checkpoint.position;
        self.last_position = checkpoint.last_position;
        self.current_kind = checkpoint.current_kind;
        self.current_flags = checkpoint.current_flags;
        self.state = checkpoint.state;
    }

    /// Calculate properties for the LexerState by examining backwards in the
    /// source.
    fn get_state_from_previous_character(&mut self) {
        if self.position == 0 {
            self.state.set_initial_conditions();
            return;
        }

        let last_char = self.text[0..self.position].chars().rev().next().unwrap();
        self.state.last_was_punctuation = if !last_char.is_ascii() {
            matches!(
                last_char.general_category_group(),
                GeneralCategoryGroup::Punctuation | GeneralCategoryGroup::Symbol
            )
        } else {
            last_char.is_ascii_punctuation()
        };

        self.state.last_was_newline = last_char == '\n';
        self.state.last_was_whitespace = last_char.is_whitespace();
        self.state.is_after_newline = self.state.last_was_newline
            || (self.state.is_after_newline && self.state.last_was_whitespace)
    }

    /// Advance the lexer by one unicode character.
    fn advance(&mut self) {
        let previous = self.current();
        if previous.is_ascii() {
            self.position += 1;
        } else {
            let current_char = self.current_char();
            self.position += current_char.len_utf8();
        }
    }

    /// Advance n bytes in the source text. A shortcut for calling `advance`
    /// multiple times when the exact number of bytes involved is known ahead
    /// of time.
    #[inline(always)]
    fn advance_n_bytes(&mut self, n: usize) {
        self.position += n;
    }

    /// Advance the lexer by one unicode character as long as the current
    /// character matches the provided char. Returns true if the character
    /// matched and the lexer advanced, otherwise returns false.
    fn advance_if(&mut self, byte: u8) -> bool {
        if self.current() == byte {
            self.advance();
            true
        } else {
            false
        }
    }

    pub fn advance_block_bound(&mut self) {
        self.state.block_bound_index += 1;
    }

    /// Returns a range representing the byte span of the current token.
    pub fn current_byte_span(&self) -> std::ops::Range<usize> {
        self.last_position..self.position
    }

    /// Creates a new token of the given `kind` from the current positions in
    /// the source text.
    ///
    /// After consuming, the state of the lexer is reset and advanced to the
    /// next position in the source.
    pub fn extract_current_token(&mut self) -> SyntaxToken {
        self.get_state_from_previous_character();
        let token = self.token_from_range(self.current_kind, self.current_byte_span());
        self.reset_state();
        token
    }

    fn reset_state(&mut self) {
        self.last_position = self.position;
        self.current_flags = TokenFlags::default();
        self.current_kind = SyntaxKind::TOMBSTONE;
    }
    //#endregion

    /// Create and return a new token from the given range. This is the default
    /// way that new tokens are created iteratively during lexing, but can also
    /// be used to generate arbitrary tokens from a given range, effectively re-
    /// lexing the content, but without destroying old tokens, either.
    pub fn token_from_range(&self, kind: SyntaxKind, range: std::ops::Range<usize>) -> SyntaxToken {
        SyntaxToken::new(kind, range).with_flags(self.current_flags)
    }
}
