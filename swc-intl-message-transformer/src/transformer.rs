use std::collections::HashSet;
use swc_core::common::DUMMY_SP;
use swc_core::ecma::ast::{
    ComputedPropName, Expr, Id, ImportDecl, ImportSpecifier, Lit, MemberExpr, MemberProp, Str,
};
use swc_core::ecma::visit::{VisitMut, VisitMutWith};
use intl_message_utils::{hash_message_key, is_message_definitions_file};

pub struct IntlMessageConsumerTransformer {
    /// Set of identifiers that represent messages objects (i.e., are imported
    /// from a messages file). Id is used as the value type to ensure that the
    /// matched identifiers exactly resolve to the ones that are imported,
    /// without any other variables shadowing it from other scopes.
    messages_object_receivers: HashSet<Id>,
}

impl IntlMessageConsumerTransformer {
    pub fn new() -> Self {
        let set = HashSet::new();

        return Self {
            messages_object_receivers: set,
        };
    }
}

impl VisitMut for IntlMessageConsumerTransformer {
    fn visit_mut_import_decl(&mut self, import_decl: &mut ImportDecl) {
        let import_source_path = &import_decl.src.value;
        if !is_message_definitions_file(&import_source_path) {
            return;
        }

        for spec in import_decl.specifiers.iter() {
            match spec {
                ImportSpecifier::Default(default_specifier) => {
                    self.messages_object_receivers
                        .insert(default_specifier.local.to_id());
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
    use swc_core::ecma::transforms::testing::test_inline_input_output;
    use swc_core::ecma::visit::as_folder;

    use super::IntlMessageConsumerTransformer;

    #[test]
    fn rewrite_message_keys_in_consumers() {
        test_inline_input_output(
            Default::default(),
            |_| as_folder(IntlMessageConsumerTransformer::new()), // Input codes
            r#"
        import {messages} from "some/module.messages";
        import {messages as differentMess} from "different.messages";
        import {other} from "another/place";
        console.log(messages.SOME_STRING);
        console.log(other.NOT_A_STRING);
        something.messages.WHAT;
        messages.whatever.anotherThing;
        differentMess.YES_STRINGS;
        "#,
            // Output codes after transformed with plugin
            r#"
        import {messages} from "some/module.messages";
        import {messages as differentMess} from "different.messages";
        import {other} from "another/place";
        console.log(messages["LvzMmy"]);
        console.log(other.NOT_A_STRING);
        something.messages.WHAT;
        messages["rj00eY"].anotherThing;
        differentMess["7hnnbN"];
        "#,
        )
    }
}
