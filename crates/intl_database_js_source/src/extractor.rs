use std::borrow::Borrow;
use swc_common::source_map::SmallPos;
use swc_common::sync::Lrc;
use swc_common::{BytePos, FileName, Loc, SourceMap, Spanned};
use swc_core::atoms::Atom;
use swc_core::ecma::ast::{
    ExportDecl, ExportDefaultExpr, Expr, Id, ImportDecl, ImportSpecifier, Lit, Module, ObjectLit,
    PropName,
};
use swc_core::ecma::parser::{lexer::Lexer, PResult, Parser, StringInput, Syntax};
use swc_core::ecma::visit::{noop_visit_type, Visit, VisitWith};

use intl_database_core::{
    key_symbol, FilePosition, KeySymbol, MessageMeta, MessageSourceError, MessageSourceResult,
    RawMessageDefinition, SourceFileMeta, SourceOffsetList,
};
use intl_message_utils::RUNTIME_PACKAGE_NAME;

pub fn parse_message_definitions_file(
    file_name: &str,
    source: &str,
) -> PResult<(Lrc<SourceMap>, Module)> {
    let cm: Lrc<SourceMap> = Default::default();

    let fm = cm.new_source_file(Lrc::new(FileName::Custom(file_name.into())), source.into());
    let lexer = Lexer::new(
        Syntax::Es(Default::default()),
        Default::default(),
        StringInput::from(&*fm),
        None,
    );

    let mut parser = Parser::new_from(lexer);
    let module = parser.parse_module()?;
    Ok((cm, module))
}

pub fn extract_message_definitions(
    source_file_path: &str,
    source_file: Lrc<SourceMap>,
    module: Module,
) -> MessageDefinitionsExtractor {
    let mut extractor = MessageDefinitionsExtractor::new(source_file_path, source_file);
    module.visit_with(&mut extractor);
    extractor
}

struct MessageStringValue<'a> {
    processed: &'a Atom,
    raw: &'a Atom,
    position: FilePosition,
}

impl MessageStringValue<'_> {
    /// SAFETY: This method assumes the input is a valid string in JavaScript's grammar.
    fn make_source_offsets(&self) -> SourceOffsetList {
        if self.raw == self.processed {
            return SourceOffsetList::default();
        }
        let mut list: Vec<(u32, u32)> = vec![];

        let bytes = self.raw.as_bytes();
        let mut total_offset = 0;
        let mut last_checked_byte = 0;
        for idx in memchr::memchr_iter(b'\\', bytes) {
            let Some(byte) = bytes.get(idx + 1) else {
                continue;
            };
            if idx <= last_checked_byte {
                continue;
            }

            // https://developer.mozilla.org/en-US/docs/Web/JavaScript/Guide/Grammar_and_types#using_special_characters_in_strings
            let added_offset = match byte {
                b'0' | b'b' | b'f' | b'n' | b'r' | b't' | b'v' | b'\'' | b'"' | b'\\' => 1,
                b'u' => {
                    if matches!(
                        bytes.get(idx + 2),
                        Some(b'0'..b'9' | b'A'..b'F' | b'a'..b'f')
                    ) {
                        // \uHHHH, 4-digit hexadecimal encoding
                        5
                    } else {
                        // \u{XXXXXX} codepoint escapes
                        let count = bytes[idx + 3..=idx + 9]
                            .iter()
                            .position(|b| *b == b'}')
                            .unwrap_or(0);
                        // `count` is the number of hex digits, plus 3 for u{}
                        count + 3
                    }
                }
                // \xA9 hexadecimal escapes
                b'x' => 3,
                // \OOO octal escapes
                b'0'..b'7' => {
                    let mut octal_count = 1;
                    for byte in &bytes[idx + 2..idx + 3] {
                        if !matches!(byte, b'0'..b'7') {
                            break;
                        }
                        octal_count += 1;
                    }
                    octal_count
                }
                _ => 0,
            };

            last_checked_byte = idx + added_offset;
            if added_offset > 0 {
                total_offset += added_offset;
                list.push((idx as u32, total_offset as u32));
            }
        }
        SourceOffsetList::new(list)
    }
}

