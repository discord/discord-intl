use arcstr::Substr;

use crate::{ast, SyntaxKind};
use crate::ast::{CodeBlockKind, HeadingKind, IcuPluralKind, LinkKind, TextOrPlaceholder};
use crate::html_entities::get_html_entity;
use crate::token::Token;
use crate::tree_builder::{cst, TokenSpan};

use super::util::unescape;

#[derive(Clone, Debug, Default)]
pub struct AstProcessingContext {
    allow_hard_line_breaks: bool,
    allow_icu_pound: bool,
}

impl AstProcessingContext {
    fn with_context<M, F, T>(&mut self, mut mutator: M, func: F) -> T
    where
        M: FnMut(&mut Self),
        F: FnOnce(&mut Self) -> T,
    {
        let old_allow_hard_line_breaks = self.allow_hard_line_breaks;
        let old_allow_icu_pound = self.allow_icu_pound;
        mutator(self);

        let result = func(self);
        self.allow_icu_pound = old_allow_icu_pound;
        self.allow_hard_line_breaks = old_allow_hard_line_breaks;
        result
    }
}

pub fn process_cst_to_ast(cst: &cst::Document) -> ast::Document {
    let mut context = AstProcessingContext::default();
    let mut blocks = vec![];
    for node in cst.children() {
        match node {
            // Top-level tokens can't mean anything in a document, so this is ignored.
            cst::NodeOrToken::Token(_) => {}
            cst::NodeOrToken::Node(node) => {
                let ast_node = match node {
                    cst::Node::ThematicBreak(_) => ast::BlockNode::ThematicBreak,
                    cst::Node::InlineContent(content) => {
                        ast::BlockNode::InlineContent(process_inline_content(&mut context, content))
                    }
                    cst::Node::Paragraph(paragraph) => {
                        ast::BlockNode::Paragraph(process_paragraph(&mut context, paragraph))
                    }
                    cst::Node::AtxHeading(atx_heading) => {
                        ast::BlockNode::Heading(process_atx_heading(&mut context, atx_heading))
                    }
                    cst::Node::SetextHeading(setext_heading) => ast::BlockNode::Heading(
                        process_setext_heading(&mut context, setext_heading),
                    ),
                    cst::Node::IndentedCodeBlock(code_block) => ast::BlockNode::CodeBlock(
                        process_indented_code_block(&mut context, code_block),
                    ),
                    cst::Node::FencedCodeBlock(code_block) => ast::BlockNode::CodeBlock(
                        process_fenced_code_block(&mut context, code_block),
                    ),
                    node => unreachable!(
                        "Inline nodes can't appear directly under a document. Found:\n{:#?}",
                        node
                    ),
                };
                blocks.push(ast_node);
            }
        }
    }

    ast::Document { blocks }
}

pub fn process_paragraph(
    context: &mut AstProcessingContext,
    paragraph: &cst::Paragraph,
) -> ast::Paragraph {
    context.with_context(
        |context| context.allow_hard_line_breaks = true,
        |context| ast::Paragraph(process_inline_content(context, &paragraph.children)),
    )
}

pub fn process_atx_heading(
    context: &mut AstProcessingContext,
    atx_heading: &cst::AtxHeading,
) -> ast::Heading {
    ast::Heading {
        kind: HeadingKind::Atx,
        level: atx_heading.level() as u8,
        content: process_inline_content(context, &atx_heading.children),
    }
}

pub fn process_setext_heading(
    context: &mut AstProcessingContext,
    setext_heading: &cst::SetextHeading,
) -> ast::Heading {
    ast::Heading {
        kind: HeadingKind::Atx,
        level: setext_heading.underline.level() as u8,
        content: process_inline_content(context, &setext_heading.children),
    }
}

