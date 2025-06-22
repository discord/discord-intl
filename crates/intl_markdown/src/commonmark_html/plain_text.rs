use crate::syntax::{
    MinimalTextIter, Syntax, SyntaxIterator, SyntaxNodeTokenIter, TextPointer, TokenTextIter,
    TokenTextIterOptions,
};
use crate::{AnyInlineNode, InlineContent, SyntaxToken, Visit, VisitWith};

pub fn iter_tokens<N: Syntax>(node: &N) -> SyntaxNodeTokenIter {
    node.syntax().iter_tokens()
}

pub fn iter_node_text<N: Syntax>(
    node: &N,
    options: TokenTextIterOptions,
) -> MinimalTextIter<TokenTextIter<SyntaxNodeTokenIter>> {
    iter_tokens(node)
        .into_text_iter()
        .with_options(options)
        .minimal()
}

pub struct PlainTextFormatter {
    pointers: Vec<TextPointer>,
    text_options: TokenTextIterOptions,
}

impl PlainTextFormatter {
    fn new(text_options: TokenTextIterOptions) -> Self {
        Self {
            pointers: Vec::with_capacity(4),
            text_options,
        }
    }

    pub fn format<N: VisitWith<Self>>(
        node: &N,
        text_options: TokenTextIterOptions,
    ) -> MinimalTextIter<std::vec::IntoIter<TextPointer>> {
        let mut f = Self::new(text_options);
        node.visit_with(&mut f);
        f.finish()
    }

    pub fn finish(self) -> MinimalTextIter<std::vec::IntoIter<TextPointer>> {
        MinimalTextIter::new(self.pointers.into_iter())
    }

    fn collect_token_pointers(&mut self, tokens: impl Iterator<Item = SyntaxToken>) {
        for token in tokens {
            self.pointers.push(token.text_pointer().clone())
        }
    }
}

impl Visit for PlainTextFormatter {
    /// Processes the list of inline elements by taking only the visual text that appears within each
    /// item. For example, a `Strong` element like `**hello**` would just be written as `hello` rather
    /// than `<strong>hello</strong>` as it might in an HTML format.
    fn visit_inline_content(&mut self, node: &InlineContent) {
        for (position, child) in node.children().with_positions() {
            match child {
                AnyInlineNode::TextSpan(node) => {
                    // Text spans are the only non-delimited kinds of text possible in inline
                    // content, so they're the only ones that need to potentially apply trims. All
                    // other trivia is preserved since it is important within delimiters.
                    for (span_position, text_child) in node.children().with_positions() {
                        self.pointers.push(text_child.trimmed_text_pointer(
                            self.text_options.trim_kind(
                                position.is_first() && span_position.is_first(),
                                position.is_last() && span_position.is_last(),
                            ),
                        ))
                    }
                }
                AnyInlineNode::Autolink(node) => {
                    self.pointers.push(node.uri_token().text_pointer().clone());
                }
                AnyInlineNode::CodeSpan(node) => {
                    self.collect_token_pointers(node.content().children());
                }
                AnyInlineNode::Link(node) => node.content().visit_with(self),
                AnyInlineNode::Image(node) => node.content().visit_with(self),
                AnyInlineNode::Hook(node) => node.content().visit_with(self),
                AnyInlineNode::Emphasis(node) => node.content().visit_with(self),
                AnyInlineNode::Strong(node) => node.content().visit_with(self),
                AnyInlineNode::Strikethrough(node) => node.content().visit_with(self),
                AnyInlineNode::Icu(_) => todo!(),
            }
        }
    }
}
