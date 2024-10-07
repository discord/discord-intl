use intl_markdown::{
    CstDocument, Document, format_ast, format_icu_string, ICUMarkdownParser, process_cst_to_ast,
};

pub fn parse(content: &str, include_blocks: bool) -> CstDocument {
    let mut parser = ICUMarkdownParser::new(content, include_blocks);
    parser.parse();
    parser.into_cst()
}

pub fn parse_to_ast(content: &str, include_blocks: bool) -> Document {
    let mut parser = ICUMarkdownParser::new(content, include_blocks);
    let source = parser.source().clone();
    parser.parse();
    process_cst_to_ast(source, &parser.into_cst())
}

/// Test that the input is parsed and formatted as HTML as given.
#[allow(unused)]
pub fn run_spec_test(input: &str, expected: &str) {
    // AST-based formatting
    let ast = parse_to_ast(input, true);
    let output = format_ast(&ast).unwrap();

    assert_eq!(expected, output);
}

/// Test that the input is parsed and formatted as an ICU string as given.
#[allow(unused)]
pub fn run_icu_string_test(input: &str, expected: &str, include_blocks: bool) {
    // AST-based formatting
    let ast = parse_to_ast(input, include_blocks);
    let output = format_icu_string(&ast).unwrap();

    assert_eq!(expected, output);
}

/// Test that the input is parsed and formatted as an ICU AST as given.
#[allow(unused)]
pub fn run_icu_ast_test(input: &str, expected: &str, include_blocks: bool) {
    // AST-based formatting
    let ast = parse_to_ast(input, include_blocks);
    let output = serde_json::to_string(&ast).unwrap();

    assert_eq!(expected, output);
}