pub fn process_indented_code_block(
    context: &mut AstProcessingContext,
    code_block: &cst::IndentedCodeBlock,
) -> ast::CodeBlock {
    ast::CodeBlock {
        kind: CodeBlockKind::Indented,
        language: None,
        info_string: None,
        content: process_code_block_content(context, &code_block.content),
    }
}

pub fn process_fenced_code_block(
    context: &mut AstProcessingContext,
    code_block: &cst::FencedCodeBlock,
) -> ast::CodeBlock {
    let (language, info_string) = process_code_block_info_string(context, &code_block.info_string);
    let mut content = String::new();
    // Fenced code blocks can have leading newlines, which are attached to the end of the opening
    // delimiter or the end of the info string, but need to get mapped into the content here.
    // Note that this skips the preceding trivia up until the first newline is found, since
    // everything up to and including that is really part of the delimiter or info string. For a
    // newline to be inserted in the content, it has to part of its own line. LEADING_WHITESPACE
    // will also be skipped, since it's generally ignored.
    if let Some(token) = code_block.content.first_token() {
        let trivia = token.preceding_contiguous_trivia();
        let mut past_first_line_ending = false;
        for piece in trivia {
            match piece.kind() {
                // Leading whitespace is always skipped
                SyntaxKind::LEADING_WHITESPACE => continue,
                // Skip everything up until the first line ending
                SyntaxKind::LINE_ENDING if !past_first_line_ending => {
                    past_first_line_ending = true;
                }
                _ if !past_first_line_ending => continue,
                // Everything else gets added directly.
                _ => content.push_str(piece.text()),
            }
        }
    }
    content.push_str(&process_code_block_content(context, &code_block.content));

    ast::CodeBlock {
        kind: CodeBlockKind::Fenced,
        language,
        info_string,
        content,
    }
}

pub fn process_code_block_content(
    _: &mut AstProcessingContext,
    content: &cst::CodeBlockContent,
) -> String {
    let mut buffer = String::with_capacity(content.children().len() * 10);
    for child in content.children() {
        // The only leading trivia that a token in a code block can have is a LEADING_WHITESPACE
        // trivia, which is intentionally left out of the content of a code block anyway, so we
        // can always skip all leading trivia of these tokens safely.
        buffer.push_str(child.text_with_trailing_trivia().as_str());
    }

    // Code blocks always have a trailing newline, so add it if it isn't already present.
    if !buffer.is_empty() && !buffer.ends_with('\n') {
        buffer.push('\n');
    }

    buffer
}

pub fn process_code_block_info_string(
    context: &mut AstProcessingContext,
    info_string: &Option<cst::CodeFenceInfoString>,
) -> (Option<String>, Option<String>) {
    let Some(info_string) = info_string else {
        return (None, None);
    };

    // This _should_ never be encountered, but in case the info string is empty, then there's
    // nothing else to process.
    if info_string.is_empty() {
        return (None, None);
    }

    // Now, collect the entire info string by merging all the token text together. Then split out
    // the language from that string as the characters up until the first whitespace.
    let info_string_text =
        take_tokens_verbatim_with_entities_replaced(context, info_string.children(), true);
    match info_string_text.split_once(|c: char| c.is_ascii_whitespace()) {
        None => (None, None),
        Some((language, info_string)) => (
            (!language.is_empty()).then_some(unescape(language)),
            (!info_string.is_empty()).then_some(unescape(info_string.into())),
        ),
    }
}

#[inline(always)]
pub fn process_inline_content(
    context: &mut AstProcessingContext,
    content: &cst::InlineContent,
) -> Vec<ast::InlineContent> {
    process_inline_children(context, content.children())
}

