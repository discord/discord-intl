use crate::syntax::SyntaxKind;

use super::ICUMarkdownParser;

/// Prospectively parse ahead through the document, collecting the list of
/// tokens until a matching closing delimiter is found. If there is no match,
/// then the parser is rewound to the opener, it is deactivated to plain text,
/// and parsing continues as normal afterward.
pub(super) fn parse_code_span(p: &mut ICUMarkdownParser, kind: SyntaxKind) -> Option<()> {
    if !p.at(kind) {
        return None;
    }

    // If the parser is still at a backtick after that, then start consuming
    // them to create the opening delimiter. As soon as a non-escaped backtick
    // is encountered, no further backticks in the run can be escaped, either.
    let marker = p.mark();
    let open_delimiter_start = p.mark();
    let mut open_count = 0;
    while p.current() == kind {
        p.bump();
        open_count += 1;
    }
    let open_delimiter_end = p.mark();

    // Parsing the content of the codespan is predictive, meaning we don't know
    // if it will actually become a codespan until we've reached a closer. If
    // a closer is never found, then we need to rewind parsing back to the start
    // to be able to re-interpret the content as real Markdown syntax rather
    // than plain text.
    let checkpoint = p.checkpoint();

    let did_complete = loop {
        match p.current() {
            // EOF means the codespan wasn't matched. Spans are also bounded as
            // inline elements, so the end of a block terminates its reach.
            SyntaxKind::EOF | SyntaxKind::BLOCK_END => break false,
            // If another delimiter is found, try to match it and complete the
            // codespan, otherwise just continue consuming it.
            SyntaxKind::BACKTICK | SyntaxKind::ESCAPED_BACKTICK => {
                let codespan_content_end_mark = p.mark();
                let close_delimiter = p.mark();
                p.bump();
                let mut close_count = 1;
                while p.current() == kind {
                    p.bump();
                    close_count += 1;
                }
                // If a match is found, complete the marker and stop parsing,
                // indicating that the marker was completed.
                if open_count == close_count {
                    // ORDER:
                    // - Complete the closing delimiter first, since it's now guaranteed to exist.
                    // - Complete the opening delimiter.
                    // - Complete the content.
                    close_delimiter.complete(p, SyntaxKind::CODE_SPAN_DELIMITER);
                    open_delimiter_start
                        .span_to(open_delimiter_end)
                        .complete(p, SyntaxKind::CODE_SPAN_DELIMITER);
                    open_delimiter_end
                        .span_to(codespan_content_end_mark)
                        .complete(p, SyntaxKind::CODE_SPAN_CONTENT);
                    marker.complete(p, SyntaxKind::CODE_SPAN);
                    break true;
                }
            }
            _ => p.bump(),
        }
    };

    // Reaching this point means the code span wasn't closed, so the parser must
    // be rewound for the caller to continue parsing normally.
    if !did_complete {
        p.rewind(checkpoint);
        return None;
    }

    Some(())
}
