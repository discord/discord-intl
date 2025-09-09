#![feature(portable_simd)]
#![feature(iter_collect_into)]
#![feature(substr_range)]
extern crate core;
extern crate intl_allocator;

pub use cst::*;
pub use intl_markdown_syntax::{
    FromSyntax, SourceText, SyntaxKind, SyntaxNode, SyntaxToken, TextPointer,
};
pub use parser::ICUMarkdownParser;

use crate::compiler::CompiledElement;

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
    pub compiled: CompiledElement,
}

/// Parse an intl message into a final AST representing the semantics of the message.
pub fn parse_intl_message(content: &str, include_blocks: bool) -> MarkdownDocument {
    let mut parser = ICUMarkdownParser::new(SourceText::from(content), include_blocks);
    parser.parse();
    let document = parser.finish().to_document();
    let compiled = compiler::compile_document(&document);
    MarkdownDocument {
        cst: document,
        compiled,
    }
}

/// Return a new MarkdownDocument with the given content as the only value, treated as a raw string
/// with no parsing or semantics applied.
pub fn raw_string_to_document(content: &str) -> MarkdownDocument {
    let cst = AnyDocument::from_syntax(SyntaxNode::new(
        SyntaxKind::INLINE_CONTENT,
        [SyntaxToken::from_str(SyntaxKind::TEXT, content).into()].into_iter(),
    ));
    let compiled = CompiledElement::Literal(TextPointer::from_str(content));
    MarkdownDocument { cst, compiled }
}
