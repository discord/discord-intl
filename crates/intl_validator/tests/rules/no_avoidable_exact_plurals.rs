use crate::harness;
use intl_validator::validators::NoAvoidableExactPlurals;
use intl_validator::{DiagnosticName, ValueDiagnostic};

fn validate(content: &str) -> Vec<ValueDiagnostic> {
    let message = harness::define_single_message("MESSAGE_KEY", content);
    harness::validate_with(message, NoAvoidableExactPlurals::new()).unwrap_or(vec![])
}

macro_rules! assert_has_diagnostic {
    ($diagnostics:expr, $span:tt) => {{
        let diagnostics = &$diagnostics;
        let name = DiagnosticName::NoAvoidableExactPlurals;
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
fn selectordinal() {
    // Ordinals are not used for the same kind of pluralization, so it's not checked.
    let diagnostics = validate("{count, selectordinal, =1 {1st} other {{count}nd}}");
    assert_eq!(diagnostics.len(), 0);
}

#[test]
fn select_is_not_plural() {
    let diagnostics = validate("{count, select, =1 {1 item}}");
    assert_eq!(diagnostics.len(), 0);
}

#[test]
fn valid_exact_zero() {
    let diagnostics = validate("{count, plural, =0 {No items} other {# items}}");
    println!("{diagnostics:?}");
    assert_eq!(diagnostics.len(), 0);
}

#[test]
fn valid_exact_one() {
    assert_eq!(
        validate("{count, plural, =1 {Only 1 item left!} other {# items}}").len(),
        0
    );
}

#[test]
fn leading_exact_one() {
    let diagnostics = validate("{count, plural, =1 {1 second} other {# seconds}}");
    assert_eq!(diagnostics.len(), 1);
    assert_has_diagnostic!(diagnostics, (16, 18));
}

#[test]
fn leading_exact_zero() {
    let diagnostics = validate("{count, plural, =0 {0 second} other {# seconds}}");
    assert_eq!(diagnostics.len(), 1);
    assert_has_diagnostic!(diagnostics, (16, 18));
}

#[test]
fn multiple_matches() {
    let diagnostics = validate("{count, plural, =0 {0 second} =1 {1 second} other {# seconds}}");
    assert_eq!(diagnostics.len(), 2);
    assert_has_diagnostic!(diagnostics, (16, 18));
    assert_has_diagnostic!(diagnostics, (30, 32));
}

#[test]
fn exact_zero_value() {
    let diagnostics =
        validate("{count, plural, zero {0 seconds} one {# second} other {# seconds}}");
    assert_eq!(diagnostics.len(), 1);
    assert_has_diagnostic!(diagnostics, (22, 23));
}

#[test]
fn exact_one_value() {
    let diagnostics = validate("{count, plural, one {1 second} other {# seconds}}");
    assert_eq!(diagnostics.len(), 1);
    assert_has_diagnostic!(diagnostics, (21, 22));
}

#[test]
fn preserves_trivia() {
    let diagnostics = validate("{count, plural, one {   1  {unit} second} other {# seconds}}");
    assert_eq!(diagnostics.len(), 1);
    assert_has_diagnostic!(diagnostics, (24, 25));
}

#[test]
fn mismatched_exact_one_zero() {
    let diagnostics = validate("{count, plural, =0 {1 second} =1 {0 second}}");
    assert_eq!(diagnostics.len(), 0);
}

#[test]
fn negative_selector() {
    let diagnostics = validate("{count, plural, =-1 {invalid value} other {some items}}");
    assert_eq!(diagnostics.len(), 0);
}

#[test]
fn higher_selector() {
    let diagnostics = validate("{count, plural, =2 {2 large number} other {some items}}");
    assert_eq!(diagnostics.len(), 0);
}

#[test]
fn pound_in_exact_zero() {
    let diagnostics = validate("{count, plural, =0 { # item} other {some items}}");
    assert_eq!(diagnostics.len(), 1);
    assert_has_diagnostic!(diagnostics, (16, 18));
}

#[test]
fn pound_in_exact_one() {
    let diagnostics = validate("{count, plural, =1 { # item} other {some items}}");
    assert_eq!(diagnostics.len(), 1);
    assert_has_diagnostic!(diagnostics, (16, 18));
}

#[test]
fn exact_one_with_no_literal() {
    let diagnostics = validate("{count, plural, =1 {an item} other {some items}}");
    assert_eq!(diagnostics.len(), 1);
    assert_has_diagnostic!(diagnostics, (16, 18));
}
