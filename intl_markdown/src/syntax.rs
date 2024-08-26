#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum SyntaxKind {
    // Tokens
    TOMBSTONE = 0, // The start of the input text, or an emptied token.
    EOF,           // The end of the input text.
    // Trivia
    WHITESPACE,         // Any non-textual, non-newline space character.
    LINE_ENDING,        // \n, \r, or \r\n
    LEADING_WHITESPACE, // ASCII whitespace occurring at the start of a line matching an expected line depth.
    BLANK_LINE,         // A complete line containing only whitespace and a line ending.
    ESCAPED,            // Any valid, backslash-escaped character.
    // Block Bounds
    BLOCK_START,  // A zero-width marker of the start of a block element.
    BLOCK_END,    // A zero-width representing the end of a block element.
    INLINE_START, // A zero-width marker of the start of inline content.
    INLINE_END,   // A zero-width marker of the end of inline content.
    // Tokens
    TEXT,             // Any string of contiguous plain text.
    HARD_LINE_ENDING, // A line ending preceded immediately by two or more spaces.
    BACKSLASH_BREAK,  // A line ending preceded immediately by a backslash character.
    HTML_ENTITY,      // HTML &-prefixed entity names, like `&amp;` and `&copy`.
    DEC_CHAR_REF,     // Decimal numeric character references, like `&#35;`.
    HEX_CHAR_REF,     // Hexadecimal numeric character reference, like `&#X22;`.
    ABSOLUTE_URI,     // An absolute URI, used in autolinks.
    EMAIL_ADDRESS,    // An email address, used in autolinks.
    VERBATIM_LINE,    // A line that is consumed as a whole with no interpretation.
    // Punctuation
    STAR,          // *
    UNDER,         // _
    TILDE,         // ~
    MINUS,         // -
    EQUAL,         // =
    HASH,          // #
    COLON,         // :
    QUOTE,         // '
    DOUBLE_QUOTE,  // "
    AMPER,         // &
    LSQUARE,       // [
    RSQUARE,       // ]
    LPAREN,        // (
    RPAREN,        // )
    LANGLE,        // <
    RANGLE,        // >
    LCURLY,        // {
    RCURLY,        // }
    UNSAFE_LCURLY, // !!{
    UNSAFE_RCURLY, // }!!
    BACKTICK,      // `
    DOLLAR,        // $
    COMMA,         // ,
    EXCLAIM,       // !

    // Nodes:
    //
    // All token kinds should be placed _above_ this point. All node kinds
    // should be placed _below_ it (beneath `DOCUMENT`). This lets the parser
    // easily determine if a kind represents a token or a node.

    // CommonMark block nodes
    DOCUMENT,
    /// Any segment of inline content, either contained within a block node or
    /// at the top level on its own.
    INLINE_CONTENT,
    /// 4.1 Thematic breaks
    ///
    /// A line consisting of optionally up to three spaces of indentation,
    /// followed by a sequence of three or more matching -, _, or * characters,
    /// each followed optionally by any number of spaces or tabs, forms a
    /// thematic break.
    THEMATIC_BREAK,
    /// 4.2 ATX headings
    ///
    /// An ATX heading consists of a string of characters, parsed as inline
    /// content, between an opening sequence of 1–6 unescaped # characters and
    /// an optional closing sequence of any number of unescaped # characters.
    /// ...
    /// The opening # character may be preceded by up to three spaces of
    /// indentation.
    ATX_HEADING,
    /// 4.3 Setext headings
    ///
    /// A setext heading consists of one or more lines of text, not interrupted
    /// by a blank line, of which the first line does not have more than 3
    /// spaces of indentation, followed by a setext heading underline.
    SETEXT_HEADING,
    /// 4.4 Indented code blocks
    ///
    /// An indented code block is composed of one or more indented chunks
    /// separated by blank lines. The contents of the code block are the literal
    /// contents of the lines, including trailing line endings, minus four
    /// spaces of indentation. An indented code block has no info string.
    INDENTED_CODE_BLOCK,
    /// 4.5 Fenced code blocks
    ///
    /// A code fence is a sequence of at least three consecutive backtick
    /// characters (`) or tildes (~). (Tildes and backticks cannot be mixed.)
    /// A fenced code block begins with a code fence, preceded by up to three
    /// spaces of indentation.
    FENCED_CODE_BLOCK,
    /// 4.6 HTML blocks
    ///
    /// An HTML block is a group of lines that is treated as raw HTML (and will
    /// not be escaped in HTML output).
    HTML_BLOCK,
    /// 4.7 Link reference definitions
    ///
    /// A link reference definition consists of a link label, optionally
    /// preceded by up to three spaces of indentation, followed by a colon (:),
    /// optional spaces or tabs (including up to one line ending), a link
    /// destination, optional spaces or tabs (including up to one line ending),
    /// and an optional link title, which if it is present must be separated
    /// from the link destination by spaces or tabs. No further character may
    /// occur.
    LINK_REFERENCE_DEFINITION,
    /// 4.8 Paragraphs
    ///
    /// A sequence of non-blank lines that cannot be interpreted as other kinds
    /// of blocks forms a paragraph. The contents of the paragraph are the
    /// result of parsing the paragraph’s raw content as inlines. The
    /// paragraph’s raw content is formed by concatenating the lines and
    /// removing initial and final spaces or tabs.
    PARAGRAPH,
    /// 4.9 Blank Lines
    ///
    /// Blank lines that appear _between_ block-level nodes are ignored.
    BLANK_LINES,

    // Container blocks
    /// 5.1 Block quotes
    ///
    /// A block quote marker, optionally preceded by up to three spaces of
    /// indentation, consists of (a) the character > together with a following
    /// space of indentation, or (b) a single character > not followed by a
    /// space of indentation.
    BLOCK_QUOTE,
    /// 5.2 List items
    ///
    /// A list marker is a bullet list marker or an ordered list marker.
    /// A bullet list marker is a -, +, or * character.
    /// An ordered list marker is a sequence of 1–9 arabic digits (0-9),
    /// followed by either a . character or a ) character.
    LIST_ITEM,
    BULLET_LIST_MARKER,
    ORDERED_LIST_MARKER,
    /// 5.3 Lists
    ///
    /// A list is a sequence of one or more list items of the same type. The
    /// list items may be separated by any number of blank lines.
    LIST,

    // Everything above this point is a Block-level node. Everything below here
    // is an Inline-level node.

    // CommonMark inline nodes
    EMPHASIS,
    STRONG,
    IMAGE,
    LINK,
    LINK_RESOURCE,
    LINK_DESTINATION,
    STATIC_LINK_DESTINATION,
    DYNAMIC_LINK_DESTINATION,
    LINK_TITLE,
    LINK_TITLE_CONTENT,
    AUTOLINK,
    CODE_SPAN,
    CODE_SPAN_DELIMITER,

    // Markdown extension nodes
    STRIKETHROUGH,
    ATX_HASH_SEQUENCE,
    SETEXT_HEADING_UNDERLINE,
    CODE_FENCE_DELIMITER,
    CODE_FENCE_INFO_STRING,
    CODE_BLOCK_CONTENT,

    // Syntax extension nodes
    HOOK,
    HOOK_NAME,

    // ICU extension nodes
    ICU,        // The overall container node for any ICU content.
    ICU_UNSAFE, // An additional wrapping node for the `!!{...}!!` syntax.
    // ICU keywords
    ICU_NUMBER_KW,         // number
    ICU_DATE_KW,           // date
    ICU_TIME_KW,           // time
    ICU_SELECT_KW,         // select
    ICU_SELECT_ORDINAL_KW, // selectordinal
    ICU_PLURAL_KW,         // plural
    // ICU tokens
    ICU_DOUBLE_COLON, // ::
    // ICU literals
    ICU_IDENT,           // Any user-created identifier, used for variable names.
    ICU_PLURAL_CATEGORY, // `one`, `zero`, `other`, etc. in a plural or select ordinal.
    ICU_PLURAL_EXACT,    // Exact value match in a plural block, like `=0`.
    ICU_STYLE_ARGUMENT,  // Any third argument to a number, date, or time variable.
    ICU_STYLE_TEXT,      // The text token of the ICU_STYLE_ARGUMENT node above.
    ICU_DATE_TIME_STYLE, // Either a keyword like `short` or a skeleton like `::hmsGy`
    ICU_NUMBER_STYLE,    // A number style argument, almost always a skeleton like `::.##`.
    // ICU Nodes
    ICU_DATE,           // {var, date} or {var, date, format}
    ICU_TIME,           // {var, time} or {var, time, format}
    ICU_NUMBER,         // {var, number} or {var, number, format}
    ICU_PLACEHOLDER,    // {var}
    ICU_PLURAL,         // {var, plural, ...}
    ICU_SELECT,         // {var, select, ...}
    ICU_SELECT_ORDINAL, // {var, selectordinal, ...}
    ICU_VARIABLE,       // `var` in `{var}` or `{var, plural}` and so on.
    // ICU_PLURAL_ARMS,  // The list of arms in a plural or select node.
    ICU_PLURAL_ARM,   // The `one {inner}` in `{var, plural, one {inner}}`
    ICU_PLURAL_VALUE, // The `inner` in `{var, plural, one {inner}}`
}

