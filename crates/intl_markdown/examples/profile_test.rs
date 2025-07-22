use intl_markdown::{format, parse_intl_message, MarkdownDocument};

fn main() {
    // Use only a few runs in debug mode to keep runs reasonably-lengthed.
    // In release mode, we want to exercise it more with longer runs.
    #[cfg(debug_assertions)]
    let max = 500;
    #[cfg(not(debug_assertions))]
    let max = 2000;

    for _ in 0..max {
        let MarkdownDocument { compiled, .. } =
            parse_intl_message(include_str!("../benches/spec.md"), true);
        let _output = format::to_html(&compiled);
    }
}
