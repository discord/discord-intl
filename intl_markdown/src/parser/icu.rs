use crate::lexer::LexContext;
use crate::parser::inline::parse_inline;
use crate::SyntaxKind;

use super::ICUMarkdownParser;

pub(super) fn is_at_normal_icu(p: &mut ICUMarkdownParser) -> bool {
    (p.at(SyntaxKind::LCURLY) || p.at(SyntaxKind::UNSAFE_LCURLY)) && !p.current_flags().is_escaped()
}

pub(super) fn parse_icu(p: &mut ICUMarkdownParser) -> Option<()> {
    let end_kind = match p.current() {
        SyntaxKind::LCURLY => SyntaxKind::RCURLY,
        SyntaxKind::UNSAFE_LCURLY => SyntaxKind::UNSAFE_RCURLY,
        _ => return None,
    };

    let icu_mark = p.mark();
    p.bump();
    // Mark a checkpoint after the opening curly brace in case any part of the ICU content fails.
    // This will be the rewind point to let the parser retry all the content as plain markdown.
    let checkpoint = p.checkpoint();

    parse_icu_inner(p)
        .and_then(|_| {
            p.expect(end_kind)?;
            icu_mark.complete(p, SyntaxKind::ICU)
        })
        .or_else(|| {
            p.rewind(checkpoint);
            None
        })
}

fn parse_icu_inner(p: &mut ICUMarkdownParser) -> Option<()> {
    p.relex_with_context(LexContext::Icu);
    p.skip_whitespace_as_trivia_with_context(LexContext::Icu);

    let outer_mark = p.mark();
    if p.at(SyntaxKind::ICU_IDENT) || p.current().is_icu_keyword() {
        let var_start = p.mark();
        p.bump_as(SyntaxKind::ICU_IDENT, LexContext::Icu);
        var_start.complete(p, SyntaxKind::ICU_VARIABLE)?;
    } else {
        return None;
    }

    p.skip_whitespace_as_trivia_with_context(LexContext::Icu);
    if p.at(SyntaxKind::COMMA) {
        p.bump_with_context(LexContext::Icu);
        p.skip_whitespace_as_trivia_with_context(LexContext::Icu);
        let completed_kind = parse_complex_icu_placeholder(p)?;
        outer_mark.complete(p, completed_kind)?;
    }

    p.skip_whitespace_as_trivia_with_context(LexContext::Icu);
    Some(())
}

fn parse_complex_icu_placeholder(p: &mut ICUMarkdownParser) -> Option<SyntaxKind> {
    match p.current() {
        SyntaxKind::ICU_DATE_KW => parse_icu_date(p),
        SyntaxKind::ICU_TIME_KW => parse_icu_time(p),
        SyntaxKind::ICU_NUMBER_KW => parse_icu_number(p),
        SyntaxKind::ICU_PLURAL_KW => parse_icu_plural(p),
        _ => None,
    }
}

fn parse_icu_date(p: &mut ICUMarkdownParser) -> Option<SyntaxKind> {
    p.expect_with_context(SyntaxKind::ICU_DATE_KW, LexContext::Icu)?;
    p.skip_whitespace_as_trivia_with_context(LexContext::Icu);
    // TODO: Support format args here
    // if p.at(SyntaxKind::COMMA) {
    //     p.bump_with_context(LexContext::Icu);
    //     p.skip_whitespace_as_trivia_with_context(LexContext::Icu);
    // }
    Some(SyntaxKind::ICU_DATE)
}

fn parse_icu_time(p: &mut ICUMarkdownParser) -> Option<SyntaxKind> {
    p.expect_with_context(SyntaxKind::ICU_TIME_KW, LexContext::Icu)?;
    p.skip_whitespace_as_trivia_with_context(LexContext::Icu);
    // TODO: Support format args here
    // if p.at(SyntaxKind::COMMA) {
    //     p.bump_with_context(LexContext::Icu);
    //     p.skip_whitespace_as_trivia_with_context(LexContext::Icu);
    // }
    Some(SyntaxKind::ICU_TIME)
}

fn parse_icu_number(p: &mut ICUMarkdownParser) -> Option<SyntaxKind> {
    p.expect_with_context(SyntaxKind::ICU_NUMBER_KW, LexContext::Icu)?;
    p.skip_whitespace_as_trivia_with_context(LexContext::Icu);
    // TODO: Support format args here
    // if p.at(SyntaxKind::COMMA) {
    //     p.bump_with_context(LexContext::Icu);
    //     p.skip_whitespace_as_trivia_with_context(LexContext::Icu);
    // }
    Some(SyntaxKind::ICU_NUMBER)
}

fn parse_icu_plural(p: &mut ICUMarkdownParser) -> Option<SyntaxKind> {
    p.expect_with_context(SyntaxKind::ICU_PLURAL_KW, LexContext::Icu)?;
    p.skip_whitespace_as_trivia_with_context(LexContext::Icu);
    p.expect_with_context(SyntaxKind::COMMA, LexContext::Icu)?;
    p.skip_whitespace_as_trivia_with_context(LexContext::Icu);

    loop {
        if !p.at(SyntaxKind::ICU_IDENT) && !p.at(SyntaxKind::ICU_PLURAL_EXACT) {
            break;
        }

        let arm_mark = p.mark();
        p.bump_with_context(LexContext::Icu);
        p.skip_whitespace_as_trivia_with_context(LexContext::Icu);
        // Using Regular context here because we're entering a value section where the content
        // is expected to be regular markdown.
        p.expect(SyntaxKind::LCURLY)?;
        let value_mark = p.mark();
        parse_inline(p, true);
        value_mark.complete(p, SyntaxKind::ICU_PLURAL_VALUE)?;
        // The closing curly needs to be lexed in the Icu context, though, since we're now back in
        // the ICU control section.
        p.expect_with_context(SyntaxKind::RCURLY, LexContext::Icu)?;

        p.skip_whitespace_as_trivia_with_context(LexContext::Icu);
        arm_mark.complete(p, SyntaxKind::ICU_PLURAL_ARM)?;
    }

    Some(SyntaxKind::ICU_PLURAL)
}
