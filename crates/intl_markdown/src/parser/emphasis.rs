use std::ops::Range;

use super::ICUMarkdownParser;
use crate::delimiter::AnyDelimiter;
use crate::delimiter::Delimiter;
use crate::parser::marker::MarkerSpan;
use crate::parser::strikethrough::match_strikethrough;
use crate::syntax::SyntaxKind;

/// Process the delimiter stack entries within the given `range`, matching
/// emphasis nodes as much as possible.
pub(super) fn process_emphasis(p: &mut ICUMarkdownParser, range: Range<usize>) {
    for closer_index in range.clone() {
        let closer = &p.delimiter_stack()[closer_index];
        if !closer.can_close() || !closer.is_active() {
            continue;
        }
        let closer_kind = closer.syntax_kind();

        // Then start looking backwards to find active openers for that
        // closer, either until there are no more openers or until the
        // closer becomes inactive.
        for opener_index in (range.start..closer_index).rev() {
            let opener = &p.delimiter_stack()[opener_index];
            if !opener.is_active() || !opener.can_open() || opener.syntax_kind() != closer_kind {
                continue;
            }

            let match_result = match opener {
                AnyDelimiter::Strikethrough(_) => {
                    match_strikethrough(p, opener_index, closer_index)
                }
                _ => match_emphasis(p, opener_index, closer_index),
            };

            match match_result {
                EmphasisMatchResult::ConsumedCloser | EmphasisMatchResult::ConsumedBoth => break,
                _ => continue,
            }
        }
    }
}

pub(super) enum EmphasisMatchResult {
    NoMatch,
    ConsumedCloser,
    ConsumedOpener,
    ConsumedBoth,
}

/// Given two indices in the delimiter stack that are known to have the same
/// kind and are _able_ to match, this method will check if the delimiters
/// are _allowed_ to be consumed, then consume the maximum number of
/// elements from each, adding the appropriate events in their place until
/// one is fully consumed.
///
/// Returns true if the closer is _fully consumed_ after this match.
pub(super) fn match_emphasis(
    p: &mut ICUMarkdownParser,
    opener_index: usize,
    closer_index: usize,
) -> EmphasisMatchResult {
    // Determine how many characters should be consumed by first checking
    // the rules for matching, then taking the smaller of the two run
    // lengths if they're allowed to match.
    let mut to_consume = {
        let delimiter_stack = &p.delimiter_stack();
        let opener = &delimiter_stack[opener_index];
        let closer = &delimiter_stack[closer_index];

        let total_length = opener.count() + closer.count();

        // "If one of the delimiters can both open and close emphasis, then
        // the sum of the lengths of the delimiter runs containing the
        // opening and closing delimiters must not be a multiple of 3 unless
        // both lengths are multiples of 3.
        if opener.can_open_and_close() || closer.can_open_and_close() {
            // Inverted condition to exit early if not met.
            if total_length % 3 == 0 && (opener.count() % 3 != 0 || closer.count() % 3 != 0) {
                return EmphasisMatchResult::NoMatch;
            }
        }

        std::cmp::min(opener.count(), closer.count())
    };

    while to_consume > 0 {
        let this_consume = std::cmp::min(to_consume, 2);
        let tag_kind = if this_consume == 1 {
            SyntaxKind::EMPHASIS
        } else {
            SyntaxKind::STRONG
        };

        complete_emphasis_and_content_marker_pairs(
            p,
            tag_kind,
            opener_index,
            closer_index,
            this_consume,
        );

        to_consume -= this_consume;
    }

    // Deactivate all the markers between the opener and the closer, since they
    // would've had to complete entirely within that range, which has now been
    // passed over.
    for i in opener_index + 1..closer_index {
        p.deactivate_delimiter(i)
    }

    let delimiter_stack = p.delimiter_stack();
    let opener = &delimiter_stack[opener_index];
    let closer = &delimiter_stack[closer_index];

    if opener.is_active() {
        EmphasisMatchResult::ConsumedCloser
    } else if closer.is_active() {
        EmphasisMatchResult::ConsumedOpener
    } else {
        EmphasisMatchResult::ConsumedBoth
    }
}

/// Consume the given `count` from the given opening and closing delimiters, marking the inner
/// bounds as `INLINE_CONTENT` and the outer bounds as the given `kind`.
pub(super) fn complete_emphasis_and_content_marker_pairs(
    p: &mut ICUMarkdownParser,
    kind: SyntaxKind,
    opener_index: usize,
    closer_index: usize,
    count: usize,
) {
    let (item_open, content_open) = p.delimiter_stack()[opener_index].consume_opening(count);
    let (item_close, content_close) = p.delimiter_stack()[closer_index].consume_closing(count);
    MarkerSpan::new(content_open, content_close).complete(p, SyntaxKind::INLINE_CONTENT);
    MarkerSpan::new(item_open, item_close).complete(p, kind);
}
