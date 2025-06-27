use super::util::{encode_body_text, encode_href};
use crate::compiler::{
    CompiledElement, CompiledNode, IcuNode, LinkDestination, LinkKind, LinkNode, MarkdownNode,
    SelectKind,
};
use crate::syntax::{PositionalIterator, TextPointer};

pub struct HtmlFormatter {
    result: String,
}

impl HtmlFormatter {
    pub fn new() -> Self {
        Self {
            result: String::new(),
        }
    }

    pub fn finish(self) -> String {
        self.result
    }

    pub fn format_element(&mut self, element: &CompiledElement) {
        match element {
            CompiledElement::BlockList(list) => self.format_block_list(list),
            CompiledElement::List(list) => self.format_list(list),
            CompiledElement::Node(CompiledNode::Markdown(node)) => self.format_markdown_node(node),
            CompiledElement::Node(CompiledNode::Icu(node)) => self.format_icu_node(node),
            CompiledElement::Literal(literal) => self.format_literal(literal),
        }
    }

    pub fn format_block_list(&mut self, list: &[CompiledElement]) {
        for (position, element) in list.iter().with_positions() {
            if !position.is_first() {
                self.result.push('\n');
            }
            self.format_element(element);
        }
    }

    pub fn format_list(&mut self, list: &[CompiledElement]) {
        for element in list.iter() {
            self.format_element(element)
        }
    }

    pub fn format_markdown_node(&mut self, node: &MarkdownNode) {
        match node {
            MarkdownNode::Paragraph(paragraph) => {
                self.result.push_str("<p>");
                self.format_element(&paragraph.content);
                self.result.push_str("</p>");
            }
            MarkdownNode::CodeBlock(block) => {
                if let Some(info_string) = &block.info_string {
                    // If this exists, there will be at least some text.
                    let language = info_string.split_ascii_whitespace().next().unwrap();
                    self.result.reserve(23 + language.len());
                    self.result.push_str("<pre><code class=\"language-");
                    push_str_iter(&mut self.result, encode_body_text(language));
                    self.result.push_str("\">")
                } else {
                    self.result.push_str("<pre><code>");
                }
                self.format_element(&block.content);
                self.result.push_str("</code></pre>");
            }
            MarkdownNode::Heading(heading) => {
                self.result
                    .push_str(HEADER_OPEN_TAGS[heading.level as usize - 1]);
                self.format_element(&heading.content);
                self.result
                    .push_str(HEADER_CLOSE_TAGS[heading.level as usize - 1]);
            }
            MarkdownNode::ThematicBreak => self.result.push_str("<hr />"),
            MarkdownNode::Strong(strong) => {
                self.result.push_str("<strong>");
                self.format_element(&strong.content);
                self.result.push_str("</strong>");
            }
            MarkdownNode::Emphasis(emphasis) => {
                self.result.push_str("<em>");
                self.format_element(&emphasis.content);
                self.result.push_str("</em>");
            }
            MarkdownNode::Strikethrough(strikethrough) => {
                self.result.push_str("<del>");
                self.format_element(&strikethrough.content);
                self.result.push_str("</del>");
            }
            MarkdownNode::Code(code) => {
                self.result.push_str("<code>");
                self.format_element(&code.content);
                self.result.push_str("</code>");
            }
            MarkdownNode::LineBreak => {
                self.result.push_str("<br />\n");
            }
            MarkdownNode::Link(link) => {
                let title = link.title.as_deref().map(encode_body_text);
                let alt = link.alt.as_deref().map(encode_body_text);

                match link.kind {
                    LinkKind::Link | LinkKind::Email => {
                        self.result.push_str("<a href=\"");
                        self.format_link_destination(&link);
                        self.result.push('\"');
                        if let Some(title) = title {
                            self.result.push_str(" title=\"");
                            push_str_iter(&mut self.result, title);
                            self.result.push('\"');
                        }
                        self.result.push('>');
                        self.format_element(&link.content);
                        self.result.push_str("</a>");
                    }
                    LinkKind::Image => {
                        self.result.push_str("<img src=\"");
                        self.format_link_destination(&link);
                        self.result.push('\"');
                        self.result.push_str(" alt=\"");
                        if let Some(alt) = alt {
                            push_str_iter(&mut self.result, alt);
                        }
                        self.result.push('\"');
                        if let Some(title) = title {
                            self.result.push_str(" title=\"");
                            push_str_iter(&mut self.result, title);
                            self.result.push('\"');
                        }
                        self.result.push_str(" />");
                    }
                }
            }
            MarkdownNode::Hook(hook) => {
                self.result.push_str("<a href=\"");
                self.result.push_str(&hook.name);
                self.result.push_str("\">");
                self.format_element(&hook.content);
                self.result.push_str("</a>");
            }
        }
    }

