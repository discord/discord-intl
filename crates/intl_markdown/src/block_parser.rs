use std::collections::VecDeque;

use super::syntax::SyntaxKind;

/// An indicator of the start or end of a block, including the byte position in
/// the source text where the bound occurs, and the syntax kind it represents.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BlockBound {
    Start(usize, SyntaxKind),
    End(usize, SyntaxKind),
    // For blocks that have leading and trailing syntax, the _content_ bounds
    // indicate where parsing should begin and end for inline content. Tokens
    // outside of these bounds will be parsed as block syntax only.
    InlineStart(usize, SyntaxKind),
    InlineEnd(usize, SyntaxKind),
}

impl BlockBound {
    pub(crate) fn position(&self) -> &usize {
        match self {
            BlockBound::Start(position, _) => position,
            BlockBound::End(position, _) => position,
            BlockBound::InlineStart(position, _) => position,
            BlockBound::InlineEnd(position, _) => position,
        }
    }

    pub(crate) fn kind(&self) -> SyntaxKind {
        match self {
            BlockBound::Start(_, kind) => *kind,
            BlockBound::End(_, kind) => *kind,
            BlockBound::InlineStart(_, kind) => *kind,
            BlockBound::InlineEnd(_, kind) => *kind,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Line {
    offset: usize,
    leading_offset: usize,
    /// The length of the line, excluding any trailing newline character.
    line_length: usize,
    leading_spaces: usize,
    /// True if this is the last line of the input, meaning there is no
    /// trailing newline character after it.
    is_last_line: bool,
    starting_icu_brace_balance: usize,
    ending_icu_brace_balance: usize,
}

impl Line {
    fn is_blank(&self) -> bool {
        self.line_length == self.leading_offset
    }

    fn end_offset(&self) -> usize {
        self.offset + self.line_length
    }

    fn content_offset(&self) -> usize {
        self.offset + self.leading_offset
    }

    #[allow(unused)]
    fn get_from<'a>(&self, text: &'a str) -> &'a str {
        &text[self.offset..self.end_offset()]
    }

    fn get_content<'a>(&self, text: &'a str) -> &'a str {
        &text[self.content_offset()..self.end_offset()]
    }

    /// Returns true if this line starts with content that is part of an ICU control section, where
    /// Markdown rules won't apply. If the line does not start in an ICU context, it may be
    /// considered for block semantics, but if a line _ends_ with an ICU context, then the following
    /// lines will inherently be continuations of whatever block is currently open.
    fn ends_inside_icu_context(&self) -> bool {
        self.ending_icu_brace_balance > 0
    }

    //#region Markdown Block semantic checks

    fn is_indented_code_block(&self) -> bool {
        self.leading_spaces >= 4 && !self.is_blank()
    }

    fn is_fenced_code_block(&self, text: &str) -> bool {
        let bytes = self.get_content(text).as_bytes();

        if bytes.len() == 0 {
            return false;
        }

        let expected = bytes[0];
        // Only ~ and ` are valid as fences
        if !matches!(expected, b'~' | b'`') {
            return false;
        }

        let mut offset = 0;
        let mut matched_count = 0;
        for byte in &bytes[offset..] {
            if *byte == expected {
                offset += 1;
                matched_count += 1;
            } else {
                break;
            }
        }

        // There must be at least three ticks to create the block.
        if matched_count < 3 {
            return false;
        }

        // Then the rest of the line can contain any text, but if this is
        // a backtick fence then it cannot include a backtick.
        if expected == b'`' {
            for byte in &bytes[offset..] {
                if *byte == expected {
                    return false;
                }
                offset += 1;
            }
        }

        true
    }

    fn is_fenced_code_block_ending(
        &self,
        text: &str,
        expected: char,
        opening_count: usize,
    ) -> bool {
        // Code fences cannot end with an indent of 4 or more.
        if self.leading_spaces >= 4 || self.is_blank() {
            return false;
        }

        let content = self.get_content(text);
        let count = content.find(|c| c != expected).unwrap_or(content.len());
        if count < opening_count {
            return false;
        }

        // If any other non-whitespace characters appear, it doesn't count.
        content[count..]
            .find(|c: char| !c.is_ascii_whitespace())
            .is_none()
    }

    fn is_setext_heading_underline(&self, text: &str) -> bool {
        if self.leading_spaces >= 4 {
            return false;
        }

        let bytes = self.get_content(text).as_bytes();

        if bytes.len() == 0 {
            return false;
        }

        let expected = bytes[0];
        // Only = and - are valid as setext heading chars.
        if !matches!(expected, b'=' | b'-') {
            return false;
        }

        // The heading underline can...
        bytes
            .iter()
            // consist of any number of matching = or - characters
            .skip_while(|b| **b == expected)
            // followed by any amount of whitespace
            .skip_while(|b| b.is_ascii_whitespace())
            // and then the end of the line must be reached (i.e., there cannot
            // be any more bytes on the line.
            .next()
            .is_none()
    }

    fn is_atx_heading(&self, text: &str) -> bool {
        if self.leading_spaces >= 4 {
            return false;
        }

        let content = self.get_content(text);
        // The line must start with a #.
        if !content.starts_with('#') {
            return false;
        }

        for (index, c) in content.bytes().enumerate() {
            match c {
                b'#' => {
                    // Only up to 6 hashes can be used in a heading, which would
                    // be index 5 in the string.
                    if index > 5 {
                        return false;
                    }
                }
                // If the first non-hash character is a space or tab, and there
                // haven't been too many hashes, then this is a valid heading.
                b' ' | b'\t' => break,
                // Any other character prevents a heading from forming.
                _ => return false,
            }
        }

        true
    }

    fn is_thematic_break(&self, text: &str) -> bool {
        if self.leading_spaces >= 4 {
            return false;
        }

        let content = self.get_content(text);

        let mut count = 0;
        let mut expected: u8 = b' ';
        for byte in content.bytes() {
            match byte {
                // Any amount of space characters can be interspersed.
                b' ' | b'\t' => continue,
                // -, _, and * are the valid delimiting characters, and the
                // expected char is initialized if this is the first one.
                b'-' | b'_' | b'*' => {
                    if expected == b' ' {
                        expected = byte;
                    }

                    if byte == expected {
                        count += 1;
                    } else {
                        break;
                    }
                }
                // Every char in the break must match if one has been found.
                b if b == expected => count += 1,
                // Any other character at any point in the line means this is
                // not a thematic break.
                _ => return false,
            }
        }

        count >= 3
    }

    /// Returns true if the line represents a new block that can interrupt a
    /// paragraph.
    fn can_interrupt_paragraph(&self, text: &str) -> bool {
        self.is_fenced_code_block(text)
            || self.is_thematic_break(text)
            || self.is_atx_heading(text)
            || self.is_blank()
    }

    //#endregion
}

