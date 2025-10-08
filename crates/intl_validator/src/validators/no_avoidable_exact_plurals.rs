use crate::macros::cst_validation_rule;
use crate::{DiagnosticFix, DiagnosticName, DiagnosticSeverity, TextRange, ValueDiagnostic};
use intl_markdown::{IcuPlural, Visit, VisitWith};
use intl_markdown_syntax::{Syntax, SyntaxNode, SyntaxToken};

cst_validation_rule!(NoAvoidableExactPlurals);

// ICU Plurals can use an "exact" syntax, like `=1`, to specify a specific number to match for a
// plural arm compared to the normal method of LDML Plural Selectors, like `one`. In the vast
// majority of cases, the exact number is used as a "clearer shortcut" rather than actually being
// a necessary selector. This has no real impact in English, since the only value that matches
// `one` is 1 itself, but other languages have multiple numbers that fit the "singular" semantic
// of `one`, like in Russian most numbers _ending_ with a 1 count (21, 31, 101, 1001, etc.).
//
// To help translators clearly understand the intent of a message and avoid accidental
// mistranslations between languages with different plural rules, this validator reports whenever a
// message uses an exact plural where it is not necessary. Exceptions that _are_ considered
// necessary include:
// - `=0` when the value of the arm contains no literal 0 (e.g., `{count, plural, =0 {no items}}`,
//   rather than `{count, plural, =0 {0 items}}` which could be replaced).
// - `=1` when
// - `=2` or any other value above 1, where there is no exact matching LDML plural rule to replace
//   the selector with.
// - `=-1` or any other negative number, since these are typically used as controls rather than
//   actual plurals, even though the mechanism should prefer using `select` or multiple messages
//   instead.
//
// Additionally, this rule will detect the inverse problem, where the `zero` or `one` selector is
// used but the value contains a literal `1`, which can result in tangibly-incorrect content being
// shown to users in other languages if they fall back to a default translation (e.g.,
// `{count, plural, one {1 item}, other {# items}}` rendered in Russian would show "1 item" even if
// `count == 21`). These cases should always prefer using a `#` to get both localized number
// formatting _and_ accurate rendering in other locales.
//
// A full list of rules as defined by Unicode LDML and used by most localization implementations is
// available at: https://www.unicode.org/cldr/charts/43/supplemental/language_plural_rules.html.

impl NoAvoidableExactPlurals {
    fn report_avoidable_exact_plural(
        &mut self,
        selector: &SyntaxToken,
        literal_range: TextRange,
        replacement_selector: &str,
    ) {
        let selector_text = selector.text();
        let selector_position = selector.source_position();
        self.diagnostics.push(ValueDiagnostic {
            name: DiagnosticName::NoAvoidableExactPlurals,
            span: Some(selector_position),
            severity: DiagnosticSeverity::Warning,
            description: format!("Exact selector {selector_text} should be written as '{replacement_selector}' and use `#` as the value inside"),
            help: None,
            fixes: vec![
                DiagnosticFix::replace_text(selector_position, replacement_selector),
                DiagnosticFix::replace_text(literal_range, "#")
            ],
        });
    }

    fn report_literal_in_value(&mut self, literal_range: TextRange, found: &str, selector: &str) {
        self.diagnostics.push(ValueDiagnostic {
            name: DiagnosticName::NoAvoidableExactPlurals,
            span: Some(literal_range),
            severity: DiagnosticSeverity::Warning,
            description: format!("Literal value {found} will not always align with selector {selector} in all locales. Use # to represent the value instead"),
            help: None,
            fixes: vec![DiagnosticFix::replace_text(literal_range, "#")],
        });
    }
}

impl Visit for NoAvoidableExactPlurals {
    fn visit_icu_plural(&mut self, node: &IcuPlural) {
        for arm in node.arms().children() {
            let selector = arm.selector_token();
            let content = arm.value().content();
            let value = content.syntax();
            match selector.text() {
                "=0" => {
                    if let Some(number_range) = find_literal_number_at_start_of_content(value, "0")
                    {
                        self.report_avoidable_exact_plural(&selector, number_range, "zero");
                    }
                }
                "=1" => {
                    if let Some(number_range) = find_literal_number_at_start_of_content(value, "1")
                    {
                        self.report_avoidable_exact_plural(&selector, number_range, "one");
                    }
                }
                "zero" => {
                    if let Some(number_range) = find_literal_number_at_start_of_content(value, "0")
                    {
                        self.report_literal_in_value(number_range, "0", "zero");
                    }
                }
                "one" => {
                    if let Some(number_range) = find_literal_number_at_start_of_content(value, "1")
                    {
                        self.report_literal_in_value(number_range, "1", "one");
                    }
                }
                _ => (),
            };
            arm.visit_with(self);
        }
    }
}

/// Returns the source position of the `expected` text if it appears at the very start of the given
/// node's contained text.
fn find_literal_number_at_start_of_content(node: &SyntaxNode, expected: &str) -> Option<TextRange> {
    let first_child = node.iter_tokens().next()?;
    if first_child.text().starts_with(expected) {
        let mut position = first_child.source_position();
        position.1 = position.0 + expected.len();
        Some(position)
    } else {
        None
    }
}