/// A Visitor to extract message definitions from a source AST.
pub struct MessageDefinitionsExtractor {
    file_key: KeySymbol,
    pub message_definitions: Vec<RawMessageDefinition>,
    pub failed_definitions: Vec<MessageSourceError>,
    pub root_meta: SourceFileMeta,
    define_messages_id: Option<Id>,
    source_map: Lrc<SourceMap>,
}

impl MessageDefinitionsExtractor {
    fn new(source_file_path: &str, source_map: Lrc<SourceMap>) -> Self {
        MessageDefinitionsExtractor {
            file_key: key_symbol(source_file_path),
            define_messages_id: None,
            message_definitions: vec![],
            failed_definitions: vec![],
            root_meta: SourceFileMeta::new(source_file_path),
            source_map,
        }
    }

    fn raw_definition_from_value(
        &self,
        key: &str,
        value: MessageStringValue,
        meta: MessageMeta,
    ) -> RawMessageDefinition {
        RawMessageDefinition::new(
            key.into(),
            value.position,
            value.processed,
            value.raw,
            meta,
            value.make_source_offsets(),
        )
    }

    /// Parses the given `object` into a set of MessageDefinitions, storing
    /// that result in `self.message_definitions`.
    fn parse_definitions_object(&mut self, object: &ObjectLit) {
        for property in object.props.iter() {
            let Some(kv) = property.as_prop().and_then(|prop| prop.as_key_value()) else {
                continue;
            };
            let name = match &kv.key {
                PropName::Ident(name) => &name.sym,
                PropName::Str(name) => &name.value,
                _ => continue,
            };

            let parse_result = match &*kv.value {
                Expr::Object(object) => self.parse_complete_definition(&name, &object),
                expr => self.parse_message_value(expr, expr.span_lo()).map(|value| {
                    self.raw_definition_from_value(name.as_str(), value, self.clone_meta())
                }),
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
    ) -> MessageSourceResult<RawMessageDefinition> {
        let mut default_value: Option<MessageStringValue> = None;
        let mut local_meta = self.clone_meta();

        for property in object.props.iter() {
            let Some(kv) = property.as_prop().and_then(|prop| prop.as_key_value()) else {
                continue;
            };
            let Some(name) = kv.key.as_ident() else {
                continue;
            };

            match name.sym.as_str() {
                "message" => {
                    default_value =
                        Some(self.parse_message_value(kv.value.borrow(), kv.value.span_lo())?)
                }
                name => self.parse_message_meta_property(name, kv.value.borrow(), &mut local_meta),
            }
        }

        // If no `message` was provided in the object definition, it's invalid
        // and can't be used.
        let Some(default_value) = default_value else {
            return Err(MessageSourceError::NoMessageValue(key.into()));
        };

        Ok(self.raw_definition_from_value(key.into(), default_value, local_meta))
    }

    fn parse_message_value<'a>(
        &self,
        value: &'a Expr,
        pos: BytePos,
    ) -> MessageSourceResult<MessageStringValue<'a>> {
        let loc = self.source_map.lookup_char_pos(pos);
        match value {
            Expr::Lit(Lit::Str(string)) => Ok(MessageStringValue {
                processed: &string.value,
                // SAFETY: `raw` is always set by the parse and the Option is used for internal
                // semantics.
                raw: string.raw.as_ref().unwrap(),
                position: self.adjust_position_from_quote(loc, 1),
            }),
            Expr::Tpl(template) => {
                // With JS, you can write static strings as template strings to
                // avoid needing to escape different quotes, like:
                //     SOME_STRING: `"this" is valid, isn't it?`
                // We want to support that syntax, but we can't allow templates
                // that have embedded expressions or multiple elements.
                if template.quasis.len() != 1 || template.exprs.len() != 0 {
                    Err(MessageSourceError::DefinitionRestrictionViolated("Encountered non-static template string. Interpolations are currently invalid".into()))
                } else {
                    let expr = &template.quasis[0];
                    Ok(MessageStringValue {
                        // SAFETY: This should always be set for a single string literal
                        processed: expr.cooked.as_ref().unwrap(),
                        raw: &expr.raw,
                        position: self.adjust_position_from_quote(loc, 1),
                    })
                }
            }
            _ => Err(MessageSourceError::DefinitionRestrictionViolated(format!(
                "Encountered an unknown message value expression at {loc:?}"
            ))),
        }
    }

