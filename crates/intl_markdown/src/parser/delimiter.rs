use std::ops::Range;

use super::ICUMarkdownParser;
use crate::cjk::is_cjk_codepoint;
use crate::delimiter::Delimiter;
use crate::parser::emphasis::process_emphasis;
use crate::token::TextSpan;
use crate::{cjk, delimiter::EmphasisDelimiter, event::Event, SyntaxKind};

/// Returns the two chars preceding `first` and two chars following `last` in
/// the source text. If the spans are at the bounds of the text, this will
/// return [`None`] for the characters in those positions instead.
fn get_surrounding_chars(
    p: &mut ICUMarkdownParser,
    first: TextSpan,
    last: TextSpan,
) -> [Option<char>; 3] {
    let mut result: [Option<char>; 3] = [None; 3];
    if let Some(mut preceding) = p.source.get(..first.start as usize).map(|i| i.chars()) {
        result[1] = preceding.next_back();
        result[0] = preceding.next_back();
    }
    if let Some(mut following) = p.source.get(last.end as usize..).map(|i| i.chars()) {
        result[2] = following.next();
    }
    result
}

fn is_preceding_cjk(prev2: Option<char>, prev: Option<char>) -> bool {
    let Some(prev) = prev else {
        return false;
    };

    if let Some(prev2) = prev2 {
        is_cjk_codepoint(
            prev2,
            // If `prev2` is a quote, check if `prev` is a variation selector to
            // make it right-justified.
            // https://github.com/tats-u/markdown-cjk-friendly/blob/aa749152266ed889be41a8de40802659ec35758d/packages/markdown-it-cjk-friendly/src/index.ts#L350-L352
            prev as u32 == 0xfe01 && cjk::is_cjk_quotemark(prev2),
        )
    } else {
        is_cjk_codepoint(prev, cjk::is_ivs_codepoint(prev))
    }
}

/// Consume a sequence of contiguous delimiter tokens with the same kind to
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
    let first_span = p.lexer.current_byte_span();

    let index = p.buffer_index() + 1;
    let mut last_flags = first_flags;
    let mut last_span = first_span.clone();
    let mut count = 0;

    while p.current() == kind {
        last_flags = p.current_flags();
        last_span = p.lexer.current_byte_span();
        count += 1;

        // Wrap each token with a Start and Finish event that the Delimiter will
        // point to when inserting the actual event kinds while processing.
        p.push_event(Event::Start(SyntaxKind::TOMBSTONE));
        p.bump();
        p.push_event(Event::Finish(SyntaxKind::TOMBSTONE));
    }

    // Using the expanded spec from:
    // https://github.com/tats-u/markdown-cjk-friendly/blob/main/specification.md

    let [mut prev2, prev_raw, next] = get_surrounding_chars(p, first_span, last_span);
    let mut prev = prev_raw;
    // "Sequences" in this spec say that if `prev` is a non-emoji general use variation selector,
    // then it's effectively transparent and `prev2` becomes the important character.
    // https://github.com/tats-u/markdown-cjk-friendly/blob/aa749152266ed889be41a8de40802659ec35758d/packages/markdown-it-cjk-friendly/src/index.ts#L401-L406
    if prev.map_or(false, cjk::is_non_emoji_general_use_variation_selector) {
        // If prev2 is whitespace or the start of the text, it's not meaningful.
        if !prev2.map_or(true, char::is_whitespace) {
            prev = prev2
        }
    } else {
        // If the previous character isn't a variation selector, then the value
        // preceding that also isn't usable.
        prev2 = None;
    }

    let has_preceding_cjk = is_preceding_cjk(prev2, prev_raw);

    let has_following_cjk = next.map_or(false, |c| is_cjk_codepoint(c, false));
    let has_preceding_punctuation = prev.map_or(false, cjk::is_punctuation)
        // NOTE(faulty): This is a little strange, but for cases of unicode art
        // messages like `¯\\_(ツ)_/¯`, we don't want the underscores to end up
        // turning into emphasis on the face, but by Markdown's rules, that's
        // exactly what should happen. To get around this, we're making a small
        // exception that `\` is not considered a punctuation character in the
        // context of emphasis delimiters. This check works because the only
        // other meaning of a `\` would be to escape the delimiter mark itself,
        // which gives the same effect as just disallowing `\` as punctuation
        // anyway. All CommonMark tests still pass with this in place, so it's
        // clearly not a common nor meaningful semantic that needs to exist.
        && !prev.is_some_and(|p| p == '\\');
    let has_following_punctuation = last_flags.has_following_punctuation();
    let has_preceding_non_cjk_punctuation = has_preceding_punctuation && !has_preceding_cjk;
    let has_following_non_cjk_punctuation = has_following_punctuation && !has_following_cjk;

    let has_preceding_whitespace = first_flags.has_preceding_whitespace();
    let has_following_whitespace = last_flags.has_following_whitespace();

    // Underscores are not able to create intra-word emphasis, meaning strings
    // like `foo_bar_` do not create emphasis, but `foo*bar*` does.
    if kind == SyntaxKind::UNDER {
        if !has_preceding_whitespace
            && !has_preceding_punctuation
            && !has_following_punctuation
            && !has_following_whitespace
        {
            return None;
        }
    }

    // Right-flanking definition:
    // 1) not preceded by Unicode whitespace AND
    let is_right_flanking = !has_preceding_whitespace
        // 2. Either:
        && (
            // 2a) not preceded by a non-CJK punctuation sequence. OR
            !has_preceding_non_cjk_punctuation ||
                // 2b) preceded by a non-CJK punctuation character and followed by
                //   2bα: Unicode whitespace
                //   2bβ: a non-CJK punctuation character
                //   2bγ: a CJK character.
                has_following_whitespace || has_following_non_cjk_punctuation || has_following_cjk
        );

    // Left-flanking definition
    // 1. Not followed by whitespace AND
    let is_left_flanking = !has_following_whitespace
        // 2. Either:
        && (
        // 2a) not followed by a non-CJK punctuation character. OR
        !has_following_non_cjk_punctuation
            // 2b) followed by a non-CJK punctuation character and preceded by
            //   2bα: Unicode whitespace
            //   2bβ: a non-CJK punctuation sequence
            //   2bγ: a CJK character
            //   2bδ: an Ideographic Variation Selector
            //   2bε: a CJK sequence
            || has_preceding_whitespace || has_preceding_non_cjk_punctuation || has_preceding_cjk
    );

    // Using the determined flanking and context flags and the `kind` of the
    // token, determine if it can be used to open and/or close emphasis.
    let can_open_emphasis = match kind {
        // Rule 2. A single _ character can open emphasis iff it is part of
        // a left-flanking delimiter run and either (a) not part of a
        // right-flanking delimiter run or (b) part of a right-flanking
        // delimiter run preceded by a Unicode punctuation character.
        SyntaxKind::UNDER => is_left_flanking && (!is_right_flanking || has_preceding_punctuation),
        SyntaxKind::STAR => is_left_flanking,
        _ => false,
    };

    let can_close_emphasis = match kind {
        SyntaxKind::UNDER => is_right_flanking && (!is_left_flanking || has_following_punctuation),
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
