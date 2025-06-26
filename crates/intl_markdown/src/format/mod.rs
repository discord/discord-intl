mod html;
mod serialize;
mod util;

use crate::compiler::CompiledElement;
use crate::format::html::HtmlFormatter;

pub fn to_html(element: &CompiledElement) -> String {
    let mut formatter = HtmlFormatter::new();
    formatter.format_element(element);
    formatter.finish()
}
