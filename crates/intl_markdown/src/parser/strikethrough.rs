use crate::delimiter::{Delimiter, StrikethroughDelimiter};
use crate::parser::delimiter::get_surrounding_delimiter_state;
use crate::parser::emphasis::{complete_emphasis_and_content_marker_pairs, EmphasisMatchResult};
use crate::parser::marker::MarkerSpan;
use crate::{syntax::SyntaxKind, ICUMarkdownParser};

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
pub(super) fn parse_strikethrough_delimiter_run(
    p: &mut ICUMarkdownParser,
    kind: SyntaxKind,
) -> Option<()> {
    let marker_index = p.tree_index();

    // Determining whether the run can open or close relies on the fact that
    // the property is transitive across the sequence of delimiter tokens. If
    // the first token in the run can open emphasis, then all other tokens
    // in the run _must_ be able to open emphasis, and the same for the last
    // token being able to close emphasis. Note that this is only true
    // because delimiters are considered "removed from the text" when they
    // are consumed, so once one is consumed, the following ones shift into
    // their place.

    let first_span = p.lexer.current_byte_span();
    let mut last_span = first_span.clone();
    let mut count = 0;
    while p.current() == kind {
        last_span = p.lexer.current_byte_span();
        count += 1;
        p.bump();
    }

    // Strikethrough delimiters are capped at two characters. They can't nest,
    // and they can't be partially consumed, so if there were more than two
    // tokens matched, this can't be a delimiter, and no more work needs to be
    // done.
    if count > 2 {
        return None;
    }

    let delimiter_state = get_surrounding_delimiter_state(p, first_span, last_span);
    // Double tildes for strikethroughs are flanking and can open so long as they are not
    // surrounded by whitespace. Single tildes match the normal flanking rules.
    let can_open_emphasis = if count == 1 {
        delimiter_state.is_left_flanking()
    } else {
        delimiter_state.has_following_whitespace
    };

    let can_close_emphasis = if count == 1 {
        delimiter_state.is_right_flanking()
    } else {
        delimiter_state.has_preceding_whitespace
    };

    p.push_delimiter(
        StrikethroughDelimiter::new(
            kind,
            count,
            can_open_emphasis,
            can_close_emphasis,
            marker_index,
        )
        .into(),
    );

    Some(())
}

pub(super) fn match_strikethrough(
    p: &mut ICUMarkdownParser,
    opener_index: usize,
    closer_index: usize,
) -> EmphasisMatchResult {
    let count = {
        let delimiter_stack = &p.delimiter_stack();
        let opener = &delimiter_stack[opener_index];
        let closer = &delimiter_stack[closer_index];

        // The counts must match and there can be no more than two delimiters to
        // create a strikethrough.
        if opener.count() != closer.count() {
            return EmphasisMatchResult::NoMatch;
        }
        opener.count()
    };

    // If both of those conditions are met, then these can be consumed as
    // a strikethrough pair.
    complete_emphasis_and_content_marker_pairs(
        p,
        SyntaxKind::STRIKETHROUGH,
        opener_index,
        closer_index,
        count,
    );

    // Deactivate all the markers between the opener and the closer, since they
    // would've had to complete entirely within that range, which has now been
    // passed over.
    for i in opener_index + 1..closer_index {
        p.deactivate_delimiter(i)
    }

    EmphasisMatchResult::ConsumedBoth
}
