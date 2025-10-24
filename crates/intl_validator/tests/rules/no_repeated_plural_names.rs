use crate::harness;
use intl_validator::validators::NoRepeatedPluralNames;
use intl_validator::{DiagnosticName, ValueDiagnostic};

fn validate(content: &str) -> Vec<ValueDiagnostic> {
    let message = harness::define_single_message("MESSAGE_KEY", content);
    harness::validate_with(&message, NoRepeatedPluralNames::new()).unwrap_or(vec![])
}

macro_rules! assert_has_diagnostic {
    ($diagnostics:expr, $span:tt) => {{
        let diagnostics = &$diagnostics;
        let name = DiagnosticName::NoRepeatedPluralNames;
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
        validate("{count, plural, one {# item} other {# items}}").len(),
        0
    );
}
#[test]
fn other_variables() {
    assert_eq!(
        validate("{count, plural, one {{bar} item} other {{bar} items}}").len(),
        0
    );
}

#[test]
fn one_repeated_plural() {
    let diagnostics = validate("{count, plural, one {{count} item} other {# items}}");

    assert_eq!(diagnostics.len(), 1);
    assert_has_diagnostic!(diagnostics, (21, 28));
}

#[test]
fn multiple_repeated_plurals() {
    let diagnostics = validate("{count, plural, one {{count} item} other {{count} items}}");

    assert_eq!(diagnostics.len(), 2);
    assert_has_diagnostic!(diagnostics, (21, 28));
    assert_has_diagnostic!(diagnostics, (42, 49));
}

#[test]
fn repeated_number() {
    let diagnostics = validate("{count, plural, one {{count, number} item} other {foo}}");

    assert_eq!(diagnostics.len(), 1);
    assert_has_diagnostic!(diagnostics, (21, 36));
}

#[test]
fn repeated_number_with_style() {
    let diagnostics =
        validate("{count, plural, one {{count, number, ::currency/USD} item} other {foo}}");

    assert_eq!(diagnostics.len(), 0);
}

#[test]
fn repeated_other_type() {
    // NOTE: this kind of conversion is arguably also invalid, but it's not captured by this
    // validation rule, since it is not correctly fixed by using `#` instead.
    let diagnostics = validate("{count, plural, one {{count, date} item} other {foo}}");

    assert_eq!(diagnostics.len(), 0);
}

#[test]
fn nested_plurals() {
    // Only the immediate child can be replaced with `#`, since the outer plural is shadowed by the
    // inner one.
    let diagnostics = validate("{count, plural, one {{other, plural, one {{other} {count} item} other {foo}} other {{count} {other} foo}}");

    assert_eq!(diagnostics.len(), 2);
    assert_has_diagnostic!(diagnostics, (42, 49));
    assert_has_diagnostic!(diagnostics, (84, 91));
}

#[test]
fn selectordinal() {
    let diagnostics = validate("{count, selectordinal, one {{count}st} other {{count}nd}}");

    assert_eq!(diagnostics.len(), 2);
    assert_has_diagnostic!(diagnostics, (28, 35));
    assert_has_diagnostic!(diagnostics, (46, 53));
}

#[test]
fn select_is_not_plural() {
    let diagnostics = validate("{count, select, one {{count}st} other {{count}nd}}");

    assert_eq!(diagnostics.len(), 0);
}
