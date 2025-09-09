use intl_markdown_syntax::{FromSyntax, PositionalIterator, SyntaxNodeChildren};
use std::marker::PhantomData;

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

impl<T: FromSyntax> ExactSizeIterator for TypedNodeChildren<'_, T> {
    fn len(&self) -> usize {
        self.children.len()
    }
}
impl<T: FromSyntax> PositionalIterator for TypedNodeChildren<'_, T> {}
