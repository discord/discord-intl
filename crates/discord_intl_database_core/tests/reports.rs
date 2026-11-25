use discord_intl_database_core::*;
#[test]
fn offsets_and_diagnostic_positions_are_exportable() {
    let offsets = SourceOffsetList::new(vec![(3, 1), (8, 6)]);
    assert_eq!(offsets.as_slice(), &[(3, 1), (8, 6)]);
    assert_eq!(offsets.total_offset_at(3), 0);
    assert_eq!(offsets.total_offset_at(4), 1);
    let d = SourceDiagnostic::new(
        DiagnosticCode::ParseError,
        key_symbol("a"),
        "\n😀x",
        (5, 6),
        "bad",
    );
    assert_eq!((d.line, d.column, d.span), (2, 2, (5, 6)));
}
