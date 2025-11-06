use super::plural_option_data::{CARDINAL_PLURAL_SELECTORS, ORDINAL_PLURAL_SELECTORS};
use std::collections::HashSet;

pub fn is_valid_cardinal_selector(selector: &str, locale: &str) -> bool {
    get_valid_cardinal_selectors(locale).map_or(false, |options| options.contains(selector))
}

pub fn is_valid_ordinal_selector(selector: &str, locale: &str) -> bool {
    get_valid_cardinal_selectors(locale).map_or(false, |options| options.contains(selector))
}

pub fn canonicalized_locale_names_for_plurals(locale: &str) -> [String; 3] {
    [
        String::from(locale),
        locale.replace('-', "_"),
        locale.split('-').next().unwrap().to_string(),
    ]
}
pub fn get_valid_cardinal_selectors(locale: &str) -> Option<HashSet<&'static str>> {
    for locale in canonicalized_locale_names_for_plurals(locale) {
        if let Some(options) = CARDINAL_PLURAL_SELECTORS.get(&locale.as_str()) {
            return Some(options.clone());
        }
    }
    None
}

pub fn get_valid_ordinal_selectors(locale: &str) -> Option<HashSet<&'static str>> {
    for locale in canonicalized_locale_names_for_plurals(locale) {
        if let Some(options) = ORDINAL_PLURAL_SELECTORS.get(&locale.as_str()) {
            return Some(options.clone());
        }
    }
    None
}