impl SyntaxKind {
    pub const fn is_html_special_entity(&self) -> bool {
        matches!(
            self,
            SyntaxKind::DOUBLE_QUOTE | SyntaxKind::LANGLE | SyntaxKind::RANGLE | SyntaxKind::AMPER
        )
    }

    pub const fn is_token(&self) -> bool {
        (*self as u8) >= (Self::EOF as u8) && (*self as u8) < (Self::DOCUMENT as u8)
    }

    pub const fn is_node(&self) -> bool {
        (*self as u8) >= (Self::DOCUMENT as u8)
    }

    pub const fn is_trivia(&self) -> bool {
        matches!(
            self,
            SyntaxKind::BLANK_LINE
                | SyntaxKind::LEADING_WHITESPACE
                | SyntaxKind::WHITESPACE
                | SyntaxKind::LINE_ENDING
        )
    }

    pub const fn is_same_line_whitespace(&self) -> bool {
        matches!(self, SyntaxKind::WHITESPACE | SyntaxKind::LINE_ENDING)
    }

    pub const fn is_block_level(&self) -> bool {
        (*self as u8) >= (Self::DOCUMENT as u8) && (*self as u8) <= (Self::EMPHASIS as u8)
    }

    pub const fn is_inline_level(&self) -> bool {
        (*self as u8) >= (Self::EMPHASIS as u8)
            && (*self as u8) <= (Self::CODE_SPAN_DELIMITER as u8)
    }

