mod compiler;
mod element;
mod element_ext;
mod util;
mod visitor;

use crate::{AnyDocument, VisitWith};
pub use compiler::Compiler;
pub use element::*;
pub use visitor::{FoldCompiled, VisitCompiled, VisitCompiledWith};

pub fn compile_document(document: &AnyDocument) -> CompiledElement {
    let mut compiler = Compiler::new();
    document.visit_with(&mut compiler);
    compiler.finish()
}
