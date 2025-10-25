use crate::lexer::LexContext;
use crate::parser::inline::parse_inline;
use intl_markdown_syntax::SyntaxKind;

use super::ICUMarkdownParser;

pub(super) fn is_at_normal_icu(p: &mut ICUMarkdownParser) -> bool {
    p.at(SyntaxKind::LCURLY) || p.at(SyntaxKind::UNSAFE_LCURLY)
}

pub(super) fn parse_icu(p: &mut ICUMarkdownParser) -> Option<()> {
    let is_unsafe = match p.current() {
        SyntaxKind::LCURLY => false,
        SyntaxKind::UNSAFE_LCURLY => true,
        _ => return None,
    };

    let checkpoint = p.checkpoint();
    let icu_mark = p.mark();
    let lex_context = LexContext::Icu { is_unsafe };
    p.bump_with_context(lex_context);

    parse_icu_inner(p, lex_context)
        .and_then(|_| {
            p.expect(if is_unsafe {
                SyntaxKind::UNSAFE_RCURLY
            } else {
                SyntaxKind::RCURLY
            })?;
            icu_mark.complete(p, SyntaxKind::ICU)
        })
        .or_else(|| {
            p.rewind(checkpoint);
            // We need to bump past the LCURLY again, otherwise the parser will get stuck looping
            // over this same token again and again. We can't take a checkpoint _after_ the LCURLY
            // token because it will have trivia attached, and the parser isn't able to rewind
            // trivia for now.
            p.bump();
            None
        })
}

fn parse_icu_inner(p: &mut ICUMarkdownParser, lex_context: LexContext) -> Option<()> {
    p.skip_icu_whitespace_as_trivia(lex_context);

    let outer_mark = p.mark();
    if !p.at(SyntaxKind::ICU_IDENT) && !p.current().is_icu_keyword() {
        return None;
    }
    p.bump_as(SyntaxKind::ICU_IDENT, lex_context);
    p.skip_icu_whitespace_as_trivia(lex_context);
    if p.at(SyntaxKind::COMMA) {
        p.bump_with_context(lex_context);
        p.skip_icu_whitespace_as_trivia(lex_context);
        let completed_kind = parse_complex_icu_expression(p, lex_context)?;
        outer_mark.complete(p, completed_kind)?;
    } else {
        outer_mark.complete(p, SyntaxKind::ICU_PLACEHOLDER)?;
    }

    p.skip_icu_whitespace_as_trivia(lex_context);
    Some(())
}

fn parse_complex_icu_expression(
    p: &mut ICUMarkdownParser,
    lex_context: LexContext,
) -> Option<SyntaxKind> {
    match p.current() {
        SyntaxKind::ICU_DATE_KW => parse_icu_date(p, lex_context),
        SyntaxKind::ICU_TIME_KW => parse_icu_time(p, lex_context),
        SyntaxKind::ICU_NUMBER_KW => parse_icu_number(p, lex_context),
        SyntaxKind::ICU_PLURAL_KW => parse_icu_plural(
            p,
            SyntaxKind::ICU_PLURAL_KW,
            SyntaxKind::ICU_PLURAL,
            lex_context,
        ),
        SyntaxKind::ICU_SELECT_KW => parse_icu_plural(
            p,
            SyntaxKind::ICU_SELECT_KW,
            SyntaxKind::ICU_SELECT,
            lex_context,
        ),
        SyntaxKind::ICU_SELECT_ORDINAL_KW => parse_icu_plural(
            p,
            SyntaxKind::ICU_SELECT_ORDINAL_KW,
            SyntaxKind::ICU_SELECT_ORDINAL,
            lex_context,
        ),
        _ => None,
    }
}

fn parse_icu_date(p: &mut ICUMarkdownParser, lex_context: LexContext) -> Option<SyntaxKind> {
    p.expect_with_context(SyntaxKind::ICU_DATE_KW, lex_context)?;
    p.skip_icu_whitespace_as_trivia(lex_context);
    parse_optional_icu_style_argument(p, SyntaxKind::ICU_DATE, lex_context);
    Some(SyntaxKind::ICU_DATE)
}

fn parse_icu_time(p: &mut ICUMarkdownParser, lex_context: LexContext) -> Option<SyntaxKind> {
    p.expect_with_context(SyntaxKind::ICU_TIME_KW, lex_context)?;
    p.skip_icu_whitespace_as_trivia(lex_context);
    parse_optional_icu_style_argument(p, SyntaxKind::ICU_TIME, lex_context);
    Some(SyntaxKind::ICU_TIME)
}

