#![feature(portable_simd)]

pub use ast::format::format_ast;
pub use ast::process::process_cst_to_ast;
pub use ast::*;
pub use icu::compile::compile_to_format_js;
pub use icu::format::format_icu_string;
pub use icu::tags::DEFAULT_TAG_NAMES;
pub use parser::ICUMarkdownParser;
pub use syntax::SyntaxKind;
pub use token::SyntaxToken;
pub use tree_builder::cst::Document as CstDocument;

pub mod ast;
mod block_parser;
mod byte_lookup;
mod cjk;
mod delimiter;
mod event;
mod html_entities;
mod icu;
mod lexer;
mod parser;
mod syntax;
mod token;
mod tree_builder;

/// Parse an intl message into a final AST representing the semantics of the message.
pub fn parse_intl_message(content: &str, include_blocks: bool) -> Document {
    let mut parser = ICUMarkdownParser::new(content, include_blocks);
    let source = parser.source().clone();
    parser.parse();
    let cst = parser.into_cst();
    process_cst_to_ast(source, &cst)
}

/// Return a new Document with the given content as the only value, treated as a raw string with
/// no parsing or semantics applied.
pub fn raw_string_to_document(content: &str) -> Document {
    Document::from_literal(content)
}

pub fn format_to_icu_string(document: &Document) -> Result<String, std::fmt::Error> {
    format_icu_string(document)
}
