use crate::spec::extensions::harness;

#[test]
fn basic_hook() {
    let input = "$[text](someHook)";
    let expected = r#"[[8,"someHook",["text"]]]"#;

    assert_eq!(expected, harness::parse_inline(input));
}
#[test]
fn empty_hook() {
    let input = "$[](someHook)";
    let expected = r#"[[8,"someHook",[]]]"#;

    assert_eq!(expected, harness::parse_inline(input));
}
#[test]
fn link_inside_hook() {
    let input = "$[text [link](./foo)](someHook)";
    let expected = r#"[[8,"someHook",["text ",[8,"$link",["link"],["./foo"]]]]]"#;

    assert_eq!(expected, harness::parse_inline(input));
}
#[test]
fn hook_inside_link() {
    let input = "[link $[text](someHook)](./foo)";
    let expected = r#"[[8,"$link",["link ",[8,"someHook",["text"]]],["./foo"]]]"#;

    assert_eq!(expected, harness::parse_inline(input));
}
#[test]
fn hook_inside_hook() {
    let input = "$[outer $[inner](hook1)](hook2)";
    let expected = r#"[[8,"hook2",["outer ",[8,"hook1",["inner"]]]]]"#;

    assert_eq!(expected, harness::parse_inline(input));
}
#[test]
fn disallow_dynamic_hook() {
    let input = "$[inner]({target})";
    let expected = r#"["$[inner](",[1,"target"],")"]"#;

    assert_eq!(expected, harness::parse_inline(input));
}
#[test]
fn allow_dynamic_content() {
    let input = "$[{target}](someHook)";
    let expected = r#"[[8,"someHook",[[1,"target"]]]]"#;

    assert_eq!(expected, harness::parse_inline(input));
}
