use quote::quote;
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};
use xtask::util;
use xtask::util::Codegen;

#[derive(Deserialize)]
struct PluralSupplementalData {
    plurals: SourcePluralRulesContainer,
}

#[derive(Deserialize)]
struct SourcePluralRulesContainer {
    #[allow(unused)]
    #[serde(rename = "@type")]
    ty: String,
    #[serde(rename = "pluralRules")]
    plural_rules: Vec<SourcePluralRuleGroup>,
}

#[derive(Deserialize)]
struct SourcePluralRuleGroup {
    #[serde(rename = "@locales")]
    locales: Vec<String>,
    #[serde(rename = "pluralRule")]
    plural_rule: Vec<SourcePluralRule>,
}

#[derive(Deserialize)]
struct SourcePluralRule {
    #[serde(rename = "@count")]
    count: String,
    #[allow(unused)]
    #[serde(rename = "#text", default)]
    definition: String,
}

type OptionMap = BTreeMap<String, BTreeSet<String>>;

fn compile_plural_rules(source: &str) -> OptionMap {
    let rules: PluralSupplementalData =
        serde_xml_rs::from_str(source).expect("Failed to parse plural rules XML");

    let mut option_map: OptionMap = BTreeMap::new();
    for rule in rules.plurals.plural_rules {
        let options: BTreeSet<String> =
            BTreeSet::from_iter(rule.plural_rule.iter().map(|rule| rule.count.clone()));
        for locale in rule.locales.iter().cloned() {
            option_map.insert(locale.to_string(), options.clone());
        }
    }

    option_map
}

fn transpile_map_to_source(map: OptionMap) -> proc_macro2::TokenStream {
    let mut entries = vec![];
    for (locale, options) in map {
        entries.push(quote! {
            (#locale, HashSet::from([#(#options),*]))
        });
    }

    quote! {HashMap::from([#(#entries),*])}
}

fn try_main() -> anyhow::Result<()> {
    let ordinals = compile_plural_rules(include_str!("ordinal-rules.xml"));
    let cardinals = compile_plural_rules(include_str!("cardinal-rules.xml"));

    let ordinal_declaration = transpile_map_to_source(ordinals.clone());
    let cardinal_declaration = transpile_map_to_source(cardinals.clone());

    let content = quote! {
        use std::collections::{HashMap, HashSet};
        use lazy_static::lazy_static;

        lazy_static! {
            pub(super) static ref CARDINAL_PLURAL_SELECTORS: HashMap<&'static str, HashSet<&'static str>> = #cardinal_declaration;
            pub(super) static ref ORDINAL_PLURAL_SELECTORS: HashMap<&'static str, HashSet<&'static str>> = #ordinal_declaration;
        }
    };

    let mut codegen = Codegen::new(util::repo_root().join("crates/intl_validator"));
    codegen.write_file("src/util/plural_option_data.rs", content.to_string())?;
    codegen.finish()
}

fn main() {
    try_main().unwrap_or_else(|e| {
        eprintln!("{}", e);
        std::process::exit(1);
    });
}
