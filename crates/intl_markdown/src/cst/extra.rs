use crate::cst::*;
use crate::SyntaxKind;
use intl_markdown_syntax::{Syntax, SyntaxToken};

impl AnyHeading {
    pub fn level(&self) -> u8 {
        match self {
            AnyHeading::AtxHeading(node) => node.level(),
            AnyHeading::SetextHeading(node) => node.level(),
        }
    }
}

impl AtxHeading {
    /// Returns the heading level (1-6, inclusive) that this heading should
    /// have according to the opening sequence
    pub fn level(&self) -> u8 {
        self.opening_run_token().text_len() as u8
    }
}

impl SetextHeading {
    pub fn level(&self) -> u8 {
        self.underline().level()
    }
}

impl SetextHeadingUnderline {
    /// Returns the heading level (1 or 2) that this heading should have
    /// according to the type of underline.
    pub fn level(&self) -> u8 {
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

impl AnyCodeBlock {
    pub fn info_string(&self) -> Option<CodeBlockInfoString> {
        match self {
            AnyCodeBlock::IndentedCodeBlock(_) => None,
            AnyCodeBlock::FencedCodeBlock(block) => block.info_string(),
        }
    }
}

impl Link {
    pub fn destination(&self) -> Option<AnyLinkDestination> {
        self.resource().destination()
    }
    pub fn title(&self) -> Option<LinkTitle> {
        self.resource().title()
    }
}

impl Image {
    pub fn destination(&self) -> Option<AnyLinkDestination> {
        self.resource().destination()
    }
    pub fn title(&self) -> Option<LinkTitle> {
        self.resource().title()
    }
}

impl Autolink {
    pub fn is_email(&self) -> bool {
        self.uri_token().kind() == SyntaxKind::EMAIL_ADDRESS
    }
}

impl Icu {
    pub fn ident_token(&self) -> SyntaxToken {
        self.value().ident_token()
    }

    pub fn is_unsafe(&self) -> bool {
        self.l_curly_token().kind() == SyntaxKind::UNSAFE_LCURLY
            && self.r_curly_token().kind() == SyntaxKind::UNSAFE_RCURLY
    }
}

impl AnyIcuExpression {
    pub fn ident_token(&self) -> SyntaxToken {
        match self {
            AnyIcuExpression::IcuPlaceholder(placeholder) => placeholder.ident_token(),
            AnyIcuExpression::IcuPlural(plural) => plural.ident_token(),
            AnyIcuExpression::IcuSelectOrdinal(ordinal) => ordinal.ident_token(),
            AnyIcuExpression::IcuSelect(select) => select.ident_token(),
            AnyIcuExpression::IcuDate(date) => date.ident_token(),
            AnyIcuExpression::IcuTime(time) => time.ident_token(),
            AnyIcuExpression::IcuNumber(number) => number.ident_token(),
        }
    }
}
