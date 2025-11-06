use crate::harness;
use intl_validator::validators::NoMissingPluralOther;
use intl_validator::{DiagnosticName, ValueDiagnostic};

fn validate(content: &str) -> Vec<ValueDiagnostic> {
    let message = harness::define_single_message("MESSAGE_KEY", content);
    harness::validate_with(&message, NoMissingPluralOther::new("en-US".into())).unwrap_or(vec![])
}

macro_rules! assert_has_diagnostic {
    ($diagnostics:expr, $span:tt) => {{
        let diagnostics = &$diagnostics;
        let name = DiagnosticName::NoMissingPluralOther;
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
fn missing_other() {
    let diagnostics = validate("{count, plural, one {{count} item}}");

    assert_eq!(diagnostics.len(), 1);
    assert_has_diagnostic!(diagnostics, (1, 34));
}

#[test]
fn nested_missing_other_inner() {
    let diagnostics =
        validate("{count, plural, one {{count2, plural, one {foo}} item} other {# items}}");

    assert_eq!(diagnostics.len(), 1);
    assert_has_diagnostic!(diagnostics, (22, 47));
}

#[test]
fn nested_missing_other_outer() {
    let diagnostics =
        validate("{count, plural, one {{count2, plural, one {foo} other {bar} item}}}");

    assert_eq!(diagnostics.len(), 1);
    assert_has_diagnostic!(diagnostics, (1, 65));
}
