use super::{delimiter::process_closed_delimiter, ICUMarkdownParser};
use crate::lexer::LexContext;
use crate::parser::icu::{is_at_normal_icu, parse_icu};
use crate::parser::marker::{Marker, MarkerSpan};
use crate::{
    delimiter::{Delimiter, LinkDelimiter},
    syntax::SyntaxKind,
};

fn is_link_kind(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::LINK | SyntaxKind::IMAGE | SyntaxKind::HOOK
    )
}

fn is_allowed_link_title_whitespace(kind: SyntaxKind) -> bool {
    matches!(kind, SyntaxKind::WHITESPACE | SyntaxKind::LINE_ENDING)
}

pub(super) fn parse_hook_open(p: &mut ICUMarkdownParser) -> Option<()> {
    let hook_start = p.mark();
    p.expect(SyntaxKind::DOLLAR)?;
    parse_link_like_open(p, SyntaxKind::HOOK, hook_start)
}

pub(super) fn parse_image_open(p: &mut ICUMarkdownParser) -> Option<()> {
    let image_start = p.mark();
    p.expect(SyntaxKind::EXCLAIM)?;
    parse_link_like_open(p, SyntaxKind::IMAGE, image_start)
}

pub(super) fn parse_link_open(p: &mut ICUMarkdownParser) -> Option<()> {
    let link_start = p.mark();
    parse_link_like_open(p, SyntaxKind::LINK, link_start)
}

pub(super) fn parse_link_like_open(
    p: &mut ICUMarkdownParser,
    kind: SyntaxKind,
    start_marker: Marker,
) -> Option<()> {
    p.expect(SyntaxKind::LSQUARE)?;
    // Links have two wrapping events. The Link container in total, and then
    // a container for just the content. Needed because the content can contain
    // balanced square braces, which would be indistinguishable when processing
    // the event stream otherwise.
    let content_start = p.mark();

    let delimiter = LinkDelimiter::new(kind, false, *start_marker, *content_start);
    p.push_delimiter(delimiter.into());

    Some(())
}

pub(super) fn parse_link_like_close(p: &mut ICUMarkdownParser) -> Option<()> {
    let content_end_index = p.mark();
    p.expect(SyntaxKind::RSQUARE)?;

    // Find a potential opener for this link
    let Some(opener_index) = p
        .delimiter_stack()
        .iter()
        .rposition(|delim| is_link_kind(delim.syntax_kind()) && delim.count() > 0)
    else {
        // If no opener is found, then this can't be a link no matter what, so
        // there's nothing left to do, it's just plain text.
        return None;
    };

    let opening_kind = {
        // This delimiter will be matched no matter what, so go ahead and consume it
        // so that it won't be matched in the future.
        let delimiter = &mut p.delimiter_stack()[opener_index];
        delimiter.consume_opening(1);

        // If the delimiter isn't active, then it gets consumed, but the result
        // is just plain text, so the rest of parsing for it can be skipped.
        if !delimiter.is_active() {
            return None;
        }

        delimiter.syntax_kind()
    };

    // Otherwise, try to finish out the link as an inline resource. If the
    // resource ends up being incomplete, then this still isn't a link, the
    // parser gets rewound to the closing brace, and it's treated as plain text.
    let checkpoint = p.checkpoint();
    let resource = parse_link_or_hook_resource(p, opening_kind);
    if resource.is_none() {
        // Since there was a matching opener, it becomes a balanced pair, and
        // the opener gets deactivated to avoid matching future braces.
        p.deactivate_delimiter(opener_index);
        p.rewind(checkpoint);
        return None;
    }

    // Finally, if everything was valid _and_ the link can be completed, then
    // all the interior content can be processed.
    let allow_nesting = matches!(opening_kind, SyntaxKind::IMAGE | SyntaxKind::HOOK);
    complete_link_like(
        p,
        opening_kind,
        opener_index,
        content_end_index,
        allow_nesting,
    );

    Some(())
}

pub(super) fn complete_link_like(
    p: &mut ICUMarkdownParser,
    kind: SyntaxKind,
    open_delimiter_index: usize,
    content_end_index: Marker,
    allow_nesting: bool,
) {
    process_closed_delimiter(
        p,
        open_delimiter_index..p.delimiter_stack_length(),
        kind,
        !allow_nesting,
    );

    let (content_marker, link_marker) = {
        let delimiter = p.delimiter_stack()[open_delimiter_index]
            .as_link_delimiter()
            .unwrap();
        (*delimiter.content_cursor(), *delimiter.link_cursor())
    };

    // Link content uses the `closing_cursor` of the delimiters.
    MarkerSpan::new(content_marker, *content_end_index).complete(p, SyntaxKind::INLINE_CONTENT);

    // The link wrapper uses the `opening_cursor` of each delimiter.
    Marker::new(link_marker).complete(p, kind);
}

fn parse_link_or_hook_resource(p: &mut ICUMarkdownParser, kind: SyntaxKind) -> Option<()> {
    match kind {
        SyntaxKind::LINK | SyntaxKind::IMAGE => parse_link_resource(p),
        SyntaxKind::HOOK => parse_hook_name(p),
        _ => unreachable!("parse_link_or_hook_resource can only be called with a known link type"),
    }
}

