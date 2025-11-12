use crate::macros::cst_validation_rule;
use crate::{util, DiagnosticCategory, DiagnosticFix, DiagnosticName, TextRange, ValueDiagnostic};
use intl_markdown::{Icu, IcuPlural, IcuPluralArm, IcuPluralValue, IcuPound, Visit, VisitWith};
use intl_markdown_syntax::{Syntax, SyntaxNode, SyntaxToken};
use once_cell::sync::Lazy;
use regex::Regex;

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
// - `=0`/`=1` when the value of the arm contains no literal 0 or 1 (e.g.,
//  `{count, plural, =0 {no items}}`, rather than `{count, plural, =0 {0 items}}` which could be
//   replaced).
// - `=2` or any other value above 1, where there is no exact matching LDML plural rule to replace
//   the selector with.
// - `=-1` or any other negative number, since these are typically used as controls rather than
//   actual plurals, even though the mechanism should prefer using `select` or multiple messages
//   instead.
//
// Additionally, this rule will detect the inverse problem, where the `zero` or `one` selector is
// used but the value contains a literal `1`, which can result in tangibly-incorrect content being
// shown to users in other languages if they fall back to a default translation (e.g.,
// `{count, plural, one {1 item} other {# items}}` rendered in Russian would show "1 item" even if
// `count == 21`). These cases should always prefer using a `#` to get both localized number
// formatting _and_ accurate rendering in other locales.
//
// A full list of rules as defined by Unicode and used by most localization implementations is
// available at: https://www.unicode.org/cldr/charts/43/supplemental/language_plural_rules.html.

impl NoAvoidableExactPlurals {
    fn report_too_many_exact_selectors(
        &mut self,
        node: &IcuPlural,
        exact_count: usize,
        include_select_fix: bool,
    ) {
        let fixes = if include_select_fix {
            util::ops::replace_plural_with_select(node)
        } else {
            vec![]
        };

        self.context.report(ValueDiagnostic {
            name: DiagnosticName::NoAvoidableExactPlurals,
            span: Some(node.syntax().source_position()),
            category: DiagnosticCategory::Suspicious,
            description: format!("Too many exact selectors in this plural ({exact_count}). Consider breaking each case into a separate message or using `select` instead"),
            help: None,
            fixes,
        })
    }

    fn report_avoidable_exact_plural(
        &mut self,
        selector: &SyntaxToken,
        literal_replacement_range: Option<TextRange>,
        replacement_selector: &str,
    ) {
        let selector_text = selector.text();
        let selector_position = selector.source_position();
        let mut fixes = vec![DiagnosticFix::replace_text(
            selector_position,
            replacement_selector,
        )];
        if let Some(range) = literal_replacement_range {
            fixes.push(DiagnosticFix::replace_text(range, "#"));
        }

        self.context.report(ValueDiagnostic {
            name: DiagnosticName::NoAvoidableExactPlurals,
            span: Some(selector_position),
            category: DiagnosticCategory::Correctness,
            description: format!("Exact selector {selector_text} should be written as '{replacement_selector}' and use `#` as the value inside"),
            help: None,
            fixes,
        });
    }

    fn report_literal_in_value(&mut self, literal_range: TextRange, selector: &str) {
        self.context.report(ValueDiagnostic {
            name: DiagnosticName::NoAvoidableExactPlurals,
            span: Some(literal_range),
            category: DiagnosticCategory::Correctness,
            description: format!("Literal value will not always align with selector {selector} in all locales. Use # to represent the value instead"),
            help: None,
            fixes: vec![DiagnosticFix::replace_text(literal_range, "#")],
        });
    }
}

impl Visit for NoAvoidableExactPlurals {
    fn visit_icu_plural(&mut self, node: &IcuPlural) {
        node.visit_children_with(self);

        let mut has_only_exact_selectors = true;
        let mut exact_selector_count = 0;
        for arm in node.arms().children() {
            if arm.is_exact_selector() {
                exact_selector_count += 1;
            } else if !arm.is_other_selector() {
                has_only_exact_selectors = false;
            }
        }

        // If there are too many exact selectors, that's the only diagnostic we want to report,
        // as the "correct fixes" applied below are less accurate than the guidance to prefer
        // splitting up the message or using a `select` instead.
        if exact_selector_count > 2 {
            self.report_too_many_exact_selectors(
                node,
                exact_selector_count,
                has_only_exact_selectors,
            );
            return;
        }

        for arm in node.arms().children() {
            arm.visit_children_with(self);

            let mut scan = PluralArmLiteralScan::default();
            arm.visit_with(&mut scan);
            let Some(selector) = scan.selector_value else {
                continue;
            };

            // An exact selector with no exact number within it does not necessitate the exact
            // selector, so it should be replaced with a category selector instead. Or, if a number
            // in the value is considered "replaceable" (meaning only grammatically significant and
            // not semantically significant), then the selector and value should be replaced, too.
            if selector.is_exact() {
                // Non-numerics only apply to non-zero selectors since a zero value could be
                // "they're all gone" rather than "0 remain", for example.
                if (scan.is_simply_non_numeric && !selector.is_zero())
                    // NOTE: Having a pound does not preclude having a numeric literal, so this can
                    // be inaccurate, though it's very unlikely.
                    || scan.pound.is_some()
                    || scan.replaceable_literal_number.is_some()
                {
                    let category_selector = selector.to_category();
                    if util::plural_rules::is_valid_cardinal_selector(
                        category_selector,
                        &self.context.locale,
                    ) {
                        self.report_avoidable_exact_plural(
                            &arm.selector_token(),
                            scan.replaceable_literal_number,
                            category_selector,
                        )
                    }
                }
            } else {
                // For category selectors, a literal number inside the value is tangibly incorrect,
                // and should always be replaced by a pound.
                if let Some(literal_range) = scan.replaceable_literal_number {
                    self.report_literal_in_value(literal_range, "#");
                }
            }
        }
    }
}

