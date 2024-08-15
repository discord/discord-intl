//! Tests for Markdown syntax extensions, specifically hooks (`$[]()`), unsafe variables (`!!{}!!`),
//! and strikethroughs (a la GFM, `~~deleted~~`).

use test_case::test_case;

use harness::run_icu_string_test;

use crate::harness::run_icu_ast_test;

mod harness;

#[test_case("$[text](someHook)","<someHook>text</someHook>"; "basic_hook")]
#[test_case("$[](someHook)","<someHook></someHook>"; "empty_hook")]
#[test_case("$[text [link](foo)](someHook)","<someHook>text <link>foo{_}link</link></someHook>"; "link_inside_hook")]
#[test_case("[link $[text](someHook)](foo)","<link>foo{_}link <someHook>text</someHook></link>"; "hook_inside_link")]
#[test_case("$[outer $[inner](hook1)](hook2)","<hook2>outer <hook1>inner</hook1></hook2>"; "hook_inside_hook")]
#[test_case("$[inner]({target})","$[inner]({target})"; "disallow_dynamic_hook")]
#[test_case("$[{target}](someHook)","<someHook>{target}</someHook>"; "allow_dynamic_content")]
fn hooks(input: &str, output: &str) {
    run_icu_string_test(input, output, false);
}

#[test_case("!!{username}!!","{username}"; "basic_unsafe")]
#[test_case("**!!{username}!!**","<b>{username}</b>"; "wrapped_unsafe")]
#[test_case("{count, plural, one {hi !!{username}!!}}","{count, plural, one {hi {username}}}"; "nested_unsafe")]
fn unsafe_variables_strings(input: &str, output: &str) {
    run_icu_string_test(input, output, false);
}

#[test_case("!!{username}!!","[{\"type\":1,\"value\":\"username\"}]"; "basic_unsafe")]
fn unsafe_variables_ast(input: &str, output: &str) {
    run_icu_ast_test(input, output, false);
}

#[test_case("~one tilde~","<del>one tilde</del>"; "basic_strikethrough")]
#[test_case("~~two tildes~~","<del>two tildes</del>"; "double_strikethrough")]
#[test_case("~~~not strikethrough~~~","~~~not strikethrough~~~"; "too_many")]
#[test_case("intra~~word~~strike","intra<del>word</del>strike"; "intra_word")]
#[test_case("~~intra~~word~~strike~~","<del>intra</del>word<del>strike</del>"; "matched_intra_word")]
#[test_case("~~no mixed~","~~no mixed~"; "no_mixed")]
#[test_case("~~~can't use part of a run~~","~~~can't use part of a run~~"; "no_partial_usage")]
#[test_case("~~~","~~~"; "no_single_run")]
#[test_case("~~first ~wins~~ easy~","<del>first ~wins</del> easy~"; "first_wins")]
#[test_case("~~nesting ~works~ with bounds~~","<del>nesting <del>works</del> with bounds</del>"; "nesting")]
#[test_case("~~~direct nesting ~doesn't work~~~","~~~direct nesting ~doesn't work~~~"; "no_direct_nesting")]
#[test_case("~~no *boundary~~ crossing*","<del>no *boundary</del> crossing*"; "no_boundary_crossing")]
#[test_case("*no ~boundary* crossing~","<i>no ~boundary</i> crossing~"; "no_reverse_boundary_crossing")]
#[test_case("~~this is \\~\\~escaped~~","<del>this is ~~escaped</del>"; "escaped")]
#[test_case("\\~this is escaped~","~this is escaped~"; "escaped_open")]
#[test_case("~this is escaped\\~","~this is escaped~"; "escaped_close")]
#[test_case("~\\~this is escaped~~","~~this is escaped~~"; "escaped_split")]
#[test_case("\\~~this is escaped~~","~~this is escaped~~"; "escaped_leading")]
#[test_case("~~this is escaped~\\~","~~this is escaped~~"; "escaped_trailing")]
#[test_case("\\~~this is escaped~","~<del>this is escaped</del>"; "escaped_matches_single")]
#[test_case("flanked punctuation~~!~~","flanked punctuation<del>!</del>"; "punctuation_flanking_double")]
#[test_case("flanked punctuation single~!~","flanked punctuation single~!~"; "punctuation_flanking_single")]
fn strikethrough(input: &str, output: &str) {
    run_icu_string_test(input, output, false);
}
