use swc_core::ecma::ast::Program;
use swc_core::ecma::visit::VisitMutWith;
use swc_core::plugin::{plugin_transform, proxies::TransformPluginProgramMetadata};

use config::IntlMessageTransformerConfig;

mod config;
mod transformer;

#[plugin_transform]
pub fn process_transform(
    mut program: Program,
    metadata: TransformPluginProgramMetadata,
) -> Program {
    let config = serde_json::from_str::<IntlMessageTransformerConfig>(
        &metadata
            .get_transform_plugin_config()
            .expect("failed to get swc-intl-message-transformer plugin config"),
    )
    .expect("failed to parse swc-intl-message-transformer config");

    program.visit_mut_with(&mut transformer::IntlMessageConsumerTransformer::new(
        config,
    ));

    program
}
