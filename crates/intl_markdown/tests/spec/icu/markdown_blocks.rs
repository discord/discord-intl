use crate::spec::icu::harness;

#[test]
fn atx_heading() {
    let input = "# Heading\nwith a paragraph";
    let expected = "<h1>Heading</h1>\n<p>with a paragraph</p>";

    assert_eq!(expected, harness::parse_blocks(input));
}
#[test]
fn setext_heading() {
    let input = "Setext *Heading*\n---";
    let expected = "<h2>Setext <em>Heading</em></h2>";

    assert_eq!(expected, harness::parse_blocks(input));
}
#[test]
fn indented_code_block_ignores() {
    let input = "    {\n    novariable\n}";
    let expected = "<pre><code>{\nnovariable\n</code></pre>\n<p>}</p>";

    assert_eq!(expected, harness::parse_blocks(input));
}
#[test]
fn fenced_code_block_info_string_ignores() {
    let input = "``` {novariable}\n```";
    let expected = r#"<pre><code class="language-{novariable}"></code></pre>"#;

    assert_eq!(expected, harness::parse_blocks(input));
}
#[test]
fn fenced_code_block_content_ignores() {
    let input = "```\n{\nnovariable\n}\n```";
    let expected = "<pre><code>{\nnovariable\n}\n</code></pre>";

    assert_eq!(expected, harness::parse_blocks(input));
}