enum RelevantPluralSelector {
    Zero,
    One,
    ExactZero,
    ExactOne,
}

impl RelevantPluralSelector {
    fn from_str(value: &str) -> Self {
        match value {
            "=0" => RelevantPluralSelector::ExactZero,
            "=1" => RelevantPluralSelector::ExactOne,
            "zero" => RelevantPluralSelector::Zero,
            "one" => RelevantPluralSelector::One,
            _ => unreachable!("Don't call from_str with a non-relevant plural selector"),
        }
    }

    fn to_value(&self) -> &'static str {
        match self {
            RelevantPluralSelector::Zero | RelevantPluralSelector::ExactZero => "0",
            RelevantPluralSelector::One | RelevantPluralSelector::ExactOne => "1",
        }
    }

    fn to_category(&self) -> &'static str {
        match self {
            RelevantPluralSelector::Zero | RelevantPluralSelector::ExactZero => "zero",
            RelevantPluralSelector::One | RelevantPluralSelector::ExactOne => "one",
        }
    }

    fn is_exact(&self) -> bool {
        matches!(
            self,
            RelevantPluralSelector::ExactZero | RelevantPluralSelector::ExactOne
        )
    }

    fn is_zero(&self) -> bool {
        matches!(
            self,
            RelevantPluralSelector::Zero | RelevantPluralSelector::ExactZero
        )
    }
}

#[derive(Default)]
struct PluralArmLiteralScan {
    /// If the selector for this arm is an exact selector, extract the numeric value of it.
    selector_value: Option<RelevantPluralSelector>,
    /// Location of a literal number that should be replaced with `#` in the plural content.
    replaceable_literal_number: Option<TextRange>,
    /// Location of a pound node within the plural content, indicating a dynamic number that does
    /// not depend on an exact selector.
    pound: Option<TextRange>,
    /// True when there is no reference to a numeric value within the plural content at all,
    /// meaning an exact `=1` selector likely has no effect and should be replaced by a generic
    /// `one` instead. This only applies to `=1` and not `=0` because `=0` can say something like
    /// "Sorry, they're gone." instead of "None left" or "zero remaining", so there's too much
    /// ambiguity to confidently mark it as incorrect.
    is_simply_non_numeric: bool,
}

static NUMERIC_CONTENT_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r#"\b(\d+|one)\b"#).unwrap());

/// This visit implementation is _only_ valid for starting from an `IcuPluralArm`. It will not
/// detect or traverse content to discover other plural arms for checking.
impl Visit for PluralArmLiteralScan {
    // Nested ICU "resets" the plural context, so no need to investigate further. This override
    // will prevent the visitor from recursing through these children.
    fn visit_icu(&mut self, _: &Icu) {
        ()
    }

    fn visit_icu_pound(&mut self, node: &IcuPound) {
        self.pound = Some(node.hash_token().source_position())
    }

    fn visit_icu_plural_arm(&mut self, arm: &IcuPluralArm) {
        let selector_token = arm.selector_token();
        match selector_token.text() {
            "=0" | "=1" | "one" | "zero" => {
                let selector = RelevantPluralSelector::from_str(selector_token.text());
                self.replaceable_literal_number = find_literal_number_at_start_of_content(
                    arm.value().content().syntax(),
                    selector.to_value(),
                );
                self.selector_value = Some(selector);
            }
            _ => {}
        };

        arm.value().visit_with(self);
    }

    fn visit_icu_plural_value(&mut self, node: &IcuPluralValue) {
        node.visit_children_with(self);
        // TODO: This is a relatively naive check and doesn't account for simple Markdown within
        // the node, which may still be "simply non-numeric" and worth preventing.
        let content = node.content();
        let mut tokens = content.syntax().iter_tokens();
        match tokens.next() {
            Some(token) if tokens.next().is_none() => {
                let has_numeric_reference = NUMERIC_CONTENT_REGEX.is_match(token.text());
                self.is_simply_non_numeric = !has_numeric_reference;
            }
            _ => {}
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