    /// Return a clone of the root meta, or a new object with the default
    /// values if none existed.
    fn clone_meta(&self) -> MessageMeta {
        MessageMeta::from(&self.root_meta)
    }

    // Parses the given `object` as a meta definition, then stores the result
    // in `self.root_meta`.
    fn parse_root_meta_initializer(&mut self, object: &ObjectLit) {
        for property in object.props.iter() {
            let Some(kv) = property.as_prop().and_then(|prop| prop.as_key_value()) else {
                continue;
            };
            let Some(name) = kv.key.as_ident() else {
                continue;
            };

            self.parse_source_file_meta_property(&name.sym, kv.value.borrow());
        }
    }

    /// Interpret a given name/value pair to see if it represents a SourceFileMeta
    /// property. If it does, apply the value to the corresponding field in
    /// `target`. Otherwise, nothing is done.
    fn parse_source_file_meta_property(&mut self, name: &str, value: &Expr) {
        // NOTE: This effectively relies on TypeScript's checker to provide
        // hints when the value types would be incorrect (e.g., `secret` is
        // given a number value instead of a boolean).
        match name {
            "secret" => self
                .parse_boolean_value(value)
                .map(|value| self.root_meta.secret = value),
            "translate" => self
                .parse_boolean_value(value)
                .map(|value| self.root_meta.translate = value),
            "translationsPath" => self
                .parse_string_value(value)
                .map(|value| self.root_meta.translations_path = value.into()),
            "description" => self
                .parse_string_value(value)
                .map(|value| self.root_meta.description = Some(value)),
            _ => None,
        };
    }

    /// Interpret a given name/value pair to see if it represents a MessageMeta
    /// property. If it does, apply the value to the corresponding field in
    /// `target`. Otherwise, nothing is done.
    fn parse_message_meta_property(&self, name: &str, value: &Expr, target: &mut MessageMeta) {
        // NOTE: This effectively relies on TypeScript's checker to provide
        // hints when the value types would be incorrect (e.g., `secret` is
        // given a number value instead of a boolean).
        match name {
            "secret" => self
                .parse_boolean_value(value)
                .map(|value| target.secret = value),
            "translate" => self
                .parse_boolean_value(value)
                .map(|value| target.translate = value),
            "description" => self
                .parse_string_value(value)
                .map(|value| target.description = Some(value)),
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
            Some(Lit::Str(string)) => Some(string.value.to_string()),
            _ => None,
        }
    }

