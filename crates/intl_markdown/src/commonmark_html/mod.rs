mod util;

use crate::ast::util::{escape_body_text, escape_href};
use crate::commonmark_html::util::{fast_replace, unescaped_chunks};
use crate::cst::*;
use crate::html_entities::get_html_entity;
use crate::syntax::{ContiguousTokenChunksIteratorOptions, Syntax, SyntaxTokenChildren, TrimKind};
use crate::{SyntaxKind, SyntaxToken};
use intl_markdown_macros::impl_format;
use std::fmt::Write;

type FormatResult<T = ()> = Result<T, std::fmt::Error>;

macro_rules! write_tag {
    ($f:ident, $tag:literal, $fmt:expr) => {{
        write!($f, "<{}>", $tag)?;
        HtmlFormat::fmt($fmt, $f)?;
        write!($f, "</{}>", $tag)
    }};
    ($f:ident, $tag:ident, $fmt:expr) => {{
        write!($f, "<{}>", $tag)?;
        HtmlFormat::fmt($fmt, $f)?;
        write!($f, "</{}>", $tag)
    }};
    (&mut $f:ident, $tag:ident, $fmt:expr) => {{
        write!(&mut $f, "<{}>", $tag)?;
        HtmlFormat::fmt($fmt, &mut $f)?;
        write!(&mut $f, "</{}>", $tag)
    }};
}

//#region HtmlFormat Trait
pub trait HtmlFormat {
    fn fmt(&self, f: &mut impl Write) -> FormatResult;
}

impl<T: HtmlFormat> HtmlFormat for TokenOr<T> {
    fn fmt(&self, f: &mut impl Write) -> FormatResult {
        match self {
            TokenOr::Token(token) => token.fmt(f),
            TokenOr::Or(node) => node.fmt(f),
        }
    }
}

impl<F: Fn(&mut dyn Write) -> FormatResult> HtmlFormat for F {
    fn fmt(&self, f: &mut impl Write) -> FormatResult {
        self(f)
    }
}

fn buffered_write<F>(size_hint: Option<usize>, func: F) -> FormatResult<String>
where
    F: FnOnce(&mut dyn Write) -> FormatResult,
{
    let mut buffer = String::with_capacity(size_hint.unwrap_or(8));
    func(&mut buffer)?;
    Ok(buffer)
}

fn token_list(
    tokens: SyntaxTokenChildren,
    options: ContiguousTokenChunksIteratorOptions,
) -> FormatTokenList {
    FormatTokenList { tokens, options }
}

struct FormatTokenList<'a> {
    tokens: SyntaxTokenChildren<'a>,
    options: ContiguousTokenChunksIteratorOptions,
}

impl HtmlFormat for FormatTokenList<'_> {
    fn fmt(&self, f: &mut impl Write) -> FormatResult {
        let mut index = 0;
        for token in self.tokens.clone() {
            let token_text = match self.options.trim_kind(index) {
                TrimKind::TrimNone => token.full_text(),
                TrimKind::TrimLeading => token.text_with_trailing_trivia(),
                TrimKind::TrimTrailing => token.text_with_leading_trivia(),
                TrimKind::TrimAll => token.text(),
            };
            let token_text = if self.options.html_entities() {
                &escape_body_text(token_text)
            } else {
                token_text
            };
            if self.options.unescape() {
                for chunk in unescaped_chunks(token_text) {
                    f.write_str(chunk)?;
                }
            } else {
                f.write_str(token_text)?;
            }
            index += 1;
        }
        Ok(())
    }
}

fn html_entity(text: &str) -> FormatHtmlEntity {
    FormatHtmlEntity(text)
}

struct FormatHtmlEntity<'a>(&'a str);
impl HtmlFormat for FormatHtmlEntity<'_> {
    fn fmt(&self, f: &mut impl Write) -> FormatResult {
        match get_html_entity(self.0.as_bytes()) {
            Some(entity) => entity.fmt(f),
            None => {
                f.write_str("&amp;")?;
                f.write_str(&self.0[1..])
            }
        }
    }
}

fn format_token(token: &SyntaxToken) -> FormatToken {
    FormatToken {
        token,
        include_trailing_trivia: true,
    }
}

struct FormatToken<'a> {
    token: &'a SyntaxToken,
    include_trailing_trivia: bool,
}

impl HtmlFormat for FormatToken<'_> {
    fn fmt(&self, f: &mut impl Write) -> FormatResult {
        match self.token.kind() {
            SyntaxKind::LANGLE => "&lt;".fmt(f),
            SyntaxKind::RANGLE => "&gt;".fmt(f),
            SyntaxKind::DOUBLE_QUOTE => "&quot;".fmt(f),
            SyntaxKind::AMPER => "&amp;".fmt(f),
            SyntaxKind::HTML_ENTITY => html_entity(self.token.text()).fmt(f),
            // This doesn't seem to happen in the spec? But does in some other
            // implementations.
            // SyntaxKind::QUOTE => "&#39;".fmt(),
            _ => unescaped(self.token.text()).fmt(f),
        }?;
        if self.include_trailing_trivia {
            f.write_str(self.token.trailing_trivia_text())?;
        }
        Ok(())
    }
}