fn parse_icu_number(p: &mut ICUMarkdownParser, lex_context: LexContext) -> Option<SyntaxKind> {
    p.expect_with_context(SyntaxKind::ICU_NUMBER_KW, lex_context)?;
    p.skip_icu_whitespace_as_trivia(lex_context);
    parse_optional_icu_style_argument(p, SyntaxKind::ICU_NUMBER, lex_context);
    Some(SyntaxKind::ICU_NUMBER)
}

/// FormatJS's interpretation of the style argument is _very_ loose. It can be completely invalid
/// and have no meaning whatsoever, but still be accepted and treated as a variable. This is a
/// _loose_ parse, which is separate from a _recovered_ parse. To implement that, we can try to
/// parse out the details of the style argument (and should, eventually, to help with
/// validations), but we can also just assume "anything until the closing brace" will create a
/// valid style argument, the same way that FormatJS does, and let the runtime figure it out
/// instead.
///
/// In the future, this can be expanded and split into appropriate parsing for both number and
/// date/time styles, with a fallback to plain text for both.
#[inline(always)]
fn parse_optional_icu_style_argument(
    p: &mut ICUMarkdownParser,
    parent_kind: SyntaxKind,
    lex_context: LexContext,
) -> Option<()> {
    // If there's no comma, then there's no style arg and this can just return immediately.
    p.optional(p.at(SyntaxKind::COMMA), |p| {
        // Otherwise, open the style marker and consume that comma.
        let style_mark = p.mark();
        p.bump_with_context(lex_context);
        p.skip_icu_whitespace_as_trivia(lex_context);
        // This relex happens first so that any potentially-significant token that may be at the
        // current position is un-lexed and treated as plain text instead. It has to happen as a relex
        // because the IcuStyle context doesn't understand whitespace and wouldn't be able to skip
        // trivia as expected if it was used in `skip_icu_whitespace_as_trivia(lex_context).
        p.relex_with_context(LexContext::IcuStyle);

        p.expect_with_context(SyntaxKind::ICU_STYLE_TEXT, lex_context)?;
        let completed_kind = match parent_kind {
            SyntaxKind::ICU_DATE | SyntaxKind::ICU_TIME => SyntaxKind::ICU_DATE_TIME_STYLE,
            SyntaxKind::ICU_NUMBER => SyntaxKind::ICU_NUMBER_STYLE,
            _ => unreachable!(),
        };
        style_mark.complete(p, completed_kind)
    })
}

fn parse_icu_plural(
    p: &mut ICUMarkdownParser,
    keyword_kind: SyntaxKind,
    kind: SyntaxKind,
    lex_context: LexContext,
) -> Option<SyntaxKind> {
    p.expect_with_context(keyword_kind, lex_context)?;
    p.skip_icu_whitespace_as_trivia(lex_context);
    p.expect_with_context(SyntaxKind::COMMA, lex_context)?;
    p.skip_icu_whitespace_as_trivia(lex_context);

    let arms_mark = p.mark();
    loop {
        if !p.at(SyntaxKind::ICU_IDENT) && !p.at(SyntaxKind::ICU_PLURAL_EXACT) {
            break;
        }

        let arm_mark = p.mark();
        p.bump_with_context(lex_context);
        p.skip_icu_whitespace_as_trivia(lex_context);
        // Using Regular context here because we're entering a value section where the content
        // is expected to be regular Markdown.
        p.expect(SyntaxKind::LCURLY)?;
        let value_mark = p.mark();
        parse_inline(p, true);
        value_mark.complete(p, SyntaxKind::ICU_PLURAL_VALUE)?;
        // The closing curly needs to be lexed in the Icu context, though, since we're now back in
        // the ICU control section.
        p.expect_with_context(SyntaxKind::RCURLY, lex_context)?;

        p.skip_icu_whitespace_as_trivia(lex_context);
        arm_mark.complete(p, SyntaxKind::ICU_PLURAL_ARM)?;
    }
    arms_mark.complete(p, SyntaxKind::ICU_PLURAL_ARMS)?;

    Some(kind)
}

pub(super) fn parse_icu_pound(p: &mut ICUMarkdownParser) -> Option<()> {
    let mark = p.mark();
    p.bump();
    mark.complete(p, SyntaxKind::ICU_POUND)
}
