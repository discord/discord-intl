mod compiler;
mod element;
mod util;

use crate::{AnyDocument, VisitWith};
pub use compiler::Compiler;
pub use element::*;

pub fn compile_document(document: &AnyDocument) -> CompiledElement {
    let mut compiler = Compiler::new();
    document.visit_with(&mut compiler);
    compiler.finish()
}
