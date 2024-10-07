use std::ops::Range;

use crate::{delimiter::EmphasisDelimiter, event::Event, SyntaxKind};
use crate::delimiter::Delimiter;
use crate::parser::emphasis::process_emphasis;

use super::ICUMarkdownParser;

/// Consume a sequence of contiguous delimiter tokens of the same kind to
/// create a new Delimiter stack entry with the kind and number of tokens
/// consumed. This will also collate the bounds of whether the run can start
/// and/or end emphasis.
///
/// Because delimiter runs can be split into any number of sub-runs depending on
/// which matching delimiters are encountered later on, each token of the
/// delimiter has to be tracked separately. In addition, each token needs a
/// marker added before and after it, to allow the processor to potentially mark
/// them as actual node boundaries afterward.
pub(super) fn parse_delimiter_run(p: &mut ICUMarkdownParser, kind: SyntaxKind) -> Option<()> {
    // Determining whether the run can open or close relies on the fact that
    // the property is transitive across the sequence of delimiter tokens. If
    // the first token in the run can open emphasis, then all other tokens
    // in the run _must_ be able to open emphasis, and the same for the last
    // token being able to close emphasis. Note that this is only true
    // because delimiters are considered "removed from the text" when they
    // are consumed, so once one is consumed, the following ones shift into
    // their place.
    let first_flags = p.current_flags();

    let index = p.buffer_index() + 1;
    let mut last_flags = first_flags;
    let mut count = 0;

    while p.current() == kind {
        last_flags = p.current_flags();
        count += 1;

        // Wrap each token with a Start and Finish event that the Delimiter will
        // point to when inserting the actual event kinds while processing.
        p.push_event(Event::Start(SyntaxKind::TOMBSTONE));
        p.bump();
        p.push_event(Event::Finish(SyntaxKind::TOMBSTONE));
    }

    // Underscores are not able to create intra-word emphasis, meaning strings
    // like `foo_bar_` do not crete emphasis, but `foo*bar*` does.
    if kind == SyntaxKind::UNDER {
        if !first_flags.has_preceding_whitespace()
            && !first_flags.has_preceding_punctuation()
            && !last_flags.has_following_punctuation()
            && !last_flags.has_following_whitespace()
        {
            return None;
        }
    }

    let is_right_flanking = // Right-flanking definition:
        // 1. Not preceded by whitespace AND
        !first_flags.has_preceding_whitespace()
            // 2. Either:
            && (
                // - not preceded by a punctuation. OR
                !first_flags.has_preceding_punctuation()
                // - preceded by punctuation but followed by whitespace or punctuation
                || (last_flags.has_following_whitespace() || last_flags.has_following_punctuation())
            );

    // Left-flanking definition
    // 1. Not followed by whitespace AND
    let is_left_flanking = !last_flags.has_following_whitespace()
            // 2. Either:
            && (
                // - not followed by a punctuation. OR
                !last_flags.has_following_punctuation()
                // - followed by punctuation but preceded by whitespace or punctuation.
                || (first_flags.has_preceding_whitespace() || first_flags.has_preceding_punctuation())
            );

    // Using the determined flanking and context flags and the `kind` of the
    // token, determine if it can be used to open and/or close emphasis.
    let can_open_emphasis = match kind {
        // Rule 2. A single _ character can open emphasis iff it is part of
        // a left-flanking delimiter run and either (a) not part of a
        // right-flanking delimiter run or (b) part of a right-flanking
        // delimiter run preceded by a Unicode punctuation character.
        SyntaxKind::UNDER => is_left_flanking || first_flags.has_preceding_punctuation(),
        SyntaxKind::STAR => is_left_flanking,
        _ => false,
    };

    let can_close_emphasis = match kind {
        SyntaxKind::UNDER => is_right_flanking || last_flags.has_following_punctuation(),
        SyntaxKind::STAR => is_right_flanking,
        _ => false,
    };

    p.push_delimiter(
        EmphasisDelimiter::new(kind, count, can_open_emphasis, can_close_emphasis, index).into(),
    );

    Some(())
}

/// Update the delimiter stack based on the successful closure of a link or
/// other inline, non-nestable element (like inline code) that starts with
/// the delimiter at the given stack index and ends at the top of the stack.
///
/// If `process_inner_emphasis` is true, emphasis from the given opening
/// index to the top of the delimiter stack will be processed immediately.
///
/// If `deactivate_previous` is true, all preceding instances of `kind` in the
/// delimiter stack are also deactivated to prevent nested elements from
/// occurring.
pub(crate) fn process_closed_delimiter(
    p: &mut ICUMarkdownParser,
    delimiter_range: Range<usize>,
    kind: SyntaxKind,
    process_inner_emphasis: bool,
    deactivate_previous: bool,
) {
    // Deactivate the link delimiter since it's been completed now.
    p.deactivate_delimiter(delimiter_range.start);

    // Links act as boundaries for emphasis, so when a completed link is
    // encountered, all the pending emphasis between the opener and the end
    // of the link should get processed independently. But inline code spans do
    // not interpret any of the characters within their span, so only process
    // them if the caller requested it.
    if process_inner_emphasis {
        process_emphasis(p, delimiter_range.clone());
    }

    // Links cannot be nested, so after finding an opener, all further link
    // openers lower than it in the stack can be marked as inactive.
    // The spec algorithm suggests removing these from the stack, but we depend
    // on all delimiters staying in the stack to construct the full event list
    // after parsing.
    if deactivate_previous {
        for i in 0..delimiter_range.start {
            let delimiter = &mut p.delimiter_stack()[i];
            if delimiter.kind() == kind && delimiter.is_active() {
                p.deactivate_delimiter(i);
            }
        }
    }

    // Similarly, after processing all the emphasis between the link bounds, all the delimiters
    // should be deactivated so that they aren't considered for use when combing over the entire
    // stack at the end of parsing.
    for i in delimiter_range {
        p.deactivate_delimiter(i);
    }
}