impl HtmlFormat for &str {
    fn fmt(&self, f: &mut impl Write) -> FormatResult {
        f.write_str(self)
    }
}

impl<T: HtmlFormat> HtmlFormat for Option<T> {
    fn fmt(&self, f: &mut impl Write) -> FormatResult {
        match self {
            Some(value) => HtmlFormat::fmt(value, f),
            None => Ok(()),
        }
    }
}

fn unescaped(text: &str) -> UnescapedText {
    UnescapedText { text }
}

pub struct UnescapedText<'a> {
    text: &'a str,
}

impl HtmlFormat for UnescapedText<'_> {
    fn fmt(&self, f: &mut impl Write) -> FormatResult {
        for chunk in unescaped_chunks(self.text) {
            f.write_str(chunk)?;
        }
        Ok(())
    }
}
//#endregion

pub fn format_document(f: &mut String, document: &Document) -> FormatResult {
    let last_index = document.len() - 1;
    for (index, block) in document.children().enumerate() {
        if index < last_index {
            f.push('\n');
        }

        match &block {
            AnyBlockNode::Paragraph(node) => HtmlFormat::fmt(node, f)?,
            AnyBlockNode::InlineContent(node) => HtmlFormat::fmt(node, f)?,
            AnyBlockNode::ThematicBreak(node) => HtmlFormat::fmt(node, f)?,
            AnyBlockNode::Heading(node) => HtmlFormat::fmt(node, f)?,
            AnyBlockNode::CodeBlock(node) => HtmlFormat::fmt(node, f)?,
        }
    }
    Ok(())
}

impl_format!(HtmlFormat, SyntaxToken | f | { format_token(self).fmt(f) });
impl_format!(
    HtmlFormat,
    Paragraph | f | write_tag!(f, "p", &self.content())
);
impl_format!(
    HtmlFormat,
    InlineContent | f | {
        let last_index = self.len().saturating_sub(1);
        for (index, child) in self.children().enumerate() {
            if index < last_index {
                HtmlFormat::fmt(&child, f)?;
            } else {
                match child {
                    _ => HtmlFormat::fmt(&child, f)?,
                }
            }
        }
        Ok(())
    }
);
impl_format!(HtmlFormat, ThematicBreak | f | f.write_str("<hr />"));
impl_format!(
    HtmlFormat,
    AnyHeading | f | {
        match self {
            AnyHeading::AtxHeading(node) => HtmlFormat::fmt(node, f),
            AnyHeading::SetextHeading(node) => HtmlFormat::fmt(node, f),
        }
    }
);
impl_format!(
    HtmlFormat,
    AtxHeading | f | {
        let tag = match self.level() {
            1 => "h1",
            2 => "h2",
            3 => "h3",
            4 => "h4",
            5 => "h5",
            6 => "h6",
            level => panic!("Invalid ATX Heading level {}", level),
        };
        write_tag!(f, tag, &self.content())
    }
);
impl_format!(
    HtmlFormat,
    SetextHeading | f | {
        let tag = match self.underline().level() {
            1 => "h1",
            2 => "h2",
            level => panic!("Invalid Setext Heading level {}", level),
        };
        write_tag!(f, tag, &self.content())
    }
);
impl_format!(
    HtmlFormat,
    AnyCodeBlock | f | {
        match self {
            AnyCodeBlock::IndentedCodeBlock(node) => HtmlFormat::fmt(node, f),
            AnyCodeBlock::FencedCodeBlock(node) => HtmlFormat::fmt(node, f),
        }
    }
);
impl_format!(
    HtmlFormat,
    IndentedCodeBlock | f | {
        write!(f, "<pre><code>")?;
        self.content().fmt(f)?;
        write!(f, "</code></pre>")
    }
);
impl_format!(
    HtmlFormat,
    FencedCodeBlock | f | {
        if let Some(language) = self.info_string_token() {
            write!(f, "<pre><code class=\"language-{}\">", language.text())?;
        } else {
            write!(f, "<pre><code>")?;
        }
        self.content().fmt(f)?;
        write!(f, "</code></pre>")
    }
);
impl_format!(
    HtmlFormat,
    CodeBlockContent | f | {
        token_list(
            self.children(),
            ContiguousTokenChunksIteratorOptions::trim_all_leading().with_unescape(false),
        )
        .fmt(f)?;
        // Code blocks always have a trailing newline, so add it if it isn't already present.
        match self.get(self.len() - 1) {
            Some(last_child) => {
                if !last_child.text_with_trailing_trivia().ends_with('\n') {
                    f.write_str("\n")?;
                }
            }
            None => f.write_str("\n")?,
        }
        Ok(())
    }
);