fn parse_hook_name(p: &mut ICUMarkdownParser) -> Option<()> {
    let name_mark = p.mark();
    p.expect(SyntaxKind::LPAREN)?;
    p.expect(SyntaxKind::TEXT)?;
    p.expect(SyntaxKind::RPAREN)?;
    name_mark.complete(p, SyntaxKind::HOOK_NAME)
}

fn parse_link_resource(p: &mut ICUMarkdownParser) -> Option<()> {
    let marker = p.mark();

    p.expect(SyntaxKind::LPAREN)?;
    // Links allow whitespace and a single newline between elements of the
    // resource.
    p.skip_whitespace_as_trivia();
    if p.expect(SyntaxKind::LINE_ENDING).is_some() {
        p.skip_whitespace_as_trivia();
    }

    if p.at(SyntaxKind::RPAREN) {
        p.push_missing(); // Missing destination element
        p.push_missing(); // Missing title token
        p.bump();
        marker.complete(p, SyntaxKind::LINK_RESOURCE);
        return Some(());
    }

    parse_link_destination(p)?;

    // Whitespace and a single newline are allowed between the destination and
    // the title, and the title can _only_ appear if there is some whitespace
    // between them, so it is nested inside here.
    p.optional(is_allowed_link_title_whitespace(p.current()), |p| {
        p.skip_whitespace_as_trivia();
        parse_link_title(p)?;
        // Whitespace and a single newline are also allowed between the title
        // and the ending.
        p.skip_whitespace_as_trivia();
        Some(())
    });

    // The next token afterward _must_ be a closing parenthesis. Any other token
    // causes the link to break and be treated as plain text instead.
    p.expect(SyntaxKind::RPAREN)?;
    marker.complete(p, SyntaxKind::LINK_RESOURCE)
}

/// In Markdown, a link destination can be any series of non-whitespace tokens and gets interpreted
/// literally. But with ICU, we also want to allow a dynamic link url using an ICU variable, like
/// `[some link]({someUrl})`. _Ideally_, this syntax would actually be elided and every link url
/// would be considered a dynamic variable, but it's still likely in the short term that strings
/// will contain static links, like `[some link](some/url/path)`, which should _not_ be considered
/// ICU variables.
///
/// With the same syntax, link destinations can also become click handlers, where a single
/// identifier as a destination dictates a variable name to provide a handling function, such as
/// `[click me](onClick)`. This syntax is an extension to normal Markdown rules _and_ is separate
/// from ICU syntax.
fn parse_link_destination(p: &mut ICUMarkdownParser) -> Option<()> {
    let marker = p.mark();

    if is_at_normal_icu(p) {
        parse_icu(p)?;
        return marker.complete(p, SyntaxKind::DYNAMIC_LINK_DESTINATION);
    }

    // The LinkDestination context disallows merging consecutive text tokens,
    // meaning any whitespace in the destination always ends the token.
    p.relex_with_context(LexContext::LinkDestination);

    // Otherwise parse some text for the url. It can be any combination of
    // tokens _other_ than whitespace, newlines, or a closing parenthesis.
    let mut balance = 1;
    let mut token_count = 0;
    loop {
        match p.current() {
            SyntaxKind::EOF | SyntaxKind::BLOCK_END => break,
            SyntaxKind::WHITESPACE | SyntaxKind::LINE_ENDING => break,
            SyntaxKind::RPAREN if balance == 1 => break,
            SyntaxKind::RPAREN => balance -= 1,
            SyntaxKind::LPAREN => balance += 1,
            _ => {}
        }
        p.bump_with_context(LexContext::LinkDestination);
        token_count += 1;
    }

    // If there's only one token in the destination, it might be an identifier
    // and qualify as a click handler rather than a static link.
    if token_count == 1 {
        // SAFETY: The condition asserts that a token was pushed, so this must
        // be present _and_ be a token.
        let token = p.last_element().unwrap().token();
        // If the text _doesn't_ contain characters that _aren't_ alphanumeric,
        // then it's a valid identifier in this context and counts as a click
        // handler.
        if !token.text().contains(|c: char| !c.is_ascii_alphanumeric()) {
            return marker.complete(p, SyntaxKind::CLICK_HANDLER_LINK_DESTINATION);
        }
    }

    marker.complete(p, SyntaxKind::STATIC_LINK_DESTINATION)
}

fn parse_link_title(p: &mut ICUMarkdownParser) -> Option<()> {
    let marker = p.mark();

    // Link titles are allowed to appear on a new line after the destination, so we need to accept
    // an optional line ending token to represent that.
    if p.current().is_line_ending() {
        p.bump();
        p.skip_whitespace_as_trivia();
    } else {
        p.push_missing();
    }

    let end_quote_kind = match p.current() {
        SyntaxKind::DOUBLE_QUOTE => SyntaxKind::DOUBLE_QUOTE,
        SyntaxKind::QUOTE => SyntaxKind::QUOTE,
        SyntaxKind::LPAREN => SyntaxKind::RPAREN,
        _ => return None,
    };
    p.bump();

    let content_start = p.mark();

    while p.current() != end_quote_kind && p.current() != SyntaxKind::BLOCK_END {
        p.bump();
    }

    let content_end = p.mark();
    p.expect(end_quote_kind)?;

    content_start
        .span_to(content_end)
        .complete(p, SyntaxKind::LINK_TITLE_CONTENT);
    marker.complete(p, SyntaxKind::LINK_TITLE)
}
