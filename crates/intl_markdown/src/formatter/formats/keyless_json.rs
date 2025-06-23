use crate::formatter::format_element::{FormatElement, FormatTag, LinkKind};
use crate::formatter::util::encode_json_string_literal;

fn write_tag_start(output: &mut String, name: &str) {
    output.reserve(7 + name.len());
    output.push_str("[8,\"");
    push_str_iter(output, encode_json_string_literal(name));
    output.push('\"');
}

fn write_empty_tag(output: &mut String, name: &str) {
    output.reserve(5 + name.len());
    output.push_str("[8,\"");
    output.push_str(name);
    output.push(']');
}

const HEADER_OPEN_TAGS: [&str; 6] = ["$h1", "$h2", "$h3", "$h4", "$h5", "$h6"];

struct KeylessJsonFormatter<'a> {
    buffer: &'a mut String,
    close_tag_stack: Vec<&'a FormatTag>,
}

impl<'a> KeylessJsonFormatter<'a> {
    fn new(buffer: &'a mut String) -> Self {
        Self {
            buffer,
            close_tag_stack: vec![],
        }
    }

    fn format_elements(&mut self, elements: &'a Vec<FormatElement>) {
        // Single elements don't get wrapped in an array. It's assumed this is always a Text element.
        if elements.len() == 1 {
            self.format_single_element(&elements[0]);
            return;
        }

        self.buffer.reserve(elements.len() * 2);
        self.buffer.push('[');
        for (index, element) in elements.iter().enumerate() {
            self.format_single_element(element);
            if !matches!(elements.get(index + 1), Some(FormatElement::EndTag) | None) {
                self.buffer.push_str(",");
            }
        }
        self.buffer.push(']');
    }

    fn format_single_element(&mut self, element: &'a FormatElement) {
        match element {
            FormatElement::StartTag(tag) => {
                self.format_tag_start(tag);
                self.close_tag_stack.push(tag);
            }
            FormatElement::EndTag => {
                let close_tag = self
                    .close_tag_stack
                    .pop()
                    .expect("Closed a tag with no remaining opener");
                self.format_tag_close(close_tag);
            }
            FormatElement::ThematicBreak => write_empty_tag(self.buffer, "$hr"),
            FormatElement::HardLineBreak => write_empty_tag(self.buffer, "$br"),
            // Soft line breaks don't mean anything in the compiled AST. If they end up being
            // important, they should be encoded into text elements with `\n` as content.
            FormatElement::SoftLineBreak => {}
            FormatElement::Text(text) => {
                self.buffer.push('\"');
                // No special encoding is done for text. It's expected that that reader supports
                // Unicode strings and
                push_str_iter(self.buffer, encode_json_string_literal(text));
                self.buffer.push('\"');
            }
        }
    }
    fn format_tag_start(&mut self, tag: &FormatTag) {
        let tag_name = match tag {
            FormatTag::Heading { level } => HEADER_OPEN_TAGS[*level as usize - 1],
            FormatTag::CodeBlock { .. } => "$codeBlock",
            FormatTag::Paragraph => "$p",
            FormatTag::Emphasis => "$i",
            FormatTag::Strong => "$b",
            FormatTag::Strikethrough => "$del",
            FormatTag::CodeSpan => "$code",
            FormatTag::Link {
                kind: LinkKind::Link,
                ..
            } => "$link",
            // The destination of a hook is it's name, which is used as the tag name in the
            // compiled AST.
            // `$[content](fooBar)` => `[8,"fooBar",["content"],[]]`
            FormatTag::Link {
                kind: LinkKind::Hook,
                destination,
                ..
            } => destination,
            // Images are just links in the compiled AST.
            FormatTag::Image { .. } => "$link",
        };

        write_tag_start(self.buffer, tag_name);
    }

    fn format_tag_close(&mut self, close_tag: &FormatTag) {
        match close_tag {
            // Links/images have their destination as the `control` segment of the compiled node.
            FormatTag::Image { destination, .. }
            | FormatTag::Link {
                kind: LinkKind::Link,
                destination,
                ..
            } => {
                // TODO: No support for link titles yet.
                self.buffer.push_str(",[\"");
                push_str_iter(self.buffer, encode_json_string_literal(destination));
                self.buffer.push_str("\"]]");
            }
            // Everything else is just closing the element.
            _ => self.buffer.push(']'),
        }
    }
}

pub fn format_elements(buffer: &mut String, elements: &Vec<FormatElement>) {
    KeylessJsonFormatter::new(buffer).format_elements(elements);
}

fn push_str_iter<'a>(buffer: &mut String, iter: impl Iterator<Item = &'a str>) {
    for chunk in iter {
        buffer.push_str(chunk);
    }
}
