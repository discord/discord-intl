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
        let document = result.to_document();
        println!("CST:\n----\n{:#?}\n", document);
        let compiled = compiler::compile_document(&document);
        println!("Compiled:\n---------\n{:#?}\n", compiled);
        let output = format::to_html(&compiled);
        println!("Input:\n------\n{}\n", input);
        println!("HTML Format:\n------------\n{}\n{:?}", output, output);
        output
    }
}
mod atx_headings;
mod autolinks;
mod backslash_escapes;
mod blank_lines;
mod block_quotes;
mod code_spans;
mod emphasis_and_strong_emphasis;
mod entity_and_numeric_character_references;
mod fenced_code_blocks;
mod hard_line_breaks;
mod html_blocks;
mod images;
mod indented_code_blocks;
mod inlines;
mod link_reference_definitions;
mod links;
mod list_items;
mod lists;
mod paragraphs;
mod precedence;
mod raw_html;
mod setext_headings;
mod soft_line_breaks;
mod tabs;
mod textual_content;
mod thematic_breaks;
