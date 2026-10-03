use crate::{SyntaxElement, SyntaxKind, SyntaxNode};

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

pub trait EqIgnoreSpan {
    /// Return true if two elements are equal, ignoring their location in the
    /// source text.
    fn eq_ignore_span(&self, rhs: &Self) -> bool;
}

impl<T: Syntax> EqIgnoreSpan for T {
    fn eq_ignore_span(&self, rhs: &Self) -> bool {
        self.syntax().eq_ignore_span(rhs.syntax())
    }
}
