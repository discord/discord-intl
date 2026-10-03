use crate::compiler::{CompiledElement, CompiledNode, IcuNode};

impl CompiledElement {
    /// Convenience method for creating "empty content". Typically used for tags where a list value
    /// is expected but none is present. This ensures that empty values do not become `null` in a
    /// serialized AST.
    pub fn empty_list() -> CompiledElement {
        CompiledElement::List(Box::new([]))
    }

    /// Conditionally wraps the `content` to ensure that it is a `List` variant. If the content is
    /// already a list, it is returned as-is. Otherwise, it is wrapped into a new Box slice and
    /// that list element is returned.
    pub fn list_from(content: impl Into<CompiledElement>) -> CompiledElement {
        match content.into() {
            element @ CompiledElement::BlockList(_) | element @ CompiledElement::List(_) => element,
            element => CompiledElement::List(Box::from([element])),
        }
    }
}

impl From<Box<[CompiledElement]>> for CompiledElement {
    fn from(list: Box<[CompiledElement]>) -> Self {
        CompiledElement::List(list)
    }
}

// TODO: Make this a `try_from`.
impl From<CompiledElement> for IcuNode {
    fn from(value: CompiledElement) -> Self {
        match value {
            CompiledElement::Node(CompiledNode::Icu(node)) => node,
            t => panic!("Converting {t:?} to IcuNode is not possible"),
        }
    }
}
