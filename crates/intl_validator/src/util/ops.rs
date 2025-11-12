use crate::DiagnosticFix;
use intl_markdown::IcuPlural;

pub(crate) fn replace_plural_with_select(plural: &IcuPlural) -> Vec<DiagnosticFix> {
    let mut fixes = plural
        .arms()
        .children()
        .filter_map(|arm| {
            if arm.is_other_selector() {
                None
            } else {
                Some(DiagnosticFix::replace_token(
                    &arm.selector_token(),
                    &arm.selector_token().text()[1..],
                ))
            }
        })
        .collect::<Vec<_>>();

    fixes.push(DiagnosticFix::replace_token(
        &plural.format_token(),
        "select",
    ));

    fixes
}
