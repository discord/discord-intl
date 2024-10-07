use crate::SyntaxKind;

use super::ICUMarkdownParser;

/// Prospectively parse ahead through the document, collecting the list of
/// tokens until a matching closing delimiter is found. If there is no match,
/// then the parser is rewound to the opener, it is deactivated to plain text,
/// and parsing continues as normal afterward.
pub(super) fn parse_code_span(p: &mut ICUMarkdownParser, kind: SyntaxKind) -> Option<()> {
    // If the first backticks are escaped, then bump those tokens out, since
    // they won't be included in the delimiter.
    while p.current() == kind && p.current_flags().is_escaped() {
        p.bump();
    }

    if !p.at(kind) {
        return None;
    }

    // If the parser is still at a backtick after that, then start consuming
    // them to create the opening delimiter. As soon as a non-escaped backtick
    // is encountered, no further backticks in the run can be escaped, either.
    let marker = p.mark();
    let open_delimiter_start = p.mark();
    let mut open_count = 0;
    while p.current() == kind && !p.current_flags().is_escaped() {
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
            // inline elements, so the end of a block terminates it's reach.
            SyntaxKind::EOF | SyntaxKind::BLOCK_END => break false,
            // If another delimiter is found, try to match it and complete the
            // codespan, otherwise just continue consuming it.
            SyntaxKind::BACKTICK => {
                let close_delimiter = p.mark();
                p.bump();
                let mut close_count = 1;
                while p.current() == kind && !p.current_flags().is_escaped() {
                    p.bump();
                    close_count += 1;
                }
                // If a match is found, complete the marker and stop parsing,
                // indicating that the marker was completed.
                if open_count == close_count {
                    open_delimiter_start
                        .span_to(open_delimiter_end)
                        .complete(p, SyntaxKind::CODE_SPAN_DELIMITER);
                    close_delimiter.complete(p, SyntaxKind::CODE_SPAN_DELIMITER);
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
