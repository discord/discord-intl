use crate::harness;
use intl_validator::validators::NoUnnecessaryPlural;
use intl_validator::{apply_fixes, DiagnosticName, ValueDiagnostic};

fn validate(content: &str) -> Vec<ValueDiagnostic> {
    let message = harness::define_single_message("MESSAGE_KEY", content);
    harness::validate_with(&message, NoUnnecessaryPlural::new("en-US".into())).unwrap_or(vec![])
}

macro_rules! assert_has_diagnostic {
    ($diagnostics:expr, $span:tt) => {{
        let diagnostics = &$diagnostics;
        let name = DiagnosticName::NoUnnecessaryPlural;
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
fn valid_with_no_other() {
    let message = "Hello {count, plural, one {# item}}";
    let diagnostics = validate(message);

    assert_eq!(diagnostics.len(), 0);
}

#[test]
fn only_other() {
    let message = "Hello {count, plural, other {item}} bar";
    let diagnostics = validate(message);

    assert_eq!(diagnostics.len(), 1);
    assert_has_diagnostic!(diagnostics, (6, 35));
    let fixed = apply_fixes(message, &diagnostics[0].fixes);
    assert_eq!("Hello item bar", fixed);
}

#[test]
fn other_with_pound() {
    let message = "Hello {count, plural, other { # item}}";
    let diagnostics = validate(message);

    assert_eq!(diagnostics.len(), 1);
    assert_has_diagnostic!(diagnostics, (6, 38));
    let fixed = apply_fixes(message, &diagnostics[0].fixes);
    assert_eq!("Hello {count, number} item", fixed);
}

#[test]
fn multiple_identical() {
    let message = "Hello {count, plural, =0 {# item} one {# item} other {# item}}";
    let diagnostics = validate(message);

    assert_eq!(diagnostics.len(), 1);
    assert_has_diagnostic!(diagnostics, (6, 62));
    println!("{:?}", diagnostics[0].fixes);
    let fixed = apply_fixes(message, &diagnostics[0].fixes);
    assert_eq!("Hello {count, number} item", fixed);
}
