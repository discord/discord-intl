#[cfg(test)]
mod harness {
    use intl_markdown::compiler::CompiledElement;
    use intl_markdown::{compiler, format, ICUMarkdownParser, SourceText};

    pub fn parse(input: &str, include_blocks: bool) -> CompiledElement {
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
        compiled
    }

    pub fn parse_inline(input: &str) -> String {
        let compiled = parse(input, false);
        println!("Input:\n------\n{}\n", input);
        format_html(compiled)
    }

    pub fn parse_blocks(input: &str) -> String {
        let compiled = parse(input, true);
        println!("Input:\n------\n{}\n", input);
        format_html(compiled)
    }

    pub fn format_html(element: CompiledElement) -> String {
        let output = format::to_html(&element);
        println!("HTML Format:\n------------\n{}\n{:?}", output, output);
        output
    }
}

mod blocks;
mod escapes;
mod inline;
mod markdown_blocks;
mod markdown_headings;
mod nodes;
mod unsafe_variables;
mod variable_formats;
