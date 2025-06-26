use super::util::{
    fast_replace_pointer, iter_node_text, iter_tokens, replace_entity_reference,
    unescaped_pointer_chunks, PlainTextFormatter,
};
use crate::compiler::{
    ArgumentNode, CodeBlockNode, CodeNode, CompiledElement, DateNode, EmphasisNode, HeadingNode,
    HookNode, IcuNode, IcuOption, LinkDestination, LinkKind, LinkNode, MarkdownNode, NumberNode,
    ParagraphNode, SelectKind, SelectableNode, StrikethroughNode, StrongNode, TimeNode,
};
use crate::cst::*;
use crate::syntax::{PositionalIterator, Syntax, TextPointer, TokenTextIterOptions, TrimKind};
use crate::SyntaxKind;

#[derive(Default, Clone)]
pub struct CompilerContext {
    is_last_inline_node: bool,
    /// True when visiting InlineContent where trailing trivia does not need to be removed, such
    /// as within an emphasis node.
    trailing_trivia_allowed: bool,
}

impl CompilerContext {
    fn should_trim_trailing_trivia(&self) -> bool {
        self.is_last_inline_node && !self.trailing_trivia_allowed
    }
}

pub struct Compiler {
    children: Vec<CompiledElement>,
    contexts: Vec<CompilerContext>,
}

impl Compiler {
    pub fn new() -> Self {
        Self {
            children: Vec::with_capacity(8),
            contexts: vec![CompilerContext::default()],
        }
    }

    pub fn finish(mut self) -> CompiledElement {
        if self.children.len() == 1
            && matches!(
                self.children[0],
                CompiledElement::List(_) | CompiledElement::BlockList(_)
            )
        {
            self.children.pop().unwrap()
        } else {
            CompiledElement::List(self.children.into_boxed_slice())
        }
    }

    /// Apply temporary changes to the current context using the given `mutator`.
    fn with_context<F: FnOnce(&mut CompilerContext)>(&mut self, mutator: F) {
        self.contexts.push(self.contexts.last().unwrap().clone());
        mutator(self.contexts.last_mut().unwrap());
    }

    fn pop_context(&mut self) {
        self.contexts.pop();
    }

    fn context(&self) -> &CompilerContext {
        self.contexts.last().unwrap()
    }

    fn context_mut(&mut self) -> &mut CompilerContext {
        self.contexts.last_mut().unwrap()
    }

    /// Push a text element into the format buffer. This method automatically takes care of
    /// escaping the text content. Use [`Self::push_raw_text`] for anything that is not intended to
    /// be escaped (like code block text content).
    fn push_text(&mut self, text: TextPointer) {
        let unescaped: TextPointer = unescaped_pointer_chunks(&text).collect();
        self.children.push(unescaped.into());
    }

    fn push_raw_text(&mut self, text: TextPointer) {
        self.children.push(text.into())
    }

    fn mark(&mut self) -> usize {
        self.children.len()
    }

    fn collect_children(&mut self, mark: usize) -> Box<[CompiledElement]> {
        Box::from_iter(self.children.drain(mark..))
    }

    fn icu_variable_name(&self, variable: &IcuVariable) -> TextPointer {
        variable
            .ident_token()
            .trimmed_text_pointer(TrimKind::TrimAll)
    }
}

impl Visit for Compiler {
    fn visit_block_document(&mut self, node: &BlockDocument) {
        for node in node.children() {
            match node.syntax().kind() {
                SyntaxKind::BLOCK_SPACE => {}
                _ => node.visit_with(self),
            }
        }
        let block_children = std::mem::take(&mut self.children);
        self.children.push(CompiledElement::BlockList(
            block_children.into_boxed_slice(),
        ))
    }

    fn visit_inline_content(&mut self, node: &InlineContent) {
        // Not making a new list within this node because the parent nodes should include them as
        // direct children. This intermediary is just useful for applying the `with_positions()`
        // iterator consistently.
        for (position, node) in node.children().with_positions() {
            self.context_mut().is_last_inline_node = position.is_last();
            node.visit_children_with(self);
        }
    }

    fn visit_paragraph(&mut self, node: &Paragraph) {
        let mark = self.mark();
        node.visit_children_with(self);
        let paragraph = ParagraphNode::new(self.collect_children(mark));
        self.children.push(paragraph.into());
    }

