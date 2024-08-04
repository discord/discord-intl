use test_case::test_case;

use crate::harness::run_icu_ast_test;

mod harness;
#[test_case("plaintext",r#"[{"type":0,"value":"plaintext"}]"#; "literal")]
#[test_case("{username}",r#"[{"type":1,"value":"username"}]"#; "argument")]
#[test_case("{count, number}",r#"[{"type":2,"value":"count"}]"#; "number")]
#[test_case("{count, number, sign-always currency/USD}",r#"[{"type":2,"value":"count","style":"sign-always currency/USD"}]"#; "number_style")]
#[test_case("{count, number, +! K currency/GBP }",r#"[{"type":2,"value":"count","style":"+! K currency/GBP"}]"#; "number_style_shorthand")]
#[test_case("{today, date}",r#"[{"type":3,"value":"today"}]"#; "date")]
#[test_case("{today, date, medium}",r#"[{"type":3,"value":"today","style":"medium"}]"#; "date_style")]
#[test_case("{today, date,  ::hhmsyG }",r#"[{"type":3,"value":"today","style":"::hhmsyG"}]"#; "date_skeleton")]
#[test_case("{rightNow, time}",r#"[{"type":4,"value":"rightNow"}]"#; "time")]
#[test_case("{rightNow, time, short}",r#"[{"type":4,"value":"rightNow","style":"short"}]"#; "time_style")]
#[test_case("{rightNow, time, ::GMDY}",r#"[{"type":4,"value":"rightNow","style":"::GMDY"}]"#; "time_skeleton")]
#[test_case("{count, plural, one {#}}",r#"[{"type":6,"value":"count","options":{"one":{"value":[{"type":7}]}},"offset":0,"pluralType":"cardinal"}]"#; "plural")]
#[test_case("{time, number}",r#"[{"type":2,"value":"time"}]"#; "keyword_as_name")]
fn icu_formatjs_types(input: &str, output: &str) {
    run_icu_ast_test(input, output, false);
}

#[test_case("*hello*",r#"[{"type":8,"value":"i","children":[{"type":0,"value":"hello"}]}]"#; "emphasis")]
#[test_case("**hello**",r#"[{"type":8,"value":"b","children":[{"type":0,"value":"hello"}]}]"#; "strong")]
#[test_case("`hello`",r#"[{"type":8,"value":"code","children":[{"type":0,"value":"hello"}]}]"#; "code_span")]
#[test_case("[hello](target)",r#"[{"type":8,"value":"link","children":[{"type":0,"value":"target"},{"type":0,"value":"hello"}]}]"#; "static_link")]
#[test_case("[hello]({target})",r#"[{"type":8,"value":"link","children":[{"type":1,"value":"target"},{"type":0,"value":"hello"}]}]"#; "dynamic_link")]
fn icu_markdown_types(input: &str, output: &str) {
    run_icu_ast_test(input, output, false);
}

#[test_case("$[text](someHook)",r#"[{"type":8,"value":"someHook","children":[{"type":0,"value":"text"}]}]"#; "basic_hook")]
#[test_case("$[](someHook)",r#"[{"type":8,"value":"someHook","children":[]}]"#; "empty_hook")]
fn icu_md_extensions(input: &str, output: &str) {
    run_icu_ast_test(input, output, false);
}
