use crate::DiagnosticFix;
use intl_markdown::{IcuPlural, IcuPound, Visit, VisitWith};

struct IcuPoundFinder {
    has_pound: bool,
}

impl IcuPoundFinder {
    fn new() -> Self {
        Self { has_pound: false }
    }
}

impl Visit for IcuPoundFinder {
    fn visit_icu_pound(&mut self, _node: &IcuPound) {
        self.has_pound = true;
    }

    fn visit_icu_plural(&mut self, _node: &IcuPlural) {
        // Skip any nested plurals, since that resets the # context.
        return;
    }
}

pub(crate) fn replace_plural_with_select(plural: &IcuPlural) -> Vec<DiagnosticFix> {
    // If any of the plural arms contain a `#`, they cannot be converted to a `select` because it
    // does not set the current plural value. Technically it can work for a select inside another
    // plural, but that is not a case worth linting for now.
    let mut pound_finder = IcuPoundFinder::new();
    plural.visit_children_with(&mut pound_finder);
    if pound_finder.has_pound {
        return vec![];
    }

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