    fn visit_thematic_break(&mut self, _: &ThematicBreak) {
        self.children.push(MarkdownNode::ThematicBreak.into());
    }

    fn visit_any_heading(&mut self, node: &AnyHeading) {
        self.with_context(|context| context.trailing_trivia_allowed = false);
        let mark = self.mark();
        node.visit_children_with(self);
        self.pop_context();
        let heading = HeadingNode::new(node.level(), self.collect_children(mark));
        self.children.push(heading.into());
    }

    fn visit_any_code_block(&mut self, node: &AnyCodeBlock) {
        let info_string = node
            .info_string()
            .and_then(|info| {
                let text_iter = iter_node_text(&info, Default::default()).collect();
                unescaped_pointer_chunks(&text_iter).collect()
            })
            .filter(|text| !text.trim().is_empty());
        let mark = self.mark();
        node.visit_children_with(self);
        let code_block = CodeBlockNode::new(info_string, self.collect_children(mark));
        self.children.push(code_block.into());
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
                self.children
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
        // TODO: Figure out how to do this without having to force a copy.
        let mut pointer = TextPointer::default();
        for (position, token) in node.children().with_positions() {
            let trim_kind = text_options.trim_kind(position.is_first(), position.is_last());
            match token.kind() {
                SyntaxKind::HARD_LINE_ENDING | SyntaxKind::BACKSLASH_BREAK => {
                    self.push_text(std::mem::take(&mut pointer).into());
                    self.children.push(MarkdownNode::LineBreak.into());
                }
                SyntaxKind::LINE_ENDING => pointer = pointer.extend_back("\n"),
                SyntaxKind::HTML_ENTITY | SyntaxKind::HEX_CHAR_REF | SyntaxKind::DEC_CHAR_REF => {
                    // TODO: Don't allow trivia on references, since they will always be copied.
                    if trim_kind.allow_leading() {
                        pointer = pointer.extend_back(token.leading_trivia_text());
                    }
                    pointer = pointer.extend_back(&replace_entity_reference(&token));
                    if trim_kind.allow_trailing() {
                        pointer = pointer.extend_back(token.trailing_trivia_text());
                    }
                }
                _ => {
                    pointer = pointer.extend_back(&token.trimmed_text_pointer(trim_kind));
                    continue;
                }
            }
        }
        if !pointer.is_empty() {
            self.push_text(pointer.into());
        }
    }

    fn visit_emphasis(&mut self, node: &Emphasis) {
        let mark = self.mark();
        self.with_context(|context| context.trailing_trivia_allowed = true);
        node.visit_children_with(self);
        self.pop_context();
        let emphasis = EmphasisNode::new(self.collect_children(mark));
        self.children.push(emphasis.into());
    }

    fn visit_strong(&mut self, node: &Strong) {
        let mark = self.mark();
        self.with_context(|context| context.trailing_trivia_allowed = true);
        node.visit_children_with(self);
        self.pop_context();
        let strong = StrongNode::new(self.collect_children(mark));
        self.children.push(strong.into());
    }

    fn visit_link(&mut self, node: &Link) {
        let title = node
            .title()
            .and_then(|title| iter_node_text(&title.content(), Default::default()).collect())
            .and_then(|destination| unescaped_pointer_chunks(&destination).collect());
        let destination = self.collect_link_destination(node.destination().as_ref());

        let mark = self.mark();
        node.content().visit_with(self);
        let content = self.collect_children(mark);
        let link = LinkNode::new(
            LinkKind::Link,
            destination,
            title,
            None,
            Some(content.into()),
        );
        self.children.push(link.into())
    }

    fn visit_image(&mut self, node: &Image) {
        let title = node
            .title()
            .and_then(|title| iter_tokens(&title.content()).into_text_iter().collect());
        let destination = self.collect_link_destination(node.destination().as_ref());
        let alt = PlainTextFormatter::format(&node.content(), Default::default()).collect();
        let link = LinkNode::new(LinkKind::Image, destination, title, alt, None);
        self.children.push(link.into())
    }

