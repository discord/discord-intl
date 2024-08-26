use std::borrow::{Borrow, Cow};

use swc_common::{FileName, SourceMap, Spanned};
use swc_common::sync::Lrc;
use swc_core::ecma::ast::{
    ExportDecl, ExportDefaultExpr, Expr, Id, ImportDecl, ImportSpecifier, Lit, Module, ObjectLit,
};
use swc_core::ecma::parser::{lexer::Lexer, Parser, PResult, StringInput, Syntax};
use swc_core::ecma::visit::{noop_visit_type, Visit, VisitWith};
use unescape_zero_copy::unescape_default;

use intl_message_utils::RUNTIME_PACKAGE_NAME;

use crate::messages::{MessageKey, MessageMeta, MessagesError, MessagesResult};

pub fn parse_message_definitions_file(file_name: &str, source: &str) -> PResult<Module> {
    let cm: Lrc<SourceMap> = Default::default();

    let fm = cm.new_source_file(FileName::Custom(file_name.into()), source.into());
    let lexer = Lexer::new(
        Syntax::Es(Default::default()),
        Default::default(),
        StringInput::from(&*fm),
        None,
    );

    let mut parser = Parser::new_from(lexer);
    parser.parse_module()
}

pub fn extract_message_definitions(module: Module) -> MessageDefinitionsExtractor {
    let mut extractor = MessageDefinitionsExtractor::new();
    module.visit_with(&mut extractor);
    extractor
}

/// A temporary representation of a message directly from a definition file.
/// This includes all of the resolved meta and other information of a
/// [NormalMessage], but does _not_ do any parsing or processing of the data
/// beyond that, since that would create intertwined lifetime dependencies.
pub struct ExtractedMessage {
    pub name: MessageKey,
    pub offset: u32,
    pub value: String,
    pub meta: MessageMeta,
}

/// A Visitor to extract message definitions from a source AST.
pub struct MessageDefinitionsExtractor {
    pub message_definitions: Vec<ExtractedMessage>,
    pub failed_definitions: Vec<MessagesError>,
    pub root_meta: Option<MessageMeta>,
    define_messages_id: Option<Id>,
}

impl MessageDefinitionsExtractor {
    fn new() -> Self {
        MessageDefinitionsExtractor {
            define_messages_id: None,
            message_definitions: vec![],
            failed_definitions: vec![],
            root_meta: None,
        }
    }

    /// Parses the given `object` into a set of MessageDefinitions, storing
    /// that result in `self.message_definitions`.
    fn parse_definitions_object(&mut self, object: &ObjectLit) {
        for property in object.props.iter() {
            let Some(keyvalue) = property.as_prop().and_then(|prop| prop.as_key_value()) else {
                continue;
            };
            let name = if let Some(name) = keyvalue.key.as_ident() {
                &name.sym
            } else if let Some(name) = keyvalue.key.as_str() {
                &name.value
            } else {
                continue;
            };

            let parse_result = if let Some(object) = keyvalue.value.as_object() {
                self.parse_complete_definition(&name, &object)
            } else if let Some(lit @ Lit::Str(string)) = keyvalue.value.as_lit() {
                self.parse_oneline_definition(&name, &string.value, lit.span_lo().0)
            } else if let Some(template) = keyvalue.value.as_tpl() {
                // With JS, you can write static strings as template strings to
                // avoid needing to escape different quotes, like:
                //     SOME_STRING: `"this" is valid, isn't it?`
                // We want to support that syntax, but we can't allow templates
                // that have embedded expressions or multiple elements.
                let string_value = template.quasis.get(0).map(|expr| &expr.raw);
                let is_static = template.quasis.len() == 1 && template.exprs.len() == 0;

                match string_value {
                    Some(string) if is_static => self.parse_oneline_definition(&name, &string, template.span_lo().0),
                    _ => Err(MessagesError::DefinitionRestrictionViolated("Encountered non-static template string. Interpolations are currently invalid".into()))
                }
            } else {
                Err(MessagesError::DefinitionRestrictionViolated(
                    "Encountered an unknown message definition structure".into(),
                ))
            };

            match parse_result {
                Ok(definition) => self.message_definitions.push(definition),
                Err(error) => self.failed_definitions.push(error),
            }
        }
    }

    /// Parse a single message definition into a structured object, resolving
    /// all meta information needed for it.
    fn parse_complete_definition(
        &self,
        key: &str,
        object: &ObjectLit,
    ) -> MessagesResult<ExtractedMessage> {
        let mut default_value: Option<String> = None;
        let mut local_meta = self.clone_meta();

        for property in object.props.iter() {
            let Some(keyvalue) = property.as_prop().and_then(|prop| prop.as_key_value()) else {
                continue;
            };
            let Some(name) = keyvalue.key.as_ident() else {
                continue;
            };

            match name.sym.as_str() {
                "message" => {
                    self.parse_string_value(keyvalue.value.borrow())
                        .map(|value| default_value = Some(value));
                }
                name => self.parse_meta_property(name, keyvalue.value.borrow(), &mut local_meta),
            }
        }

        // If no `message` was provided in the object definition, it's invalid
        // and can't be used.
        let Some(default_value) = default_value else {
            return Err(MessagesError::NoMessageValue(key.into()));
        };

        Ok(ExtractedMessage {
            name: key.into(),
            value: default_value,
            offset: object.span.lo.0,
            meta: local_meta,
        })
    }

    /// Parse a message definition using the shorthand `name: "value"`
    fn parse_oneline_definition(
        &self,
        key: &str,
        value: &str,
        offset: u32,
    ) -> MessagesResult<ExtractedMessage> {
        Ok(ExtractedMessage {
            name: key.into(),
            value: self.apply_string_escapes(value).to_string(),
            offset,
            meta: self.clone_meta(),
        })
    }

