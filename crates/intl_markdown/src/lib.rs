#![feature(portable_simd)]
#![feature(iter_collect_into)]
#![feature(substr_range)]
extern crate core;
extern crate intl_allocator;

use crate::compiler::{compile_document, CompiledElement};
pub use cst::*;
pub use intl_markdown_syntax::{
    FromSyntax, SourceText, SyntaxKind, SyntaxNode, SyntaxToken, TextPointer,
};
pub use parser::ICUMarkdownParser;

mod block_parser;
mod byte_lookup;
mod cjk;
pub mod compiler;
mod cst;
mod delimiter;
pub mod format;
mod lexer;
mod parser;

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct MarkdownDocument {
    pub cst: AnyDocument,
}

impl MarkdownDocument {
    pub fn as_compiled(&self) -> CompiledElement {
        compile_document(&self.cst)
    }
}

/// Parse an intl message into a final AST representing the semantics of the message.
pub fn parse_intl_message(content: &str, include_blocks: bool) -> MarkdownDocument {
    let mut parser = ICUMarkdownParser::new(SourceText::from(content), include_blocks);
    parser.parse();
    let document = parser.finish().to_document();
    MarkdownDocument { cst: document }
}

/// Return a new MarkdownDocument with the given content as the only value, treated as a raw string
/// with no parsing or semantics applied.
pub fn raw_string_to_document(content: &str) -> MarkdownDocument {
    let cst = AnyDocument::from_syntax(SyntaxNode::new(
        SyntaxKind::INLINE_CONTENT,
        0,
        [SyntaxToken::from_str(SyntaxKind::TEXT, 0, content).into()].into_iter(),
    ));
    MarkdownDocument { cst }
}
