#![feature(substr_range)]
#![feature(iter_collect_into)]

mod element;
pub mod html_entities;
mod iterators;
mod kind;
mod node;
pub mod support;
mod text;
mod token;
mod traits;
mod tree;

pub use element::{SyntaxElement, SyntaxNodeChildren, SyntaxTokenChildren};
pub use iterators::{
    MinimalTextIter, PositionalIterator, SyntaxNodeTokenIter, TokenTextIter, TokenTextIterOptions,
};
pub use kind::SyntaxKind;
pub use node::{SyntaxNode, SyntaxNodeHeader};
pub use text::TextPointer;
pub use token::{SourceText, SyntaxToken, TextSize, TextSpan, TrimKind};
pub use traits::{EqIgnoreSpan, FromSyntax, FromSyntaxElement, Syntax};
pub use tree::{TreeBuilder, TreeMarker};