    fn visit_autolink(&mut self, node: &Autolink) {
        let content = node.uri_token().trimmed_text_pointer(TrimKind::TrimAll);
        let destination = LinkDestination::Static(content.clone());
        let kind = if node.is_email() {
            LinkKind::Email
        } else {
            LinkKind::Link
        };
        let link = LinkNode::new(kind, destination, None, None, Some(content.into()));
        self.children.push(link.into());
    }

    fn visit_code_span(&mut self, node: &CodeSpan) {
        let mark = self.mark();
        node.visit_children_with(self);
        let code_span = CodeNode::new(self.collect_children(mark));
        self.children.push(code_span.into());
    }

    fn visit_hook(&mut self, node: &Hook) {
        let mark = self.mark();
        node.content().visit_with(self);
        let hook = HookNode::new(
            node.name()
                .name_token()
                .trimmed_text_pointer(TrimKind::TrimAll),
            self.collect_children(mark),
        );
        self.children.push(hook.into());
    }

    fn visit_strikethrough(&mut self, node: &Strikethrough) {
        let mark = self.mark();
        self.with_context(|context| context.trailing_trivia_allowed = true);
        node.visit_children_with(self);
        self.pop_context();
        let strikethrough = StrikethroughNode::new(self.collect_children(mark));
        self.children.push(strikethrough.into());
    }

    fn visit_icu_pound(&mut self, _: &IcuPound) {
        self.children.push(IcuNode::Pound.into());
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

    // NOTE: Visit has to be done on the enum here, otherwise `IcuVariable` gets visited for every
    // variable name, even inside other Icu elements. Check the CST structure in `markdown.ungram`
    // to see the nesting that occurs.
    fn visit_any_icu_placeholder(&mut self, node: &AnyIcuPlaceholder) {
        match node {
            AnyIcuPlaceholder::IcuVariable(variable) => self.children.push(
                ArgumentNode {
                    name: self.icu_variable_name(variable),
                }
                .into(),
            ),
            AnyIcuPlaceholder::IcuNumber(number) => self.children.push(
                NumberNode {
                    name: self.icu_variable_name(&number.variable()),
                    style: number.style().map(|style| {
                        style
                            .style_text_token()
                            .trimmed_text_pointer(TrimKind::TrimAll)
                    }),
                }
                .into(),
            ),
            AnyIcuPlaceholder::IcuDate(date) => self.children.push(
                DateNode {
                    name: self.icu_variable_name(&date.variable()),
                    style: date.style().map(|style| {
                        style
                            .style_text_token()
                            .trimmed_text_pointer(TrimKind::TrimAll)
                    }),
                }
                .into(),
            ),
            AnyIcuPlaceholder::IcuTime(time) => self.children.push(
                TimeNode {
                    name: self.icu_variable_name(&time.variable()),
                    style: time.style().map(|style| {
                        style
                            .style_text_token()
                            .trimmed_text_pointer(TrimKind::TrimAll)
                    }),
                }
                .into(),
            ),
            AnyIcuPlaceholder::IcuSelect(select) => {
                self.with_context(|context| context.trailing_trivia_allowed = true);
                let options = self.collect_icu_options(&select.arms());
                self.children.push(
                    SelectableNode {
                        kind: SelectKind::Select,
                        name: self.icu_variable_name(&select.variable()),
                        // TODO: implement offsets
                        offset: None,
                        options,
                    }
                    .into(),
                );
            }
            AnyIcuPlaceholder::IcuSelectOrdinal(select_ordinal) => {
                self.with_context(|context| context.trailing_trivia_allowed = true);
                let options = self.collect_icu_options(&select_ordinal.arms());
                self.children.push(
                    SelectableNode {
                        kind: SelectKind::SelectOrdinal,
                        name: self.icu_variable_name(&select_ordinal.variable()),
                        // TODO: implement offsets
                        offset: None,
                        options,
                    }
                    .into(),
                );
            }
            AnyIcuPlaceholder::IcuPlural(plural) => {
                self.with_context(|context| context.trailing_trivia_allowed = true);
                let options = self.collect_icu_options(&plural.arms());
                self.children.push(
                    SelectableNode {
                        kind: SelectKind::Plural,
                        name: self.icu_variable_name(&plural.variable()),
                        // TODO: implement offsets
                        offset: None,
                        options,
                    }
                    .into(),
                );
            }
        }
    }
}

impl Compiler {
    fn collect_icu_options(&mut self, arms: &IcuPluralArms) -> Box<[IcuOption]> {
        let mut options = Vec::with_capacity(arms.len());
        for arm in arms.children() {
            let mark = self.mark();
            arm.value().visit_with(self);
            options.push(IcuOption {
                name: arm.selector_token().trimmed_text_pointer(TrimKind::TrimAll),
                value: Box::new(CompiledElement::from(self.collect_children(mark))),
            });
        }
        options.into_boxed_slice()
    }

