mod util;

use crate::syntax::{
    FromSyntax, FromSyntaxElement, SyntaxElement, SyntaxElementChildren, SyntaxKind, SyntaxNode,
    SyntaxNodeChildren, SyntaxToken, SyntaxTokenChildren,
};
use intl_markdown_macros::cst_node_debug;
use util::*;

//#region Macros
macro_rules! cst_field {
    ([$slot:literal, $field_name:ident : Token]) => {
        pub fn $field_name(&self) -> SyntaxToken {
            required_token(&self.syntax, $slot)
        }
    };
    ([$slot:literal, $field_name:ident : Option<Token>]) => {
        pub fn $field_name(&self) -> Option<SyntaxToken> {
            optional_token(&self.syntax, $slot)
        }
    };
    ([$slot:literal, $field_name:ident : $ty:ident]) => {
        pub fn $field_name(&self) -> $ty {
            required_node(&self.syntax, $slot)
        }
    };
    ([$slot:literal, $field_name:ident : Option < $ty:ident >]) => {
        pub fn $field_name(&self) -> Option<$ty> {
            optional_node(&self.syntax, $slot)
        }
    };
}
macro_rules! cst_node {
    (@no_debug $node_name:ident) => {
        #[repr(transparent)]
        pub struct $node_name {
            syntax: SyntaxNode,
        }

        impl FromSyntax for $node_name {
            fn from_syntax(syntax: SyntaxNode) -> Self { Self { syntax } }
        }
    };
    ($node_name:ident) => {
        cst_node!(@no_debug $node_name);
        impl std::fmt::Debug for $node_name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str(stringify!($node_name))
            }
        }
    };
    ($node_name:ident, $( $fields:tt ),*) => {
        cst_node!(@no_debug $node_name);
        impl $node_name {
            $(cst_field!($fields);)*
        }
        cst_node_debug!($node_name, $($fields,)*);
    };
}

macro_rules! cst_list_node {
    ($node_name:ident, Token) => {
        cst_node!(@no_debug $node_name);
        impl $node_name {
            pub fn content(&self) -> SyntaxTokenChildren {
                SyntaxTokenChildren::new(self.syntax.children())
            }
            pub fn len(&self) -> usize {
                self.syntax.len()
            }
        }
        cst_list_node!(@debug, $node_name);
    };
    ($node_name:ident, Mixed<$node_ty:ident>) => {
        cst_node!(@no_debug $node_name);
        impl $node_name {
            pub fn content(&self) -> AstElementChildren<$node_ty> {
                AstElementChildren::new(SyntaxElementChildren::new(self.syntax.children()))
            }
            pub fn len(&self) -> usize {
                self.syntax.len()
            }
        }
        cst_list_node!(@debug, $node_name);
    };
    ($node_name:ident, $node_ty:ident) => {
        cst_node!(@no_debug $node_name);
        impl $node_name {
            pub fn content(&self) -> AstNodeChildren<$node_ty> {
                AstNodeChildren::new(SyntaxNodeChildren::new(self.syntax.children()))
            }
            pub fn len(&self) -> usize {
                self.syntax.len()
            }
        }
        cst_list_node!(@debug, $node_name);
    };
    (@debug, $node_name:ident) => {
        impl std::fmt::Debug for $node_name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str(stringify!($node_name))?;
                f.write_str(" ")?;
                f.debug_list().entries(self.content()).finish()
            }
        }
    }
}

