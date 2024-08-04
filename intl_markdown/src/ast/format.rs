use std::fmt::Write;

use crate::ast::{
    BlockNode, CodeBlock, CodeSpan, Document, Emphasis, Heading, Hook, Icu, IcuDate,
    IcuDateTimeStyle, IcuNumber, IcuNumberStyle, IcuPlural, IcuPluralArm, IcuPluralKind, IcuTime,
    IcuVariable, InlineContent, Link, LinkKind, Paragraph, Strikethrough, Strong,
    TextOrPlaceholder,
};

use super::util::{escape_body_text, escape_href, format_plain_text};

macro_rules! write {
    ($dst:expr, [$($arg:expr),+ $(,)?]) => {{
        $(
            let _ = $arg.fmt(&mut $dst)?;
        )*
        Ok(())
    }}
}

pub(crate) type FormatResult<T> = Result<T, std::fmt::Error>;

trait FormatHtml {
    fn fmt(&self, f: &mut dyn Write) -> FormatResult<()>;
}

impl FormatHtml for char {
    #[inline(always)]
    fn fmt(&self, f: &mut dyn Write) -> FormatResult<()> {
        f.write_char(*self)
    }
}
impl FormatHtml for &str {
    #[inline(always)]
    fn fmt(&self, f: &mut dyn Write) -> FormatResult<()> {
        f.write_str(self)
    }
}
impl FormatHtml for String {
    #[inline(always)]
    fn fmt(&self, mut f: &mut dyn Write) -> FormatResult<()> {
        write!(f, [self.as_str()])
    }
}
impl<T: ?Sized> FormatHtml for &T
where
    T: FormatHtml,
{
    #[inline(always)]
    fn fmt(&self, mut f: &mut dyn Write) -> FormatResult<()> {
        write!(f, [*self])
    }
}
impl<T: FormatHtml> FormatHtml for Option<T> {
    #[inline(always)]
    fn fmt(&self, mut f: &mut dyn Write) -> FormatResult<()> {
        match self {
            Some(t) => write!(f, [t]),
            None => Ok(()),
        }
    }
}
// Implementing for vectors and slices lets elements format multiple elements at
// once, such as a subset of their children, without looping over them manually.
impl<T: FormatHtml> FormatHtml for Vec<T> {
    fn fmt(&self, mut f: &mut dyn Write) -> FormatResult<()> {
        for child in self {
            write!(f, [child])?;
        }

        Ok(())
    }
}
impl<T: FormatHtml> FormatHtml for [T] {
    fn fmt(&self, mut f: &mut dyn Write) -> FormatResult<()> {
        for child in self {
            write!(f, [child])?;
        }

        Ok(())
    }
}

pub fn format_ast(document: &Document) -> FormatResult<String> {
    let mut f = String::new();

    for (index, block) in document.blocks().iter().enumerate() {
        if index > 0 {
            f.push('\n');
        }

        match block {
            BlockNode::Paragraph(paragraph) => write!(f, [paragraph])?,
            BlockNode::Heading(heading) => write!(f, [heading])?,
            BlockNode::CodeBlock(code_block) => write!(f, [code_block])?,
            BlockNode::ThematicBreak => write!(f, ["<hr />"])?,
            BlockNode::InlineContent(content) => write!(f, [content])?,
        }
    }

    Ok(f)
}

impl FormatHtml for Paragraph {
    fn fmt(&self, mut f: &mut dyn Write) -> FormatResult<()> {
        write!(f, ["<p>", self.content(), "</p>"])
    }
}

impl FormatHtml for Heading {
    fn fmt(&self, mut f: &mut dyn Write) -> FormatResult<()> {
        std::write!(f, "<h{}>", self.level)?;
        write!(f, [self.content])?;
        std::write!(f, "</h{}>", self.level)
    }
}

impl FormatHtml for CodeBlock {
    fn fmt(&self, mut f: &mut dyn Write) -> FormatResult<()> {
        std::write!(f, "<pre><code")?;
        if let Some(language) = self.language() {
            write!(f, [" class=\"language-", language, '"'])?;
        }
        std::write!(f, ">{}</code></pre>", escape_body_text(self.content()))
    }
}

impl FormatHtml for InlineContent {
    fn fmt(&self, mut f: &mut dyn Write) -> FormatResult<()> {
        match self {
            InlineContent::Text(text) => write!(f, [escape_body_text(text)]),
            InlineContent::Emphasis(emphasis) => write!(f, [emphasis]),
            InlineContent::Strong(strong) => write!(f, [strong]),
            InlineContent::Link(link) => write!(f, [link]),
            InlineContent::CodeSpan(code_span) => write!(f, [code_span]),
            InlineContent::HardLineBreak => write!(f, ["<br />\n"]),
            InlineContent::Hook(hook) => write!(f, [hook]),
            InlineContent::Strikethrough(strikethrough) => write!(f, [strikethrough]),
            InlineContent::Icu(icu) => write!(f, [icu]),
            InlineContent::IcuPound => write!(f, ['#']),
        }
    }
}