pub fn process_inline_children(
    context: &mut AstProcessingContext,
    content: &Vec<cst::NodeOrToken>,
) -> Vec<ast::InlineContent> {
    let mut children = vec![];
    let mut last_node_token = None;
    // If multiple plain text tokens appear in a row, they can be merged together into a single
    // `InlineContent::Text` node, saving space and time during traversals. Entities and other kinds
    // of text tokens are _not_ merged since they have significance and might be treated differently
    // depending on the context.
    let mut last_token_kind: Option<SyntaxKind> = None;

    for (index, child) in content.iter().enumerate() {
        // If the last child was a node, then its trailing trivia won't have been picked up. In that
        // case, it gets added as a new text element before the next child is processed.
        //
        // This is also true when the last token is interpreted as a unit struct, like IcuPound,
        // where the token is effectively removed, so trivia needs to be added back again.
        let last_was_trivia = if let Some(token) = last_node_token {
            children.push(process_trailing_trivia(context, token));
            true
        } else {
            false
        };

        let is_last = index == content.len() - 1;
        match child {
            cst::NodeOrToken::Node(node) => {
                let result = process_inline_node(context, node);
                last_node_token = node.last_token();
                last_token_kind = None;
                children.push(result);
            }
            cst::NodeOrToken::Token(token) => {
                let result = process_inline_token(context, token, !is_last);
                match result {
                    ast::InlineContent::IcuPound => {
                        // IcuPound is really a Node type, but is represented as a single token in the CST,
                        // so it gets set as the last node token in this branch.
                        last_node_token = Some(token);
                        last_token_kind = None;
                        children.push(result);
                    }
                    // If this is a text element and the last one was also a plain and simple text
                    // element, then this new element gets merged into the previous one.
                    ast::InlineContent::Text(new_text)
                        if last_was_trivia
                            || token.kind().can_merge_as_text()
                                && last_token_kind
                                    .as_ref()
                                    .is_some_and(SyntaxKind::can_merge_as_text) =>
                    {
                        if let Some(ast::InlineContent::Text(previous_text)) = children.last_mut() {
                            previous_text.push_str(&new_text);
                        }
                        last_node_token = None;
                        last_token_kind = Some(token.kind())
                    }
                    _ => {
                        children.push(result);
                        last_node_token = None;
                        last_token_kind = Some(token.kind())
                    }
                };
            }
        };
    }

    children
}

pub fn process_trailing_trivia(_: &mut AstProcessingContext, token: &Token) -> ast::InlineContent {
    ast::InlineContent::Text(token.trailing_trivia_text().to_string())
}

pub fn process_inline_node(
    context: &mut AstProcessingContext,
    node: &cst::Node,
) -> ast::InlineContent {
    match node {
        cst::Node::Emphasis(emphasis) => {
            ast::InlineContent::Emphasis(process_emphasis(context, emphasis))
        }
        cst::Node::Strong(strong) => ast::InlineContent::Strong(process_strong(context, strong)),
        cst::Node::Link(link) => ast::InlineContent::Link(process_link(context, link)),
        cst::Node::Image(image) => ast::InlineContent::Link(process_image(context, image)),
        cst::Node::Autolink(autolink) => {
            ast::InlineContent::Link(process_autolink(context, autolink))
        }
        cst::Node::CodeSpan(code_span) => {
            ast::InlineContent::CodeSpan(process_code_span(context, code_span))
        }
        cst::Node::Hook(hook) => ast::InlineContent::Hook(process_hook(context, hook)),
        cst::Node::Strikethrough(strikethrough) => {
            ast::InlineContent::Strikethrough(process_strikethrough(context, strikethrough))
        }
        cst::Node::Icu(icu) => ast::InlineContent::Icu(process_icu(context, icu)),
        node => unreachable!("Inline nodes cannot be block nodes. found: {:?}", node),
    }
}

