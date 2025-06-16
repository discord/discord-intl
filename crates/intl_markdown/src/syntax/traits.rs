use crate::syntax::SyntaxNode;

pub trait FromSyntax {
    fn from_syntax(node: SyntaxNode) -> Self;
}
