mod unsafe_variable_strings {
    use crate::harness::ast_test;
    ast_test!(basic_unsafe, "!!{username}!!", r#"[[1,"username"]]"#);
    ast_test!(
        nested_unsafe,
        "{count, plural, one {hi !!{username}!!}}",
        r#"[[6,"count",{"one":["hi ",[1,"username"]]},0,"cardinal"]]"#
    );
    ast_test!(
        wrapped_unsafe,
        "**!!{username}!!**",
        r#"[[8,"$b",[[1,"username"]]]]"#
    );
}
