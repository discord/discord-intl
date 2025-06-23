mod harness;

mod icu_formatjs_types {
    use crate::harness::ast_test;

    ast_test!(literal, "plaintext", r#"["plaintext"]"#);
    ast_test!(argument, "{username}", r#"[[1,"username"]]"#);
    ast_test!(number, "{count, number}", r#"[[2,"count"]]"#);
    ast_test!(
        number_style,
        "{count, number, sign-always currency/USD}",
        r#"[[2,"count","sign-always currency/USD"]]"#
    );
    ast_test!(
        number_style_shorthand,
        "{count, number, +! K currency/GBP }",
        r#"[[2,"count","+! K currency/GBP"]]"#
    );
    ast_test!(date, "{today, date}", r#"[[3,"today"]]"#);
    ast_test!(
        date_style,
        "{today, date, medium}",
        r#"[[3,"today","medium"]]"#
    );
    ast_test!(
        date_skeleton,
        "{today, date,  ::hhmsyG }",
        r#"[[3,"today","::hhmsyG"]]"#
    );
    ast_test!(time, "{rightNow, time}", r#"[[4,"rightNow"]]"#);
    ast_test!(
        time_style,
        "{rightNow, time, short}",
        r#"[[4,"rightNow","short"]]"#
    );
    ast_test!(
        time_skeleton,
        "{rightNow, time, ::GMDY}",
        r#"[[4,"rightNow","::GMDY"]]"#
    );
    ast_test!(
        plural,
        "{count, plural, one {#}}",
        r#"[[6,"count",{"one":[[7]]},0,"cardinal"]]"#
    );
    ast_test!(
        plural_exact,
        "{count, plural, =-1 {#} =5 {five}}",
        r#"[[6,"count",{"=-1":[[7]],"=5":["five"]},0,"cardinal"]]"#
    );
    ast_test!(
        plural_surrounded_pound,
        "{count, plural, one {put the # in the middle of an arm}}",
        r#"[[6,"count",{"one":["put the ",[7]," in the middle of an arm"]},0,"cardinal"]]"#
    );
    ast_test!(
        selectordinal,
        "{count, selectordinal, one {#}}",
        r#"[[6,"count",{"one":[[7]]},0,"ordinal"]]"#
    );
    ast_test!(
        select,
        "{color, select, orange {fluffy}}",
        r#"[[5,"color",{"orange":["fluffy"]}]]"#
    );
    ast_test!(keyword_as_name, "{time, number}", r#"[[2,"time"]]"#);

    ast_test!(
        unclosed_argument,
        "{username unclosed",
        r#"["{username unclosed"]"#
    );
}

mod icu_markdown_types {
    #[test]
    fn emphasis() {
        let input = "*hello*";
        let expected = r#"[[8,"$i",["hello"]]]"#;

        assert_eq!(expected, input);
    }
    #[test]
    fn strong() {
        let input = "**hello**";
        let expected = r#"[[8,"$b",["hello"]]]"#;

        assert_eq!(expected, input);
    }
    #[test]
    fn code_span() {
        let input = "`hello`";
        let expected = r#"[[8,"$code",["hello"]]]"#;

        assert_eq!(expected, input);
    }
    #[test]
    fn static_link() {
        let input = "[hello](./target)";
        let expected = r#"[[8,"$link",["hello"],["./target"]]]"#;

        assert_eq!(expected, input);
    }
    #[test]
    fn handler_link() {
        let input = "[hello](onClick)";
        let expected = r#"[[8,"$link",["hello"],[[1,"onClick"]]]]"#;

        assert_eq!(expected, input);
    }
    #[test]
    fn dynamic_link() {
        let input = "[hello]({target})";
        let expected = r#"[[8,"$link",["hello"],[[1,"target"]]]]"#;

        assert_eq!(expected, input);
    }
}
