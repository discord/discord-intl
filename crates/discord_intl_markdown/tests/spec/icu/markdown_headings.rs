use crate::spec::icu::harness;

#[test]
fn same_line_icu_variable() {
    let input = "# Heading {variable}";
    let expected = "<h1>Heading {variable}</h1>";

    assert_eq!(expected, harness::parse_blocks(input));
}
#[test]
fn multiline_icu_variable() {
    let input = "# Heading {\n\nvariable}";
    let expected = "<h1>Heading {variable}</h1>";

    assert_eq!(expected, harness::parse_blocks(input));
}
#[test]
fn content_after_icu() {
    let input = "# {\n\nvariable} and more";
    let expected = "<h1>{variable} and more</h1>";

    assert_eq!(expected, harness::parse_blocks(input));
}
#[test]
fn setext_heading() {
    let input = "{\n\nvariable}\nsetext heading\n===";
    let expected = "<h1>{variable}\nsetext heading</h1>";

    assert_eq!(expected, harness::parse_blocks(input));
}
