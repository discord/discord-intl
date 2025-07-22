#[cfg(test)]
mod harness {
    use intl_markdown::{compiler, format, ICUMarkdownParser, SourceText};
    pub fn parse_inline(input: &str) -> String {
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
        let compiled = compiler::compile_document(&ast);
        println!("Compiled:\n---------\n{:#?}\n", compiled);
        println!("Input:\n------\n{}\n", input);
        let output = format::to_html(&compiled);
        println!("HTML Format:\n------------\n{}\n{:?}", output, output);
        output
    }
}

mod cjk_emphasis;
mod hooks;
mod strikethrough;
