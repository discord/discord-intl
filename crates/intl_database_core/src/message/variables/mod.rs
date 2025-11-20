use std::ops::Deref;

use crate::database::symbol::{KeySymbol, KeySymbolMap};
use crate::message::variables::visitor::MessageVariablesVisitor;
use crate::{key_symbol, KeySymbolSet};
use intl_markdown::{AnyDocument, VisitWith};
use intl_markdown_syntax::TextSpan;
use rustc_hash::FxHashSet;
use serde::Serialize;

mod visitor;

#[derive(Clone, Debug, Serialize, Hash, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum MessageVariableType {
    /// Any value is accepted for this variable. Generally used when the
    /// required type of the variable can't be determined.
    Any,
    /// Any type of numeric value is valid. Accepts both integers and floats.
    Number,
    /// A value used for a Plural evaluation. Generally a number, or something
    /// that can be directly cast to a number.
    Plural,
    /// A value that must match one of the defined values in this vec. Enums
    /// that support fallbacks are determined by the runtime, but most use the
    /// option `"other"` to represent that.
    Enum {
        values: Vec<String>,
        allow_other: bool,
    },
    /// Like a regular Enum, but the values must be numbers, similar to Plurals
    /// except only a fixed set of options are allowed. If `allow_other` is
    /// true, then any number is allowed as a value, but anything not given in
    /// `options` will always use the `other` clause.
    NumericEnum {
        values: Vec<usize>,
        allow_other: bool,
    },
    /// A Date type must be supplied. The runtime can decide whether the type
    /// can be parsed from a String or must be a Date object.
    Date,
    /// A Time type must be supplied. The runtime can decide whether the type
    /// can be parsed from a String or must be a specific Time object.
    Time,
    /// A function that provides some structured replacement of content,
    /// normally used for applying styles or injecting custom objects into the
    /// result string.
    HookFunction,
    /// A specialization of [MessageVariableType::HookFunction] that represents
    /// a Link, which requires specific handling in most cases.
    LinkFunction,
    /// A function that handles some action. Not used for any rendered content,
    /// the return value of this function is ignored.
    HandlerFunction,
}

impl MessageVariableType {
    /// Returns true if the variable type is a reference that directly renders
    /// the content of the variable.
    pub fn is_visible_reference(&self) -> bool {
        !matches!(
            self,
            Self::Enum { .. } | Self::NumericEnum { .. } | Self::Plural
        )
    }
}

/// A representation of a single _instance_ of a variable in a message. Each
/// time a variable appears in a message, even if it is a variable that has
/// already been seen, a new MessageVariable is created.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MessageVariableInstance {
    /// The location in the message where this variable is used. Each instance
    /// of a variable in a string has its own struct, so each stores its own
    /// span as well.
    pub span: Option<TextSpan>,
    /// `true` if this variable is a system-defined variable, typically for
    /// rich text formatting tags like `$b` and `$link`, which are almost never
    /// intended for a user to provide and/or only represent formatting points,
    /// but can be given as an override regardless.
    pub is_builtin: bool,
    /// The specific kind of the variable, used for generating types.
    pub kind: MessageVariableType,
}

impl MessageVariableInstance {
    /// Returns true if the content of the variable is rendered as a result of
    /// this instance. For example, `{count}` is visible because the content of
    /// `count` is rendered into the string, but `{count, plural, other {foo}}`
    /// is _not_ visible, because the value of count only controls the plural
    /// and is not actually rendered in any of the arms.
    ///
    /// Builtin variables are _not_ included as visible, because they are
    /// always stylistic and may not have the same colloquial meaning across
    /// languages. Note that link and hook functions _do_ contain builtins, but
    /// _also_ contain other variable references when the content is affected
    /// by the variable values.
    pub fn is_visible(&self) -> bool {
        self.kind.is_visible_reference() && !self.is_builtin
    }
}

#[derive(Clone, Debug, Serialize)]
#[serde(transparent)]
pub struct MessageVariables {
    variables: KeySymbolMap<Vec<MessageVariableInstance>>,
}

impl MessageVariables {
    pub fn new() -> Self {
        Self {
            variables: KeySymbolMap::default(),
        }
    }

