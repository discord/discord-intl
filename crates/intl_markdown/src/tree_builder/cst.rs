use intl_markdown_macros::ReadFromEvents;

use crate::event::{Event, EventBuffer};
use crate::syntax::SyntaxKind;
use crate::token::{SourceText, Token};
use crate::token::TriviaList;
use crate::tree_builder::{ReadFromEventBuf, TokenSpan};

//#region Boilerplate
pub enum NodeOrToken {
    Node(Node),
    Token(Token),
}

impl NodeOrToken {
    pub fn kind(&self) -> SyntaxKind {
        match self {
            NodeOrToken::Node(node) => node.kind(),
            NodeOrToken::Token(token) => token.kind(),
        }
    }

    pub fn token(&self) -> &Token {
        match &self {
            NodeOrToken::Token(token) => token,
            NodeOrToken::Node(_) => panic!("Called `NodeOrToken::token` on a Node value"),
        }
    }

    pub fn node(&self) -> &Node {
        match &self {
            NodeOrToken::Node(node) => node,
            NodeOrToken::Token(_) => panic!("Called `NodeOrToken::node` on a Token value"),
        }
    }

    pub fn as_token(&self) -> Option<&Token> {
        match &self {
            NodeOrToken::Token(token) => Some(token),
            NodeOrToken::Node(_) => None,
        }
    }

    pub fn as_node(&self) -> Option<&Node> {
        match &self {
            NodeOrToken::Node(node) => Some(node),
            NodeOrToken::Token(_) => None,
        }
    }

    pub fn into_token(self) -> Token {
        match self {
            NodeOrToken::Token(token) => token,
            NodeOrToken::Node(_) => panic!("Called `NodeOrToken::into_token` on a Node value"),
        }
    }

    pub fn into_node(self) -> Node {
        match self {
            NodeOrToken::Node(node) => node,
            NodeOrToken::Token(_) => panic!("Called `NodeOrToken::into_node` on a Token value"),
        }
    }
}

impl TokenSpan for NodeOrToken {
    fn first_token(&self) -> Option<&Token> {
        match self {
            NodeOrToken::Node(node) => node.first_token(),
            NodeOrToken::Token(token) => token.first_token(),
        }
    }

    fn last_token(&self) -> Option<&Token> {
        match self {
            NodeOrToken::Node(node) => node.last_token(),
            NodeOrToken::Token(token) => token.last_token(),
        }
    }
}

impl ReadFromEventBuf for NodeOrToken {
    #[inline(always)]
    fn read_from<I: Iterator<Item = Event>>(buf: &mut EventBuffer<I>) -> Self {
        if matches!(buf.peek(), Some(Event::Token(_))) {
            Self::Token(Token::read_from(buf))
        } else {
            Self::Node(Node::read_from(buf))
        }
    }
}

impl std::fmt::Debug for NodeOrToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Node(v) => v.fmt(f),
            Self::Token(v) => v.fmt(f),
        }
    }
}

macro_rules! cst_token_list {
    ($node_name:ident) => {
        #[derive(Debug, ReadFromEvents)]
        #[repr(transparent)]
        pub struct $node_name {
            children: Vec<Token>,
        }

        impl $node_name {
            pub fn children(&self) -> &Vec<Token> {
                &self.children
            }

            pub fn is_empty(&self) -> bool {
                self.children.is_empty()
            }
        }
    };
}

macro_rules! cst_block_node {
    ($node_name:tt) => {
        #[derive(Debug, ReadFromEvents)]
        #[repr(transparent)]
        pub struct $node_name {
            children: Vec<NodeOrToken>,
        }

        impl $node_name {
            pub fn children(&self) -> &Vec<NodeOrToken> {
                &self.children
            }
        }
    };
}
//#endregion

//#region Markdown Block Nodes
cst_block_node!(Document);
cst_block_node!(InlineContent);
cst_token_list!(ThematicBreak);

#[derive(Debug, ReadFromEvents)]
pub struct Paragraph {
    pub children: InlineContent,
}

#[derive(Debug, ReadFromEvents)]
pub struct IndentedCodeBlock {
    pub content: CodeBlockContent,
}

