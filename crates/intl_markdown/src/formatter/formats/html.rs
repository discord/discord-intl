use crate::formatter::format_element::{FormatElement, FormatTag, LinkKind};
use crate::formatter::util::{encode_body_text, encode_href};

pub fn format_elements(mut buffer: &mut String, elements: &Vec<FormatElement>) {
    let mut close_tag_stack = vec![];
    for element in elements {
        match element {
            FormatElement::StartTag(tag) => {
                let end_tag = format_tag(&mut buffer, tag);
                close_tag_stack.push(end_tag);
            }
            FormatElement::EndTag => {
                let Some(close_tag) = close_tag_stack.pop() else {
                    panic!("Closed a tag with no remaining opener");
                };
                buffer.push_str(close_tag);
            }
            FormatElement::ThematicBreak => buffer.push_str("<hr />"),
            FormatElement::HardLineBreak => buffer.push_str("<br />\n"),
            FormatElement::SoftLineBreak => buffer.push_str("\n"),
            FormatElement::Text(text) => push_str_iter(buffer, encode_body_text(&text)),
        }
    }
}

const HEADER_OPEN_TAGS: [&str; 6] = ["<h1>", "<h2>", "<h3>", "<h4>", "<h5>", "<h6>"];
const HEADER_CLOSE_TAGS: [&str; 6] = ["</h1>", "</h2>", "</h3>", "</h4>", "</h5>", "</h6>"];

fn format_tag(buffer: &mut String, tag: &FormatTag) -> &'static str {
    match tag {
        FormatTag::Paragraph => {
            buffer.push_str("<p>");
            "</p>"
        }
        FormatTag::Heading { level } => {
            buffer.push_str(HEADER_OPEN_TAGS[*level as usize - 1]);
            HEADER_CLOSE_TAGS[*level as usize - 1]
        }
        FormatTag::CodeBlock { info_string, .. } => {
            if let Some(info_string) = info_string {
                // If this exists, there will be at least some text.
                let language = info_string.split_ascii_whitespace().next().unwrap();
                buffer.reserve(23 + language.len());
                buffer.push_str("<pre><code class=\"language-");
                push_str_iter(buffer, encode_body_text(language));
                buffer.push_str("\">")
            } else {
                buffer.push_str("<pre><code>");
            }
            "</code></pre>"
        }
        FormatTag::Link {
            kind,
            destination,
            title,
        } => match kind {
            LinkKind::Link => {
                buffer.reserve(11 + destination.len() + title.as_ref().map_or(0, |t| t.len() + 9));
                buffer.push_str("<a href=\"");
                push_str_iter(buffer, encode_href(destination));
                buffer.push_str("\"");
                if let Some(title) = title {
                    buffer.push_str(" title=\"");
                    push_str_iter(buffer, encode_body_text(title));
                    buffer.push_str("\"");
                }
                buffer.push_str(">");
                "</a>"
            }
            LinkKind::Hook => {
                buffer.reserve(11 + destination.len());
                buffer.push_str("<a href=\"");
                buffer.push_str(destination);
                buffer.push_str("\">");
                "</a>"
            }
        },
        FormatTag::Image {
            destination,
            title,
            alt,
        } => {
            buffer.reserve(
                18 + destination.len()
                    + alt.as_ref().map_or(0, |alt| alt.len())
                    + title.as_ref().map_or(0, |t| t.len() + 7),
            );
            buffer.push_str("<img src=\"");
            push_str_iter(buffer, encode_href(destination));
            buffer.push_str("\"");
            // The `alt` attribute is always added for images, even if there is none.
            buffer.push_str(" alt=\"");
            if let Some(alt) = alt {
                push_str_iter(buffer, encode_body_text(alt));
            }
            buffer.push_str("\"");
            if let Some(title) = title {
                buffer.push_str(" title=\"");
                push_str_iter(buffer, encode_body_text(title));
                buffer.push_str("\"");
            }
            buffer.push_str(" />");
            // Image tags are self-closing in HTML
            ""
        }
        FormatTag::Emphasis => {
            buffer.push_str("<em>");
            "</em>"
        }
        FormatTag::Strong => {
            buffer.push_str("<strong>");
            "</strong>"
        }
        FormatTag::Strikethrough => {
            buffer.push_str("<del>");
            "</del>"
        }
        FormatTag::CodeSpan => {
            buffer.push_str("<code>");
            "</code>"
        }
    }
}

fn push_str_iter<'a>(buffer: &mut String, iter: impl Iterator<Item = &'a str>) {
    for chunk in iter {
        buffer.push_str(chunk);
    }
}
