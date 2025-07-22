use crate::spec::icu::harness;

#[test]
fn beginning_of_paragraph() {
    let input = "{variable}\nlazy paragraph continuation";
    let expected = "<p>{variable}\nlazy paragraph continuation</p>";
    assert_eq!(expected, harness::parse_blocks(input));
}
#[test]
fn end_of_paragraph() {
    let input = "paragraph start\n{variable}";
    let expected = "<p>paragraph start\n{variable}</p>";

    assert_eq!(expected, harness::parse_blocks(input));
}
#[test]
fn blank_line_start() {
    let input = "paragraph one\n\n{variable} paragraph two";
    let expected = "<p>paragraph one</p>\n<p>{variable} paragraph two</p>";

    assert_eq!(expected, harness::parse_blocks(input));
}
#[test]
fn blank_line_end() {
    let input = "{variable}\n\nparagraph two";
    let expected = "<p>{variable}</p>\n<p>paragraph two</p>";

    assert_eq!(expected, harness::parse_blocks(input));
}
#[test]
fn continuation_blank_line_inside_var() {
    let input = "paragraph one {\n\nvariable\n\n} still paragraph one";
    let expected = "<p>paragraph one {variable} still paragraph one</p>";

    assert_eq!(expected, harness::parse_blocks(input));
}
#[test]
fn continuation_blank_line_inside_plural_control() {
    let input = "paragraph one {count,\n\n    plural,\n\n one {same par} other\n\n{same paragraph}} still paragraph one";
    let expected = "<p>paragraph one {count, plural, one {same par} other {same paragraph}} still paragraph one</p>";

    assert_eq!(expected, harness::parse_blocks(input));
}
#[test]
fn continuation_blank_line_inside_plural_value() {
    let input = "paragraph one {count, plural, one {\n\n# false heading\n\n ends with paragraph}} still paragraph one";
    let expected = "<p>paragraph one {count, plural, one {\n\n# false heading\n\nends with paragraph}} still paragraph one</p>";

    assert_eq!(expected, harness::parse_blocks(input));
}
#[test]
fn continuation_blank_line_inside_select_value() {
    let input = "this cat is {color, select, orange {orange flavored\n\n} black {void}}";
    let expected = "<p>this cat is {color, select, orange {orange flavored\n\n} black {void}}</p>";

    assert_eq!(expected, harness::parse_blocks(input));
}
