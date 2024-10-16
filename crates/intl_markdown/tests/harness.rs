use intl_markdown::{
    compile_to_format_js, format_ast, format_icu_string, process_cst_to_ast, CstDocument, Document,
    ICUMarkdownParser,
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
    let output = keyless_json::to_string(&compile_to_format_js(&ast)).unwrap();

    assert_eq!(expected, output);
}

macro_rules! ast_test {
    ($name:ident, $input:literal, $output:literal) => {
        #[test]
        fn $name() {
            crate::harness::run_icu_ast_test($input, $output, false);
        }
    };
}
macro_rules! icu_string_test {
    ($name:ident, $input:literal, $output:literal) => {
        #[test]
        fn $name() {
            crate::harness::run_icu_string_test($input, $output, false);
        }
    };
}
macro_rules! icu_block_string_test {
    ($name:ident, $input:literal, $output:literal) => {
        #[test]
        fn $name() {
            crate::harness::run_icu_string_test($input, $output, true);
        }
    };
}

pub(crate) use ast_test;
pub(crate) use icu_block_string_test;
pub(crate) use icu_string_test;
