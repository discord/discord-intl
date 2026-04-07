use crate::harness;
use intl_database_core::{key_symbol, Message, MessageMeta};
use intl_validator::{validate_message, DiagnosticName, MessageDiagnostic};

fn build_message(source_content: &str, translation_content: &str) -> Message {
    let key = "MESSAGE_KEY";
    let source = harness::define_single_message(key, source_content);
    let translation = harness::define_single_message(key, translation_content);

    let mut message = Message::from_definition(
        key_symbol(key),
        source,
        key_symbol("en-US"),
        MessageMeta::default(),
    );
    message.set_translation(key_symbol("fr"), translation);
    message
}

fn validate(source_content: &str, translation_content: &str) -> Vec<MessageDiagnostic> {
    let message = build_message(source_content, translation_content);
    validate_message(&message)
        .into_iter()
        .filter(|d| d.name == DiagnosticName::NoVariableTypeMismatches)
        .collect()
}

#[test]
fn safe_new_source_and_old_translation_same_type_variables() {
    assert_eq!(validate("{user}", "{user}").len(), 0);
    assert_eq!(validate("hello {user}", "bonjour {user}").len(), 0);
    assert_eq!(validate("{count, number}", "{count, number}").len(), 0);
    assert_eq!(validate("{today, date}", "{today, date}").len(), 0);
    assert_eq!(
        validate(
            "{count, plural, one {# item} other {# items}}",
            "{count, plural, one {# item} other {# items}}",
        )
        .len(),
        0
    );
    assert_eq!(
        validate(
            "{gender, select, male {He} female {She} other {They}}",
            "{gender, select, male {He} female {She} other {They}}",
        )
        .len(),
        0
    );
    assert_eq!(
        validate("$[click here](myVar)", "$[cliquez ici](myVar)").len(),
        0
    );
    assert_eq!(
        validate("[click here](myVar)", "[cliquez ici](myVar)").len(),
        0
    );
}

#[test]
fn safe_new_source_specialized_and_old_translation_any_type_variables() {
    assert_eq!(validate("{count, number}", "{count}").len(), 0);
    assert_eq!(validate("{today, date}", "{today}").len(), 0);
    assert_eq!(validate("{now, time}", "{now}").len(), 0);
    assert_eq!(
        validate("{count, plural, one {# item} other {# items}}", "{count}").len(),
        0
    );
    assert_eq!(
        validate(
            "{gender, select, male {He} female {She} other {They}}",
            "{gender}"
        )
        .len(),
        0
    );
}

#[test]
fn unsafe_new_source_any_and_old_translation_number_variable() {
    let diagnostics = validate("{count}", "{count, number}");
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(
        diagnostics[0].name,
        DiagnosticName::NoVariableTypeMismatches
    );
}

#[test]
fn unsafe_new_source_any_and_old_translation_date_variable() {
    let diagnostics = validate("{today}", "{today, date}");
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(
        diagnostics[0].name,
        DiagnosticName::NoVariableTypeMismatches
    );
}

#[test]
fn unsafe_new_source_any_and_old_translation_time_variable() {
    let diagnostics = validate("{now}", "{now, time}");
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(
        diagnostics[0].name,
        DiagnosticName::NoVariableTypeMismatches
    );
}

#[test]
fn unsafe_new_source_number_and_old_translation_date_variable() {
    let diagnostics = validate("{val, number}", "{val, date}");
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(
        diagnostics[0].name,
        DiagnosticName::NoVariableTypeMismatches
    );
}

#[test]
fn safe_new_source_function_and_old_translation_any_variable() {
    assert_eq!(validate("$[text](myVar)", "{myVar}").len(), 0);
    assert_eq!(validate("[text](myVar)", "{myVar}").len(), 0);
}

#[test]
fn unsafe_new_source_any_and_old_translation_hook_variable() {
    let diagnostics = validate("{var1}", "$[text](var1)");
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(
        diagnostics[0].name,
        DiagnosticName::NoVariableTypeMismatches
    );
}

#[test]
fn unsafe_new_source_any_and_old_translation_handler_variable() {
    let diagnostics = validate("{var1}", "[link](var1)");
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(
        diagnostics[0].name,
        DiagnosticName::NoVariableTypeMismatches
    );
}

#[test]
fn unsafe_new_source_hook_and_old_translation_handler_variable() {
    let diagnostics = validate("$[click here](myVar)", "[click here](myVar)");
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(
        diagnostics[0].name,
        DiagnosticName::NoVariableTypeMismatches
    );
}

#[test]
fn unsafe_new_source_handler_and_old_translation_hook_variable() {
    let diagnostics = validate("[click here](myVar)", "$[click here](myVar)");
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(
        diagnostics[0].name,
        DiagnosticName::NoVariableTypeMismatches
    );
}