//#region Markdown nodes
pub fn process_inline_token(
    context: &mut AstProcessingContext,
    token: &Token,
    include_trailing_trivia: bool,
) -> ast::InlineContent {
    match token.kind() {
        SyntaxKind::BACKSLASH_BREAK | SyntaxKind::HARD_LINE_ENDING
            if context.allow_hard_line_breaks =>
        {
            return ast::InlineContent::HardLineBreak;
        }
        SyntaxKind::HASH if context.allow_icu_pound => {
            return ast::InlineContent::IcuPound;
        }
        _ => {}
    }

    let mut text = get_text_with_replaced_references(context, &token);
    if include_trailing_trivia {
        // If the token has a trailing newline, then none of the other trivia matter and it all
        // is just replaced by a single newline.
        if token.has_trailing_newline() {
            text.push_str("\n");
        } else {
            text.push_str(&token.trailing_trivia_text());
        }
    }

    ast::InlineContent::Text(unescape(&text))
}

fn get_text_with_replaced_references(context: &mut AstProcessingContext, token: &Token) -> String {
    match token.kind() {
        SyntaxKind::DEC_CHAR_REF => {
            return process_char_ref(context, &token.text()[2..token.text().len() - 1], 10)
        }
        SyntaxKind::HEX_CHAR_REF => {
            return process_char_ref(context, &token.text()[3..token.text().len() - 1], 16)
        }
        SyntaxKind::HTML_ENTITY => return process_html_entity(context, token).into(),
        _ => token.text().to_string(),
    }
}

pub fn process_char_ref(_: &mut AstProcessingContext, ref_text: &str, radix: u32) -> String {
    u32::from_str_radix(ref_text, radix)
        .ok()
        .and_then(|c| {
            if c > 0 {
                char::from_u32(c)
            } else {
                Some(char::REPLACEMENT_CHARACTER)
            }
        })
        .unwrap_or(char::REPLACEMENT_CHARACTER)
        .into()
}

pub fn process_html_entity<'a>(_: &mut AstProcessingContext, token: &'a Token) -> &'a str {
    match get_html_entity(token.text().as_bytes()) {
        Some(replacement) => replacement,
        None => &token.text(),
    }
}

pub fn process_emphasis(
    context: &mut AstProcessingContext,
    emphasis: &cst::Emphasis,
) -> ast::Emphasis {
    ast::Emphasis(process_inline_content(context, &emphasis.children))
}

pub fn process_strong(context: &mut AstProcessingContext, strong: &cst::Strong) -> ast::Strong {
    ast::Strong(process_inline_content(context, &strong.children))
}

pub fn process_link(context: &mut AstProcessingContext, link: &cst::Link) -> ast::Link {
    let label = process_inline_content(context, &link.content);
    let destination = process_link_destination(context, &link.resource.destination);
    let title = process_link_title(context, &link.resource.title);

    ast::Link {
        kind: LinkKind::Link,
        label,
        destination,
        title,
    }
}

pub fn process_image(context: &mut AstProcessingContext, image: &cst::Image) -> ast::Link {
    let label = process_inline_content(context, &image.content);
    let destination = process_link_destination(context, &image.resource.destination);
    let title = process_link_title(context, &image.resource.title);

    ast::Link {
        kind: LinkKind::Image,
        label,
        destination,
        title,
    }
}

pub fn process_autolink(_: &mut AstProcessingContext, image: &cst::Autolink) -> ast::Link {
    let link_kind = match image.uri.kind() {
        SyntaxKind::ABSOLUTE_URI => LinkKind::Autolink,
        SyntaxKind::EMAIL_ADDRESS => LinkKind::Email,
        _ => unreachable!("Invalid syntax kind for autolink URI"),
    };

    let label = vec![ast::InlineContent::Text(image.uri.text().into())];
    let mut destination = String::new();
    if link_kind == LinkKind::Email {
        destination.push_str("mailto:");
    }
    destination.push_str(image.uri.text());

    ast::Link {
        kind: link_kind,
        label,
        destination: TextOrPlaceholder::Text(destination),
        title: None,
    }
}

