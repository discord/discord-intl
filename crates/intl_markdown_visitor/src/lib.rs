use intl_markdown::Document;

pub use crate::visit_with::VisitWith;
pub use crate::visitor::Visit;

mod visit_with;
mod visitor;

pub fn visit_with_mut<V: Visit>(document: &Document, visitor: &mut V) {
    document.visit_with(visitor);
}
