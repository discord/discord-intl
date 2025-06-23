use crate::{Document, VisitWith};
use format_element::FormatElement;
use formats::{html, keyless_json};
use formatter::Formatter;

mod format_element;
mod formats;
mod formatter;
mod plain_text;
mod util;

pub type FormattedDocument = Vec<FormatElement>;

pub fn format_document(document: &Document) -> FormattedDocument {
    let mut formatter = Formatter::new();
    document.visit_with(&mut formatter);
    formatter.finish()
}

pub fn to_html(document: &Document) -> String {
    let formatted = format_document(document);
    let mut output = String::new();
    html::format_elements(&mut output, &formatted);
    output
}

pub fn to_keyless_json(document: &Document) -> String {
    let formatted = format_document(document);
    let mut output = String::new();
    keyless_json::format_elements(&mut output, &formatted);
    output
}
