//! Tests for Markdown syntax extensions, specifically hooks (`$[]()`), unsafe variables (`!!{}!!`),
//! and strikethroughs (a la GFM, `~~deleted~~`).

mod harness;

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

mod strikethrough {
    use crate::harness::icu_string_test;
    icu_string_test!(basic_strikethrough, "~one tilde~", "<del>one tilde</del>");
    icu_string_test!(
        double_strikethrough,
        "~~two tildes~~",
        "<del>two tildes</del>"
    );
    icu_string_test!(
        too_many,
        "~~~not strikethrough~~~",
        "~~~not strikethrough~~~"
    );
    icu_string_test!(
        intra_word,
        "intra~~word~~strike",
        "intra<del>word</del>strike"
    );
    icu_string_test!(
        matched_intra_word,
        "~~intra~~word~~strike~~",
        "<del>intra</del>word<del>strike</del>"
    );
    icu_string_test!(no_mixed, "~~no mixed~", "~~no mixed~");
    icu_string_test!(
        no_partial_usage,
        "~~~can't use part of a run~~",
        "~~~can't use part of a run~~"
    );
    icu_string_test!(no_single_run, "~~~", "~~~");
    icu_string_test!(
        first_wins,
        "~~first ~wins~~ easy~",
        "<del>first ~wins</del> easy~"
    );
    icu_string_test!(
        nesting,
        "~~nesting ~works~ with bounds~~",
        "<del>nesting <del>works</del> with bounds</del>"
    );
    icu_string_test!(
        no_direct_nesting,
        "~~~direct nesting ~doesn't work~~~",
        "~~~direct nesting ~doesn't work~~~"
    );
    icu_string_test!(
        no_boundary_crossing,
        "~~no *boundary~~ crossing*",
        "<del>no *boundary</del> crossing*"
    );
    icu_string_test!(
        no_reverse_boundary_crossing,
        "*no ~boundary* crossing~",
        "<i>no ~boundary</i> crossing~"
    );
    icu_string_test!(
        escaped,
        "~~this is \\~\\~escaped~~",
        "<del>this is ~~escaped</del>"
    );
    icu_string_test!(escaped_open, "\\~this is escaped~", "~this is escaped~");
    icu_string_test!(escaped_close, "~this is escaped\\~", "~this is escaped~");
    icu_string_test!(
        escaped_split,
        "~\\~this is escaped~~",
        "~~this is escaped~~"
    );
    icu_string_test!(
        escaped_leading,
        "\\~~this is escaped~~",
        "~~this is escaped~~"
    );
    icu_string_test!(
        escaped_trailing,
        "~~this is escaped~\\~",
        "~~this is escaped~~"
    );
    icu_string_test!(
        escaped_matches_single,
        "\\~~this is escaped~",
        "~<del>this is escaped</del>"
    );
    icu_string_test!(
        punctuation_flanking_double,
        "flanked punctuation~~!~~",
        "flanked punctuation<del>!</del>"
    );
    icu_string_test!(
        punctuation_flanking_single,
        "flanked punctuation single~!~",
        "flanked punctuation single~!~"
    );
}