/// A one-shot parser to build a structure of block elements from a Markdown
/// source text. The result is a list of indices in the text representing block
/// boundaries, which the full parser is then able to use as delimiters when
/// parsing inline content.
pub(crate) struct BlockParser<'a> {
    text: &'a str,
    bounds: Vec<BlockBound>,
    lines: VecDeque<Line>,
    previous_line: Option<Line>,
}

impl<'a> BlockParser<'a> {
    pub(crate) fn new(text: &'a str) -> Self {
        Self {
            text,
            bounds: vec![],
            lines: create_lines(text),
            previous_line: None,
        }
    }

    #[inline(never)]
    pub(crate) fn parse_into_block_bounds(mut self) -> Vec<BlockBound> {
        while !self.is_eof() {
            match self.current_line() {
                // "Blank lines between block-level elements are ignored, except for
                // the role they play in determining whether a list is tight or
                // loose."
                //
                // In the case of a list, the parsing will be handled by an inner
                // method, meaning a blank line at this position _must_ be between
                // block-level elements.
                line if line.is_blank() => {
                    self.advance();
                }
                // Lines starting with 4 spaces of indentation are Indented Code
                // Blocks. These _cannot_ interrupt paragraphs, so the only time
                // they should be encountered is here at the top level.
                line if line.is_indented_code_block() => self.consume_indented_code_block(),
                line if line.is_fenced_code_block(self.text) => self.consume_fenced_code_block(),
                line if line.is_thematic_break(self.text) => {
                    self.consume_line_as(SyntaxKind::THEMATIC_BREAK)
                }
                line if line.is_atx_heading(self.text) => self.consume_atx_heading(),
                // A sequence of non-blank lines that cannot be interpreted as
                // other kinds of blocks forms a paragraph.
                _ => self.consume_paragraph_or_setext_heading(),
            }
        }

        self.bounds
    }

    /// Consumes a single line as the given kind of block.
    fn consume_line_as(&mut self, kind: SyntaxKind) {
        self.push_start(kind);
        self.advance();
        self.push_end(kind);
    }

