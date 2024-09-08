use std::collections::HashMap;

use criterion::{Criterion, criterion_group, criterion_main};

use intl_markdown::{Document, format_ast, ICUMarkdownParser, process_cst_to_ast};

fn parse_to_ast(content: &str, include_blocks: bool) -> Document {
    let mut parser = ICUMarkdownParser::new(content, include_blocks);
    let source = parser.source().clone();
    parser.parse();
    let document = parser.into_cst();
    process_cst_to_ast(source, &document)
}

/// NOTE: To run this test, copy the commonmark spec text from
/// https://github.com/commonmark/commonmark-spec/blob/master/spec.txt into
/// a new file `./spec.md`.
fn long_documents(c: &mut Criterion) {
    let mut group = c.benchmark_group("long documents");
    group.bench_function("intl-markdown", |b| {
        b.iter(|| {
            let content = include_str!("./spec.md");
            let ast = parse_to_ast(content, true);
            format_ast(&ast)
        })
    });

    group.bench_function("pulldown_cmark", |b| {
        b.iter(|| {
            let content = include_str!("./spec.md");
            let parser = pulldown_cmark::Parser::new(content);
            let mut html_output = String::new();
            pulldown_cmark::html::push_html(&mut html_output, parser);
        })
    });
    group.finish();
}

fn short_inlines(c: &mut Criterion) {
    let mut group = c.benchmark_group("inlines");
    group.bench_function("intl-markdown", |b| {
        b.iter(|| {
            let content = "*this ***has some* various things* that** [create multiple elements](while/inline 'but without') taking _too_ much ![effort] to parse, and should `be a decent` test` ``of ``whether this works quickly.";
            let ast = parse_to_ast(content, true);
            format_ast(&ast)
        })
    });
    group.bench_function("intl-markdown no blocks", |b| {
        b.iter(|| {
            let content = "*this ***has some* various things* that** [create multiple elements](while/inline 'but without') taking _too_ much ![effort] to parse, and should `be a decent` test` ``of ``whether this works quickly.";
    let ast = parse_to_ast(content, false);

            format_ast(&ast)
        })
    });
    group.bench_function("pulldown_cmark", |b| {
        b.iter(|| {
            let content = "*this ***has some* various things* that** [create multiple elements](while/inline 'but without') taking _too_ much ![effort] to parse, and should `be a decent` test` ``of ``whether this works quickly.";
            let parser = pulldown_cmark::Parser::new(content);
            let mut html_output = String::new();
            pulldown_cmark::html::push_html(&mut html_output, parser);
        })
    });
    group.finish();
}

fn real_messages(c: &mut Criterion) {
    let mut group = c.benchmark_group("real messages");
    let messages: HashMap<String, String> = serde_json::from_str(
        &std::fs::read_to_string("../intl_message_database/data/input/fr.jsona")
            .expect("No data file exists"),
    )
    .expect("failed to parse JSON data file");

    group.bench_function("intl-markdown", |b| {
        b.iter(|| {
            for message in messages.values() {
                let ast = parse_to_ast(message, true);
                format_ast(&ast).ok();
            }
        })
    });
    group.bench_function("intl-markdown no blocks", |b| {
        b.iter(|| {
            for message in messages.values() {
                let ast = parse_to_ast(message, false);
                format_ast(&ast).ok();
            }
        })
    });
    group.bench_function("pulldown_cmark", |b| {
        b.iter(|| {
            for message in messages.values() {
                let parser = pulldown_cmark::Parser::new(&message);
                let mut html_output = String::new();
                pulldown_cmark::html::push_html(&mut html_output, parser);
            }
        })
    });
    group.finish();
}

criterion_group!(benches, long_documents, short_inlines, real_messages);
criterion_main!(benches);
