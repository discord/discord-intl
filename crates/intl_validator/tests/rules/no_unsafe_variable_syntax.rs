use crate::harness;
use intl_validator::validators::NoUnsafeVariableSyntax;
use intl_validator::{DiagnosticName, ValueDiagnostic};

fn validate(content: &str) -> Vec<ValueDiagnostic> {
    let message = harness::define_single_message("MESSAGE_KEY", content);
    harness::validate_with(&message, NoUnsafeVariableSyntax::new("en-US".into())).unwrap_or(vec![])
}

macro_rules! assert_has_diagnostic {
    ($diagnostics:expr, $span:tt) => {{
        let diagnostics = &$diagnostics;
        let name = DiagnosticName::NoUnsafeVariableSyntax;
        let span = $span;
        assert_eq!(
            harness::has_matching_diagnostic(diagnostics, name, span),
            true,
            "Did not find matching diagnostic {name}({span:?}) in {diagnostics:#?}"
        );
    }};
}

#[test]
fn plain_variable() {
    assert_eq!(validate("{foo}").len(), 0);
    assert_eq!(validate("hello, {user}").len(), 0);
    assert_eq!(validate("{user} is here").len(), 0);
}

#[test]
fn ambiguous_writing() {
    assert_eq!(validate("{user}!!").len(), 0);
    assert_eq!(validate("!!{user}").len(), 0);
    assert_eq!(validate("!{user}!").len(), 0);
    assert_eq!(validate("!{user}!!!").len(), 0);
    assert_eq!(validate("hi!! {user}!!").len(), 0);
}

#[test]
fn unsafe_at_start() {
    let diagnostics = validate("!!{user}!! is here");
    assert_eq!(diagnostics.len(), 1);
    assert_has_diagnostic!(diagnostics, (0, 10));
}

#[test]
fn unsafe_at_end() {
    let diagnostics = validate("hello, !!{user}!!");
    assert_eq!(diagnostics.len(), 1);
    assert_has_diagnostic!(diagnostics, (7, 17));
}

#[test]
fn unsafe_only() {
    let diagnostics = validate("!!{user}!!");
    assert_eq!(diagnostics.len(), 1);
    assert_has_diagnostic!(diagnostics, (0, 10));
}

#[test]
fn multiple_within_message() {
    let diagnostics = validate("!!{user}!! is also !!{otherUser}!!");
    assert_eq!(diagnostics.len(), 2);
    assert_has_diagnostic!(diagnostics, (0, 10));
    assert_has_diagnostic!(diagnostics, (19, 34));
}

#[test]
fn other_types() {
    let diagnostics = validate("!!{now, time}!! !!{today, number}!!");
    assert_eq!(diagnostics.len(), 2);
    assert_has_diagnostic!(diagnostics, (0, 15));
    assert_has_diagnostic!(diagnostics, (16, 35));
}

#[test]
fn plural() {
    let diagnostics = validate("!!{count, plural, one {foo} other {bar}}!!");
    assert_eq!(diagnostics.len(), 1);
    assert_has_diagnostic!(diagnostics, (0, 42));
}

#[test]
fn inside_plural() {
    let diagnostics = validate("{count, plural, one {!!{user}!! hi} other {bar}}");
    assert_eq!(diagnostics.len(), 1);
    assert_has_diagnostic!(diagnostics, (21, 31));
}