#[derive(Debug, ReadFromEvents)]
pub struct FencedCodeBlock {
    pub opening_sequence: CodeFenceDelimiter,
    pub info_string: Option<CodeFenceInfoString>,
    pub content: CodeBlockContent,
    pub closing_sequence: Option<CodeFenceDelimiter>,
}

cst_token_list!(CodeBlockContent);
cst_token_list!(CodeFenceDelimiter);
cst_token_list!(CodeFenceInfoString);

#[derive(Debug, ReadFromEvents)]
pub struct AtxHeading {
    pub open_sequence: AtxHashSequence,
    pub children: InlineContent,
    pub closing_sequence: Option<AtxHashSequence>,
}

impl AtxHeading {
    /// Returns the heading level (1-6, inclusive) that this heading should
    /// have according to the opening sequence
    pub fn level(&self) -> usize {
        self.open_sequence.children.len()
    }
}

cst_token_list!(AtxHashSequence);

#[derive(Debug, ReadFromEvents)]
pub struct SetextHeading {
    pub children: InlineContent,
    pub underline: SetextHeadingUnderline,
}

cst_token_list!(SetextHeadingUnderline);

impl SetextHeadingUnderline {
    /// Returns the heading level (1 or 2) that this heading should have
    /// according to the type of underline.
    pub fn level(&self) -> usize {
        match self.children[0].kind() {
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
#[derive(Debug, ReadFromEvents)]
pub struct Emphasis {
    pub open_1: Token,
    pub children: InlineContent,
    pub close_1: Token,
}

impl Emphasis {
    /// Returns the kind of token used for the delimiters in this node.
    pub fn token_kind(&self) -> SyntaxKind {
        self.open_1.kind()
    }
}

#[derive(Debug, ReadFromEvents)]
pub struct Strong {
    pub open_1: Token,
    pub open_2: Token,
    pub children: InlineContent,
    pub close_1: Token,
    pub close_2: Token,
}

impl Strong {
    /// Returns the kind of token used for the delimiters in this node.
    pub fn token_kind(&self) -> SyntaxKind {
        self.open_1.kind()
    }
}

#[derive(Debug, ReadFromEvents)]
pub struct Link {
    pub l_square: Token,
    pub content: InlineContent,
    pub r_square: Token,
    pub resource: LinkResource,
}

#[derive(Debug, ReadFromEvents)]
pub struct Image {
    pub exclaim: Token,
    pub l_square: Token,
    pub content: InlineContent,
    pub r_square: Token,
    pub resource: LinkResource,
}

#[derive(Debug, ReadFromEvents)]
pub struct LinkResource {
    pub l_paren: Token,
    pub destination: Option<LinkDestination>,
    pub title: Option<LinkTitle>,
    pub r_paren: Token,
}

#[derive(ReadFromEvents)]
pub enum LinkDestination {
    StaticLinkDestination(StaticLinkDestination),
    DynamicLinkDestination(DynamicLinkDestination),
    ClickHandlerLinkDestination(ClickHandlerLinkDestination),
}

#[derive(Debug, ReadFromEvents)]
pub struct StaticLinkDestination {
    pub url: Vec<Token>,
}

#[derive(Debug, ReadFromEvents)]
pub struct DynamicLinkDestination {
    pub url: Icu,
}

#[derive(Debug, ReadFromEvents)]
pub struct ClickHandlerLinkDestination {
    pub name: Token,
}

#[derive(Debug, ReadFromEvents)]
pub struct LinkTitle {
    pub opening_punctuation: Token,
    pub title: LinkTitleContent,
    pub closing_punctuation: Token,
}

cst_token_list!(LinkTitleContent);

#[derive(Debug, ReadFromEvents)]
pub struct Autolink {
    pub l_angle: Token,
    pub uri: Token,
    pub r_angle: Token,
}

#[derive(Debug, ReadFromEvents)]
pub struct CodeSpan {
    pub open_backticks: CodeSpanDelimiter,
    pub children: Vec<Token>,
    pub close_backticks: CodeSpanDelimiter,
}

cst_token_list!(CodeSpanDelimiter);
//#endregion

//#region Markdown Extensions
#[derive(Debug, ReadFromEvents)]
pub struct Hook {
    pub dollar: Token,
    pub l_square: Token,
    pub content: InlineContent,
    pub r_square: Token,
    pub name: HookName,
}

#[derive(Debug, ReadFromEvents)]
pub struct HookName {
    pub l_paren: Token,
    pub name: Token,
    pub r_paren: Token,
}

#[derive(Debug, ReadFromEvents)]
pub struct Strikethrough {
    pub l_tilde_1: Token,
    pub l_tilde_2: Option<Token>,
    pub content: InlineContent,
    pub r_tilde_1: Token,
    pub r_tilde_2: Option<Token>,
}
//#endregion

//#region ICU Nodes
#[derive(Debug, ReadFromEvents)]
pub struct Icu {
    pub l_curly: Token,
    pub value: IcuPlaceholder,
    pub r_curly: Token,
}

#[derive(ReadFromEvents)]
pub enum IcuPlaceholder {
    IcuVariable(IcuVariable),
    IcuPlural(IcuPlural),
    IcuSelectOrdinal(IcuSelectOrdinal),
    IcuSelect(IcuSelect),
    IcuDate(IcuDate),
    IcuTime(IcuTime),
    IcuNumber(IcuNumber),
}

#[derive(Debug, ReadFromEvents)]
pub struct IcuVariable {
    pub ident: Token,
}

#[derive(Debug, ReadFromEvents)]
pub struct IcuPlural {
    pub variable: IcuVariable,
    pub variable_comma: Token,
    pub format_token: Token,
    pub format_comma: Token,
    pub arms: Vec<IcuPluralArm>,
}

#[derive(Debug, ReadFromEvents)]
pub struct IcuSelect {
    pub variable: IcuVariable,
    pub variable_comma: Token,
    pub format_token: Token,
    pub format_comma: Token,
    pub arms: Vec<IcuPluralArm>,
}

#[derive(Debug, ReadFromEvents)]
pub struct IcuSelectOrdinal {
    pub variable: IcuVariable,
    pub variable_comma: Token,
    pub format_token: Token,
    pub format_comma: Token,
    pub arms: Vec<IcuPluralArm>,
}

#[derive(Debug, ReadFromEvents)]
pub struct IcuPluralArm {
    pub selector: Token,
    pub l_curly: Token,
    pub value: IcuPluralValue,
    pub r_curly: Token,
}

#[derive(Debug, ReadFromEvents)]
pub struct IcuPluralValue {
    pub content: InlineContent,
}

#[derive(Debug, ReadFromEvents)]
pub struct IcuDate {
    pub variable: IcuVariable,
    pub variable_comma: Token,
    pub format_token: Token,
    pub style: Option<IcuDateTimeStyle>,
}
#[derive(Debug, ReadFromEvents)]
pub struct IcuTime {
    pub variable: IcuVariable,
    pub variable_comma: Token,
    pub format_token: Token,
    pub style: Option<IcuDateTimeStyle>,
}

#[derive(Debug, ReadFromEvents)]
pub struct IcuDateTimeStyle {
    pub leading_comma: Token,
    pub style_text: Token,
}

#[derive(Debug, ReadFromEvents)]
pub struct IcuNumber {
    pub variable: IcuVariable,
    pub variable_comma: Token,
    pub format_token: Token,
    pub style: Option<IcuNumberStyle>,
}

#[derive(Debug, ReadFromEvents)]
pub struct IcuNumberStyle {
    pub leading_comma: Token,
    pub style_text: Token,
}
//#endregion

#[derive(ReadFromEvents)]
pub enum Node {
    Paragraph(Paragraph),
    ThematicBreak(ThematicBreak),
    AtxHeading(AtxHeading),
    SetextHeading(SetextHeading),
    IndentedCodeBlock(IndentedCodeBlock),
    FencedCodeBlock(FencedCodeBlock),
    InlineContent(InlineContent),
    Emphasis(Emphasis),
    Strong(Strong),
    Link(Link),
    Image(Image),
    Autolink(Autolink),
    CodeSpan(CodeSpan),
    Hook(Hook),
    Strikethrough(Strikethrough),
    Icu(Icu),
}

pub fn parser_events_to_cst(buf: Vec<Event>, source: SourceText, trivia: TriviaList) -> Document {
    let only_important_events = buf
        .into_iter()
        .filter(|event| !matches!(event.kind(), SyntaxKind::TOMBSTONE));
    Document::read_from(&mut EventBuffer::new(only_important_events, source, trivia))
}