    /// Consume a paragraph from the input, eating lines so long as they are valid continuations.
    /// This method checks whether the following line is able to interrupt the paragraph to assert
    /// that the continuation is valid or not.
    ///
    /// A line that ends inside an ICU context definitively causes the paragraph to continue, even
    /// if the following lines are blank.
    ///
    /// This method also checks for setext heading underlines and converts the paragraph into a
    /// heading if it is found.
    fn consume_paragraph_or_setext_heading(&mut self) {
        let start_offset = self.current_line().offset;
        let mut block_kind = SyntaxKind::PARAGRAPH;
        self.eat_lines_while(|line| {
            // Ending in an ICU context forces the paragraph to continue until the ICU content
            // ends, even if there are blank lines. This check can safely happen first, since a
            // setext heading underline cannot contain extra characters that would allow an ICU
            // content segment to start or end on that line.
            if line.ends_inside_icu_context() {
                return true;
            }

            if line.is_setext_heading_underline(self.text) {
                block_kind = SyntaxKind::SETEXT_HEADING;
                return false;
            }

            if line.can_interrupt_paragraph(self.text) {
                return false;
            }

            true
        });
        self.push_start_at(block_kind, start_offset);
        // For setext headings, exclude the underline from the inline content
        // of the heading using an INLINE_CONTENT block.
        if block_kind == SyntaxKind::SETEXT_HEADING {
            self.push_inline_start(start_offset);
            // Should be able to confidently unwrap here, since the only way to
            // get here is to have parsed a line and then another line to
            // create the heading.
            let last_line = self.previous_line.unwrap();
            self.push_inline_end(last_line.end_offset());
            // Then consume the underline to end the block as a whole.
            self.advance();
        }
        self.push_end(block_kind);
    }

    /// Consumes an ATX heading from the input, which generally consists of a single line of input
    /// unless the line contains ICU content, in which case the ICU content is consumed entirely,
    /// and the rest of the final line of the ICU content is used as the end bound of the heading.
    fn consume_atx_heading(&mut self) {
        self.push_start(SyntaxKind::ATX_HEADING);
        let offset = self.current_line().content_offset();
        let inline_start = offset
            + self
                .current_line()
                .get_content(self.text)
                .find(|c: char| c.is_ascii_whitespace())
                .map_or(self.current_line().line_length, |index| index + 1);

        self.push_inline_start(inline_start);

        // Check for ICU content inside the heading and consume lines until it is fully closed
        // before attempting to find the end of the inline content. If a line ends in an ICU
        // context, the next line must by definition start in the same context, so that does not
        // need to be checked here.
        while self.current_line().ends_inside_icu_context() {
            self.advance();
        }

        // Once the ICU content is fully passed, the now-current line can be checked to find the
        // end bound of the inline content.
        let mut end_iter = self
            .current_line()
            .get_content(self.text)
            .char_indices()
            .rev()
            .skip_while(|(_, c)| c.is_ascii_whitespace())
            .peekable();

        // Get the index after the first non-whitespace character as the fallback for the end of the
        // inline content.
        let mut inline_end = end_iter
            .peek()
            .map_or(self.current_line().line_length, |(index, c)| {
                index + c.len_utf8()
            });

        // Then, if that next one is a hash, collect all the following hashes, then check that the
        // character after that is another space to signify it as the closing hash sequence.
        if let Some((_, '#')) = end_iter.peek() {
            if let Some((index, ' ')) = end_iter.skip_while(|(_, c)| *c == '#').next() {
                inline_end = index + 1;
            }
        }

        // Because of how the end is found, it might end up being before the
        // start index if the line is blank, so just use the start index in
        // that case to create zero-length content.
        self.push_inline_end(std::cmp::max(
            inline_start,
            self.current_line().content_offset() + inline_end,
        ));
        self.advance();
        self.push_end(SyntaxKind::ATX_HEADING);
    }

    /// Consume an indented code block from the input. Code blocks ignore ICU context, since they
    /// treat all content within them as literal text.
    fn consume_indented_code_block(&mut self) {
        self.push_start(SyntaxKind::INDENTED_CODE_BLOCK);
        // `parse_into_block_bounds` has already asserted that the current line
        // is not blank, so we know at least that one must be the last
        // contentful line to keep track of.
        //
        // The code block ends with the last contentful line, though there can
        // be multiple blank lines within it, meaning we want to track where the
        // last one is.
        let mut last_contentful_line = *self.current_line();
        self.eat_lines_while(|line| {
            // This is an && check since it must both have content _and_ be part
            // of the block to count.
            if !line.is_blank() && line.leading_spaces >= 4 {
                last_contentful_line = line;
            }

            line.leading_spaces >= 4 || line.is_blank()
        });
        self.push_end_after_line(SyntaxKind::INDENTED_CODE_BLOCK, last_contentful_line);
    }