    /// Add a new instance of a variable to the set of variables in a message.
    /// If this is the first instance of that variable, a new entry will be
    /// allocated for it, otherwise it will be appended to the list of
    /// instances for that name.
    pub fn add_instance(
        &mut self,
        name: &str,
        kind: MessageVariableType,
        is_builtin: bool,
        span: Option<TextSpan>,
    ) {
        let instance = MessageVariableInstance {
            kind,
            is_builtin,
            span,
        };
        self.variables
            .entry(key_symbol(name))
            .or_insert_with(|| vec![])
            .push(instance);
    }

    /// Merge the variables from `other` into self by copying them over.
    pub fn merge(&mut self, other: &Self) {
        for (symbol, instances) in other.iter() {
            self.variables
                .entry(*symbol)
                .and_modify(|existing| existing.extend(instances.clone()))
                .or_insert(instances.clone());
        }
    }

    /// Returns a HashSet of the names of all variables in this message.
    pub fn keys(&self) -> FxHashSet<&KeySymbol> {
        self.variables.keys().collect::<FxHashSet<&KeySymbol>>()
    }

    /// Returns true if this variable set contains the given variable name
    pub fn has_key(&self, name: &KeySymbol) -> bool {
        self.variables.contains_key(name)
    }

    /// Returns the count of _uniquely-named_ variables found in the message
    pub fn count(&self) -> usize {
        self.variables.len()
    }

    pub fn get(&self, key: &KeySymbol) -> Option<&Vec<MessageVariableInstance>> {
        self.variables.get(key)
    }

    /// Return only the variables of this set that are "visible" when the
    /// message is rendered.
    pub fn visible_variable_names(&self) -> KeySymbolSet {
        self.variables
            .iter()
            .filter(|(_, instances)| instances.iter().any(|instance| instance.is_visible()))
            .map(|(name, _)| name.clone())
            .collect()
    }
}

impl Deref for MessageVariables {
    type Target = KeySymbolMap<Vec<MessageVariableInstance>>;

    fn deref(&self) -> &Self::Target {
        &self.variables
    }
}

pub fn collect_message_variables(ast: &AnyDocument) -> MessageVariables {
    let mut visitor = MessageVariablesVisitor::new();
    ast.visit_with(&mut visitor);
    visitor.into_variables()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selectable_variables_are_not_visible() {
        assert_eq!(MessageVariableType::Plural.is_visible_reference(), false);
        assert_eq!(
            MessageVariableType::NumericEnum {
                values: vec![],
                allow_other: false
            }
            .is_visible_reference(),
            false
        );
        assert_eq!(
            MessageVariableType::Enum {
                values: vec![],
                allow_other: false
            }
            .is_visible_reference(),
            false
        );
    }

    #[test]
    fn regular_variables_are_visible() {
        assert_eq!(MessageVariableType::Any.is_visible_reference(), true);
        assert_eq!(MessageVariableType::Number.is_visible_reference(), true);
        assert_eq!(MessageVariableType::Date.is_visible_reference(), true);
        assert_eq!(MessageVariableType::Time.is_visible_reference(), true);
    }

    #[test]
    fn function_variables_are_visible() {
        assert_eq!(
            MessageVariableType::HandlerFunction.is_visible_reference(),
            true
        );
        assert_eq!(
            MessageVariableType::HookFunction.is_visible_reference(),
            true
        );
        assert_eq!(
            MessageVariableType::LinkFunction.is_visible_reference(),
            true
        );
    }

    #[test]
    fn one_visible_reference_makes_variable_visible() {
        // This tests the difference between `{count, plural, other {foo}}` and
        // `{count, plural, other {# foo}}`, where only the latter has a visible reference because
        // of the `#` rendering the number.
        let mut variables = MessageVariables::new();
        variables.add_instance("foo", MessageVariableType::Plural, false, None);

        assert_eq!(variables.visible_variable_names().len(), 0);

        // Another visible instance of the same variable makes it visible
        variables.add_instance("foo", MessageVariableType::Number, false, None);
        assert_eq!(variables.visible_variable_names().len(), 1);
    }

    #[test]
    fn adding_invisible_reference_does_not_hide_variable() {
        let mut variables = MessageVariables::new();
        variables.add_instance("foo", MessageVariableType::Number, false, None);

        assert_eq!(variables.visible_variable_names().len(), 1);

        // Another visible instance of the same variable makes it visible
        variables.add_instance("foo", MessageVariableType::Plural, false, None);
        assert_eq!(variables.visible_variable_names().len(), 1);
    }
}
