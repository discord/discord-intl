use crate::spec::extensions::harness;

#[test]
fn basic_hook() {
    let input = "$[text](someHook)";
    let expected = r#"<a href="someHook">text</a>"#;

    assert_eq!(expected, harness::parse_inline(input));
}
#[test]
fn empty_hook() {
    let input = "$[](someHook)";
    let expected = r#"<a href="someHook"></a>"#;

    assert_eq!(expected, harness::parse_inline(input));
}
#[test]
fn link_inside_hook() {
    let input = "$[text [link](./foo)](someHook)";
    let expected = r#"<a href="someHook">text <a href="./foo">link</a></a>"#;

    assert_eq!(expected, harness::parse_inline(input));
}
#[test]
fn hook_inside_link() {
    let input = "[link $[text](someHook)](./foo)";
    let expected = r#"<a href="./foo">link <a href="someHook">text</a></a>"#;

    assert_eq!(expected, harness::parse_inline(input));
}
#[test]
fn hook_inside_hook() {
    let input = "$[outer $[inner](hook1)](hook2)";
    let expected = r#"<a href="hook2">outer <a href="hook1">inner</a></a>"#;

    assert_eq!(expected, harness::parse_inline(input));
}
#[test]
fn disallow_dynamic_hook() {
    let input = "$[inner]({target})";
    let expected = r#"$[inner]({target})"#;

    assert_eq!(expected, harness::parse_inline(input));
}
#[test]
fn allow_dynamic_content() {
    let input = "$[{target}](someHook)";
    let expected = r#"<a href="someHook">{target}</a>"#;

    assert_eq!(expected, harness::parse_inline(input));
}
