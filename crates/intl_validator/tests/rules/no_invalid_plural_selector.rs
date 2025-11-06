use crate::harness;
use intl_validator::validators::NoInvalidPluralSelector;
use intl_validator::{DiagnosticName, ValueDiagnostic};

fn validate(content: &str, locale: &str) -> Vec<ValueDiagnostic> {
    let message = harness::define_single_message("MESSAGE_KEY", content);
    harness::validate_with(&message, NoInvalidPluralSelector::new(locale.into())).unwrap_or(vec![])
}

macro_rules! assert_has_diagnostic {
    ($diagnostics:expr, $span:tt) => {{
        let diagnostics = &$diagnostics;
        let name = DiagnosticName::NoInvalidPluralSelector;
        let span = $span;
        assert_eq!(
            harness::has_matching_diagnostic(diagnostics, name, span),
            true,
            "Did not find matching diagnostic {name}({span:?}) in {diagnostics:#?}"
        );
    }};
}

#[test]
fn valid_plural() {
    assert_eq!(
        validate("{count, plural, one {# item} other {# items}}", "en-US").len(),
        0
    );
}

#[test]
fn other_only_locale() {
    let diagnostics = validate(
        "{count, plural, one {# item} two {# items} other {# items}}",
        "ja",
    );

    assert_eq!(diagnostics.len(), 2);
    assert_has_diagnostic!(diagnostics, (16, 29));
    assert_has_diagnostic!(diagnostics, (29, 43));
}

#[test]
fn two_selector() {
    let message = "{count, plural, one {# item} two {# items} other {# items}}";

    let diagnostics = validate(message, "br");
    assert_eq!(diagnostics.len(), 0);

    let diagnostics = validate(message, "en-US");
    assert_eq!(diagnostics.len(), 1);
    assert_has_diagnostic!(diagnostics, (29, 43));
}

#[test]
fn all_valid() {
    let diagnostics = validate(
        "{count, plural, zero {foo} one {bar} two {baz} few {aaa} many {bbb} other {# items}}",
        "cy",
    );

    assert_eq!(diagnostics.len(), 0);
}


#[test]
fn exact_selectors() {

    let diagnostics = validate(
        "{count, plural, =0 {foo} =1 {bar} =2 {baz} other {# items}}",
        "ja",
    );

    assert_eq!(diagnostics.len(), 0);
}