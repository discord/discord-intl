use intl_markdown::{compile_to_format_js, parse_intl_message};

#[test]
#[ignore]
fn test_ast() {
    let ast = parse_intl_message(include_str!("../benches/spec.md"), true);
    println!("{:?}", ast);

    // let compiled = compile_to_format_js(&ast);
    // println!("{:#?}", compiled);
    //
    // let serialized = serde_json::to_string(&compiled);
    // println!("{}", serialized.unwrap())
}
