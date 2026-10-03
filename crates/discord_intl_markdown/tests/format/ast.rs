mod icu {
    use crate::format::harness;

    #[test]
    fn literal() {
        let input = "plaintext";
        let expected = r#"["plaintext"]"#;

        harness::assert_ast(expected, &harness::parse_and_compile(input, false));
    }
    #[test]
    fn argument() {
        let input = "{username}";
        let expected = r#"[[1,"username"]]"#;

        harness::assert_ast(expected, &harness::parse_and_compile(input, false));
    }
    #[test]
    fn number() {
        let input = "{count, number}";
        let expected = r#"[[2,"count"]]"#;

        harness::assert_ast(expected, &harness::parse_and_compile(input, false));
    }
    #[test]
    fn number_style() {
        let input = "{count, number, sign-always currency/USD}";
        let expected = r#"[[2,"count","sign-always currency/USD"]]"#;

        harness::assert_ast(expected, &harness::parse_and_compile(input, false));
    }
    #[test]
    fn number_style_shorthand() {
        let input = "{count, number, +! K currency/GBP}";
        let expected = r#"[[2,"count","+! K currency/GBP"]]"#;

        harness::assert_ast(expected, &harness::parse_and_compile(input, false));
    }
    #[test]
    fn date() {
        let input = "{today, date}";
        let expected = r#"[[3,"today"]]"#;

        harness::assert_ast(expected, &harness::parse_and_compile(input, false));
    }
    #[test]
    fn date_style() {
        let input = "{today, date, medium}";
        let expected = r#"[[3,"today","medium"]]"#;

        harness::assert_ast(expected, &harness::parse_and_compile(input, false));
    }
    #[test]
    fn date_skeleton() {
        let input = "{today, date,  ::hhmsyG}";
        let expected = r#"[[3,"today","::hhmsyG"]]"#;

        harness::assert_ast(expected, &harness::parse_and_compile(input, false));
    }
    #[test]
    fn time() {
        let input = "{rightNow, time}";
        let expected = r#"[[4,"rightNow"]]"#;

        harness::assert_ast(expected, &harness::parse_and_compile(input, false));
    }
    #[test]
    fn time_style() {
        let input = "{rightNow, time, short}";
        let expected = r#"[[4,"rightNow","short"]]"#;

        harness::assert_ast(expected, &harness::parse_and_compile(input, false));
    }
    #[test]
    fn time_skeleton() {
        let input = "{rightNow, time,  ::GMDY}";
        let expected = r#"[[4,"rightNow","::GMDY"]]"#;

        harness::assert_ast(expected, &harness::parse_and_compile(input, false));
    }
    #[test]
    fn plural() {
        let input = "{count, plural, one {#}}";
        let expected = r#"[[6,"count",{"one":[[7]]},0,"cardinal"]]"#;

        harness::assert_ast(expected, &harness::parse_and_compile(input, false));
    }
    #[test]
    fn plural_exact() {
        let input = "{count, plural, =-1 {#} =5 {five}}";
        let expected = r#"[[6,"count",{"=-1":[[7]],"=5":["five"]},0,"cardinal"]]"#;

        harness::assert_ast(expected, &harness::parse_and_compile(input, false));
    }
    #[test]
    fn plural_surrounded_pound() {
        let input = "{count, plural, one {put the # in the middle of an arm}}";
        let expected =
            r#"[[6,"count",{"one":["put the ",[7]," in the middle of an arm"]},0,"cardinal"]]"#;

        harness::assert_ast(expected, &harness::parse_and_compile(input, false));
    }
    #[test]
    fn select() {
        let input = "{color, select, orange {fluffy}}";
        let expected = r#"[[5,"color",{"orange":["fluffy"]}]]"#;

        harness::assert_ast(expected, &harness::parse_and_compile(input, false));
    }
    #[test]
    fn select_ordinal() {
        let input = "{count, selectordinal, one {#}}";
        let expected = r#"[[6,"count",{"one":[[7]]},0,"ordinal"]]"#;

        harness::assert_ast(expected, &harness::parse_and_compile(input, false));
    }
    #[test]
    fn keyword_as_name() {
        let input = "{time, number}";
        let expected = r#"[[2,"time"]]"#;

        harness::assert_ast(expected, &harness::parse_and_compile(input, false));
    }
    #[test]
    fn unclosed_argument() {
        let input = "{username unclosed";
        let expected = r#"["{username unclosed"]"#;

        harness::assert_ast(expected, &harness::parse_and_compile(input, false));
    }
}

