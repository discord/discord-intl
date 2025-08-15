use crate::SyntaxElement;
use std::fmt::Write;

/// A non-semantic printer that directly writes the full text content of all tokens in a
/// `[SyntaxElement]`. No logic around node types or semantics is performed, and no formatting or
/// trimming of any kind will be applied, either.
///
/// This is most useful for re-rendering a parsed tree into a string, generally after changing the
/// structure somehow, like mutating or replacing nodes during a lint fix.
pub fn print_syntax(output: &mut impl Write, syntax: SyntaxElement) -> std::fmt::Result {
    match syntax {
        SyntaxElement::Node(node) => {
            for text in node.iter_tokens().into_text_iter() {
                output.write_str(&text)?;
            }
        }
        SyntaxElement::Token(token) => output.write_str(&token.text())?,
        SyntaxElement::Empty => {}
    }
    Ok(())
}
