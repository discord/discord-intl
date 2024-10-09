use std::cell::RefCell;
use std::rc::Rc;

use intl_database_core::KeySymbol;

use crate::DiagnosticSeverity;

pub struct MessageDiagnostic {
    pub key: KeySymbol,
    pub file_key: KeySymbol,
    pub locale: KeySymbol,
    pub severity: DiagnosticSeverity,
    pub description: String,
    pub help: Option<String>,
}

#[derive(Default)]
pub struct DiagnosticBuilder {
    pub diagnostics: Vec<MessageDiagnostic>,
    pub key: KeySymbol,
    pub file: KeySymbol,
    pub locale: KeySymbol,
}

pub type Diagnostics = Rc<RefCell<DiagnosticBuilder>>;

impl DiagnosticBuilder {
    pub fn new(key: KeySymbol, file: KeySymbol, locale: KeySymbol) -> Self {
        Self {
            diagnostics: Vec::with_capacity(4),
            key,
            file,
            locale,
        }
    }

    pub fn create(
        &mut self,
        severity: DiagnosticSeverity,
        description: impl ToString,
        help: Option<String>,
    ) {
        self.diagnostics.push(MessageDiagnostic {
            key: self.key,
            file_key: self.file,
            locale: self.locale,
            severity,
            description: description.to_string(),
            help,
        });
    }
}
