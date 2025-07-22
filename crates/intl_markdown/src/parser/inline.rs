use super::{
    code_span::parse_code_span,
    delimiter::parse_delimiter_run,
    emphasis::process_emphasis,
    icu::parse_icu,
    link::{parse_image_open, parse_link_like_close, parse_link_open},
    ICUMarkdownParser,
};
use crate::parser::icu::parse_icu_pound;
use crate::parser::link::parse_hook_open;
use crate::parser::strikethrough::parse_strikethrough_delimiter_run;
use crate::{lexer::LexContext, syntax::SyntaxKind};

/// Parse any series of inline content. This function should _only_ be called from a block context
/// or an ICU context, because it pushes a new delimiter context to use and processes it at the end
/// of the function.
pub(super) fn parse_inline(p: &mut ICUMarkdownParser, is_inside_icu: bool) {
    p.push_delimiter_stack();
    let inline_start = p.mark();

    // Consume any leading whitespace in a block content node as a separate text token so that it
    // doesn't get incidentally applied to anything inside the inline content.
    // See `tests::spec_regression::regression_1` for an example.
    if p.current().is_leading_whitespace() {
        let mark = p.mark();
        p.bump_as_trivia_only_token(LexContext::InlineWhitespace);
        mark.complete(p, SyntaxKind::TEXT_SPAN);
    }

    // First inline phase: tokenizing.
    loop {
        // Leading trivia is allowed for a complete segment of inline content, but if some trivia
        // would be considered leading while _within_ the inline content, it is instead treated as
        // actual token text. So, to start, we skip the leading whitespace as trivia, but all other
        // methods.
        p.skip_whitespace_as_trivia();

        match p.current() {
            SyntaxKind::EOF | SyntaxKind::BLOCK_END | SyntaxKind::INLINE_END => break,
            // Emphasis
            SyntaxKind::STAR | SyntaxKind::UNDER => parse_delimiter_run(p, p.current()),
            // Links and Images
            SyntaxKind::EXCLAIM => {
                // Most CommonMark rules work well and aren't a concern for conflicting with
                // natural language syntax, but sometimes things overlap a little bit. For example,
                // the image syntax `![]` is ambiguous with a natural link following an
                // exclamation, like `hello![foo](./bar)`. In reality, the correct thing to do here
                // is add a space between either the `o` and `!` to create a phrase and an image
                // _or_ between the `!` and the `[` to create a phrase and a regular link. However,
                // since most of the time we're working with untrustable user input for intl
                // messages, we want a way to more definitively distinguish them. It's also
                // exceedingly rare for an image to be intentional, so preferring the link tag is
                // more natural.
                // `peek_back(2)` gets the character _before_ the `!`.
                let prev = p.lexer.peek_back(2);
                if prev == b'\0' || prev.is_ascii_whitespace() || prev.is_ascii_punctuation() {
                    parse_image_open(p)
                } else {
                    p.bump();
                    continue;
                }
            }
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
                    p.bump();
                    Some(())
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
            SyntaxKind::HASH if is_inside_icu => parse_icu_pound(p),
            // Plain text
            // Anything else is effectively plain text, but kept separate in the event stream for
            // clarity. While these end up falling into their own Nodes in the tree, they are
            // allowed to have trailing trivia added to them because the text span is what's
            // responsible for rendering that trivia.
            SyntaxKind::TEXT | _ => {
                p.bump();
                continue;
            }
        };

        // For all _nodes_ in inline content, re-lex any trailing inline whitespace as a separate
        // token, so that it does not get attached to a token inside a nested node. Keeping them
        // separate ensures downstream processing can easily preserve this space without having to
        // traverse adjacent nodes to find trivia text.
        parse_trailing_inline_text(p);
    }

    // Second inline phase: process nestable delimiters.
    process_emphasis(p, 0..p.delimiter_stack_length());
    inline_start.complete(p, SyntaxKind::INLINE_CONTENT);
    p.pop_delimiter_stack();
}

/// Special method for parsing whitespace in contexts where leading and trailing trivia are not
/// meaningful (and typically cause more complication than they help). For example, inline
/// content is made up of heterogeneous mixes of tokens and nodes, but whitespace between nodes
/// and tokens must be respected. Since trivia is by default added as trailing trivia of a
/// _token_, ensuring that the trivia appears _outside_ of a node (like spaces after an
/// emphasis, like `**foo** bar`) becomes a complex traversal of the node tree. Similarly, if
/// it is treated as leading trivia of the _following_ token, it requires conditional logic for
/// when leading trivia should be rendered, since we typically skip leading trivia for all
/// block nodes (like the leading text of a paragraph, `   foo`, would become `foo`).
///
/// By allowing the parser to instead treat these trivia as text tokens, it can maintain
/// significance and be preserved in all formatting contexts _without_ any special processing.
fn parse_trailing_inline_text(p: &mut ICUMarkdownParser) {
    if p.current().is_trivia() {
        p.relex_with_context(LexContext::InlineWhitespace);
    }
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
