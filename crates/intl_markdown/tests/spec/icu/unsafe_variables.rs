use crate::spec::icu::harness;

#[test]
fn basic_unsafe() {
    let input = "!!{username}!!";
    let expected = "{username}";
    assert_eq!(expected, harness::parse_inline(input));
}

#[test]
fn basic_unsafe_2() {
    let input = "hello !!{username}!!. again!";
    let expected = "hello {username}. again!";

    assert_eq!(expected, harness::parse_inline(input));
}

#[test]
fn nested_unsafe() {
    let input = "{count, plural, one {hi !!{username}!!}}";
    let expected = "{count, plural, one {hi {username}}}";

    assert_eq!(expected, harness::parse_inline(input));
}

#[test]
fn wrapped_unsafe() {
    let input = "**!!{username}!!**";
    let expected = "<strong>{username}</strong>";

    assert_eq!(expected, harness::parse_inline(input));
}
