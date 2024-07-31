use swc_core::ecma::ast::Program;
use swc_core::ecma::visit::{as_folder, FoldWith};
use swc_core::plugin::{plugin_transform, proxies::TransformPluginProgramMetadata};

mod transformer;

#[plugin_transform]
pub fn process_transform(program: Program, _metadata: TransformPluginProgramMetadata) -> Program {
    program.fold_with(&mut as_folder(transformer::IntlMessageConsumerTransformer::new()))
}