    pub fn format_link_destination(&mut self, link: &LinkNode) {
        match &link.destination {
            LinkDestination::Static(text) => {
                if matches!(link.kind, LinkKind::Email) {
                    self.result.push_str("mailto:");
                }
                push_str_iter(&mut self.result, encode_href(text));
            }
            LinkDestination::Dynamic(dynamic) => self.format_icu_node(dynamic),
            // No encoding on handlers since they are intended to be replaced by something downstream.
            LinkDestination::Handler(handler) => self.result.push_str(&handler.name),
            LinkDestination::Empty => {}
        }
    }

    pub fn format_icu_node(&mut self, node: &IcuNode) {
        match node {
            IcuNode::Argument(argument) => {
                self.result.push('{');
                self.result.push_str(&argument.name);
                self.result.push('}');
            }
            IcuNode::Number(number) => {
                self.result.push('{');
                self.result.push_str(&number.name);
                self.result.push_str(", number");
                if let Some(style) = &number.style {
                    self.result.push_str(", ");
                    self.result.push_str(&style);
                }
                self.result.push('}');
            }
            IcuNode::Date(date) => {
                self.result.push('{');
                self.result.push_str(&date.name);
                self.result.push_str(", date");
                if let Some(style) = &date.style {
                    self.result.push_str(", ");
                    self.result.push_str(&style);
                }
                self.result.push('}');
            }
            IcuNode::Time(time) => {
                self.result.push('{');
                self.result.push_str(&time.name);
                self.result.push_str(", time");
                if let Some(style) = &time.style {
                    self.result.push_str(", ");
                    self.result.push_str(&style);
                }
                self.result.push('}');
            }
            IcuNode::Selectable(selectable) => {
                self.result.push('{');
                self.result.push_str(&selectable.name);
                self.result.push_str(match selectable.kind {
                    SelectKind::Plural => ", plural,",
                    SelectKind::Select => ", select,",
                    SelectKind::SelectOrdinal => ", selectordinal,",
                });
                if let Some(offset) = &selectable.offset {
                    self.result.push_str(&offset);
                    self.result.push_str(",");
                }
                for option in &selectable.options {
                    self.result.push(' ');
                    self.result.push_str(&option.name);
                    self.result.push_str(" {");
                    self.format_element(&option.value);
                    self.result.push_str("}");
                }
                self.result.push_str("}");
            }
            IcuNode::Pound => self.result.push('#'),
        }
    }

    pub fn format_literal(&mut self, literal: &TextPointer) {
        push_str_iter(&mut self.result, encode_body_text(&literal))
    }
}

const HEADER_OPEN_TAGS: [&str; 6] = ["<h1>", "<h2>", "<h3>", "<h4>", "<h5>", "<h6>"];
const HEADER_CLOSE_TAGS: [&str; 6] = ["</h1>", "</h2>", "</h3>", "</h4>", "</h5>", "</h6>"];

fn push_str_iter<'a>(buffer: &mut String, iter: impl Iterator<Item = &'a str>) {
    for chunk in iter {
        buffer.push_str(chunk);
    }
}
