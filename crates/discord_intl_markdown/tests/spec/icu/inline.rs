use crate::spec::icu::harness;

#[test]
fn basic_icu() {
    let input = "{username}";
    let expected = "{username}";

    assert_eq!(expected, harness::parse_inline(input));
}
#[test]
fn icu_whitespace() {
    let input = "{  username\n}";
    let expected = "{username}";

    assert_eq!(expected, harness::parse_inline(input));
}
#[test]
fn basic_markdown() {
    let input = "**hello**";
    let expected = "<strong>hello</strong>";

    assert_eq!(expected, harness::parse_inline(input));
}
#[test]
fn nested_markdown() {
    let input = "***hello** this has more content*";
    let expected = "<em><strong>hello</strong> this has more content</em>";

    assert_eq!(expected, harness::parse_inline(input));
}
#[test]
fn static_link() {
    let input = "[a link](to/somewhere)";
    let expected = r#"<a href="to/somewhere">a link</a>"#;

    assert_eq!(expected, harness::parse_inline(input));
}
#[test]
fn dynamic_link() {
    let input = "[a link]({variable})";
    let expected = r#"<a href="{variable}">a link</a>"#;

    assert_eq!(expected, harness::parse_inline(input));
}
#[test]
fn dynamic_link_with_spaces() {
    let input = "[a link]({  variable  })";
    let expected = r#"<a href="{variable}">a link</a>"#;

    assert_eq!(expected, harness::parse_inline(input));
}
#[test]
fn unclosed_icu() {
    let input = "{username unclosed";
    let expected = "{username unclosed";

    assert_eq!(expected, harness::parse_inline(input));
}
#[test]
fn unclosed_icu_with_spaces() {
    let input = "{  username unclosed";
    let expected = "{  username unclosed";

    assert_eq!(expected, harness::parse_inline(input));
}

#[test]
fn unclosed_icu_with_newlines() {
    let input = "{ \n username unclosed";
    let expected = "{\nusername unclosed";

    assert_eq!(expected, harness::parse_inline(input));
}
