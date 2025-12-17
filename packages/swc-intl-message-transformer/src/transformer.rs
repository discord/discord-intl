use std::collections::HashSet;

use swc_core::common::DUMMY_SP;
use swc_core::ecma::ast::{
    ComputedPropName, Expr, Id, ImportDecl, ImportSpecifier, Lit, MemberExpr, MemberProp, Str,
};
use swc_core::ecma::visit::{VisitMut, VisitMutWith};

use intl_message_utils::{hash_message_key, is_message_definitions_file};

use crate::IntlMessageTransformerConfig;

#[derive(Default)]
pub struct IntlMessageConsumerTransformer {
    /// Set of identifiers that represent messages objects (i.e., are imported
    /// from a messages file). Id is used as the value type to ensure that the
    /// matched identifiers exactly resolve to the ones that are imported,
    /// without any other variables shadowing it from other scopes.
    messages_object_receivers: HashSet<Id>,
    /// Configuration for the transformer to adjust which object are
    /// transformed and other facets.
    config: IntlMessageTransformerConfig,
}

impl IntlMessageConsumerTransformer {
    pub fn new(config: IntlMessageTransformerConfig) -> Self {
        let set = HashSet::new();

        return Self {
            messages_object_receivers: set,
            config,
        };
    }
}

impl VisitMut for IntlMessageConsumerTransformer {
    fn visit_mut_import_decl(&mut self, import_decl: &mut ImportDecl) {
        let import_source_path = &import_decl.src.value.to_string_lossy();

        let is_definitions_file = is_message_definitions_file(&import_source_path);
        let extra_import_specifiers = self
            .config
            .get_configured_names_for_import_specifier(import_source_path);

        // If this isn't a definitions import _and_ there's no configured specifiers for
        // this import, then it doesn't need to be processed.
        if !is_definitions_file && extra_import_specifiers.is_none() {
            return;
        }

        for spec in import_decl.specifiers.iter() {
            match spec {
                ImportSpecifier::Default(default_specifier) if is_definitions_file => {
                    self.messages_object_receivers
                        .insert(default_specifier.local.to_id());
                }
                ImportSpecifier::Named(named_specifier) if extra_import_specifiers.is_some() => {
                    let local_name = &named_specifier.local;
                    if extra_import_specifiers
                        .is_some_and(|extra| extra.contains(&local_name.as_ref().into()))
                    {
                        self.messages_object_receivers.insert(local_name.to_id());
                    }
                }
                _ => continue,
            }
        }
    }

    fn visit_mut_member_expr(&mut self, member_expr: &mut MemberExpr) {
        member_expr.visit_mut_children_with(self);

        // Check if this expression is accessing a known messages object, like
        // `messages.SOME_STRING`. If not, then exit early since we don't need
        // to do anything with it.
        let messages_object = member_expr.obj.as_ident();
        if !messages_object
            .is_some_and(|ident| self.messages_object_receivers.contains(&ident.to_id()))
        {
            return;
        }

        // Replace the expression with a computed access to the hashed name.
        // Computed access ensures that the hash can use any character safely,
        // without worrying about being a valid JS identifier.
        // messages.SOME_STRING => messages["abc"].
        if let Some(message_name) = member_expr.prop.as_ident() {
            let hashed_name = hash_message_key(&message_name.sym);
            member_expr.prop = MemberProp::Computed(ComputedPropName {
                span: DUMMY_SP,
                expr: Box::new(Expr::Lit(Lit::Str(Str {
                    span: DUMMY_SP,
                    value: hashed_name.into(),
                    raw: None,
                }))),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use swc_core::ecma::{transforms::testing::test_inline_input_output, visit::visit_mut_pass};

    use crate::config::IntlMessageTransformerConfig;

    use super::IntlMessageConsumerTransformer;

    #[test]
    fn rewrite_message_keys_in_consumers() {
        test_inline_input_output(
            Default::default(),
            Some(true),
            |_| visit_mut_pass(IntlMessageConsumerTransformer::default()), // Input codes
            r#"
        import messages from "some/module.messages";
        import differentMess from "different.messages";
        import other from "another/place";
        console.log(messages.SOME_STRING);
        console.log(other.NOT_A_STRING);
        something.messages.WHAT;
        messages.whatever.anotherThing;
        differentMess.YES_STRINGS;
        "#,
            // Output codes after transformed with plugin
            r#"
        import messages from "some/module.messages";
        import differentMess from "different.messages";
        import other from "another/place";
        console.log(messages["Q5kgob"]);
        console.log(other.NOT_A_STRING);
        something.messages.WHAT;
        messages["nWsV48"].anotherThing;
        differentMess["PuzRxG"];
        "#,
        )
    }

    #[test]
    fn extra_specifier_config() {
        let config = serde_json::from_str::<IntlMessageTransformerConfig>(
            r#"{"extraImports":{"@app/intl":["t","untranslated"]}}"#,
        )
        .expect("failed to parse config");

        test_inline_input_output(
            Default::default(),
            Some(true),
            |_| visit_mut_pass(IntlMessageConsumerTransformer::new(config)),
            r#"
        import {untouchedSameSpec, t} from "@app/intl";
        import {untranslated} from "somewhere/else";
        import messages from "some.messages";
        console.log(t.SOME_STRING);
        console.log(messages.SOME_STRING);
        console.log(untranslated.SOME_STRING);
        console.log(untouchedSameSpec.SOME_STRING);
        "#,
            r#"
        import {untouchedSameSpec, t} from "@app/intl";
        import {untranslated} from "somewhere/else";
        import messages from "some.messages";
        console.log(t["Q5kgob"]);
        console.log(messages["Q5kgob"]);
        console.log(untranslated.SOME_STRING);
        console.log(untouchedSameSpec.SOME_STRING);
        "#,
        )
    }
}
