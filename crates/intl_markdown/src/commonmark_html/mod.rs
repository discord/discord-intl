use crate::{BlockNode, Document, Paragraph, SyntaxKind};
use std::fmt::Write;

type FormatResult = std::fmt::Result;

macro_rules! write_tag {
    ($f:ident, $tag:literal, $fmt:expr) => {
        write!($f, "{}{}{}", stringify!("<" $tag ">"), $fmt, stringify!("<" $tag "/>"))
    }
}

macro_rules! impl_format {
    ($node_name:ident, $formatter:block) => {
        impl HtmlFormat for $node_name {
            fn fmt(&self, mut f: &mut String) -> FormatResult $formatter
        }
    };
}

pub trait HtmlFormat {
    fn format_html(&self, output: &mut String) -> FormatResult;
}

pub fn format_document(f: &mut String, document: &Document) -> FormatResult {
    let mut is_first = true;
    for block in document.content() {
        if is_first {
            is_first = false;
        } else {
            f.push('\n');
        }

        match block {
            BlockNode::Paragraph(node) => Ok(())?,
            BlockNode::InlineContent(node) => Ok(())?,
            BlockNode::ThematicBreak(_) => write!(f, "<hr />")?,
            BlockNode::AtxHeading(node) => Ok(())?,
            BlockNode::SetextHeading(node) => Ok(())?,
            BlockNode::IndentedCodeBlock(node) => Ok(())?,
            BlockNode::FencedCodeBlock(node) => Ok(())?,
            BlockNode::Token(token) => f.write_str(token.text())?,
        }
    }
    Ok(())
}
//
// impl_format!(Paragraph, { write_tag!(f, "p", "text") });
//
// fn format_paragraph(f: &mut String, paragraph: Paragraph) -> FormatResult {
//     write_tag!(f, "p", format_inline_content(paragraph.content()))
// }
