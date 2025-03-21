use std::ops::Deref;

use rustc_hash::FxHashSet;
use serde::Serialize;

use intl_markdown_visitor::visit_with_mut;

use crate::database::symbol::{KeySymbol, KeySymbolMap};
use crate::error::DatabaseResult;
use crate::message::variables::visitor::MessageVariablesVisitor;

mod visitor;

#[derive(Clone, Debug, Serialize, Hash, PartialEq, Eq)]
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
    Enum(Vec<String>),
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

/// A representation of a single _instance_ of a variable in a message. Each
/// time a variable appears in a message, even if it is a variable that has
/// already been seen, a new MessageVariable is created.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MessageVariableInstance {
    /// The location in the message where this variable is used. Each instance
    /// of a variable in a string has its own struct, so each stores its own
    /// span as well.
    /// TODO: Add this back
    pub span: Option<usize>,
    /// `true` if this variable is a system-defined variable, typically for
    /// rich text formatting tags like `$b` and `$link`, which are almost never
    /// intended for a user to provide and/or only represent formatting points,
    /// but can be given as an override regardless.
    pub is_builtin: bool,
    /// The specific kind of the variable, used for generating types.
    pub kind: MessageVariableType,
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
        name: KeySymbol,
        kind: MessageVariableType,
        is_builtin: bool,
        span: Option<usize>,
    ) {
        let instance = MessageVariableInstance {
            kind,
            is_builtin,
            span,
        };
        self.variables
            .entry(name)
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
    pub fn get_keys(&self) -> FxHashSet<&KeySymbol> {
        self.variables.keys().collect::<FxHashSet<&KeySymbol>>()
    }

    /// Returns the count of _uniquely-named_ variables found in the message
    pub fn count(&self) -> usize {
        self.variables.len()
    }

    pub fn get(&self, key: &KeySymbol) -> Option<&Vec<MessageVariableInstance>> {
        self.variables.get(key)
    }
}

impl Deref for MessageVariables {
    type Target = KeySymbolMap<Vec<MessageVariableInstance>>;

    fn deref(&self) -> &Self::Target {
        &self.variables
    }
}

pub fn collect_message_variables(
    ast: &intl_markdown::Document,
) -> DatabaseResult<MessageVariables> {
    let mut visitor = MessageVariablesVisitor::new();
    visit_with_mut(&ast, &mut visitor);
    Ok(visitor.into_variables())
}
