use crate::spec::extensions::harness;

#[test]
fn basic_strikethrough() {
    let input = "~one tilde~";
    let expected = "<del>one tilde</del>";

    assert_eq!(expected, harness::parse_inline(input));
}
#[test]
fn double_strikethrough() {
    let input = "~~two tildes~~";
    let expected = "<del>two tildes</del>";

    assert_eq!(expected, harness::parse_inline(input));
}
#[test]
fn too_many() {
    let input = "~~~not strikethrough~~~";
    let expected = "~~~not strikethrough~~~";

    assert_eq!(expected, harness::parse_inline(input));
}
#[test]
fn intra_word() {
    let input = "intra~~word~~strike";
    let expected = "intra<del>word</del>strike";

    assert_eq!(expected, harness::parse_inline(input));
}
#[test]
fn matched_intra_word() {
    let input = "~~intra~~word~~strike~~";
    let expected = "<del>intra</del>word<del>strike</del>";

    assert_eq!(expected, harness::parse_inline(input));
}
#[test]
fn no_mixed() {
    let input = "~~no mixed~";
    let expected = "~~no mixed~";

    assert_eq!(expected, harness::parse_inline(input));
}
#[test]
fn no_partial_usage() {
    let input = "~~~can't use part of a run~~";
    let expected = "~~~can't use part of a run~~";

    assert_eq!(expected, harness::parse_inline(input));
}
#[test]
fn no_single_run() {
    let input = "~~~";
    let expected = "~~~";

    assert_eq!(expected, harness::parse_inline(input));
}
#[test]
fn first_wins() {
    let input = "~~first ~wins~~ easy~";
    let expected = "<del>first ~wins</del> easy~";

    assert_eq!(expected, harness::parse_inline(input));
}
#[test]
fn nesting() {
    let input = "~~nesting ~works~ with bounds~~";
    let expected = "<del>nesting <del>works</del> with bounds</del>";

    assert_eq!(expected, harness::parse_inline(input));
}
#[test]
fn no_direct_nesting() {
    let input = "~~~direct nesting ~doesn't work~~~";
    let expected = "~~~direct nesting ~doesn't work~~~";

    assert_eq!(expected, harness::parse_inline(input));
}
#[test]
fn no_boundary_crossing() {
    let input = "~~no *boundary~~ crossing*";
    let expected = "<del>no *boundary</del> crossing*";

    assert_eq!(expected, harness::parse_inline(input));
}
#[test]
fn no_reverse_boundary_crossing() {
    let input = "*no ~boundary* crossing~";
    let expected = "<em>no ~boundary</em> crossing~";

    assert_eq!(expected, harness::parse_inline(input));
}
#[test]
fn escaped() {
    let input = "~~this is \\~\\~escaped~~";
    let expected = "<del>this is ~~escaped</del>";

    assert_eq!(expected, harness::parse_inline(input));
}
#[test]
fn escaped_open() {
    let input = "\\~this is escaped~";
    let expected = "~this is escaped~";

    assert_eq!(expected, harness::parse_inline(input));
}
#[test]
fn escaped_close() {
    let input = "~this is escaped\\~";
    let expected = "~this is escaped~";

    assert_eq!(expected, harness::parse_inline(input));
}
#[test]
fn escaped_split() {
    let input = "~\\~this is escaped~~";
    let expected = "~~this is escaped~~";

    assert_eq!(expected, harness::parse_inline(input));
}
#[test]
fn escaped_leading() {
    let input = "\\~~this is escaped~~";
    let expected = "~~this is escaped~~";

    assert_eq!(expected, harness::parse_inline(input));
}
#[test]
fn escaped_trailing() {
    let input = "~~this is escaped~\\~";
    let expected = "~~this is escaped~~";

    assert_eq!(expected, harness::parse_inline(input));
}
#[test]
fn escaped_matches_single() {
    let input = "\\~~this is escaped~";
    let expected = "~<del>this is escaped</del>";

    assert_eq!(expected, harness::parse_inline(input));
}
#[test]
fn punctuation_flanking_double() {
    let input = "flanked punctuation~~!~~";
    let expected = "flanked punctuation<del>!</del>";

    assert_eq!(expected, harness::parse_inline(input));
}
#[test]
fn punctuation_flanking_single() {
    let input = "flanked punctuation single~!~";
    let expected = "flanked punctuation single~!~";

    assert_eq!(expected, harness::parse_inline(input));
}
