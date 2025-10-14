#[cfg(test)]
mod harness {
    use intl_markdown::{compiler, format, ICUMarkdownParser, SourceText};
    pub fn parse(input: &str) -> String {
        let mut parser = ICUMarkdownParser::new(SourceText::from(input), true);
        #[cfg(feature = "debug-tracing")]
        println!("Blocks: {:?}\n", parser.lexer_block_bounds());
        parser.parse();
        #[cfg(feature = "debug-tracing")]
        println!("Tokens:\n-------\n{:#?}\n", parser.debug_token_list());
        let result = parser.finish();
        println!("Tree:\n-------\n{:#?}\n", result.tree);
        let ast = result.to_document();
        println!("AST:\n----\n{:#?}\n", ast);
        let compiled = compiler::compile_document(&ast);
        println!("Compiled:\n---------\n{:#?}\n", compiled);
        println!("Input:\n------\n{}\n", input);
        let output = format::to_html(&compiled);
        println!("HTML Format:\n------------\n{}\n{:?}", output, output);
        output
    }
}

#[test]
fn regression_1() {
    // Leading trivia on an emphasis span need to be accounted for in the text offset of where the
    // span starts. If they are not counted, the Strong node here incorrectly picks up the last
    // TEXT node inside itself because it won't have counted enough text to complete itself.
    //     0 "   *" STAR (len 4, leading trivia len 3)
    //     1 "*"    STAR (len 1)
    //     2 "Foo"  TEXT (len 3)
    //     3 "*"    STAR (len 1)
    //     4 "*"    STAR (len 1)
    //     5 " f"   TEXT (len 2)
    // Strong spans `[0, 4]`, with an expected text length of 10, but without counting the leading
    // trivia of `0` it only counts 7 bytes before reaching the last STAR, and will continue
    // picking up the next token to try to reach the end.
    //
    // This is only a problem here because the tree implementation prepends the leading trivia
    // _after_ the marker for the first STAR token was taken, so it assumes its offset if 3 instead
    // of 0 where the leading trivia starts.
    let input = "   **Foo** f";
    let expected = "<p><strong>Foo</strong> f</p>";
    let parsed = harness::parse(input);
    assert_eq!(expected, parsed);
}

#[test]
fn regression_2() {
    // Most CommonMark rules work well and aren't a concern for conflicting with natural language
    // syntax, but sometimes things overlap a little bit. For example, the image syntax `![]` is
    // ambiguous with a natural link following an exclamation, like `hello![foo](./bar)`. In
    // reality, the correct thing to do here is add a space between either the `o` and `!` to
    // create a phrase and an image _or_ between the `!` and the `[` to create a phrase and a
    // regular link. However, since most of the time we're working with untrustable user input for
    // intl messages, we want a way to more definitively distinguish them. It's also exceedingly
    // rare for an image to be intentional, so preferring the link tag is more natural.
    let input = "hello![foo](./bar)";
    let expected = r#"<p>hello!<a href="./bar">foo</a></p>"#;
    assert_eq!(expected, harness::parse(input));
}

#[test]
fn regression_3() {
    // The classic shrug, ¯\_(ツ)_/¯, is a pain to process with Markdown for two reasons: First, the
    // `\_` at the front can be considered an escaped `_` character, so the input needs to prepare
    // for that by escaping the backslash itself, leading to `\\` as the _raw_ input of the message,
    // with further escapes sometimes being necessary depending on the source language (in JS, for
    // example, the string is written as `"¯\\\\_(ツ)_/¯"` to be interpreted down to `\\_`).
    //
    // Second, the `_(ツ)_` segment can also be considered an Emphasis marker, meaning the `_`
    // characters would be removed from the text and leave only `¯\\(ツ)/¯`, which is also wrong.
    // It's up to the parser itself to understand this situation and prevent the underscore from
    // being handled. There may be other cases that this affects downstream, but by treating `\` as
    // a _non-punctuation_ character, the CommonMark rules for emphasis delimiters can still be
    // applied as normal (including CJK rules) to yield the correct result.
    let input = r#"¯\\_(ツ)_/¯"#;
    let expected = r#"<p>¯\_(ツ)_/¯</p>"#;
    assert_eq!(expected, harness::parse(input));
}
