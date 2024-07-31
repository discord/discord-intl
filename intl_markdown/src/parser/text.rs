use super::ICUMarkdownParser;

/// Consume as many sequential plain-text tokens as possible, merging them
/// into a single token before pushing that new token onto the buffer.
pub(super) fn parse_plain_text(p: &mut ICUMarkdownParser) -> Option<()> {
    p.bump();
    Some(())
}