macro_rules! cst_enum_node {
    ($node_name:ident, Token, $( $field:ident ),+) => {
        pub enum $node_name {
            $($field($field),)+
            Token(SyntaxToken)
        }

        paste::item! {
            impl FromSyntaxElement for $node_name {
                fn from_syntax_element(syntax: SyntaxElement) -> Self {
                    match syntax.kind() {
                        $(SyntaxKind::[<$field:snake:upper>] => Self::$field($field::from_syntax_element(syntax)),)+
                        kind => {
                            if kind.is_token() {
                                $node_name::Token(syntax.into_token())
                            } else {
                                unreachable!(
                                    "Encountered invalid node or token of kind {:?}",
                                    syntax.kind()
                                )
                            }
                        }
                    }
                }
            }
        }

        paste::item! {
            impl std::fmt::Debug for $node_name {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    match self {
                        $(Self::$field(node) => node.fmt(f),)+
                        Self::Token(token) => token.fmt(f),
                    }
                }
            }
        }
    };
    ($node_name:ident, $( $field:ident ),+) => {
        pub enum $node_name {
            $($field($field)),+
        }

        paste::item! {
            impl FromSyntax for $node_name {
                fn from_syntax(syntax: SyntaxNode) -> Self {
                    match syntax.kind() {
                        $(SyntaxKind::[<$field:snake:upper>] => Self::$field($field::from_syntax(syntax)),)+
                        _ => unreachable!("Encountered invalid node of kind {:?}", syntax.kind())
                    }
                }
            }
        }

        paste::item! {
            impl std::fmt::Debug for $node_name {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    match self {
                        $(Self::$field(node) => node.fmt(f)),+
                    }
                }
            }
        }
    };
}
//#endregion

//#region Markdown Block Nodes
cst_list_node!(Document, Mixed<BlockNode>);
cst_list_node!(InlineContent, Mixed<InlineNode>);
cst_list_node!(ThematicBreak, Token);

cst_node!(Paragraph, [0, content: InlineContent]);
cst_node!(IndentedCodeBlock, [0, content: CodeBlockContent]);

cst_node!(FencedCodeBlock,
    [0, opening_run: Token],
    [1, info_string: Option<CodeFenceInfoString>],
    [2, content: CodeBlockContent],
    [3, closing_run: Option<Token>]
);

cst_list_node!(CodeBlockContent, Token);
cst_list_node!(CodeFenceInfoString, Token);

cst_node!(AtxHeading,
    [0, opening_run: Token],
    [1, content: InlineContent],
    [2, closing_run: Option<Token>]
);
impl AtxHeading {
    /// Returns the heading level (1-6, inclusive) that this heading should
    /// have according to the opening sequence
    pub fn level(&self) -> usize {
        self.opening_run().len() as usize
    }
}

cst_list_node!(AtxHashSequence, Token);

cst_node!(SetextHeading, [0, content: InlineContent], [1, underline: SetextHeadingUnderline]);
cst_list_node!(SetextHeadingUnderline, Token);
impl SetextHeadingUnderline {
    /// Returns the heading level (1 or 2) that this heading should have
    /// according to the type of underline.
    pub fn level(&self) -> usize {
        match self.syntax.children()[0].kind() {
            SyntaxKind::EQUAL => 1,
            SyntaxKind::MINUS => 2,
            found => unreachable!(
                "Found a setext heading underline character that is invalid: {:?}",
                found
            ),
        }
    }
}
//#endregion

//#region Markdown Inline Nodes
cst_node!(Emphasis, [0, open_1: Token], [1, content: InlineContent], [2, close_1: Token]);
impl Emphasis {
    /// Returns the kind of token used for the delimiters in this node.
    pub fn token_kind(&self) -> SyntaxKind {
        self.open_1().kind()
    }
}

cst_node!(Strong,
    [0, open_1: Token],
    [1, open_2: Token],
    [2, content: InlineContent],
    [3, close_1: Token],
    [4, close_2: Token]
);
impl Strong {
    /// Returns the kind of token used for the delimiters in this node.
    pub fn token_kind(&self) -> SyntaxKind {
        self.open_1().kind()
    }
}

cst_node!(Link,
    [0, l_squre: Token],
    [1, content: InlineContent],
    [2, r_square: Token],
    [3, resource: LinkResource]
);
cst_node!(Image,
    [0, exclaim: Token],
    [1, l_square: Token],
    [2, content: InlineContent],
    [3, r_square: Token],
    [4, resource: LinkResource]
);

cst_node!(LinkResource,
    [0, l_paren: Token],
    [1, destination: LinkDestination],
    [2, title: Option<LinkTitle>],
    [3, r_paren: Token]
);

cst_enum_node!(
    LinkDestination,
    StaticLinkDestination,
    DynamicLinkDestination,
    ClickHandlerLinkDestination
);

cst_list_node!(StaticLinkDestination, Token);
cst_node!(DynamicLinkDestination, [0, url: Icu]);
cst_node!(ClickHandlerLinkDestination, [0, name: Token]);

