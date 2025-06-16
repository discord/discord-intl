use crate::syntax::{SyntaxElement, SyntaxNode};

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