    pub const fn is_icu(&self) -> bool {
        (*self as u8) >= (Self::ICU as u8) && (*self as u8) < (Self::ICU_PLURAL_VALUE as u8)
    }

    pub const fn is_icu_keyword(&self) -> bool {
        matches!(
            self,
            SyntaxKind::ICU_NUMBER_KW
                | SyntaxKind::ICU_DATE_KW
                | SyntaxKind::ICU_TIME_KW
                | SyntaxKind::ICU_SELECT_KW
                | SyntaxKind::ICU_SELECT_ORDINAL_KW
                | SyntaxKind::ICU_PLURAL_KW
        )
    }

    pub const fn is_hard_line_break(&self) -> bool {
        matches!(
            self,
            SyntaxKind::HARD_LINE_ENDING | SyntaxKind::BACKSLASH_BREAK
        )
    }

    /// Returns true if a token of this kind is valid to merge together into a single text element
    /// with another merge-able kind. This is true for any kind of text and most other plain tokens,
    /// other than character references and html entities, since those are often treated specially.
    pub const fn can_merge_as_text(&self) -> bool {
        !self.is_trivia()
            && !self.is_html_special_entity()
            && !matches!(
                self,
                SyntaxKind::HARD_LINE_ENDING
                    | SyntaxKind::BACKSLASH_BREAK
                    | SyntaxKind::LINE_ENDING
                    | SyntaxKind::HTML_ENTITY
                    | SyntaxKind::DEC_CHAR_REF
                    | SyntaxKind::HEX_CHAR_REF
                    | SyntaxKind::ABSOLUTE_URI
                    | SyntaxKind::EMAIL_ADDRESS
                    | SyntaxKind::VERBATIM_LINE
            )
    }
}

/// Returns true if the given byte represents a significant character that
/// could become a new type of token. This effectively just includes
/// punctuation and newline characters.
///
/// Note that these are only the characters that are significant when they
/// interrupt textual content. For example, a `-` could become a MINUS token,
/// but within a word it can never be significant, e.g. the dash in `two-part`
/// is not significant.
///
/// Whitespace in this context is _not_ considered significant.
pub fn byte_is_significant(byte: u8) -> bool {
    matches!(
        byte,
        b'\n'
            | b'['
            | b']'
            | b'}'
            | b'{'
            | b'('
            | b')'
            | b':'
            | b'<'
            | b'>'
            | b'`'
            | b'$'
            | b'*'
            | b'_'
            | b'~'
            | b'!'
            | b'\''
            | b'"'
            | b'\\'
            | b'&'
    )
}
