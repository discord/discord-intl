use crate::spec::icu::harness;

#[test]
fn icu_escapes() {
    let input = "\\{  variable  }";
    let expected = "{  variable  }";

    assert_eq!(expected, harness::parse_inline(input));
}
