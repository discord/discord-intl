#[cfg(test)]
mod harness {
    use intl_markdown::compiler::CompiledElement;
    use intl_markdown::{compiler, format, ICUMarkdownParser, SourceText};

    pub fn parse_and_compile(input: &str, include_blocks: bool) -> CompiledElement {
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
        println!("Input:\n------\n{}\n", input);
        compiled
    }

    pub fn assert_html(expected: &str, element: &CompiledElement) {
        assert_eq!(
            expected,
            format::to_html(element),
            "Formatted HTML (right) did not match expected result (left)"
        );
    }

    pub fn assert_ast(expected: &str, element: &CompiledElement) {
        let output = keyless_json::to_string(element).expect("Failed to serialize element");
        assert_eq!(
            expected, output,
            "Formatted AST (right) did not match expected result (left)"
        );
    }
}

mod ast;
