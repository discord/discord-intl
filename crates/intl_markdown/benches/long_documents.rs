use std::collections::HashMap;

use criterion::{criterion_group, criterion_main, Criterion};

use intl_markdown::{compiler, format, ICUMarkdownParser, SourceText};

fn parse_to_html(content: &str, include_blocks: bool) -> String {
    let mut parser = ICUMarkdownParser::new(SourceText::from(content), include_blocks);
    parser.parse();
    let document = parser.finish().to_document();
    let compiled = compiler::compile_document(&document);
    format::to_html(&compiled)
}

/// NOTE: To run this test, copy the commonmark spec text from
/// https://github.com/commonmark/commonmark-spec/blob/master/spec.txt into
/// a new file `./spec.md`.
fn long_documents(c: &mut Criterion) {
    let mut group = c.benchmark_group("long documents");
    group.bench_function("intl-markdown", |b| {
        b.iter(|| {
            let content = include_str!("./spec.md");
            let output = parse_to_html(content, true);
            let _len = output.len();
        })
    });

    group.bench_function("pulldown_cmark", |b| {
        b.iter(|| {
            let content = include_str!("./spec.md");
            let parser = pulldown_cmark::Parser::new(content);
            let mut html_output = String::new();
            pulldown_cmark::html::push_html(&mut html_output, parser);
            let _len = html_output.len();
        })
    });
    group.finish();
}

fn short_inlines(c: &mut Criterion) {
    let mut group = c.benchmark_group("inlines");
    group.bench_function("intl-markdown", |b| {
        b.iter(|| {
            let content = "*this ***has some* various things* that** [create multiple elements](while/inline 'but without') taking _too_ much ![effort] to parse, and should `be a decent` test` ``of ``whether this works quickly.";
            let output = parse_to_html(content, true);
            let _len = output.len();
        })
    });
    group.bench_function("intl-markdown no blocks", |b| {
        b.iter(|| {
            let content = "*this ***has some* various things* that** [create multiple elements](while/inline 'but without') taking _too_ much ![effort] to parse, and should `be a decent` test` ``of ``whether this works quickly.";
            let output = parse_to_html(content, false);
            let _len = output.len();
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
        &std::fs::read_to_string("../intl_message_database/data/input/fr.messages.jsona")
            .expect("No data file exists"),
    )
    .expect("failed to parse JSON data file");

    group.bench_function("intl-markdown", |b| {
        b.iter(|| {
            for message in messages.values() {
                let output = parse_to_html(message, true);
                let _len = output.len();
            }
        })
    });
    group.bench_function("intl-markdown no blocks", |b| {
        b.iter(|| {
            for message in messages.values() {
                let output = parse_to_html(message, false);
                let _len = output.len();
            }
        })
    });
    group.bench_function("pulldown_cmark", |b| {
        b.iter(|| {
            for message in messages.values() {
                let parser = pulldown_cmark::Parser::new(&message);
                let mut html_output = String::new();
                pulldown_cmark::html::push_html(&mut html_output, parser);
                let _len = html_output.len();
            }
        })
    });
    group.finish();
}

criterion_group!(benches, long_documents, short_inlines, real_messages);
criterion_main!(benches);