cst_node!(LinkTitle, [0, opening_punctuation: Token], [1, title: LinkTitleContent], [2, closing_punctuation: Token]);
cst_list_node!(LinkTitleContent, Token);

cst_node!(Autolink, [0, l_angle: Token], [1, uri: Token], [2, r_angle: Token]);

cst_node!(CodeSpan,
    [0, open_backticks: Token],
    [1, content: CodeSpanContent],
    [2, close_backticks: Token]
);

cst_list_node!(CodeSpanDelimiter, Token);
cst_list_node!(CodeSpanContent, Token);
//#endregion

//#region Markdown Extensions
cst_node!(Hook,
    [0, dollar: Token],
    [1, l_square: Token],
    [2, content: InlineContent],
    [3, r_square: Token],
    [4, name: HookName]
);
cst_node!(HookName,
    [0, l_paren: Token],
    [1, name: Token],
    [2, r_paren: Token]
);

cst_node!(Strikethrough,
    [0, opening_run: Token],
    [2, content: InlineContent],
    [4, closing_run: Token]
);
//#endregion

//#region ICU Nodes
cst_node!(Icu,
    [0, l_curly: Token],
    [1, value: IcuPlaceholder],
    [2, r_curly: Token]
);

cst_enum_node!(
    IcuPlaceholder,
    IcuVariable,
    IcuPlural,
    IcuSelectOrdinal,
    IcuSelect,
    IcuDate,
    IcuTime,
    IcuNumber
);

cst_node!(IcuVariable, [0, ident: Token]);
cst_node!(IcuPlural,
    [0, variable: IcuVariable],
    [1, variable_comma: Token],
    [2, format_token: Token],
    [3, format_comma: Token],
    [4, arms: IcuPluralArms]
);
cst_node!(IcuSelect,
    [0, variable: IcuVariable],
    [1, variable_comma: Token],
    [2, format_token: Token],
    [3, format_comma: Token],
    [4, arms: IcuPluralArms]
);
cst_node!(IcuSelectOrdinal,
    [0, variable: IcuVariable],
    [1, variable_comma: Token],
    [2, format_token: Token],
    [3, format_comma: Token],
    [4, arms: IcuPluralArms]
);
cst_list_node!(IcuPluralArms, IcuPluralArm);
cst_node!(IcuPluralArm,
    [0, selector: Token],
    [1, l_curly: Token],
    [2, value: IcuPluralValue],
    [3, r_curly: Token]
);
cst_node!(IcuPluralValue, [0, content: InlineContent]);

cst_node!(IcuDate,
    [0, variable: IcuVariable],
    [1, variable_comma: Token],
    [2, format_token: Token],
    [3, style: Option<IcuDateTimeStyle>]
);
cst_node!(IcuTime,
    [0, variable: IcuVariable],
    [1, variable_comma: Token],
    [2, format_token: Token],
    [3, style: Option<IcuDateTimeStyle>]
);
cst_node!(IcuDateTimeStyle,
    [0, leading_comma: Token],
    [0, style_text: Token]
);

cst_node!(IcuNumber,
    [0, variable: IcuVariable],
    [1, variable_comma: Token],
    [2, format_token: Token],
    [3, style: Option<IcuNumberStyle>]
);
cst_node!(IcuNumberStyle,
    [0, leading_comma: Token],
    [0, style_text: Token]
);
//#endregion

cst_enum_node!(
    Node,
    Paragraph,
    ThematicBreak,
    AtxHeading,
    SetextHeading,
    IndentedCodeBlock,
    FencedCodeBlock,
    InlineContent,
    Emphasis,
    Strong,
    Link,
    Image,
    Autolink,
    CodeSpan,
    Hook,
    Strikethrough,
    Icu
);

cst_enum_node!(
    BlockNode,
    Token,
    // Blocks can directly contain inline content, but this only happens when parsing _without_
    // blocks enabled. Otherwise, all inline content is wrapped in a Paragraph.
    InlineContent,
    Paragraph,
    ThematicBreak,
    AtxHeading,
    SetextHeading,
    IndentedCodeBlock,
    FencedCodeBlock
);

cst_enum_node!(
    InlineNode,
    Token,
    Emphasis,
    Strong,
    Link,
    Image,
    Autolink,
    CodeSpan,
    Hook,
    Strikethrough,
    Icu
);
