use crate::syntax::{
    FromSyntax, FromSyntaxElement, SyntaxElementChildren, SyntaxNodeChildren, SyntaxToken,
};
use crate::SyntaxNode;
use std::marker::PhantomData;

pub(super) fn required_node<N: FromSyntax>(syntax: &SyntaxNode, slot: usize) -> N {
    N::from_syntax(syntax.required_node(slot))
}
pub(super) fn optional_node<N: FromSyntax>(syntax: &SyntaxNode, slot: usize) -> Option<N> {
    syntax.optional_node(slot).map(FromSyntax::from_syntax)
}
pub(super) fn required_token(syntax: &SyntaxNode, slot: usize) -> SyntaxToken {
    syntax.required_token(slot)
}
pub(super) fn optional_token(syntax: &SyntaxNode, slot: usize) -> Option<SyntaxToken> {
    syntax.optional_token(slot)
}

pub struct TypedNodeChildren<'a, T: FromSyntax> {
    children: SyntaxNodeChildren<'a>,
    _phantom: PhantomData<&'a T>,
}
impl<'a, T: FromSyntax> TypedNodeChildren<'a, T> {
    pub fn new(children: SyntaxNodeChildren<'a>) -> Self {
        Self {
            children,
            _phantom: PhantomData,
        }
    }
}
impl<'a, T: FromSyntax> Iterator for TypedNodeChildren<'a, T> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        self.children.next().map(T::from_syntax)
    }
}

pub struct AstElementChildren<'a, T: FromSyntaxElement> {
    children: SyntaxElementChildren<'a>,
    _phantom: PhantomData<&'a T>,
}
impl<'a, T: FromSyntaxElement> AstElementChildren<'a, T> {
    pub fn new(children: SyntaxElementChildren<'a>) -> Self {
        Self {
            children,
            _phantom: PhantomData,
        }
    }
}
impl<'a, T: FromSyntaxElement> Iterator for AstElementChildren<'a, T> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        self.children.next().map(T::from_syntax_element)
    }
}
