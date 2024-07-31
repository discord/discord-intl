use test_case::test_case;

mod harness;
use crate::harness::run_icu_ast_test;

#[test_case("plaintext","[{\"type\":0,\"value\":\"plaintext\"}]"; "literal")]
#[test_case("{username}","[{\"type\":1,\"value\":\"username\"}]"; "argument")]
#[test_case("{count, number}","[{\"type\":2,\"value\":\"count\"}]"; "number")]
#[test_case("{today, date}","[{\"type\":3,\"value\":\"today\"}]"; "date")]
#[test_case("{rightNow, time}","[{\"type\":4,\"value\":\"rightNow\"}]"; "time")]
#[test_case("{count, plural, one {#}}","[{\"type\":6,\"value\":\"count\",\"options\":{\"one\":{\"value\":[{\"type\":7}]}},\"offset\":0,\"pluralType\":\"cardinal\"}]"; "plural")]
#[test_case("{time, number}","[{\"type\":2,\"value\":\"time\"}]"; "keyword_as_name")]
fn icu_formatjs_types(input: &str, output: &str) {
    run_icu_ast_test(input, output, false);
}

#[test_case("*hello*","[{\"type\":8,\"value\":\"i\",\"children\":[{\"type\":0,\"value\":\"hello\"}]}]"; "emphasis")]
#[test_case("**hello**","[{\"type\":8,\"value\":\"b\",\"children\":[{\"type\":0,\"value\":\"hello\"}]}]"; "strong")]
#[test_case("`hello`","[{\"type\":8,\"value\":\"code\",\"children\":[{\"type\":0,\"value\":\"hello\"}]}]"; "code_span")]
#[test_case("[hello](target)","[{\"type\":8,\"value\":\"link\",\"children\":[{\"type\":0,\"value\":\"target\"},{\"type\":0,\"value\":\"hello\"}]}]"; "static_link")]
#[test_case("[hello]({target})","[{\"type\":8,\"value\":\"link\",\"children\":[{\"type\":1,\"value\":\"target\"},{\"type\":0,\"value\":\"hello\"}]}]"; "dynamic_link")]
fn icu_markdown_types(input: &str, output: &str) {
    run_icu_ast_test(input, output, false);
}

#[test_case("$[text](someHook)","[{\"type\":8,\"value\":\"someHook\",\"children\":[{\"type\":0,\"value\":\"text\"}]}]"; "basic_hook")]
#[test_case("$[](someHook)","[{\"type\":8,\"value\":\"someHook\",\"children\":[]}]"; "empty_hook")]
fn icu_md_extensions(input: &str, output: &str) {
    run_icu_ast_test(input, output, false);
}