    fn adjust_position_from_quote(&self, loc: Loc, offset: u32) -> FilePosition {
        FilePosition {
            file: self.file_key,
            line: loc.line as u32,
            col: loc.col.to_u32() + offset,
        }
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
                self.failed_definitions
                    .push(MessageSourceError::InvalidSourceFileMeta);
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
    use intl_database_core::MessageSourceError;

    use super::{
        extract_message_definitions, parse_message_definitions_file, MessageDefinitionsExtractor,
    };

    fn extract_test_messages(source: &str) -> MessageDefinitionsExtractor {
        let (map, module) = parse_message_definitions_file(
            "testing.js",
            &format!(
                r#"
        import {{defineMessages}} from '{}';

        export default defineMessages({{{source}}})
        "#,
                intl_message_utils::RUNTIME_PACKAGE_NAME
            ),
        )
        .expect("successful parse of message definitions");
        extract_message_definitions("testing.js", map, module)
    }

    #[test]
    fn test_parsing() {
        parse_message_definitions_file("testing.js", "const t = hello".into())
            .expect("successful parse");
    }

    #[test]
    fn test_slash_escapes() {
        let extractor = extract_test_messages(
            r#"
            HALF: '\_ foo',
            ONE_ESCAPE: '\\_ foo',
            ONE_HALF: '\\\_ bar',
            TWO_ESCAPE: '\\\\_ two',
            TWO_HALF: '\\\\\_ half',
            UNICODE: '\ƒ\xff'
        "#,
        );

        for message in extractor.message_definitions {
            let expected = match message.name.as_str() {
                "HALF" => "_ foo",
                "ONE_ESCAPE" => "\\_ foo",
                "ONE_HALF" => "\\_ bar",
                "TWO_ESCAPE" => "\\\\_ two",
                "TWO_HALF" => "\\\\_ half",
                "UNICODE" => "ƒÿ",
                _ => unreachable!(),
            };

            assert_eq!(message.value.raw, expected);
        }
    }

    #[test]
    fn test_escaped_newlines() {
        let extractor = extract_test_messages(
            r#"
                SINGLE_NEWLINE: 'this is a\nmulti-line string',
                DOUBLE_NEWLINE: 'this is a\n\nmulti-paragraph string'
            "#,
        );

        for message in extractor.message_definitions {
            let expected = match message.name.as_str() {
                "SINGLE_NEWLINE" => "this is a\nmulti-line string",
                "DOUBLE_NEWLINE" => "this is a\n\nmulti-paragraph string",
                _ => unreachable!(),
            };

            assert_eq!(message.value.raw, expected);
            println!("{}", message.value.raw);
        }
    }

    #[test]
    fn test_template_literal_escapes() {
        // SWC Issue: https://github.com/swc-project/swc/issues/637
        // Depsite being "fixed" the behavior doesn't seem to actually be
        // correct (`\n` still creates '\\n' when parsed).
        let extractor = extract_test_messages(
            r#"
                REGULAR: 'this is a\nmulti-line string',
                TEMPLATE: `this is a\nmulti-line string`
            "#,
        );

        for message in extractor.message_definitions {
            let expected = match message.name.as_str() {
                "REGULAR" => "this is a\nmulti-line string",
                "TEMPLATE" => "this is a\nmulti-line string",
                _ => unreachable!(),
            };

            assert_eq!(message.value.raw, expected);
            println!("{}", message.value.raw);
        }
    }

    #[test]
    fn test_parse_template_string() {
        parse_message_definitions_file(
            "testing.js",
            &format!(
                r#"
        import {{defineMessages}} from '{}';

        export default defineMessages({{
            TEMPLATED: `this is a template`,
            INVALID: `this is an ${{invalidTemplate}}`,
            'string-key': 'this has a stringy key',
        }});
        "#,
                intl_message_utils::RUNTIME_PACKAGE_NAME
            ),
        )
        .expect("failed to parse source code");
    }

    #[test]
    fn test_extract_meta() {
        let file_name = "testing.js";
        let (source, module) = parse_message_definitions_file(
            file_name,
            r#"
                export const meta = {
                    translate: false,
                    description: "Hello world",
                };
                "#,
        )
        .expect("failed to parse source code");

        let extractor = extract_message_definitions(file_name, source, module);

        assert!(extractor.failed_definitions.is_empty());
        assert_eq!(false, extractor.root_meta.translate);
        assert_eq!(Some("Hello world".into()), extractor.root_meta.description);
    }

    #[test]
    fn test_extract_meta_without_initializer() {
        let file_name = "testing.js";
        let (source, module) = parse_message_definitions_file(
            file_name,
            r#"
                export const meta;
                "#,
        )
        .expect("failed to parse source code");

        let extractor = extract_message_definitions(file_name, source, module);

        assert_eq!(
            vec![MessageSourceError::InvalidSourceFileMeta,],
            extractor.failed_definitions
        );
    }
}
