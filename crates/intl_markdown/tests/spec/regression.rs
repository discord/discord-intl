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
