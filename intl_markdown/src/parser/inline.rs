use crate::{lexer::LexContext, SyntaxKind};
use crate::parser::link::parse_hook_open;
use crate::parser::strikethrough::parse_strikethrough_delimiter_run;

use super::{
    code_span::parse_code_span,
    delimiter::parse_delimiter_run,
    emphasis::process_emphasis,
    icu::parse_icu,
    ICUMarkdownParser,
    link::{parse_image_open, parse_link_like_close, parse_link_open},
    text::parse_plain_text,
};

/// Parse any series of inline content. This function should _only_ be called from a block context
/// or an ICU context, because it pushes a new delimiter context to use and processes it at the end
/// of the function.
pub(super) fn parse_inline(p: &mut ICUMarkdownParser, is_inside_icu: bool) {
    p.push_delimiter_stack();
    let inline_start = p.mark();

    // First inline phase: tokenizing.
    loop {
        p.skip_whitespace_as_trivia();

        match p.current() {
            SyntaxKind::EOF | SyntaxKind::BLOCK_END | SyntaxKind::INLINE_END => break,
            // Plain text
            SyntaxKind::TEXT => parse_plain_text(p),
            // Emphasis
            SyntaxKind::STAR | SyntaxKind::UNDER => parse_delimiter_run(p, p.current()),
            // Images or ICU unsafe variables
            // Images start with `![`, and unsafe ICU variables start with `!!{`. Because the syntax
            // overlaps, both have to be checked at the same time. The more likely use case here is
            // an ICU unsafe block, so that is checked first.
            SyntaxKind::EXCLAIM => parse_image_open(p),
            // Links
            SyntaxKind::LSQUARE => parse_link_open(p),
            SyntaxKind::RSQUARE => parse_link_like_close(p),
            // Code spans
            // These are parsed predictively, meaning they will parse ahead
            // through the rest of the input right away, trying to find a
            // matching closer. If one is found, the entire content is turned
            // into a code span, otherwise the parser is rewound and all of that
            // content is parsed again in a normal context.
            SyntaxKind::BACKTICK => parse_code_span(p, p.current()),
            // Autolinks
            // These have the same precedence as code spans, so the same
            // process is applied.
            SyntaxKind::LANGLE => {
                let checkpoint = p.checkpoint();
                parse_autolink(p).or_else(|| {
                    p.rewind(checkpoint);
                    parse_plain_text(p)
                })
            }

            // Markdown Extensions
            // Hooks
            // These are just like links or images but with an assumed variable
            // name as the destination.
            SyntaxKind::DOLLAR => parse_hook_open(p),
            // Strikethroughs
            // These are like STAR and UNDER for emphasis, but with _slightly_
            // different rules, so they need to be handled separately.
            SyntaxKind::TILDE => parse_strikethrough_delimiter_run(p, p.current()),

            // ICU
            SyntaxKind::LCURLY | SyntaxKind::UNSAFE_LCURLY => parse_icu(p),
            SyntaxKind::RCURLY if is_inside_icu => break,

            // Anything else is effectively plain text, but kept separate in
            // the event stream for clarity.
            _ => Some(p.bump()),
        };
    }

    // Second inline phase: process nestable delimiters.
    process_emphasis(p, 0..p.delimiter_stack_length());

    inline_start.complete(p, SyntaxKind::INLINE_CONTENT);
    p.pop_delimiter_stack();
}

fn parse_autolink(p: &mut ICUMarkdownParser) -> Option<()> {
    let autolink = p.mark();
    // Whitespace is not allowed within autolinks, so no trivia is skipped.
    p.expect_with_context(SyntaxKind::LANGLE, LexContext::Autolink)?;
    if matches!(
        p.current(),
        SyntaxKind::ABSOLUTE_URI | SyntaxKind::EMAIL_ADDRESS,
    ) {
        p.bump();
    } else {
        return None;
    }
    p.expect(SyntaxKind::RANGLE)?;

    autolink.complete(p, SyntaxKind::AUTOLINK);
    Some(())
}
