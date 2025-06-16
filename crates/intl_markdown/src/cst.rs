use crate::syntax::{
    FromSyntax, SyntaxElementChildren, SyntaxKind, SyntaxNode, SyntaxNodeChildren, SyntaxToken,
    SyntaxTokenChildren,
};
use std::marker::PhantomData;

//#region Utilities
fn required_node<N: FromSyntax>(syntax: &SyntaxNode, slot: usize) -> N {
    N::from_syntax(syntax.required_node(slot))
}
fn optional_node<N: FromSyntax>(syntax: &SyntaxNode, slot: usize) -> Option<N> {
    syntax.optional_node(slot).map(FromSyntax::from_syntax)
}
fn required_token(syntax: &SyntaxNode, slot: usize) -> SyntaxToken {
    syntax.required_token(slot)
}
fn optional_token(syntax: &SyntaxNode, slot: usize) -> Option<SyntaxToken> {
    syntax.optional_token(slot)
}

pub struct AstNodeChildren<'a, T: FromSyntax> {
    children: SyntaxNodeChildren<'a>,
    _phantom: PhantomData<&'a T>,
}

impl<'a, T: FromSyntax> AstNodeChildren<'a, T> {
    pub fn new(children: SyntaxNodeChildren<'a>) -> Self {
        Self {
            children,
            _phantom: PhantomData,
        }
    }
}

impl<'a, T: FromSyntax> Iterator for AstNodeChildren<'a, T> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        self.children.next().map(T::from_syntax)
    }
}
//#endregion

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
    ($node_name:ident) => {
        #[derive(Debug)]
        #[repr(transparent)]
        pub struct $node_name {
            syntax: SyntaxNode,
        }

        impl FromSyntax for $node_name {
            fn from_syntax(syntax: SyntaxNode) -> Self { Self { syntax } }
        }
    };
    ($node_name:ident, $( $fields:tt ),*) => {
        cst_node!($node_name);
        impl $node_name {
            $(cst_field!($fields);)*
        }
    };
}

macro_rules! cst_list_node {
    ($node_name:ident, Node<$node_ty:ident>) => {
        cst_node!($node_name);
        impl $node_name {
            pub fn content(&self) -> AstNodeChildren<$node_ty> {
                AstNodeChildren::new(SyntaxNodeChildren::new(self.syntax.children()))
            }
            pub fn len(&self) -> usize {
                self.syntax.len()
            }
        }
    };
    ($node_name:ident, $children:ident) => {
        cst_node!($node_name);
        impl $node_name {
            pub fn content(&self) -> $children {
                $children::new(self.syntax.children())
            }
            pub fn len(&self) -> usize {
                self.syntax.len()
            }
        }
    };
}

macro_rules! cst_enum_node {
    ($node_name:ident, $( $field:ident ),+) => {
        #[derive(Debug)]
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
    };
}
//#endregion

//#region Markdown Block Nodes
cst_list_node!(Document, SyntaxElementChildren);
cst_list_node!(InlineContent, SyntaxElementChildren);
cst_list_node!(ThematicBreak, SyntaxTokenChildren);

cst_node!(Paragraph, [0, content: InlineContent]);
cst_node!(IndentedCodeBlock, [0, content: CodeBlockContent]);

cst_node!(FencedCodeBlock,
    [0, opening_sequence: CodeFenceDelimiter],
    [1, info_string: Option<CodeFenceInfoString>],
    [2, content: CodeBlockContent],
    [3, closing_sequence: CodeFenceDelimiter]
);

cst_list_node!(CodeBlockContent, SyntaxTokenChildren);
cst_list_node!(CodeFenceDelimiter, SyntaxTokenChildren);
cst_list_node!(CodeFenceInfoString, SyntaxTokenChildren);

cst_node!(AtxHeading,
    [0, opening_sequence: AtxHashSequence],
    [1, content: InlineContent],
    [2, closing_sequence: AtxHashSequence]
);
impl AtxHeading {
    /// Returns the heading level (1-6, inclusive) that this heading should
    /// have according to the opening sequence
    pub fn level(&self) -> usize {
        self.opening_sequence().len()
    }
}

cst_list_node!(AtxHashSequence, SyntaxTokenChildren);

cst_node!(SetextHeading, [0, content: InlineContent], [1, underline: SetextHeadingUnderline]);
cst_list_node!(SetextHeadingUnderline, SyntaxTokenChildren);
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
    [1, destination: Option<LinkDestination>],
    [2, title: Option<LinkTitle>],
    [3, r_paren: Token]
);

cst_enum_node!(
    LinkDestination,
    StaticLinkDestination,
    DynamicLinkDestination,
    ClickHandlerLinkDestination
);

cst_list_node!(StaticLinkDestination, SyntaxTokenChildren);
cst_node!(DynamicLinkDestination, [0, url: Icu]);
cst_node!(ClickHandlerLinkDestination, [0, name: Token]);

cst_node!(LinkTitle, [0, opening_punctuation: Token], [1, title: LinkTitleContent], [2, closing_punctuation: Token]);
cst_list_node!(LinkTitleContent, SyntaxTokenChildren);

cst_node!(Autolink, [0, l_angle: Token], [1, uri: Token], [2, r_angle: Token]);

cst_node!(CodeSpan,
    [0, open_backticks: CodeSpanDelimiter],
    [1, content: CodeSpanContent],
    [2, close_backticks: CodeSpanDelimiter]
);

cst_list_node!(CodeSpanDelimiter, SyntaxTokenChildren);
cst_list_node!(CodeSpanContent, SyntaxTokenChildren);
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
    [0, l_tilde_1: Token],
    [1, l_tilde_2: Option<Token>],
    [2, content: InlineContent],
    [3, r_tilde_1: Token],
    [4, r_tilde_2: Option<Token>]
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
cst_list_node!(IcuPluralArms, Node<IcuPluralArm>);
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