    /// Fenced code blocks also ignore ICU context since they also treat all content within them as
    /// literal text.
    fn consume_fenced_code_block(&mut self) {
        let opening_content = self.current_line().get_content(self.text);
        // Find how many characters will be needed to close the block
        let opening_count = opening_content
            .find(|c| !matches!(c, '~' | '`'))
            .unwrap_or(opening_content.len());

        let expected = self.current_line().get_content(self.text).as_bytes()[0] as char;

        self.push_start(SyntaxKind::FENCED_CODE_BLOCK);
        self.advance();
        // It's possible that just an opening fence exists at the end of the
        // input, creating a blank fenced code block. Even though it's empty,
        // this still adds an inline content node for consistency.
        if self.is_eof() {
            let line = self.previous_line.unwrap();
            self.push_inline_start(line.end_offset());
            self.push_inline_end(line.end_offset());
            self.push_end(SyntaxKind::FENCED_CODE_BLOCK);
            return;
        }

        // Otherwise there must be content, even if it's empty.
        self.push_inline_start(self.current_line().offset);
        loop {
            // If the end of the file was reached, then there was no closing fence,
            // so the inline and block ends just appear at the end of the input.
            if self.is_eof() {
                let last_line = self.previous_line.unwrap();
                self.push_inline_end(last_line.end_offset());
                break;
            }

            // If this line is a valid code block ending, then mark the end of the inline content
            // before it and break.
            if self
                .current_line()
                .is_fenced_code_block_ending(self.text, expected, opening_count)
            {
                self.push_inline_end(self.current_line().offset);
                self.advance();
                break;
            }

            self.advance();
        }

        self.push_end(SyntaxKind::FENCED_CODE_BLOCK);
    }

    fn current_line(&self) -> &Line {
        self.lines
            .front()
            .expect("Requested current line when no more exist")
    }

    fn is_eof(&self) -> bool {
        self.lines.is_empty()
    }

    /// Continuously eat lines so long as the given `predicate` is true for that
    /// line. Looping will automatically stop if the end of the file is reached.
    fn eat_lines_while<F: FnMut(Line) -> bool>(&mut self, mut predicate: F) {
        loop {
            self.advance();

            // Stop at the end of input.
            if self.is_eof() {
                break;
            }

            // Blank lines end paragraphs unambiguously.
            if !predicate(*self.current_line()) {
                break;
            }
        }
    }

    fn advance(&mut self) {
        self.previous_line = self.lines.pop_front();
    }

    fn push_inline_start(&mut self, index: usize) {
        self.bounds
            .push(BlockBound::InlineStart(index, SyntaxKind::INLINE_START));
    }

    fn push_inline_end(&mut self, index: usize) {
        self.bounds
            .push(BlockBound::InlineEnd(index, SyntaxKind::INLINE_END));
    }

    /// Push a start bound for the given kind, using the offset of the current
    /// line as the position.
    fn push_start(&mut self, kind: SyntaxKind) {
        self.push_start_at(kind, self.current_line().offset);
    }

    /// Push a start bound for the given kind, using the given offset. This
    /// can be used for determining the kind of the starting element _after_
    /// consuming lines after it, such as for setext headings.
    fn push_start_at(&mut self, kind: SyntaxKind, index: usize) {
        self.bounds.push(BlockBound::Start(index, kind));
    }

    /// Push an end bound for the given kind. The bound will have a position of
    /// one byte _before_ the current line's offset, i.e., the newline character
    /// that ends the previous line.
    fn push_end(&mut self, kind: SyntaxKind) {
        let previous = self
            .previous_line
            .as_ref()
            .expect("Tried to create an end event when no lines have been consumed");
        self.bounds
            .push(BlockBound::End(previous.end_offset(), kind));
    }

    /// Push an end bound for the given kind, where the bound is written at the
    /// start of the following line, rather than at the end of the previous
    /// line. This is useful for blocks that include trailing line endings, like
    /// indented code blocks.
    fn push_end_after_line(&mut self, kind: SyntaxKind, line: Line) {
        let mut position = line.end_offset();
        if !line.is_last_line {
            position += 1;
        }
        self.bounds.push(BlockBound::End(position, kind));
    }
}