fn process_link_destination(
    context: &mut AstProcessingContext,
    destination: &Option<cst::LinkDestination>,
) -> TextOrPlaceholder {
    match destination {
        Some(cst::LinkDestination::StaticLinkDestination(destination)) => {
            TextOrPlaceholder::Text(unescape(&take_tokens_verbatim_with_entities_replaced(
                context,
                &destination.url,
                false,
            )))
        }
        Some(cst::LinkDestination::DynamicLinkDestination(destination)) => {
            TextOrPlaceholder::Placeholder(process_icu(context, &destination.url))
        }
        None => TextOrPlaceholder::Text("".into()),
    }
}

fn process_link_title(
    context: &mut AstProcessingContext,
    title: &Option<cst::LinkTitle>,
) -> Option<String> {
    let Some(title) = title else {
        return None;
    };

    Some(unescape(&take_tokens_verbatim_with_entities_replaced(
        context,
        &title.title.children(),
        true,
    )))
}

pub fn process_code_span(_: &mut AstProcessingContext, code_span: &cst::CodeSpan) -> ast::CodeSpan {
    let mut text = String::new();
    // The code span can contain only empty space, which would mean there are no child tokens and
    // the whitespace is attached to the last token of the opening delimiter. Even if it's _not_
    // empty, that leading whitespace is still preserved, meaning it needs to be extracted anyway.
    if let Some(last_opener) = code_span.open_backticks.last_token() {
        text.push_str(&last_opener.trailing_trivia_text());
    }
    // Then the actual content of the span can be written.
    text.push_str(&take_tokens_as_verbatim_text(&code_span.children, true));
    // Finally, the first token of the closing delimiter will be "escaped" if the content ends with
    // a backslash, but that backslash should actually be part of the content (since escapes aren't
    // valid inside of code spans), so this check just adds that in if needed.
    if code_span
        .close_backticks
        .first_token()
        .unwrap()
        .flags()
        .is_escaped()
    {
        text.push('\\');
    }

    // Line endings within a code span are converted to single spaces.
    let mut text = text.replace('\n', " ");
    // Then, _after_ replacing, if each side ends with a space but the content is not entirely made
    // of whitespace, the first and last space are stripped.
    if text.starts_with(' ')
        && text.ends_with(' ')
        && text.contains(|c: char| !c.is_ascii_whitespace())
    {
        text = text[1..text.len() - 1].to_string();
    }
    ast::CodeSpan(text)
}
//#endregion

//#region Markdown Extensions
pub fn process_hook(context: &mut AstProcessingContext, hook: &cst::Hook) -> ast::Hook {
    ast::Hook {
        content: process_inline_content(context, &hook.content),
        name: process_hook_name(context, &hook.name),
    }
}

fn process_hook_name(_: &mut AstProcessingContext, hook_name: &cst::HookName) -> String {
    hook_name.name.text().to_string()
}

fn process_strikethrough(
    context: &mut AstProcessingContext,
    strikethrough: &cst::Strikethrough,
) -> ast::Strikethrough {
    ast::Strikethrough(process_inline_content(context, &strikethrough.content))
}

//#region ICU nodes
pub fn process_icu(context: &mut AstProcessingContext, icu: &cst::Icu) -> ast::Icu {
    let is_unsafe = matches!(icu.l_curly.kind(), SyntaxKind::UNSAFE_LCURLY);
    match &icu.value {
        cst::IcuPlaceholder::IcuVariable(variable) => {
            ast::Icu::IcuVariable(process_icu_variable(context, variable, is_unsafe))
        }
        cst::IcuPlaceholder::IcuPlural(plural) => {
            ast::Icu::IcuPlural(process_icu_plural(context, plural, is_unsafe))
        }
        cst::IcuPlaceholder::IcuDate(date) => {
            ast::Icu::IcuDate(process_icu_date(context, date, is_unsafe))
        }
        cst::IcuPlaceholder::IcuTime(time) => {
            ast::Icu::IcuTime(process_icu_time(context, time, is_unsafe))
        }
        cst::IcuPlaceholder::IcuNumber(number) => {
            ast::Icu::IcuNumber(process_icu_number(context, number, is_unsafe))
        }
    }
}

