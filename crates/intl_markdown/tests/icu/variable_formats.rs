use crate::icu::harness;

#[test]
fn date_word_format() {
    let input = "{today, date, short}";
    let expected = "{today, date, short}";

    assert_eq!(expected, harness::parse(input));
}
#[test]
fn number_currency_format() {
    let input = "{count, number, currency/USD}";
    let expected = "{count, number, currency/USD}";

    assert_eq!(expected, harness::parse(input));
}
