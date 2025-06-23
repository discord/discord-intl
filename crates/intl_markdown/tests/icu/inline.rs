use crate::icu::harness;

#[test]
fn basic_icu() {
    let input = "{username}";
    let expected = "{username}";

    assert_eq!(expected, harness::parse(input));
}
#[test]
fn icu_whitespace() {
    let input = "{  username\n}";
    let expected = "{username}";

    assert_eq!(expected, harness::parse(input));
}
#[test]
fn basic_markdown() {
    let input = "**hello**";
    let expected = r#"[[8,"$b","hello"]]"#;

    assert_eq!(expected, harness::parse(input));
}
#[test]
fn nested_markdown() {
    let input = "***hello** this has more content*";
    let expected = r#"[[8,"$i",[[8,"$b",["hello"]]," this has more content"]]]"#;

    assert_eq!(expected, harness::parse(input));
}
#[test]
fn static_link() {
    let input = "[a link](to/somewhere)";
    let expected = "<link>to/somewhere{_}a link</link>";

    assert_eq!(expected, harness::parse(input));
}
#[test]
fn dynamic_link() {
    let input = "[a link]({variable})";
    let expected = "<link>{variable}a link</link>";

    assert_eq!(expected, harness::parse(input));
}
#[test]
fn autolink() {
    let input = "<https://example.com>";
    let expected = "<link>https://example.com{_}https://example.com</link>";

    assert_eq!(expected, harness::parse(input));
}
#[test]
fn no_block_atx_heading() {
    let input = "# false heading";
    let expected = "# false heading";

    assert_eq!(expected, harness::parse(input));
}
#[test]
fn no_block_setext_heading() {
    let input = "false setext\n---";
    let expected = "false setext\n---";

    assert_eq!(expected, harness::parse(input));
}
#[test]
fn unclosed_icu() {
    let input = "{username unclosed";
    let expected = "{username unclosed";

    assert_eq!(expected, harness::parse(input));
}