pub fn process_icu_variable(
    _: &mut AstProcessingContext,
    variable: &cst::IcuVariable,
    is_unsafe: bool,
) -> ast::IcuVariable {
    ast::IcuVariable {
        name: variable.ident.text().to_owned(),
        is_unsafe,
    }
}

pub fn process_icu_date(
    context: &mut AstProcessingContext,
    date: &cst::IcuDate,
    is_unsafe: bool,
) -> ast::IcuDate {
    ast::IcuDate {
        variable: process_icu_variable(context, &date.variable, is_unsafe),
        style: date.style.as_ref().map(process_icu_date_time_style),
        is_unsafe,
    }
}

pub fn process_icu_time(
    context: &mut AstProcessingContext,
    time: &cst::IcuTime,
    is_unsafe: bool,
) -> ast::IcuTime {
    ast::IcuTime {
        variable: process_icu_variable(context, &time.variable, is_unsafe),
        style: time.style.as_ref().map(process_icu_date_time_style),
        is_unsafe,
    }
}

pub fn process_icu_date_time_style(style: &cst::IcuDateTimeStyle) -> ast::IcuDateTimeStyle {
    ast::IcuDateTimeStyle {
        text: style.style_text.text().trim().into(),
    }
}

pub fn process_icu_number(
    context: &mut AstProcessingContext,
    number: &cst::IcuNumber,
    is_unsafe: bool,
) -> ast::IcuNumber {
    ast::IcuNumber {
        variable: process_icu_variable(context, &number.variable, is_unsafe),
        style: number.style.as_ref().map(process_icu_number_style),
        is_unsafe,
    }
}

pub fn process_icu_number_style(style: &cst::IcuNumberStyle) -> ast::IcuNumberStyle {
    ast::IcuNumberStyle {
        text: style.style_text.text().trim().into(),
    }
}

pub fn process_icu_plural(
    context: &mut AstProcessingContext,
    plural: &cst::IcuPlural,
    is_unsafe: bool,
) -> ast::IcuPlural {
    let arms = plural
        .arms
        .iter()
        .map(|arm| process_plural_arm(context, arm))
        .collect();
    ast::IcuPlural {
        variable: process_icu_variable(context, &plural.variable, is_unsafe),
        kind: IcuPluralKind::Plural,
        arms,
        is_unsafe,
    }
}

pub fn process_plural_arm(
    context: &mut AstProcessingContext,
    arm: &cst::IcuPluralArm,
) -> ast::IcuPluralArm {
    context.with_context(
        |context| context.allow_icu_pound = true,
        |context| ast::IcuPluralArm {
            selector: arm.selector.text().into(),
            content: process_inline_content(context, &arm.value.content),
        },
    )
}
//#endregion

//#region Utilities
fn take_tokens_verbatim_with_entities_replaced(
    context: &mut AstProcessingContext,
    tokens: &Vec<Token>,
    include_trailing_trivia: bool,
) -> String {
    let mut buffer = String::new();

    for (index, token) in tokens.iter().enumerate() {
        if token.kind() == SyntaxKind::LEADING_WHITESPACE {
            continue;
        }

        buffer.push_str(&get_text_with_replaced_references(context, &token));
        if index + 1 < tokens.len() || include_trailing_trivia {
            buffer.push_str(&token.trailing_trivia_text());
        }
    }

    buffer
}

fn take_tokens_as_verbatim_text(tokens: &Vec<Token>, include_trailing_trivia: bool) -> Substr {
    if tokens.is_empty() {
        return Substr::default();
    }

    let text = tokens[0].parent_text();
    let start = tokens[0].range().start;
    let end = if include_trailing_trivia {
        tokens[tokens.len() - 1]
            .text_with_trailing_trivia()
            .range()
            .end
    } else {
        tokens[tokens.len() - 1].range().end
    };

    text.substr(start..end)
}
//#endregion
