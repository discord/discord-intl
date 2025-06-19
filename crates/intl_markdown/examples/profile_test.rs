use intl_markdown::{commonmark_html, parse_intl_message};

fn main() {
    let source = include_str!("../benches/spec.md");

    // Use only a few runs in debug mode to keep runs reasonably-lengthed.
    // In release mode, we want to exercise it more with longer runs.
    #[cfg(debug_assertions)]
    let max = 500;
    #[cfg(not(debug_assertions))]
    let max = 2000;

    for _ in 0..max {
        let ast = parse_intl_message(include_str!("../benches/spec.md"), true);
        let mut output = String::with_capacity(source.len());
        commonmark_html::format_document(&mut output, &ast).ok();
    }
}
