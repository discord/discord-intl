use super::{inline::parse_inline, ICUMarkdownParser};
use crate::lexer::LexContext;
use crate::syntax::SyntaxKind;

/// Parse a block element of the given kind. Rules for how the content of the
/// block is parsed are applied first, then all of the contained content is
/// parsed as an inline segment.
pub(super) fn parse_block(p: &mut ICUMarkdownParser, kind: SyntaxKind) -> Option<()> {
    let result = match kind {
        SyntaxKind::ATX_HEADING => parse_atx_heading(p),
        SyntaxKind::SETEXT_HEADING => parse_setext_heading(p),
        SyntaxKind::INDENTED_CODE_BLOCK => parse_code_block(p),
        SyntaxKind::FENCED_CODE_BLOCK => parse_fenced_code_block(p),
        SyntaxKind::THEMATIC_BREAK => parse_thematic_break(p),
        _ => parse_paragraph(p),
    };

    result
}

fn parse_paragraph(p: &mut ICUMarkdownParser) -> Option<()> {
    parse_inline(p, false);
    Some(())
}

fn parse_remainder_as_token_list(p: &mut ICUMarkdownParser) -> Option<()> {
    while !p.current().is_block_bound() {
        if p.current().is_trivia() {
            p.bump_as_trivia(LexContext::Regular);
        } else {
            p.bump();
        }
    }
    Some(())
}

fn parse_thematic_break(p: &mut ICUMarkdownParser) -> Option<()> {
    parse_remainder_as_token_list(p)
}

fn parse_setext_heading(p: &mut ICUMarkdownParser) -> Option<()> {
    p.skip_whitespace_as_trivia();
    // The content of the heading is always contained in an INLINE_CONTENT.
    p.expect_block_bound(SyntaxKind::INLINE_START)?;
    parse_inline(p, false);
    p.expect_block_bound(SyntaxKind::INLINE_END)?;

    let underline = p.mark();
    parse_remainder_as_token_list(p)?;
    underline.complete(p, SyntaxKind::SETEXT_HEADING_UNDERLINE)
}

/// Parse an ATX heading line, including the opening sequence, the inner
/// content, and the optional closing sequence.
///
/// Parsing here presumes that the block parser has asserted the content of the
/// input will create a valid heading.
fn parse_atx_heading(p: &mut ICUMarkdownParser) -> Option<()> {
    p.relex_with_context(LexContext::AsciiPunctuationRun);
    p.expect(SyntaxKind::PUNCTUATION_RUN)?;

    p.skip_whitespace_as_trivia();

    // Then parse the inline content of the heading
    p.expect_block_bound(SyntaxKind::INLINE_START)?;
    parse_inline(p, false);
    p.expect_block_bound(SyntaxKind::INLINE_END)?;

    p.skip_whitespace_as_trivia();

    // Finally, collect the optional closing sequence.
    p.optional(p.at(SyntaxKind::HASH), |p| {
        p.relex_with_context(LexContext::AsciiPunctuationRun);
        p.expect(SyntaxKind::PUNCTUATION_RUN)?;
        p.skip_whitespace_as_trivia();
        Some(())
    });

    Some(())
}

/// Continuously parse tokens from the input until a BLOCK_ENDING is
/// encountered. No semantics will be applied to the tokens, and they will
/// appear in the containing node as a flat list of plain tokens.
fn parse_code_block(p: &mut ICUMarkdownParser) -> Option<()> {
    let content_mark = p.mark();
    p.set_lexer_state(|state| state.indent_depth += 4);
    parse_code_block_content(p);
    p.set_lexer_state(|state| state.indent_depth -= 4);

    content_mark.complete(p, SyntaxKind::CODE_BLOCK_CONTENT)
}

fn parse_code_block_content(p: &mut ICUMarkdownParser) {
    p.relex_with_context(LexContext::CodeBlock);
    loop {
        p.skip_whitespace_as_trivia_with_context(LexContext::CodeBlock);
        if matches!(
            p.current(),
            SyntaxKind::EOF | SyntaxKind::INLINE_END | SyntaxKind::BLOCK_END
        ) {
            break;
        }
        p.bump_with_context(LexContext::CodeBlock);
    }
}

fn parse_fenced_code_block(p: &mut ICUMarkdownParser) -> Option<()> {
    let leading_indent = if p.at(SyntaxKind::LEADING_WHITESPACE) {
        let length = p.current_token_len();
        p.bump_as_trivia(LexContext::AsciiPunctuationRun);
        length
    } else {
        p.relex_with_context(LexContext::AsciiPunctuationRun);
        0
    };
    p.set_lexer_state(|state| state.indent_depth += leading_indent);

    // The block parser has already asserted that this will create a valid sequence, either from
    // ~~~ or ```.
    p.expect(SyntaxKind::PUNCTUATION_RUN)?;
    p.skip_whitespace_as_trivia();

    // If that's not the end of the line, then consume everything else as the
    // info string.
    p.optional(
        !p.at(SyntaxKind::INLINE_START) && !p.at(SyntaxKind::BLOCK_END),
        |p| {
            let info_string = p.mark();
            parse_remainder_as_token_list(p);
            info_string.complete(p, SyntaxKind::CODE_FENCE_INFO_STRING)
        },
    );

    // Then move onto the next line, which should start the inline content.
    p.expect_block_bound(SyntaxKind::INLINE_START)?;
    let content_mark = p.mark();
    parse_code_block_content(p);
    content_mark.complete(p, SyntaxKind::CODE_BLOCK_CONTENT);
    p.expect_block_bound(SyntaxKind::INLINE_END)?;

    p.skip_whitespace_as_trivia();

    // Finally the closing delimiter, which can be missing if the block ended
    // because of the end of the input.
    p.optional(p.at(SyntaxKind::BACKTICK) || p.at(SyntaxKind::TILDE), |p| {
        p.relex_with_context(LexContext::AsciiPunctuationRun);
        p.expect(SyntaxKind::PUNCTUATION_RUN)?;
        Some(())
    });

    p.set_lexer_state(|state| state.indent_depth -= leading_indent);
    Some(())
}
