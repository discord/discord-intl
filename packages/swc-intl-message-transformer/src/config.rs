use std::collections::HashMap;

use serde::Deserialize;

#[derive(Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct IntlMessageTransformerConfig {
    pub extra_imports: Option<HashMap<String, Vec<String>>>,
}

impl IntlMessageTransformerConfig {
    pub fn get_configured_names_for_import_specifier(
        &self,
        specifier: &str,
    ) -> Option<&Vec<String>> {
        match &self.extra_imports {
            Some(extras) => extras.get(specifier),
            None => None,
        }
    }
}
