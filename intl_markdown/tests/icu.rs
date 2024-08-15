use test_case::test_case;

use harness::run_icu_string_test;

mod harness;

#[test_case("{username}","{username}"; "basic_icu")]
#[test_case("{  username\n}","{username}"; "icu_whitespace")]
#[test_case("**hello**","<b>hello</b>"; "basic_markdown")]
#[test_case("***hello** this has more content*","<i><b>hello</b> this has more content</i>"; "nested_markdown")]
#[test_case("[a link](to/somewhere)","<link>to/somewhere{_}a link</link>"; "static_link")]
#[test_case("[a link]({variable})","<link>{variable}a link</link>"; "dynamic_link")]
#[test_case("<https://example.com>","<link>https://example.com{_}https://example.com</link>"; "autolink")]
#[test_case("# false heading","# false heading"; "no_block_atx_heading")]
#[test_case("false setext\n---","false setext\n---"; "no_block_setext_heading")]
fn icu_inline(input: &str, output: &str) {
    run_icu_string_test(input, output, false);
}

#[test_case("{today, date, short}","{today, date, short}"; "date_word_format")]
fn icu_variable_formats(input: &str, output: &str) {
    run_icu_string_test(input, output, false);
}

#[test_case("# Heading\nwith a paragraph","<h1>Heading</h1>\n<p>with a paragraph</p>"; "atx_heading")]
#[test_case("Setext *Heading*\n---","<h2>Setext <i>Heading</i></h2>"; "setext_heading")]
#[test_case("    {\n    novariable\n}","<codeBlock>{\nnovariable\n</codeBlock>\n<p>}</p>"; "indented_code_block_ignores")]
#[test_case("``` {novariable}\n```","<codeBlock></codeBlock>"; "fenced_code_block_info_string_ignores")]
#[test_case("```\n{\nnovariable\n}\n```","<codeBlock>{\nnovariable\n}\n</codeBlock>"; "fenced_code_block_content_ignores")]
fn icu_markdown_blocks(input: &str, output: &str) {
    run_icu_string_test(input, output, true);
}

#[test_case("{variable}\nlazy paragraph continuation","<p>{variable}\nlazy paragraph continuation</p>"; "beginning_of_paragraph")]
#[test_case("paragraph start\n{variable}","<p>paragraph start\n{variable}</p>"; "end_of_paragraph")]
#[test_case("paragraph one\n\n{variable} paragraph two","<p>paragraph one</p>\n<p>{variable} paragraph two</p>"; "blank_line_start")]
#[test_case("{variable}\n\nparagraph two","<p>{variable}</p>\n<p>paragraph two</p>"; "blank_line_end")]
#[test_case("paragraph one {\n\nvariable\n\n} still paragraph one","<p>paragraph one {variable} still paragraph one</p>"; "continuation_blank_line_inside_var")]
#[test_case("paragraph one {count,\n\n    plural,\n\n one {same par} other\n\n{same paragraph}} still paragraph one","<p>paragraph one {count, plural, one {same par} other {same paragraph}} still paragraph one</p>"; "continuation_blank_line_inside_plural_control")]
#[test_case("paragraph one {count, plural, one {\n\n# false heading\n\n ends with paragraph}} still paragraph one","<p>paragraph one {count, plural, one {# false heading\nends with paragraph}} still paragraph one</p>"; "continuation_blank_line_inside_plural_value")]
fn icu_blocks(input: &str, output: &str) {
    run_icu_string_test(input, output, true);
}

#[test_case("# Heading {variable}","<h1>Heading {variable}</h1>"; "same_line_icu_variable")]
#[test_case("# Heading {\n\nvariable}","<h1>Heading {variable}</h1>"; "multiline_icu_variable")]
#[test_case("# {\n\nvariable} and more","<h1>{variable} and more</h1>"; "content_after_icu")]
#[test_case("{\n\nvariable}\nsetext heading\n===","<h1>{variable}\nsetext heading</h1>"; "setext_heading")]
fn icu_in_headings(input: &str, output: &str) {
    run_icu_string_test(input, output, true);
}
