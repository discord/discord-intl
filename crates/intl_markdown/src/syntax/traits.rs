use crate::syntax::{SyntaxElement, SyntaxNode};
use crate::SyntaxKind;

pub trait FromSyntax {
    fn from_syntax(node: SyntaxNode) -> Self;
}

impl<T: FromSyntax + Sized> FromSyntaxElement for T {
    fn from_syntax_element(element: SyntaxElement) -> Self {
        T::from_syntax(element.into_node())
    }
}

pub trait FromSyntaxElement {
    fn from_syntax_element(node: SyntaxElement) -> Self;
}

pub trait Syntax {
    /// Return the raw syntax node backing this item.
    fn syntax(&self) -> &SyntaxNode;

    #[allow(unused)]
    fn kind(&self) -> SyntaxKind {
        self.syntax().kind()
    }
}
