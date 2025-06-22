use crate::commonmark_html::formatter::{FormatResult, Formatter};
use crate::{Document, VisitWith};

mod format_element;
mod formatter;
mod html_format;
mod plain_text;
mod util;

pub fn format_document(output: &mut String, document: &Document) -> FormatResult {
    let mut formatter = Formatter::new();
    document.visit_with(&mut formatter);
    let elements = formatter.finish();
    html_format::format_elements(output, &elements);
    Ok(())
}
