use crate::writer::{
    write_doc, AlphabeticSymbolMap, AlphabeticSymbolSet, TypeDocFormat, TypeDocWriter, WriteResult,
};
use intl_database_core::{
    KeySymbol, KeySymbolSet, MessageVariableInstance, MessageVariableType, MessageVariables,
};

pub struct TypeDef {
    pub name: KeySymbol,
    pub variables: MessageVariables,
    pub spurious_variable_keys: KeySymbolSet,
}

impl TypeDef {
    fn get_total_type_from_variable_instances(
        &self,
        instances: &Vec<MessageVariableInstance>,
    ) -> AlphabeticSymbolSet {
        let mut set = AlphabeticSymbolSet::new();
        for instance in instances {
            add_strict_type_name(&mut set, &instance.kind)
        }
        set
    }
}

impl TypeDocFormat for TypeDef {
    fn fmt(&self, mut w: &mut TypeDocWriter) -> WriteResult {
        write_doc!(w, ["'", &self.name, "': TypedIntlMessageGetter<{"])?;

        let mut sorted_map: AlphabeticSymbolMap<AlphabeticSymbolSet> = AlphabeticSymbolMap::new();
        for (name, variable) in self.variables.iter() {
            sorted_map.insert(*name, self.get_total_type_from_variable_instances(variable));
        }

        let mut is_first = true;
        for (name, types) in sorted_map {
            if !is_first {
                write_doc!(w, [", "])?;
            } else {
                is_first = false;
            }

            // TODO: Do this once per variable rather than having to check every instance, since
            // builtin-ness is determined by the name, not the instance.
            let is_builtin = name.starts_with("$");
            // TODO: These types shouldn't actually be optional, as they'll crash at runtime.
            // Optionality is just a migration step.
            let is_optional = self.spurious_variable_keys.contains(&name);
            let undefinable = is_optional || is_builtin;
            write_doc!(w, [&name, &undefinable.then_some("?"), ": "])?;
            let mut is_first_type = true;
            for ty in types {
                write_doc!(w, [&(!is_first_type).then_some(" | "), &ty])?;
                is_first_type = false;
            }
        }

        write_doc!(w, ["}>"])
    }
}

fn add_strict_type_name(set: &mut AlphabeticSymbolSet, kind: &MessageVariableType) {
    match kind {
        MessageVariableType::Any => {
            set.insert("any".into());
        }
        MessageVariableType::Number => {
            // Number allows both `number` and `string`, because `Intl.NumberFormat` is able to
            // internally parse the string into a number before formatting. Note that this _only_
            // applies to number formatting and does not happen for dates or times or other values.
            set.insert("number".into());
            set.insert("string".into());
        }
        MessageVariableType::Plural => {
            set.insert("number".into());
        }
        MessageVariableType::Enum(values) => {
            for value in values {
                if value == "other" {
                    set.insert("string".into());
                } else {
                    set.insert(format!("'{value}'").into());
                }
            }
        }
        MessageVariableType::Date => {
            set.insert("Date".into());
            set.insert("number".into());
        }
        MessageVariableType::Time => {
            set.insert("Date".into());
            set.insert("number".into());
        }
        MessageVariableType::HookFunction => {
            set.insert("HookFunction".into());
        }
        MessageVariableType::LinkFunction => {
            set.insert("LinkFunction".into());
        }
        MessageVariableType::HandlerFunction => {
            set.insert("HandlerFunction".into());
        }
    }
}