impl_format!(
    HtmlFormat,
    AnyInlineNode | f | {
        match self {
            AnyInlineNode::TextSpan(span) => HtmlFormat::fmt(span, f),
            AnyInlineNode::EntityReference(node) => HtmlFormat::fmt(node, f),
            AnyInlineNode::Emphasis(node) => HtmlFormat::fmt(node, f),
            AnyInlineNode::Strong(node) => HtmlFormat::fmt(node, f),
            AnyInlineNode::Link(node) => HtmlFormat::fmt(node, f),
            AnyInlineNode::Image(node) => HtmlFormat::fmt(node, f),
            AnyInlineNode::Autolink(node) => HtmlFormat::fmt(node, f),
            AnyInlineNode::CodeSpan(node) => HtmlFormat::fmt(node, f),
            AnyInlineNode::Hook(node) => HtmlFormat::fmt(node, f),
            AnyInlineNode::Strikethrough(node) => HtmlFormat::fmt(node, f),
            AnyInlineNode::Icu(node) => HtmlFormat::fmt(node, f),
        }
    }
);

impl_format!(HtmlFormat, TextSpan | f | self.text_token().fmt(f));

impl_format!(HtmlFormat, EntityReference | f | self.token().fmt(f));

impl_format!(
    HtmlFormat,
    Emphasis | f | write_tag!(f, "em", &self.content())
);
impl_format!(
    HtmlFormat,
    Strong | f | write_tag!(f, "strong", &self.content())
);
impl_format!(
    HtmlFormat,
    Strikethrough | f | write_tag!(f, "del", &self.content())
);
impl_format!(HtmlFormat, Link | f | AnyLink::from(self).fmt(f));
impl_format!(HtmlFormat, Autolink | f | AnyLink::from(self).fmt(f));
impl_format!(HtmlFormat, Image | f | AnyLink::from(self).fmt(f));
impl_format!(HtmlFormat, Hook | f | write!(f, "unimplemented!"));
impl_format!(HtmlFormat, Icu | f | write!(f, "unimplemented!"));
impl_format!(
    HtmlFormat,
    CodeSpan | f | write_tag!(f, "code", &self.content())
);
impl_format!(
    HtmlFormat,
    CodeSpanContent | f | {
        // https://spec.commonmark.org/0.31.2/#code-span
        // "If the resulting string both begins and ends with a space character, but does not
        // consist entirely of space characters, a single space character is removed from the front
        // and back."
        // This can only be applied _after_ writing the text, since it relies on the newline space
        // conversion. It's possible to do this by introspecting tokens ahead of time, but that's
        // generally going to be more costly than just writing to a temporary buffer, checking for
        // spaces in the result, and removing them from there.
        let mut buffer = buffered_write(Some(self.len() * 2), |mut f| {
            token_list(self.children(), Default::default()).fmt(&mut f)
        })?;

        fast_replace(buffer.as_mut_str(), b'\n', b' ');
        if buffer.starts_with(' ')
            && buffer.ends_with(' ')
            && buffer.contains(|c: char| !c.is_ascii_whitespace())
        {
            f.write_str(&buffer[1..buffer.len() - 1])
        } else {
            f.write_str(&buffer)
        }
    }
);

impl HtmlFormat for AnyLink {
    fn fmt(&self, f: &mut impl Write) -> FormatResult {
        let title = match self.title() {
            Some(node) => {
                let title = buffered_write(None, |mut f| {
                    token_list(
                        node.content().children(),
                        ContiguousTokenChunksIteratorOptions::trim_all(),
                    )
                    .fmt(&mut f)
                })?;
                Some(escape_body_text(&title))
            }
            None => None,
        };

        let href = buffered_write(Some(8), |mut f| {
            if self.is_email() {
                f.write_str("mailto:")?;
            }
            self.destination().fmt(&mut f)
        })?;

        match self {
            AnyLink::Image(_) => {
                write!(f, "<img src=\"{href}\" alt=\"")?;
                self.label().fmt(f)?;
                if let Some(title) = title {
                    write!(f, "\" title=\"{}\"", title)?;
                }
                write!(f, "\" />")
            }
            _ => {
                write!(f, "<a href=\"{href}\"")?;
                if let Some(title) = title {
                    write!(f, " title=\"{}\"", title)?;
                }
                write!(f, ">")?;
                self.label().fmt(f)?;
                write!(f, "</a>")
            }
        }
    }
}

impl_format!(
    HtmlFormat,
    AnyLinkDestination | f | {
        match self {
            AnyLinkDestination::StaticLinkDestination(node) => {
                write!(f, "{}", escape_href(&node.destination_token().text()))
            }
            AnyLinkDestination::DynamicLinkDestination(icu) => icu.url().fmt(f),
            AnyLinkDestination::ClickHandlerLinkDestination(handler) => handler.name_token().fmt(f),
        }
    }
);
