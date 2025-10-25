use crate::spec::icu::harness;
use intl_markdown::compiler::{CompiledElement, CompiledNode, IcuNode};

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

#[test]
fn unsafe_close_reparse() {
    let input = "{username}!!";

    let CompiledElement::List(elements) = harness::parse(input, false) else {
        panic!("Parsed input was not a list of elements");
    };

    let CompiledElement::Node(CompiledNode::Icu(IcuNode::Argument(argument))) = &elements[0] else {
        panic!("Incorrect first element");
    };
    let CompiledElement::Literal(literal) = &elements[1] else {
        panic!("Incorrect second element");
    };

    assert_eq!(argument.name.as_str(), "username");
    assert_eq!(literal.as_str(), "!!");
}
