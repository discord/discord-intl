use crate::cst::*;
use crate::formatter::format_element::{FormatElement, FormatTag, LinkKind};
use crate::formatter::plain_text::{iter_node_text, iter_tokens, PlainTextFormatter};
use crate::formatter::util::{
    fast_replace_pointer, replace_entity_reference, unescaped_pointer_chunks,
};
use crate::syntax::{Syntax, SyntaxIterator, TextPointer, TokenTextIterOptions, TrimKind};
use crate::SyntaxKind;

#[derive(Default, Clone)]
pub struct FormatterContext {
    is_last_inline_node: bool,
    /// True when visiting InlineContent where trailing trivia does not need to be removed, such
    /// as within an emphasis node.
    trailing_trivia_allowed: bool,
}

impl FormatterContext {
    fn should_trim_trailing_trivia(&self) -> bool {
        self.is_last_inline_node && !self.trailing_trivia_allowed
    }
}

pub struct Formatter {
    elements: Vec<FormatElement>,
    contexts: Vec<FormatterContext>,
}

impl Formatter {
    pub fn new() -> Self {
        Self {
            elements: vec![],
            contexts: vec![FormatterContext::default()],
        }
    }

    pub fn finish(self) -> Vec<FormatElement> {
        self.elements
    }

    fn start_tag(&mut self, tag: FormatTag) {
        self.elements.push(FormatElement::StartTag(tag));
    }

    fn end_tag(&mut self) {
        self.elements.push(FormatElement::EndTag);
    }

    /// Apply temporary changes to the current context using the given `mutator`.
    fn with_context<F: FnOnce(&mut FormatterContext)>(&mut self, mutator: F) {
        self.contexts.push(self.contexts.last().unwrap().clone());
        mutator(self.contexts.last_mut().unwrap());
    }

    fn pop_context(&mut self) {
        self.contexts.pop();
    }

    fn context(&self) -> &FormatterContext {
        self.contexts.last().unwrap()
    }

    fn context_mut(&mut self) -> &mut FormatterContext {
        self.contexts.last_mut().unwrap()
    }

    /// Push a text element into the format buffer. This method automatically takes care of
    /// escaping the text content. Use [`Self::push_raw_text`] for anything that is not intended to
    /// be escaped (like code block text content).
    fn push_text(&mut self, text: TextPointer) {
        let unescaped: TextPointer = unescaped_pointer_chunks(&text).collect();
        self.elements.push(unescaped.into());
    }

    fn push_raw_text(&mut self, text: TextPointer) {
        self.elements.push(text.into())
    }
}

impl Visit for Formatter {
    fn visit_document(&mut self, node: &Document) {
        for (position, node) in node.children().with_positions() {
            match node.syntax().kind() {
                SyntaxKind::BLOCK_SPACE => {
                    if position.is_middle() {
                        self.elements.push(FormatElement::SoftLineBreak)
                    }
                }
                _ => node.visit_with(self),
            }
        }
    }

    fn visit_paragraph(&mut self, node: &Paragraph) {
        self.start_tag(FormatTag::Paragraph);
        node.visit_children_with(self);
        self.end_tag();
    }

    fn visit_thematic_break(&mut self, _: &ThematicBreak) {
        self.elements.push(FormatElement::ThematicBreak);
    }

    fn visit_any_heading(&mut self, node: &AnyHeading) {
        self.with_context(|context| context.trailing_trivia_allowed = false);
        self.start_tag(FormatTag::Heading {
            level: node.level(),
        });
        self.pop_context();
        node.visit_children_with(self);
        self.end_tag();
    }

    fn visit_any_code_block(&mut self, node: &AnyCodeBlock) {
        self.start_tag(FormatTag::CodeBlock {
            info_string: node
                .info_string()
                .and_then(|info| {
                    let text_iter = iter_node_text(&info, Default::default()).collect();
                    unescaped_pointer_chunks(&text_iter).collect()
                })
                .filter(|text| !text.trim().is_empty()),
        });
        node.visit_children_with(self);
        self.end_tag();
    }

    fn visit_inline_content(&mut self, node: &InlineContent) {
        for (position, node) in node.children().with_positions() {
            self.context_mut().is_last_inline_node = position.is_last();
            node.visit_children_with(self);
        }
    }

    fn visit_code_block_content(&mut self, node: &CodeBlockContent) {
        if node.len() == 0 {
            return;
        }

        // If a code block is indented at all, then each token is going to heave leading whitespace
        // that must get stripped. In that case, it's very _inefficient_ to collect it all into a
        // single new TextPointer, since the content will have to be copied after the first leading
        // spaces. Instead, just keep each token as a separate element.
        // _But_, if there is no leading indent, then we can concatenate them all together into a
        // single node, since all trailing trivia is preserved inside of a code block.
        let can_concatenate = node
            .get(0)
            .is_some_and(|token| token.leading_trivia_len() == 0);

        if can_concatenate {
            self.push_raw_text(
                iter_node_text(node, TokenTextIterOptions::default())
                    .collect::<TextPointer>()
                    .into(),
            );
        } else {
            for token in node.children() {
                self.elements
                    .push(token.trimmed_text_pointer(TrimKind::TrimLeading).into());
            }
        }

        // If the code block didn't end with a newline, we need to add one to it to follow
        // Markdown's convention.
        if node
            .get(node.len().saturating_sub(1))
            .is_none_or(|token| !token.full_text().ends_with("\n"))
        {
            self.push_raw_text(TextPointer::from_str("\n"));
        }
    }