impl FormatHtml for Emphasis {
    fn fmt(&self, mut f: &mut dyn Write) -> FormatResult<()> {
        write!(f, ["<em>", self.content(), "</em>"])
    }
}

impl FormatHtml for Strong {
    fn fmt(&self, mut f: &mut dyn Write) -> FormatResult<()> {
        write!(f, ["<strong>", self.content(), "</strong>"])
    }
}

impl FormatHtml for Link {
    fn fmt(&self, mut f: &mut dyn Write) -> FormatResult<()> {
        match self.kind {
            LinkKind::Image => {
                let title = self
                    .title
                    .as_ref()
                    .map(|title| format!(" title=\"{}\"", escape_body_text(&title)));

                write!(f, ["<img src=\""])?;

                match self.destination() {
                    TextOrPlaceholder::Text(text) => write!(f, [escape_href(&text)])?,
                    TextOrPlaceholder::Placeholder(icu) => write!(f, [icu])?,
                }

                write!(
                    f,
                    [
                        '"',
                        " alt=\"",
                        format_plain_text(&self.label),
                        '"',
                        title,
                        " />"
                    ]
                )
            }
            _ => {
                let title = self
                    .title
                    .as_ref()
                    .map(|title| format!(" title=\"{}\"", escape_body_text(&title)));

                write!(f, ["<a href=\""])?;
                match self.destination() {
                    TextOrPlaceholder::Text(text) => write!(f, [escape_href(&text)])?,
                    TextOrPlaceholder::Placeholder(icu) => write!(f, [icu])?,
                }
                write!(f, ['"', title, ">", self.label, "</a>"])
            }
        }
    }
}

impl FormatHtml for CodeSpan {
    fn fmt(&self, mut f: &mut dyn Write) -> FormatResult<()> {
        write!(f, ["<code>", escape_body_text(self.content()), "</code>"])
    }
}

impl FormatHtml for Hook {
    fn fmt(&self, mut f: &mut dyn Write) -> FormatResult<()> {
        write!(f, ["$[", self.content(), "](", self.name(), ")"])
    }
}

impl FormatHtml for Strikethrough {
    fn fmt(&self, mut f: &mut dyn Write) -> FormatResult<()> {
        write!(f, ["<del>", self.content(), "</del>"])
    }
}

impl FormatHtml for Icu {
    fn fmt(&self, mut f: &mut dyn Write) -> FormatResult<()> {
        f.write_str("{")?;
        match self {
            Icu::IcuVariable(variable) => write!(f, [variable])?,
            Icu::IcuPlural(plural) => write!(f, [plural])?,
            Icu::IcuDate(date) => write!(f, [date])?,
            Icu::IcuTime(time) => write!(f, [time])?,
            Icu::IcuNumber(number) => write!(f, [number])?,
        };
        f.write_str("}")
    }
}

impl FormatHtml for IcuVariable {
    fn fmt(&self, f: &mut dyn Write) -> FormatResult<()> {
        f.write_str(&self.name())
    }
}

impl FormatHtml for IcuPlural {
    fn fmt(&self, mut f: &mut dyn Write) -> FormatResult<()> {
        let kind_str = match self.kind() {
            IcuPluralKind::Plural => "plural",
            IcuPluralKind::Select => "select",
            IcuPluralKind::SelectOrdinal => "selectordinal",
        };

        write!(f, [self.name(), ", ", kind_str, ",", self.arms()])
    }
}

impl FormatHtml for IcuPluralArm {
    fn fmt(&self, mut f: &mut dyn Write) -> FormatResult<()> {
        write!(f, [" ", self.selector(), " {", self.content(), "}"])
    }
}

impl FormatHtml for IcuDate {
    fn fmt(&self, mut f: &mut dyn Write) -> FormatResult<()> {
        write!(f, [self.name(), ", date", self.style()])
    }
}

impl FormatHtml for IcuTime {
    fn fmt(&self, mut f: &mut dyn Write) -> FormatResult<()> {
        write!(f, [self.name(), ", time", self.style()])
    }
}

impl FormatHtml for IcuDateTimeStyle {
    fn fmt(&self, mut f: &mut dyn Write) -> FormatResult<()> {
        write!(f, [", ", self.text()])
    }
}

impl FormatHtml for IcuNumber {
    fn fmt(&self, mut f: &mut dyn Write) -> FormatResult<()> {
        write!(f, [self.name(), ", number", self.style()])
    }
}

impl FormatHtml for IcuNumberStyle {
    fn fmt(&self, mut f: &mut dyn Write) -> FormatResult<()> {
        write!(f, [", ", self.text()])
    }
}
