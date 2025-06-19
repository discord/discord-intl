use crate::cst::*;
use crate::syntax::{FromSyntax, FromSyntaxElement, Syntax, SyntaxElement};
use crate::{SyntaxKind, SyntaxToken};

pub enum TokenOr<T> {
    Token(SyntaxToken),
    Or(T),
}

impl<T> From<SyntaxToken> for TokenOr<T> {
    fn from(value: SyntaxToken) -> Self {
        TokenOr::Token(value)
    }
}

impl<T: FromSyntax> From<T> for TokenOr<T> {
    fn from(value: T) -> Self {
        TokenOr::Or(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AnyLink {
    Link(Link),
    Image(Image),
    Autolink(Autolink),
}

impl AnyLink {
    pub fn label(&self) -> TokenOr<InlineContent> {
        match self {
            AnyLink::Link(link) => link.content().into(),
            AnyLink::Image(image) => image.content().into(),
            AnyLink::Autolink(autolink) => autolink.uri_token().into(),
        }
    }

    pub fn destination(&self) -> Option<AnyLinkDestination> {
        match self {
            AnyLink::Link(link) => link.resource().destination(),
            AnyLink::Image(image) => image.resource().destination(),
            // TODO: fix this
            AnyLink::Autolink(_) => None,
        }
    }

    pub fn title(&self) -> Option<LinkTitle> {
        match self {
            AnyLink::Link(link) => link.resource().title(),
            AnyLink::Image(image) => image.resource().title(),
            AnyLink::Autolink(_) => None,
        }
    }

    pub fn is_email(&self) -> bool {
        match self {
            AnyLink::Autolink(link) => matches!(link.uri_token().kind(), SyntaxKind::EMAIL_ADDRESS),
            _ => false,
        }
    }
}

impl From<SyntaxElement> for AnyLink {
    fn from(value: SyntaxElement) -> Self {
        match value.kind() {
            SyntaxKind::LINK => Self::Link(Link::from_syntax_element(value)),
            SyntaxKind::IMAGE => Self::Image(Image::from_syntax_element(value)),
            SyntaxKind::AUTOLINK => Self::Autolink(Autolink::from_syntax_element(value)),
            _ => unreachable!("Invalid syntax kind for AnyLink: {:?}", value.kind()),
        }
    }
}

impl From<Link> for AnyLink {
    fn from(value: Link) -> Self {
        Self::Link(value)
    }
}
impl From<Image> for AnyLink {
    fn from(value: Image) -> Self {
        Self::Image(value)
    }
}
impl From<Autolink> for AnyLink {
    fn from(value: Autolink) -> Self {
        Self::Autolink(value)
    }
}
impl From<&Link> for AnyLink {
    fn from(value: &Link) -> Self {
        Self::Link(value.clone())
    }
}
impl From<&Image> for AnyLink {
    fn from(value: &Image) -> Self {
        Self::Image(value.clone())
    }
}
impl From<&Autolink> for AnyLink {
    fn from(value: &Autolink) -> Self {
        Self::Autolink(value.clone())
    }
}

impl FencedCodeBlock {
    fn foo() {}
}

impl AtxHeading {
    /// Returns the heading level (1-6, inclusive) that this heading should
    /// have according to the opening sequence
    pub fn level(&self) -> usize {
        self.opening_run_token().text_len() as usize
    }
}
impl SetextHeadingUnderline {
    /// Returns the heading level (1 or 2) that this heading should have
    /// according to the type of underline.
    pub fn level(&self) -> usize {
        match self.syntax().children()[0].kind() {
            SyntaxKind::EQUAL => 1,
            SyntaxKind::MINUS => 2,
            found => unreachable!(
                "Found a setext heading underline character that is invalid: {:?}",
                found
            ),
        }
    }
}