    fn visit_text_span(&mut self, node: &TextSpan) {
        let mut text_options = TokenTextIterOptions::new();
        if self.context().should_trim_trailing_trivia() {
            text_options = text_options.with_trim_last_trailing();
        }
        // TODO: Figure out how to use `MinimalTextIter` here.
        let mut pointer: Option<TextPointer> = None;
        for (position, token) in node.children().with_positions() {
            let trim_kind = text_options.trim_kind(position.is_first(), position.is_last());
            match token.kind() {
                SyntaxKind::HARD_LINE_ENDING | SyntaxKind::BACKSLASH_BREAK => {
                    self.elements.push(FormatElement::HardLineBreak);
                }
                SyntaxKind::LINE_ENDING => self.elements.push(FormatElement::SoftLineBreak),
                SyntaxKind::HTML_ENTITY | SyntaxKind::HEX_CHAR_REF | SyntaxKind::DEC_CHAR_REF => {
                    // TODO: Trivia should not be allowed on references, since they will _always_
                    // be replaced and end up being a copy. The text can and should be merged into
                    // the adjacent pointers instead.
                    let leading = token.trimmed_text_pointer(TrimKind::LeadingOnly);
                    let trailing = token.trimmed_text_pointer(TrimKind::TrailingOnly);
                    let replaced = replace_entity_reference(&token);
                    match trim_kind {
                        TrimKind::TrimLeading => {
                            self.push_raw_text([replaced, trailing].iter().collect())
                        }
                        TrimKind::TrimTrailing => {
                            self.push_raw_text([leading, replaced].iter().collect())
                        }
                        TrimKind::TrimAll => self.push_text(replaced),
                        _ => self.push_text([leading, replaced, trailing].iter().collect()),
                    }
                }
                _ => {
                    let pointer = token.trimmed_text_pointer(trim_kind);
                    self.push_text(pointer);
                    continue;
                }
            }

            if let Some(pointer) = pointer.take() {
                self.push_text(pointer)
            }
        }
        if let Some(pointer) = pointer.take() {
            self.push_text(pointer)
        }
    }

    fn visit_emphasis(&mut self, node: &Emphasis) {
        self.start_tag(FormatTag::Emphasis);
        self.with_context(|context| context.trailing_trivia_allowed = true);
        node.visit_children_with(self);
        self.pop_context();
        self.end_tag();
    }

    fn visit_strong(&mut self, node: &Strong) {
        self.start_tag(FormatTag::Strong);
        self.with_context(|context| context.trailing_trivia_allowed = true);
        node.visit_children_with(self);
        self.pop_context();
        self.end_tag();
    }

    fn visit_link(&mut self, node: &Link) {
        let title = node
            .title()
            .and_then(|title| iter_node_text(&title.content(), Default::default()).collect())
            .and_then(|destination| unescaped_pointer_chunks(&destination).collect());
        let destination = node
            .destination()
            .and_then(|destination| {
                iter_node_text(&destination, TokenTextIterOptions::new().with_trim_ends()).collect()
            })
            .and_then(|destination| unescaped_pointer_chunks(&destination).collect())
            .unwrap_or_default();

        self.start_tag(FormatTag::Link {
            kind: LinkKind::Link,
            title,
            destination,
        });
        node.content().visit_with(self);
        self.end_tag()
    }

    fn visit_image(&mut self, node: &Image) {
        let title = node
            .title()
            .and_then(|title| iter_tokens(&title.content()).into_text_iter().collect());
        let destination = node
            .destination()
            .and_then(|destination| {
                iter_node_text(&destination, TokenTextIterOptions::new().with_trim_ends()).collect()
            })
            .unwrap_or_default();

        self.start_tag(FormatTag::Image {
            title,
            destination,
            alt: PlainTextFormatter::format(&node.content(), Default::default()).collect(),
        });
        self.end_tag()
    }

    fn visit_autolink(&mut self, node: &Autolink) {
        let content = node.uri_token().trimmed_text_pointer(TrimKind::TrimAll);
        let mut destination = content.clone();
        if node.is_email() {
            destination = destination.extend_front("mailto:");
        }

        self.start_tag(FormatTag::Link {
            kind: LinkKind::Link,
            destination,
            title: None,
        });
        self.push_raw_text(content.into());
        self.end_tag();
    }

    fn visit_code_span(&mut self, node: &CodeSpan) {
        self.start_tag(FormatTag::CodeSpan);
        node.visit_children_with(self);
        self.end_tag();
    }

    fn visit_strikethrough(&mut self, node: &Strikethrough) {
        self.start_tag(FormatTag::Strikethrough);
        self.with_context(|context| context.trailing_trivia_allowed = true);
        node.visit_children_with(self);
        self.pop_context();
        self.end_tag();
    }

    fn visit_code_span_content(&mut self, node: &CodeSpanContent) {
        // https://spec.commonmark.org/0.31.2/#code-spans
        // First, line endings are converted to spaces.
        // If the resulting string both begins and ends with a space character, but does not
        // consist entirely of space characters, a single space character is removed from the
        // front and back.
        //
        // We can potentially do this as a zero-copy operation, so long as there are no newline
        // characters to replace. For now, we'll accept the forced copy of collecting into a single
        // text pointer for the simplicity of applying the rules.
        let mut text = iter_node_text(
            node,
            TokenTextIterOptions::new().with_replace_entity_references(false),
        )
        .collect();
        text = fast_replace_pointer(text, b'\n', b' ');
        if text.starts_with(' ')
            && text.ends_with(' ')
            && text.contains(|c: char| !c.is_ascii_whitespace())
        {
            text = text.substr(1..text.len() - 1);
        }
        self.push_raw_text(text.into())
    }
}
