#![feature(portable_simd)]
#![feature(iter_collect_into)]
#![feature(substr_range)]
extern crate core;

pub use cst::*;
// pub use icu::compile::compile_to_format_js;
// pub use icu::format::format_icu_string;
// pub use icu::tags::DEFAULT_TAG_NAMES;
pub use parser::ICUMarkdownParser;

pub use crate::syntax::{SourceText, SyntaxKind, SyntaxNode, SyntaxToken};
use syntax::FromSyntax;

mod block_parser;
mod byte_lookup;
mod cjk;
mod cst;
mod delimiter;
mod html_entities;
// mod icu;
pub mod compiler;
pub mod format;
mod lexer;
mod parser;
mod syntax;

extern crate intl_allocator;

/// Parse an intl message into a final AST representing the semantics of the message.
pub fn parse_intl_message(content: &str, include_blocks: bool) -> AnyDocument {
    let mut parser = ICUMarkdownParser::new(SourceText::from(content), include_blocks);
    parser.parse();
    AnyDocument::from_syntax(parser.finish().tree.node().clone())
}

/// Return a new AnyDocument with the given content as the only value, treated as a raw string with
/// no parsing or semantics applied.
pub fn raw_string_to_document(content: &str) -> AnyDocument {
    AnyDocument::from_syntax(SyntaxNode::new(SyntaxKind::INLINE_CONTENT, None))
}

// pub fn format_to_icu_string(document: &AnyDocument) -> Result<String, std::fmt::Error> {
//     format_icu_string(document)
// }
