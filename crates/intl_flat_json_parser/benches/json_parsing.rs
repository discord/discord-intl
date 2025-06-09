use criterion::{criterion_group, criterion_main, Criterion};
use std::collections::HashMap;

fn get_source_files() -> Vec<(String, String)> {
    let file_names = vec![
        "../intl_message_database/data/input/fr.messages.jsona",
        "../intl_message_database/data/input/en-GB.messages.jsona",
        "../intl_message_database/data/input/zh-CN.messages.jsona",
        "../intl_message_database/data/input/zh-TW.messages.jsona",
        "../intl_message_database/data/input/ru.messages.jsona",
        "../intl_message_database/data/input/hi.messages.jsona",
        "../intl_message_database/data/input/ja.messages.jsona",
        "../intl_message_database/data/input/ko.messages.jsona",
    ];
    file_names
        .into_iter()
        .map(|file| {
            (
                file.to_string(),
                std::fs::read_to_string("../intl_message_database/data/input/fr.messages.jsona")
                    .expect("No data file exists"),
            )
        })
        .collect()
}

fn parse_comparison(c: &mut Criterion) {
    let files = get_source_files();
    let mut group = c.benchmark_group("parse");
    group.bench_function("serde", |b| {
        b.iter(|| {
            for (_name, content) in &files {
                let _ = serde_json::from_str::<HashMap<String, String>>(&content);
            }
        })
    });
    group.bench_function("intl-flat-json", |b| {
        b.iter(|| {
            for (_name, content) in &files {
                let _ = intl_flat_json_parser::parse_flat_translation_json(&content)
                    .collect::<Vec<_>>();
            }
        })
    });
}

criterion_group!(benches, parse_comparison);
criterion_main!(benches);