mod markdown {
    use crate::format::harness;

    #[test]
    fn paragraph() {
        let input = "*hello*";
        let expected = r#"[[8,"$p",[[8,"$i",["hello"]]]]]"#;

        harness::assert_ast(expected, &harness::parse_and_compile(input, true));
    }
    #[test]
    fn heading() {
        let input = "## hello";
        let expected = r#"[[8,"$h2",["hello"]]]"#;

        harness::assert_ast(expected, &harness::parse_and_compile(input, true));
    }

    #[test]
    fn emphasis() {
        let input = "*hello*";
        let expected = r#"[[8,"$i",["hello"]]]"#;

        harness::assert_ast(expected, &harness::parse_and_compile(input, false));
    }
    #[test]
    fn strong() {
        let input = "**hello**";
        let expected = r#"[[8,"$b",["hello"]]]"#;

        harness::assert_ast(expected, &harness::parse_and_compile(input, false));
    }
    #[test]
    fn code_span() {
        let input = "`hello`";
        let expected = r#"[[8,"$code",["hello"]]]"#;

        harness::assert_ast(expected, &harness::parse_and_compile(input, false));
    }
    #[test]
    fn static_link() {
        let input = "[hello](./target)";
        let expected = r#"[[8,"$link",["hello"],["./target"]]]"#;

        harness::assert_ast(expected, &harness::parse_and_compile(input, false));
    }
    #[test]
    fn handler_link() {
        let input = "[hello](onClick)";
        let expected = r#"[[8,"$link",["hello"],[[1,"onClick"]]]]"#;

        harness::assert_ast(expected, &harness::parse_and_compile(input, false));
    }
    #[test]
    fn dynamic_link() {
        let input = "[hello]({target})";
        let expected = r#"[[8,"$link",["hello"],[[1,"target"]]]]"#;

        harness::assert_ast(expected, &harness::parse_and_compile(input, false));
    }
    #[test]
    fn plural_link() {
        let input = "[hello]({target, plural, one {foo} other {bar}})";
        let expected = r#"[[8,"$link",["hello"],[[6,"target",{"one":["foo"],"other":["bar"]},0,"cardinal"]]]]"#;

        harness::assert_ast(expected, &harness::parse_and_compile(input, false));
    }

    #[test]
    fn ambiguous_image_link() {
        let input = "hello![foo](./bar)";
        let expected = r#"["hello!",[8,"$link",["foo"],["./bar"]]]"#;

        harness::assert_ast(expected, &harness::parse_and_compile(input, false));
    }

    #[test]
    fn no_content_link() {
        let input = "hello![](./bar)";
        let expected = r#"["hello!",[8,"$link",[],["./bar"]]]"#;

        harness::assert_ast(expected, &harness::parse_and_compile(input, false));
    }

    #[test]
    fn no_destination_link() {
        let input = "hello![foo]()";
        let expected = r#"["hello!",[8,"$link",["foo"],[""]]]"#;

        harness::assert_ast(expected, &harness::parse_and_compile(input, false));
    }

    #[test]
    fn incidental_image() {
        let input = "hello ![foo](./bar)";
        let expected = r#"["hello ",[8,"$link",["foo"],["./bar"]]]"#;

        harness::assert_ast(expected, &harness::parse_and_compile(input, false));
    }

    #[test]
    fn no_content_image() {
        let input = "hello ![](./bar)";
        let expected = r#"["hello ",[8,"$link",[],["./bar"]]]"#;

        harness::assert_ast(expected, &harness::parse_and_compile(input, false));
    }
}
