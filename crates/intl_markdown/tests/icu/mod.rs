#[cfg(test)]
mod harness {
    use intl_markdown::{formatter, ICUMarkdownParser, SourceText};
    pub fn parse(input: &str) -> String {
        let mut parser = ICUMarkdownParser::new(SourceText::from(input), false);
        #[cfg(feature = "debug-tracing")]
        println!("Blocks: {:?}\n", parser.lexer_block_bounds());
        parser.parse();
        #[cfg(feature = "debug-tracing")]
        println!("Tokens:\n-------\n{:#?}\n", parser.debug_token_list());
        let result = parser.finish();
        println!("Tree:\n-------\n{:#?}\n", result.tree);
        let ast = result.to_document();
        println!("AST:\n----\n{:#?}\n", ast);
        let output = formatter::to_keyless_json(&ast);
        println!("Input:\n------\n{}\n", input);
        println!("HTML Format:\n------------\n{}\n{:?}", output, output);
        output
    }
}

mod blocks;
mod escapes;
mod inline;
mod markdown_blocks;
mod markdown_headings;
mod variable_formats;
