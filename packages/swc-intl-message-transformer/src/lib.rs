use swc_core::ecma::ast::Program;
use swc_core::ecma::visit::visit_mut_pass;
use swc_core::plugin::{plugin_transform, proxies::TransformPluginProgramMetadata};

use config::IntlMessageTransformerConfig;

mod config;
mod transformer;

#[plugin_transform]
pub fn process_transform(program: Program, metadata: TransformPluginProgramMetadata) -> Program {
    let config = serde_json::from_str::<IntlMessageTransformerConfig>(
        &metadata
            .get_transform_plugin_config()
            .expect("failed to get swc-intl-message-transformer plugin config"),
    )
    .expect("failed to parse swc-intl-message-transformer config");

    program.apply(&mut visit_mut_pass(
        transformer::IntlMessageConsumerTransformer::new(config),
    ))
}
