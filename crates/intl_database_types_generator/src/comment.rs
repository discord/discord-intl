use crate::writer::{
    write_doc, AlphabeticSymbolMap, AlphabeticSymbolSet, TypeDocFormat, TypeDocWriter, WriteResult,
};

pub struct DocComment<'a> {
    /// Hashed key of the message
    pub(super) key: &'a str,
    /// Raw text of the definition of the message
    pub(super) value: Option<&'a str>,
    /// Optional description of the message provided from the definition
    pub(super) description: Option<&'a str>,
    /// Locales where the message expected a translation but was not found
    pub(super) missing_translations: AlphabeticSymbolSet,
    /// Whether this message is marked as is_secret
    pub(super) is_secret: bool,
    /// Whether this message is marked as ready for translation
    pub(super) ready_to_translate: bool,
    /// When translations of a message contain variables different from those defined in the source
    /// message, this map contains the name of the variable mapped to the locales where it is
    /// defined.
    pub(super) spurious_variables: AlphabeticSymbolMap<AlphabeticSymbolSet>,
}

impl TypeDocFormat for DocComment<'_> {
    fn fmt(&self, mut w: &mut TypeDocWriter) -> WriteResult {
        w.push_prefix(" * ");
        write_doc!(w, ["/**\nKey: `", &self.key, "`"])?;
        write_doc!(
            w,
            ["\n\n### Definition\n", "```text\n", &self.value, "\n```"]
        )?;

        if !self.ready_to_translate {
            write_doc!(w, ["\n\n**Not ready for translation**"])?;
        }

        let is_missing_translations = !self.missing_translations.is_empty();
        let has_spurious_variables = !self.spurious_variables.is_empty();
        let has_problems = is_missing_translations || has_spurious_variables;

        if has_problems {
            write_doc!(w, ["\n\n### Problems"])?;
            if self.ready_to_translate && is_missing_translations {
                let locales = self
                    .missing_translations
                    .iter()
                    .map(|locale| ["`", &locale, "`"]);
                write_doc!(w, ["\n\nMissing translations: "])?;
                let mut is_first = true;
                for locale in locales {
                    if !is_first {
                        write_doc!(w, [", "])?;
                    } else {
                        is_first = false;
                    }
                    write_doc!(w, [&locale])?;
                }
            }
            if has_spurious_variables {
                write_doc!(w, ["\n\nSpurious variables from translations:"])?;
                for (variable, locales) in &self.spurious_variables {
                    let locales = locales.iter().map(|locale| ["`", &locale, "`"]);
                    write_doc!(w, ["\n  - `", &variable, "`: "])?;
                    let mut is_first = true;
                    for locale in locales {
                        write_doc!(w, [&(!is_first).then_some(", "), &locale])?;
                        is_first = false;
                    }
                }
            }
        } else {
            if self.ready_to_translate && !is_missing_translations {
                write_doc!(w, ["\n\n**Translated in all locales**"])?;
            }
        }

        if let Some(description) = self.description {
            write_doc!(w, ["\n@description - ", description])?;
        }

        if self.value.is_none() {
            write_doc!(
                w,
                ["\n@deprecated - This message has no definition, only translations. It should not be used until a definition is added."]
            )?;
        }

        if self.is_secret {
            write_doc!(w, ["\n@experimental - This message is marked as *secret*. It will be obfuscated in production builds"])?;
        }

        w.pop_prefix();
        write_doc!(w, ["\n */"])?;
        Ok(())
    }
}
