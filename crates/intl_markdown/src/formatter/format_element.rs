use crate::syntax::TextPointer;

pub enum LinkKind {
    Link,
    Hook,
}

pub enum FormatTag {
    Heading {
        level: u8,
    },
    CodeBlock {
        info_string: Option<TextPointer>,
    },
    Paragraph,
    Link {
        kind: LinkKind,
        destination: TextPointer,
        title: Option<TextPointer>,
    },
    Image {
        destination: TextPointer,
        title: Option<TextPointer>,
        alt: Option<TextPointer>,
    },
    Emphasis,
    Strong,
    Strikethrough,
    CodeSpan,
}

/// An individual element of processed IntlMarkdown content. Output formats can implement
/// processing for this limited set of elements and automatically support converting all valid
/// Markdown documents into their own format (e.g., HTML, compiled ASTs, JSON, etc.).
pub enum FormatElement {
    /// Start of a new content segment that may have its own rules for formatting.
    StartTag(FormatTag),
    /// End of the most recently started tag.
    EndTag,
    /// A hard line break that _always_ wraps to a new line. In contexts where whitespace can be
    /// collapsed, this is often represented separately in the output, like a `<br />` tag in HTML.
    HardLineBreak,
    /// A soft line break that _may_ wrap to a new line. These are often just preserved as-is in
    /// the output format, but indicate that some form of newline should be allowed. For example,
    /// in HTML, this can be represented as a literal newline character, since HTML itself will
    /// collapse the whitespace when appropriate.
    SoftLineBreak,
    /// A semantic break between content. This represents a literal Markdown element created by
    /// lines like `***`. In HTML, this is represented by the `<hr />` tag.
    ThematicBreak,
    /// Raw text to be added to the output. This text has already been processed by Markdown's
    /// handling rules, like replacing HTML entities and character references, and removing leading
    /// whitespace
    Text(TextPointer),
    // NOTE: This list is incomplete compared to Markdown's full list of node types, as the parser
    // does not currently support most Block nodes (e.g., list items, block quotes).
}

impl From<TextPointer> for FormatElement {
    fn from(pointer: TextPointer) -> Self {
        FormatElement::Text(pointer)
    }
}

impl From<&TextPointer> for FormatElement {
    fn from(pointer: &TextPointer) -> Self {
        FormatElement::Text(pointer.clone())
    }
}
