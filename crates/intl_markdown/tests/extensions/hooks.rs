mod hooks {
    use crate::harness::ast_test;
    ast_test!(
        basic_hook,
        "$[text](someHook)",
        r#"[[8,"someHook",["text"]]]"#
    );
    ast_test!(empty_hook, "$[](someHook)", r#"[[8,"someHook",[]]]"#);
    ast_test!(
        link_inside_hook,
        "$[text [link](./foo)](someHook)",
        r#"[[8,"someHook",["text ",[8,"$link",["link"],["./foo"]]]]]"#
    );
    ast_test!(
        hook_inside_link,
        "[link $[text](someHook)](./foo)",
        r#"[[8,"$link",["link ",[8,"someHook",["text"]]],["./foo"]]]"#
    );
    ast_test!(
        hook_inside_hook,
        "$[outer $[inner](hook1)](hook2)",
        r#"[[8,"hook2",["outer ",[8,"hook1",["inner"]]]]]"#
    );
    ast_test!(
        disallow_dynamic_hook,
        "$[inner]({target})",
        r#"["$[inner](",[1,"target"],")"]"#
    );
    ast_test!(
        allow_dynamic_content,
        "$[{target}](someHook)",
        r#"[[8,"someHook",[[1,"target"]]]]"#
    );
}