    /// Return a clone of the root meta, or a new object with the default
    /// values if none existed.
    fn clone_meta(&self) -> MessageMeta {
        self.root_meta
            .as_ref()
            .map_or_else(Default::default, Clone::clone)
    }

    // Parses the given `object` as a meta definition, then stores the result
    // in `self.root_meta`.
    fn parse_root_meta_initializer(&mut self, object: &ObjectLit) {
        let mut result = MessageMeta::default();

        for property in object.props.iter() {
            let Some(keyvalue) = property.as_prop().and_then(|prop| prop.as_key_value()) else {
                continue;
            };
            let Some(name) = keyvalue.key.as_ident() else {
                continue;
            };

            self.parse_meta_property(&name.sym, keyvalue.value.borrow(), &mut result);
        }

        self.root_meta = Some(result);
    }

    /// Interpret a given name/value pair to see if it represents a MessageMeta
    /// property. If it does, apply the value to the corresponding field in
    /// `target`. Otherwise, nothing is done.
    fn parse_meta_property(&self, name: &str, value: &Expr, target: &mut MessageMeta) {
        // NOTE: This effectively relies on TypeScript's checker to provide
        // hints when the value types would be incorrect (e.g., `secret` is
        // given a number value instead of a boolean).
        match name {
            "secret" => self
                .parse_boolean_value(value)
                .map(|value| target.secret = value),
            "bundleSecrets" => self
                .parse_boolean_value(value)
                .map(|value| target.bundle_secrets = value),
            "translate" => self
                .parse_boolean_value(value)
                .map(|value| target.translate = value),
            "translationsPath" => self
                .parse_string_value(value)
                .map(|value| target.translations_path = value),
            _ => None,
        };
    }

    /// If the given expression is a boolean literal, it is interpreted into an
    /// actual boolean value. Any other expression will return None.
    fn parse_boolean_value(&self, expr: &Expr) -> Option<bool> {
        match expr.as_lit() {
            Some(Lit::Bool(bool)) => Some(bool.value),
            _ => None,
        }
    }

    /// If the given expression is a string literal, the value of that literal
    /// is returned. Any other expression will return None.
    fn parse_string_value(&self, expr: &Expr) -> Option<String> {
        match expr.as_lit() {
            Some(Lit::Str(string)) => Some(self.apply_string_escapes(&string.value).to_string()),
            _ => None,
        }
    }

    /// Apply literal escape sequences like `\n` from the string value.
    fn apply_string_escapes<'a>(&self, value: &'a str) -> Cow<'a, str> {
        unescape_default(value).unwrap_or(Cow::from(value))
    }
}

impl Visit for MessageDefinitionsExtractor {
    noop_visit_type!();

    // Captures `meta` declarations.
    fn visit_export_decl(&mut self, export: &ExportDecl) {
        let Some(decl) = export.decl.as_var() else {
            return;
        };

        for decl in decl.decls.iter() {
            let is_meta_declaration = decl
                .name
                .as_ident()
                .is_some_and(|id| id.id.sym.as_str() == "meta");
            if !is_meta_declaration {
                continue;
            }

            if let Some(initializer) = decl.init.as_ref().and_then(|init| init.as_object()) {
                self.parse_root_meta_initializer(initializer);
            } else {
                // We've found the meta and determined it didn't have an
                // initializer, so we don't need to continue iterating.
                // TODO: Use this error.
                drop(MessagesError::NoMetaInitializer);
                break;
            }
        }
    }

    // Captures `defineMessages` calls as the default export.
    fn visit_export_default_expr(&mut self, default_export: &ExportDefaultExpr) {
        let Some(call_expr) = default_export.expr.as_call() else {
            return;
        };

        // This is almost definitely set before reaching here. If not, it's an
        // error anyway.
        let Some(define_messages_id) = &self.define_messages_id else {
            return;
        };

        // Ensure this is the default-exported `defineMessages()` call that we
        // want to parse.
        if !call_expr
            .callee
            .as_expr()
            .and_then(|callee| callee.as_ident())
            .is_some_and(|ident| ident.to_id() == *define_messages_id)
        {
            return;
        }

        // If it has an object expression as the first argument
        if let Some(definition_object) = call_expr.args.get(0).and_then(|arg| arg.expr.as_object())
        {
            self.parse_definitions_object(definition_object);
        }
    }

    fn visit_import_decl(&mut self, import_decl: &ImportDecl) {
        let import_source_path = &import_decl.src.value;
        if import_source_path != RUNTIME_PACKAGE_NAME {
            return;
        }

        for spec in import_decl.specifiers.iter() {
            match spec {
                ImportSpecifier::Named(specifier) => {
                    self.define_messages_id = Some(specifier.local.to_id());
                }
                _ => continue,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::messages::global_intern_string;

    use super::parse_message_definitions_file;

    #[test]
    fn test_parsing() {
        let module = parse_message_definitions_file("testing.js", "const t = hello".into());
        println!("{:#?}", module);
    }

    #[test]
    fn test_template_string() {
        let module = parse_message_definitions_file(
            "testing.js",
            &format!(
                r#"
        import {{defineMessages}} from '{RUNTIME_PACKAGE_NAME}';

        export default defineMessages({{
            TEMPLATED: `this is a template`,
            INVALID: `this is an ${{invalidTemplate}}`,
            'string-key': 'this has a stringy key',
        }});
        "#
            ),
        )
        .expect("failed to parse source code");

        let file_symbol = global_intern_string("testing.js").expect("couldn't intern symbol");
    }
}
