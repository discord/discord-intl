#![feature(portable_simd)]
#![feature(iter_collect_into)]
#![feature(substr_range)]
extern crate core;

pub use cst::*;
pub use parser::ICUMarkdownParser;

use crate::compiler::CompiledElement;
pub use crate::syntax::{SourceText, SyntaxKind, SyntaxNode, SyntaxToken};
use syntax::FromSyntax;

mod block_parser;
mod byte_lookup;
mod cjk;
pub mod compiler;
mod cst;
mod delimiter;
pub mod format;
mod html_entities;
mod lexer;
mod parser;
mod syntax;

extern crate intl_allocator;

/// Parse an intl message into a final AST representing the semantics of the message.
pub fn parse_intl_message(content: &str, include_blocks: bool) -> (AnyDocument, CompiledElement) {
    let mut parser = ICUMarkdownParser::new(SourceText::from(content), include_blocks);
    parser.parse();
    let document = parser.finish().to_document();
    let compiled = compiler::compile_document(&document);
    (document, compiled)
}

/// Return a new AnyDocument with the given content as the only value, treated as a raw string with
/// no parsing or semantics applied.
pub fn raw_string_to_document(content: &str) -> AnyDocument {
    AnyDocument::from_syntax(SyntaxNode::new(SyntaxKind::INLINE_CONTENT, None))
}

// pub fn format_to_icu_string(document: &AnyDocument) -> Result<String, std::fmt::Error> {
//     format_icu_string(document)
// }
