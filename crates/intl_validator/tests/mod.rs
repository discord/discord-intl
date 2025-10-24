#[cfg(test)]
mod harness {
    use intl_database_core::{
        key_symbol, KeySymbolMap, MessageDefinitionSource, MessageTranslationSource, MessageValue,
    };
    use intl_database_js_source::JsMessageSource;
    use intl_database_json_source::JsonMessageSource;
    use intl_validator::validators::validator::Validator;
    use intl_validator::{DiagnosticName, TextRange, ValueDiagnostic};

    pub fn json_source_file(file_name: &str, content: &str) -> KeySymbolMap<MessageValue> {
        let Ok(raw_messages) =
            JsonMessageSource.extract_translations(key_symbol(file_name), content)
        else {
            panic!("Unparseable JSON messages content");
        };

        let mut messages = KeySymbolMap::default();
        for message in raw_messages {
            messages.insert(message.name, message.value);
        }
        messages
    }
    pub fn js_source_file(file_name: &str, content: &str) -> KeySymbolMap<MessageValue> {
        let Ok((meta, raw_messages)) =
            JsMessageSource.extract_definitions(key_symbol(file_name), content)
        else {
            panic!("Unparseable JSON messages content");
        };

        let mut messages = KeySymbolMap::default();
        for message in raw_messages {
            messages.insert(message.name, message.value);
        }
        messages
    }

    pub fn define_single_message(key: &str, content: &str) -> MessageValue {
        let mut messages = json_source_file(
            "single_message.json",
            &format!(
                r#"{{
                    "{key}": "{content}"
                }}"#
            ),
        );
        let key = key_symbol(key);
        messages.remove(&key).unwrap()
    }

    pub fn has_matching_diagnostic(
        diagnostics: &Vec<ValueDiagnostic>,
        name: DiagnosticName,
        span: TextRange,
    ) -> bool {
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.name == name && diagnostic.span.is_some_and(|s| s == span))
    }

    pub fn validate_with(
        message: &MessageValue,
        mut validator: impl Validator,
    ) -> Option<Vec<ValueDiagnostic>> {
        validator.validate_cst(&message)
    }
}

mod offsets;
mod rules;