    fn collect_link_destination(
        &mut self,
        destination: Option<&AnyLinkDestination>,
    ) -> LinkDestination {
        let Some(destination) = destination else {
            return LinkDestination::Empty;
        };

        match destination {
            AnyLinkDestination::StaticLinkDestination(destination) => {
                let text =
                    iter_node_text(destination, TokenTextIterOptions::new().with_trim_ends())
                        .collect::<Option<TextPointer>>();
                match text {
                    Some(text) => LinkDestination::Static(
                        unescaped_pointer_chunks(&text).collect::<TextPointer>(),
                    ),
                    None => LinkDestination::Empty,
                }
            }
            AnyLinkDestination::DynamicLinkDestination(destination) => {
                destination.url().value().visit_with(self);
                let icu_node = IcuNode::from(self.children.pop().unwrap());
                LinkDestination::Dynamic(icu_node)
            }
            AnyLinkDestination::ClickHandlerLinkDestination(destination) => {
                let name = destination
                    .name_token()
                    .trimmed_text_pointer(TrimKind::TrimAll);
                LinkDestination::Handler(ArgumentNode { name }.into())
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::{CompiledElement, Compiler};
    use crate::compiler::{CompiledNode, MarkdownNode};
    use crate::{ICUMarkdownParser, SourceText, VisitWith};

    fn parse(input: &str, include_blocks: bool) -> CompiledElement {
        let mut parser = ICUMarkdownParser::new(SourceText::from(input), include_blocks);
        parser.parse();
        let document = parser.finish().to_document();
        let mut compiler = Compiler::new();
        document.visit_with(&mut compiler);
        compiler.finish()
    }
    #[test]
    fn finish_returns_plain_text_as_list() {
        let compiled = parse("hello", false);
        let CompiledElement::List(list) = compiled else {
            panic!("Expected Compiler.finish() to return a list element");
        };
        assert!(matches!(list[0], CompiledElement::Literal(_)));
    }

    #[test]
    fn finish_returns_inline_list_unchanged() {
        let compiled = parse("**foo** bar", false);
        let CompiledElement::List(list) = compiled else {
            panic!("Expected Compiler.finish() to return a list element");
        };
        assert!(matches!(
            list[0],
            CompiledElement::Node(CompiledNode::Markdown(MarkdownNode::Strong(_)))
        ));
        assert!(matches!(list[1], CompiledElement::Literal(_)));
    }

    #[test]
    fn finish_returns_single_inline_node_as_list() {
        let compiled = parse("**hello**", false);
        let CompiledElement::List(list) = compiled else {
            panic!("Expected Compiler.finish() to return a list element");
        };
        assert!(matches!(
            list[0],
            CompiledElement::Node(CompiledNode::Markdown(MarkdownNode::Strong(_)))
        ));
    }
    #[test]
    fn finish_returns_single_block_node_as_list() {
        let compiled = parse("hello", true);
        println!("{:?}", compiled);
        let CompiledElement::BlockList(list) = compiled else {
            panic!("Expected Compiler.finish() to return a list element");
        };
        assert!(matches!(
            list[0],
            CompiledElement::Node(CompiledNode::Markdown(MarkdownNode::Paragraph(_)))
        ));
    }
    #[test]
    fn finish_returns_multiple_block_nodes_as_list() {
        let compiled = parse("# hello\n\nfoo bar", true);
        println!("{:?}", compiled);
        let CompiledElement::BlockList(list) = compiled else {
            panic!("Expected Compiler.finish() to return a list element");
        };
        assert!(matches!(
            list[0],
            CompiledElement::Node(CompiledNode::Markdown(MarkdownNode::Heading(_)))
        ));
        assert!(matches!(
            list[1],
            CompiledElement::Node(CompiledNode::Markdown(MarkdownNode::Paragraph(_)))
        ));
    }
}
