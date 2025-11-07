use crate::harness;
use intl_validator::validators::NoNonExhaustivePlurals;
use intl_validator::{apply_fixes, DiagnosticName, ValueDiagnostic};

fn validate(content: &str) -> Vec<ValueDiagnostic> {
    let message = harness::define_single_message("MESSAGE_KEY", content);
    harness::validate_with(&message, NoNonExhaustivePlurals::new("en-US".into())).unwrap_or(vec![])
}

macro_rules! assert_has_diagnostic {
    ($diagnostics:expr, $span:tt) => {{
        let diagnostics = &$diagnostics;
        let name = DiagnosticName::NoNonExhaustivePlurals;
        let span = $span;
        assert_eq!(
            harness::has_matching_diagnostic(diagnostics, name, span),
            true,
            "Did not find matching diagnostic {name}({span:?}) in {diagnostics:#?}"
        );
    }};
}

#[test]
fn exact_and_other() {
    assert_eq!(
        validate("{count, plural, =1 {# item} other {# items}}").len(),
        0
    );
    assert_eq!(
        validate("{count, plural, =5 {# item} other {# items}}").len(),
        0
    );
}

#[test]
fn multiple_exact_and_other() {
    // This case would convert `=1` to `one` from the NoAvoidableExactPlurals rule, but this
    // rule does not flag it.
    assert_eq!(
        validate("{count, plural, =0 {# items} =1 {# item} other {# items}}").len(),
        0
    );
}

#[test]
fn only_one_exact_selector() {
    let message = "{count, plural, =1 {why plural}}";
    let diagnostics = validate(message);

    assert_eq!(diagnostics.len(), 1);
    assert_has_diagnostic!(diagnostics, (1, 31));
    let fixed = apply_fixes(message, &diagnostics[0].fixes);
    assert_eq!(fixed, "{count, select, 1 {why plural}}");
}

#[test]
fn multiple_exact_selectors() {
    let message = "{count, plural, =1 {why plural} =2 {this} =3 {is a} =4 {select}}";
    let diagnostics = validate(message);

    assert_eq!(diagnostics.len(), 1);
    assert_has_diagnostic!(diagnostics, (1, 63));
    println!("{:#?}", diagnostics[0]);
    let fixed = apply_fixes(message, &diagnostics[0].fixes);
    assert_eq!(
        fixed,
        "{count, select, 1 {why plural} 2 {this} 3 {is a} 4 {select}}"
    );
}
