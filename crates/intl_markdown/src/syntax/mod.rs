mod element;
mod iterators;
mod kind;
mod node;
pub mod support;
mod text;
mod token;
mod traits;
mod tree;

pub(crate) use traits::{FromSyntax, FromSyntaxElement, Syntax};
pub(crate) use tree::{TreeBuilder, TreeMarker};

pub use element::{SyntaxElement, SyntaxElementRef, SyntaxNodeChildren, SyntaxTokenChildren};
pub use iterators::{
    MinimalTextIter, SyntaxIterator, SyntaxNodeTokenIter, TokenTextIter, TokenTextIterOptions,
};
pub use kind::SyntaxKind;
#[allow(unused)]
pub use node::{SyntaxNode, SyntaxNodeHeader};
pub use text::TextPointer;
#[allow(unused)]
pub use token::{SourceText, SyntaxToken, TextSize, TextSpan, TrimKind};
