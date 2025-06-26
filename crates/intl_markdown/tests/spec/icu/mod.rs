#[cfg(test)]
mod harness {
    use intl_markdown::{compiler, format, ICUMarkdownParser, SourceText};

    fn parse(input: &str, include_blocks: bool) -> String {
        let mut parser = ICUMarkdownParser::new(SourceText::from(input), include_blocks);
        #[cfg(feature = "debug-tracing")]
        if include_blocks {
            println!("Blocks: {:?}\n", parser.lexer_block_bounds());
        }
        parser.parse();
        #[cfg(feature = "debug-tracing")]
        println!("Tokens:\n-------\n{:#?}\n", parser.debug_token_list());
        let result = parser.finish();
        println!("Tree:\n-------\n{:#?}\n", result.tree);
        let ast = result.to_document();
        println!("AST:\n----\n{:#?}\n", ast);
        let compiled = compiler::compile_document(&ast);
        println!("Compiled:\n---------\n{:#?}\n", compiled);
        let output = format::to_html(&compiled);
        println!("Input:\n------\n{}\n", input);
        println!("HTML Format:\n------------\n{}\n{:?}", output, output);
        output
    }

    pub fn parse_inline(input: &str) -> String {
        parse(input, false)
    }

    pub fn parse_blocks(input: &str) -> String {
        parse(input, true)
    }
}

mod blocks;
mod escapes;
mod inline;
mod markdown_blocks;
mod markdown_headings;
mod nodes;
mod variable_formats;