/// Iterate the given text, creating a new Line struct with information about
/// position, length, and other characteristics for each line it contains.
fn create_lines(text: &str) -> VecDeque<Line> {
    let mut offset = 0;

    let mut lines = vec![];

    let mut icu_brace_balance: usize = 0;

    while offset < text.len() {
        let starting_icu_brace_balance = icu_brace_balance;
        let line_text = &text[offset..];
        let mut leading_offset = 0;
        let mut leading_spaces = 0;
        let mut newline_index: Option<usize> = None;
        let mut has_found_content = false;

        let line_bytes = line_text[0..].as_bytes();
        let mut index = 0;
        while index < line_bytes.len() {
            match line_bytes[index] {
                b'\t' => {
                    if !has_found_content {
                        leading_offset += 1;
                        // Tabs are stopped at 4 space characters in leading spaces.
                        leading_spaces += 4 - (leading_spaces % 4);
                    }
                }
                b' ' => {
                    if !has_found_content {
                        leading_offset += 1;
                        leading_spaces += 1
                    }
                }
                b'\n' => {
                    newline_index = Some(index);
                    break;
                }
                b'{' => icu_brace_balance += 1,
                b'}' => icu_brace_balance = icu_brace_balance.saturating_sub(1),
                _ => has_found_content = true,
            }

            index += 1;
        }

        let line_length = newline_index.unwrap_or(line_text.len());
        lines.push(Line {
            offset,
            leading_offset,
            line_length,
            leading_spaces,
            is_last_line: newline_index.is_none(),
            starting_icu_brace_balance,
            ending_icu_brace_balance: icu_brace_balance,
        });

        offset += line_length + 1;
    }

    // If the last line ended with a newline character, then there is one last
    // empty line to add at the end of the list.
    if lines.last().is_some_and(|line| !line.is_last_line) {
        lines.push(Line {
            offset,
            leading_offset: 0,
            line_length: 0,
            leading_spaces: 0,
            is_last_line: true,
            starting_icu_brace_balance: icu_brace_balance,
            ending_icu_brace_balance: icu_brace_balance,
        })
    }

    VecDeque::from(lines)
}

#[cfg(test)]
mod test {
    use super::{create_lines, BlockBound, BlockParser};
    use crate::syntax::SyntaxKind;
    use test_case::test_case;

    #[test]
    fn print_test() {
        let text = "some text\n\n\non multiple\n  lines";
        let lines = create_lines(text);

        println!("{:#?}", lines)
    }

    #[test]
    fn creates_lines() {
        let text = "some text\non multiple\n  lines\n";
        let lines = create_lines(text);

        let known_lines = vec!["some text", "on multiple", "  lines", ""];

        for (index, line) in lines.iter().enumerate() {
            let found_line = &text[line.offset..line.end_offset()];

            assert_eq!(known_lines[index], found_line);
        }

        assert_eq!(lines[0].leading_spaces, 0);
        assert_eq!(lines[1].leading_spaces, 0);
        assert_eq!(lines[2].leading_spaces, 2);
        assert_eq!(lines[3].leading_spaces, 0);
    }

    fn block_bounds_test(text: &str, bounds: &[(usize, usize, SyntaxKind)]) {
        let expected = bounds
            .into_iter()
            .flat_map(|(start, end, kind)| {
                [
                    BlockBound::Start(*start, *kind),
                    BlockBound::End(*end, *kind),
                ]
            })
            .collect::<Vec<BlockBound>>();

        let parser = BlockParser::new(text);
        let bounds = parser.parse_into_block_bounds();

        assert_eq!(bounds, expected);
    }

    #[test_case(
        "one paragraph on one line\n\nNow continuing\non multiple lines", & [(0, 25, SyntaxKind::PARAGRAPH), (27, 59, SyntaxKind::PARAGRAPH)]; "multiple_paragraphs"
    )]
    #[test_case(
        "paragraph one\n  \nparagraph two", & [(0, 13, SyntaxKind::PARAGRAPH), (17, 30, SyntaxKind::PARAGRAPH)]; "blank_lines_with_spaces"
    )]
    #[test_case(
        "with\n  leading\n   spaces\n", & [(0, 24, SyntaxKind::PARAGRAPH)]; "leading_spaces"
    )]
    fn paragraphs(text: &str, bounds: &[(usize, usize, SyntaxKind)]) {
        block_bounds_test(text, bounds);
    }

    #[test_case(
        "    const foo;", & [(0, 14, SyntaxKind::INDENTED_CODE_BLOCK)]; "single indented code block"
    )]
    #[test_case(
        "    const foo;\n", & [(0, 15, SyntaxKind::INDENTED_CODE_BLOCK)]; "includes trailing line"
    )]
    fn indented_code_blocks(text: &str, bounds: &[(usize, usize, SyntaxKind)]) {
        block_bounds_test(text, bounds);
    }
}
