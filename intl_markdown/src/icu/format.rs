use std::fmt::Write;

use crate::ast::{
    BlockNode, CodeBlock, CodeSpan, Document, Emphasis, Heading, Hook, Icu, IcuDate,
    IcuDateTimeStyle, IcuNumber, IcuNumberStyle, IcuPlural, IcuPluralArm, IcuPluralKind, IcuTime,
    IcuVariable, InlineContent, Link, LinkKind, Paragraph, Strikethrough, Strong,
    TextOrPlaceholder,
};
use crate::ast::util::{escape_body_text, escape_href};

macro_rules! write {
    ($dst:expr, [$($arg:expr),+ $(,)?]) => {{
        $(
            let _ = $arg.fmt(&mut $dst)?;
        )*
        Ok(())
    }}
}

pub(crate) type FormatResult<T> = Result<T, std::fmt::Error>;

trait FormatIcuString {
    fn fmt(&self, f: &mut dyn Write) -> FormatResult<()>;
}

impl FormatIcuString for char {
    #[inline(always)]
    fn fmt(&self, f: &mut dyn Write) -> FormatResult<()> {
        f.write_char(*self)
    }
}
impl FormatIcuString for &str {
    #[inline(always)]
    fn fmt(&self, f: &mut dyn Write) -> FormatResult<()> {
        f.write_str(self)
    }
}
impl FormatIcuString for String {
    #[inline(always)]
    fn fmt(&self, mut f: &mut dyn Write) -> FormatResult<()> {
        write!(f, [self.as_str()])
    }
}
impl<T: ?Sized> FormatIcuString for &T
where
    T: FormatIcuString,
{
    #[inline(always)]
    fn fmt(&self, mut f: &mut dyn Write) -> FormatResult<()> {
        write!(f, [*self])
    }
}
impl<T: FormatIcuString> FormatIcuString for Option<T> {
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
impl<T: FormatIcuString> FormatIcuString for Vec<T> {
    fn fmt(&self, mut f: &mut dyn Write) -> FormatResult<()> {
        for child in self {
            write!(f, [child])?;
        }

        Ok(())
    }
}
impl<T: FormatIcuString> FormatIcuString for [T] {
    fn fmt(&self, mut f: &mut dyn Write) -> FormatResult<()> {
        for child in self {
            write!(f, [child])?;
        }

        Ok(())
    }
}

pub fn format_icu_string(document: &Document) -> FormatResult<String> {
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

impl FormatIcuString for Paragraph {
    fn fmt(&self, mut f: &mut dyn Write) -> FormatResult<()> {
        write!(f, ["<p>", self.content(), "</p>"])
    }
}

impl FormatIcuString for Heading {
    fn fmt(&self, mut f: &mut dyn Write) -> FormatResult<()> {
        std::write!(f, "<h{}>", self.level())?;
        write!(f, [self.content()])?;
        std::write!(f, "</h{}>", self.level())
    }
}

impl FormatIcuString for CodeBlock {
    fn fmt(&self, f: &mut dyn Write) -> FormatResult<()> {
        std::write!(
            f,
            "<codeBlock>{}</codeBlock>",
            escape_body_text(self.content())
        )
    }
}

impl FormatIcuString for InlineContent {
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

impl FormatIcuString for Emphasis {
    fn fmt(&self, mut f: &mut dyn Write) -> FormatResult<()> {
        write!(f, ["<i>", self.content(), "</i>"])
    }
}

impl FormatIcuString for Strong {
    fn fmt(&self, mut f: &mut dyn Write) -> FormatResult<()> {
        write!(f, ["<b>", self.content(), "</b>"])
    }
}

fn format_text_or_placeholder<F: Fn(&str) -> String>(
    node: &TextOrPlaceholder,
    text_mutator: F,
) -> FormatTextOrPlaceholder<F> {
    FormatTextOrPlaceholder { node, text_mutator }
}
struct FormatTextOrPlaceholder<'a, F: Fn(&str) -> String> {
    node: &'a TextOrPlaceholder,
    text_mutator: F,
}
impl<F: Fn(&str) -> String> FormatIcuString for FormatTextOrPlaceholder<'_, F> {
    fn fmt(&self, mut f: &mut dyn Write) -> FormatResult<()> {
        match self.node {
            TextOrPlaceholder::Text(text) => f.write_str(&(self.text_mutator)(&text)),
            TextOrPlaceholder::Placeholder(icu) => write!(f, [icu]),
        }
    }
}

impl FormatIcuString for Link {
    fn fmt(&self, mut f: &mut dyn Write) -> FormatResult<()> {
        let destination = format_text_or_placeholder(self.destination(), escape_href);
        match self.kind() {
            LinkKind::Image => {
                write!(f, ["<img>", destination, "</img>"])
            }
            _ => {
                write!(
                    f,
                    [
                        "<link>",
                        destination,
                        // Insert a delimiter between the destination and the label in case the
                        // destination is a plain string. Otherwise they would get merged together
                        // when parsing with FormatJS.
                        self.destination().is_text().then_some("{_}"),
                        self.label(),
                        "</link>"
                    ]
                )
            }
        }
    }
}

impl FormatIcuString for CodeSpan {
    fn fmt(&self, mut f: &mut dyn Write) -> FormatResult<()> {
        write!(f, ["<code>", &escape_body_text(self.content()), "</code>"])
    }
}

impl FormatIcuString for Hook {
    fn fmt(&self, mut f: &mut dyn Write) -> FormatResult<()> {
        std::write!(f, "<{}>", self.name())?;
        write!(f, [self.content()])?;
        std::write!(f, "</{}>", self.name())
    }
}

impl FormatIcuString for Strikethrough {
    fn fmt(&self, mut f: &mut dyn Write) -> FormatResult<()> {
        write!(f, ["<del>", self.content(), "</del>"])
    }
}

impl FormatIcuString for Icu {
    fn fmt(&self, mut f: &mut dyn Write) -> crate::ast::format::FormatResult<()> {
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

impl FormatIcuString for IcuVariable {
    fn fmt(&self, f: &mut dyn Write) -> crate::ast::format::FormatResult<()> {
        f.write_str(&self.name())
    }
}

impl FormatIcuString for IcuPlural {
    fn fmt(&self, mut f: &mut dyn Write) -> crate::ast::format::FormatResult<()> {
        let kind_str = match self.kind() {
            IcuPluralKind::Plural => "plural",
            IcuPluralKind::Select => "select",
            IcuPluralKind::SelectOrdinal => "selectordinal",
        };

        write!(f, [self.name(), ", ", kind_str, ",", self.arms()])
    }
}

impl FormatIcuString for IcuPluralArm {
    fn fmt(&self, mut f: &mut dyn Write) -> crate::ast::format::FormatResult<()> {
        write!(f, [" ", self.selector(), " {", self.content(), "}"])
    }
}

impl FormatIcuString for IcuDate {
    fn fmt(&self, mut f: &mut dyn Write) -> crate::ast::format::FormatResult<()> {
        write!(f, [self.name(), ", date", self.style()])
    }
}

impl FormatIcuString for IcuTime {
    fn fmt(&self, mut f: &mut dyn Write) -> crate::ast::format::FormatResult<()> {
        write!(f, [self.name(), ", time", self.style()])
    }
}

impl FormatIcuString for IcuDateTimeStyle {
    fn fmt(&self, mut f: &mut dyn Write) -> FormatResult<()> {
        write!(f, [", ", self.text()])
    }
}

impl FormatIcuString for IcuNumber {
    fn fmt(&self, mut f: &mut dyn Write) -> crate::ast::format::FormatResult<()> {
        write!(f, [self.name(), ", number", self.style()])
    }
}

impl FormatIcuString for IcuNumberStyle {
    fn fmt(&self, mut f: &mut dyn Write) -> FormatResult<()> {
        write!(f, [", ", self.text()])
    }
}
