use criterion::{criterion_group, criterion_main, Criterion, Throughput};

use intl_markdown::parse_intl_message;

fn parse_message(message: &str) {
    parse_intl_message(message, false);
}

fn parse_bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("messages");
    group.throughput(Throughput::Elements(1));
    group.bench_function("complex messages", |b| {
        b.iter(|| {
        parse_message(r#"
            {gender_of_host, select,
                female {
                    {num_guests, plural,
                        =0 {{host} does not give a party.}
                        =1 {{host} invites <em>{guest}</em> to her party.}
                        =2 {{host} invites <em>{guest}</em> and <em>one</em> other person to her party.}
                        other {{host} invites <em>{guest}</em> and <em>#</em> other people to her party.}
                    }
                }
                male {
                    {num_guests, plural,
                        =0 {{host} does not give a party.}
                        =1 {{host} invites <em>{guest}</em> to his party.}
                        =2 {{host} invites <em>{guest}</em> and one other person to his party.}
                        other {{host} invites <em>{guest}</em> and <em>#</em> other people to his party.}
                    }
                }
                other {
                    {num_guests, plural,
                        =0 {{host} does not give a party.}
                        =1 {{host} invites <em>{guest}</em> to their party.}
                        =2 {{host} invites <em>{guest}</em> and one other person to their party.}
                        other {{host} invites <em>{guest}</em> and <em>#</em> other people to their party.}
                    }
                }
            }"#
        )});
    });

    group.bench_function("normal message", |b| {
        b.iter(|| {
            parse_message(
                r#"
            Yo, {firstName} {lastName} has
            {numBooks, number, integer}
            {numBooks, plural,
                one {book}
                other {books}
            }
        "#,
            )
        });
    });
    group.bench_function("simple message", |b| {
        b.iter(|| parse_message(r#"Hello, {name}"#));
    });
    group.bench_function("string message", |b| {
        b.iter(|| parse_message(r#"Hello, world"#));
    });
}
criterion_group!(benches, parse_bench);
criterion_main!(benches);
