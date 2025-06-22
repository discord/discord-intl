use crate::cst::util::*;
use crate::syntax::*;
#[derive(Clone, Eq, PartialEq)]
#[repr(transparent)]
pub struct Document {
    syntax: SyntaxNode,
}
impl Syntax for Document {
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl FromSyntax for Document {
    fn from_syntax(syntax: SyntaxNode) -> Self {
        Self { syntax }
    }
}
impl Document {
    pub fn len(&self) -> usize {
        self.syntax.len()
    }
    pub fn children(&self) -> TypedNodeChildren<AnyBlockNode> {
        TypedNodeChildren::new(SyntaxNodeChildren::new(self.syntax.children()))
    }
    pub fn get(&self, index: usize) -> Option<AnyBlockNode> {
        self.syntax
            .get(index)
            .map(|node| AnyBlockNode::from_syntax_element(node.clone()))
    }
}
impl std::ops::Index<usize> for Document {
    type Output = SyntaxToken;
    fn index(&self, index: usize) -> &Self::Output {
        &self.syntax[index].token()
    }
}
impl std::fmt::Debug for Document {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Document")?;
        f.debug_list().entries(self.children()).finish()
    }
}
#[derive(Clone, Eq, PartialEq)]
pub enum AnyBlockNode {
    Paragraph(Paragraph),
    ThematicBreak(ThematicBreak),
    Heading(AnyHeading),
    CodeBlock(AnyCodeBlock),
    InlineContent(InlineContent),
}
impl Syntax for AnyBlockNode {
    fn syntax(&self) -> &SyntaxNode {
        match self {
            Self::Paragraph(node) => node.syntax(),
            Self::ThematicBreak(node) => node.syntax(),
            Self::Heading(node) => node.syntax(),
            Self::CodeBlock(node) => node.syntax(),
            Self::InlineContent(node) => node.syntax(),
        }
    }
}
impl FromSyntax for AnyBlockNode {
    fn from_syntax(syntax: SyntaxNode) -> Self {
        match syntax.kind() {
            SyntaxKind::PARAGRAPH => Self::Paragraph(Paragraph::from_syntax(syntax)),
            SyntaxKind::THEMATIC_BREAK => Self::ThematicBreak(ThematicBreak::from_syntax(syntax)),
            SyntaxKind::ATX_HEADING => Self::Heading(AnyHeading::from_syntax(syntax)),
            SyntaxKind::SETEXT_HEADING => Self::Heading(AnyHeading::from_syntax(syntax)),
            SyntaxKind::INDENTED_CODE_BLOCK => Self::CodeBlock(AnyCodeBlock::from_syntax(syntax)),
            SyntaxKind::FENCED_CODE_BLOCK => Self::CodeBlock(AnyCodeBlock::from_syntax(syntax)),
            SyntaxKind::INLINE_CONTENT => Self::InlineContent(InlineContent::from_syntax(syntax)),
            kind => unreachable!(
                "Invalid syntax kind {:?} encountered when constructing enum node {}",
                kind, "AnyBlockNode"
            ),
        }
    }
}
impl From<Paragraph> for AnyBlockNode {
    fn from(value: Paragraph) -> Self {
        Self::Paragraph(value)
    }
}
impl From<ThematicBreak> for AnyBlockNode {
    fn from(value: ThematicBreak) -> Self {
        Self::ThematicBreak(value)
    }
}
impl From<AnyHeading> for AnyBlockNode {
    fn from(value: AnyHeading) -> Self {
        Self::Heading(value)
    }
}
impl From<AnyCodeBlock> for AnyBlockNode {
    fn from(value: AnyCodeBlock) -> Self {
        Self::CodeBlock(value)
    }
}
impl From<InlineContent> for AnyBlockNode {
    fn from(value: InlineContent) -> Self {
        Self::InlineContent(value)
    }
}
impl std::fmt::Debug for AnyBlockNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut tuple = f.debug_tuple("AnyBlockNode");
        match self {
            Self::Paragraph(node) => tuple.field(node),
            Self::ThematicBreak(node) => tuple.field(node),
            Self::Heading(node) => tuple.field(node),
            Self::CodeBlock(node) => tuple.field(node),
            Self::InlineContent(node) => tuple.field(node),
        };
        tuple.finish()
    }
}
#[derive(Clone, Eq, PartialEq)]
#[repr(transparent)]
pub struct Paragraph {
    syntax: SyntaxNode,
}
impl Syntax for Paragraph {
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl FromSyntax for Paragraph {
    fn from_syntax(syntax: SyntaxNode) -> Self {
        Self { syntax }
    }
}
impl Paragraph {
    pub fn content(&self) -> InlineContent {
        support::required_node(&self.syntax, 0usize)
    }
}
impl std::fmt::Debug for Paragraph {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Paragraph")
            .field("[0] content", &self.content())
            .finish()
    }
}
#[derive(Clone, Eq, PartialEq)]
#[repr(transparent)]
pub struct ThematicBreak {
    syntax: SyntaxNode,
}
impl Syntax for ThematicBreak {
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl FromSyntax for ThematicBreak {
    fn from_syntax(syntax: SyntaxNode) -> Self {
        Self { syntax }
    }
}
impl ThematicBreak {
    pub fn len(&self) -> usize {
        self.syntax.len()
    }
    pub fn children(&self) -> SyntaxTokenChildren {
        SyntaxTokenChildren::new(self.syntax.children())
    }
    pub fn get(&self, index: usize) -> Option<&SyntaxToken> {
        self.syntax.get(index).map(|element| element.token())
    }
}
impl std::ops::Index<usize> for ThematicBreak {
    type Output = SyntaxToken;
    fn index(&self, index: usize) -> &Self::Output {
        &self.syntax[index].token()
    }
}
impl std::fmt::Debug for ThematicBreak {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("ThematicBreak")?;
        f.debug_list().entries(self.children()).finish()
    }
}
#[derive(Clone, Eq, PartialEq)]
pub enum AnyHeading {
    AtxHeading(AtxHeading),
    SetextHeading(SetextHeading),
}
impl Syntax for AnyHeading {
    fn syntax(&self) -> &SyntaxNode {
        match self {
            Self::AtxHeading(node) => node.syntax(),
            Self::SetextHeading(node) => node.syntax(),
        }
    }
}
impl FromSyntax for AnyHeading {
    fn from_syntax(syntax: SyntaxNode) -> Self {
        match syntax.kind() {
            SyntaxKind::ATX_HEADING => Self::AtxHeading(AtxHeading::from_syntax(syntax)),
            SyntaxKind::SETEXT_HEADING => Self::SetextHeading(SetextHeading::from_syntax(syntax)),
            kind => unreachable!(
                "Invalid syntax kind {:?} encountered when constructing enum node {}",
                kind, "AnyHeading"
            ),
        }
    }
}
impl From<AtxHeading> for AnyHeading {
    fn from(value: AtxHeading) -> Self {
        Self::AtxHeading(value)
    }
}
impl From<SetextHeading> for AnyHeading {
    fn from(value: SetextHeading) -> Self {
        Self::SetextHeading(value)
    }
}
impl std::fmt::Debug for AnyHeading {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut tuple = f.debug_tuple("AnyHeading");
        match self {
            Self::AtxHeading(node) => tuple.field(node),
            Self::SetextHeading(node) => tuple.field(node),
        };
        tuple.finish()
    }
}
#[derive(Clone, Eq, PartialEq)]
pub enum AnyCodeBlock {
    IndentedCodeBlock(IndentedCodeBlock),
    FencedCodeBlock(FencedCodeBlock),
}
impl Syntax for AnyCodeBlock {
    fn syntax(&self) -> &SyntaxNode {
        match self {
            Self::IndentedCodeBlock(node) => node.syntax(),
            Self::FencedCodeBlock(node) => node.syntax(),
        }
    }
}
impl FromSyntax for AnyCodeBlock {
    fn from_syntax(syntax: SyntaxNode) -> Self {
        match syntax.kind() {
            SyntaxKind::INDENTED_CODE_BLOCK => {
                Self::IndentedCodeBlock(IndentedCodeBlock::from_syntax(syntax))
            }
            SyntaxKind::FENCED_CODE_BLOCK => {
                Self::FencedCodeBlock(FencedCodeBlock::from_syntax(syntax))
            }
            kind => unreachable!(
                "Invalid syntax kind {:?} encountered when constructing enum node {}",
                kind, "AnyCodeBlock"
            ),
        }
    }
}
impl From<IndentedCodeBlock> for AnyCodeBlock {
    fn from(value: IndentedCodeBlock) -> Self {
        Self::IndentedCodeBlock(value)
    }
}
impl From<FencedCodeBlock> for AnyCodeBlock {
    fn from(value: FencedCodeBlock) -> Self {
        Self::FencedCodeBlock(value)
    }
}
impl std::fmt::Debug for AnyCodeBlock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut tuple = f.debug_tuple("AnyCodeBlock");
        match self {
            Self::IndentedCodeBlock(node) => tuple.field(node),
            Self::FencedCodeBlock(node) => tuple.field(node),
        };
        tuple.finish()
    }
}
#[derive(Clone, Eq, PartialEq)]
#[repr(transparent)]
pub struct InlineContent {
    syntax: SyntaxNode,
}
impl Syntax for InlineContent {
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl FromSyntax for InlineContent {
    fn from_syntax(syntax: SyntaxNode) -> Self {
        Self { syntax }
    }
}
impl InlineContent {
    pub fn len(&self) -> usize {
        self.syntax.len()
    }
    pub fn children(&self) -> TypedNodeChildren<AnyInlineNode> {
        TypedNodeChildren::new(SyntaxNodeChildren::new(self.syntax.children()))
    }
    pub fn get(&self, index: usize) -> Option<AnyInlineNode> {
        self.syntax
            .get(index)
            .map(|node| AnyInlineNode::from_syntax_element(node.clone()))
    }
}
impl std::ops::Index<usize> for InlineContent {
    type Output = SyntaxToken;
    fn index(&self, index: usize) -> &Self::Output {
        &self.syntax[index].token()
    }
}
impl std::fmt::Debug for InlineContent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("InlineContent")?;
        f.debug_list().entries(self.children()).finish()
    }
}
#[derive(Clone, Eq, PartialEq)]
#[repr(transparent)]
pub struct AtxHeading {
    syntax: SyntaxNode,
}
impl Syntax for AtxHeading {
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl FromSyntax for AtxHeading {
    fn from_syntax(syntax: SyntaxNode) -> Self {
        Self { syntax }
    }
}
impl AtxHeading {
    pub fn opening_run_token(&self) -> SyntaxToken {
        support::required_token(&self.syntax, 0usize)
    }
    pub fn content(&self) -> InlineContent {
        support::required_node(&self.syntax, 1usize)
    }
    pub fn closing_run_token(&self) -> Option<SyntaxToken> {
        support::optional_token(&self.syntax, 2usize)
    }
}
impl std::fmt::Debug for AtxHeading {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AtxHeading")
            .field("[0] opening_run_token", &self.opening_run_token())
            .field("[1] content", &self.content())
            .field("[2] closing_run_token", &self.closing_run_token())
            .finish()
    }
}
#[derive(Clone, Eq, PartialEq)]
#[repr(transparent)]
pub struct SetextHeading {
    syntax: SyntaxNode,
}
impl Syntax for SetextHeading {
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl FromSyntax for SetextHeading {
    fn from_syntax(syntax: SyntaxNode) -> Self {
        Self { syntax }
    }
}
impl SetextHeading {
    pub fn content(&self) -> InlineContent {
        support::required_node(&self.syntax, 0usize)
    }
    pub fn underline(&self) -> SetextHeadingUnderline {
        support::required_node(&self.syntax, 1usize)
    }
}
impl std::fmt::Debug for SetextHeading {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SetextHeading")
            .field("[0] content", &self.content())
            .field("[1] underline", &self.underline())
            .finish()
    }
}
#[derive(Clone, Eq, PartialEq)]
#[repr(transparent)]
pub struct SetextHeadingUnderline {
    syntax: SyntaxNode,
}
impl Syntax for SetextHeadingUnderline {
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl FromSyntax for SetextHeadingUnderline {
    fn from_syntax(syntax: SyntaxNode) -> Self {
        Self { syntax }
    }
}
impl SetextHeadingUnderline {
    pub fn len(&self) -> usize {
        self.syntax.len()
    }
    pub fn children(&self) -> SyntaxTokenChildren {
        SyntaxTokenChildren::new(self.syntax.children())
    }
    pub fn get(&self, index: usize) -> Option<&SyntaxToken> {
        self.syntax.get(index).map(|element| element.token())
    }
}
impl std::ops::Index<usize> for SetextHeadingUnderline {
    type Output = SyntaxToken;
    fn index(&self, index: usize) -> &Self::Output {
        &self.syntax[index].token()
    }
}
impl std::fmt::Debug for SetextHeadingUnderline {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("SetextHeadingUnderline")?;
        f.debug_list().entries(self.children()).finish()
    }
}
#[derive(Clone, Eq, PartialEq)]
#[repr(transparent)]
pub struct IndentedCodeBlock {
    syntax: SyntaxNode,
}
impl Syntax for IndentedCodeBlock {
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl FromSyntax for IndentedCodeBlock {
    fn from_syntax(syntax: SyntaxNode) -> Self {
        Self { syntax }
    }
}
impl IndentedCodeBlock {
    pub fn content(&self) -> CodeBlockContent {
        support::required_node(&self.syntax, 0usize)
    }
}
impl std::fmt::Debug for IndentedCodeBlock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IndentedCodeBlock")
            .field("[0] content", &self.content())
            .finish()
    }
}
#[derive(Clone, Eq, PartialEq)]
#[repr(transparent)]
pub struct FencedCodeBlock {
    syntax: SyntaxNode,
}
impl Syntax for FencedCodeBlock {
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl FromSyntax for FencedCodeBlock {
    fn from_syntax(syntax: SyntaxNode) -> Self {
        Self { syntax }
    }
}
impl FencedCodeBlock {
    pub fn opening_run_token(&self) -> SyntaxToken {
        support::required_token(&self.syntax, 0usize)
    }
    pub fn info_string(&self) -> Option<CodeBlockInfoString> {
        support::optional_node(&self.syntax, 1usize)
    }
    pub fn content(&self) -> CodeBlockContent {
        support::required_node(&self.syntax, 2usize)
    }
    pub fn closing_run_token(&self) -> Option<SyntaxToken> {
        support::optional_token(&self.syntax, 3usize)
    }
}
impl std::fmt::Debug for FencedCodeBlock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FencedCodeBlock")
            .field("[0] opening_run_token", &self.opening_run_token())
            .field("[1] info_string", &self.info_string())
            .field("[2] content", &self.content())
            .field("[3] closing_run_token", &self.closing_run_token())
            .finish()
    }
}
#[derive(Clone, Eq, PartialEq)]
#[repr(transparent)]
pub struct CodeBlockContent {
    syntax: SyntaxNode,
}
impl Syntax for CodeBlockContent {
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl FromSyntax for CodeBlockContent {
    fn from_syntax(syntax: SyntaxNode) -> Self {
        Self { syntax }
    }
}
impl CodeBlockContent {
    pub fn len(&self) -> usize {
        self.syntax.len()
    }
    pub fn children(&self) -> SyntaxTokenChildren {
        SyntaxTokenChildren::new(self.syntax.children())
    }
    pub fn get(&self, index: usize) -> Option<&SyntaxToken> {
        self.syntax.get(index).map(|element| element.token())
    }
}
impl std::ops::Index<usize> for CodeBlockContent {
    type Output = SyntaxToken;
    fn index(&self, index: usize) -> &Self::Output {
        &self.syntax[index].token()
    }
}
impl std::fmt::Debug for CodeBlockContent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("CodeBlockContent")?;
        f.debug_list().entries(self.children()).finish()
    }
}
#[derive(Clone, Eq, PartialEq)]
#[repr(transparent)]
pub struct CodeBlockInfoString {
    syntax: SyntaxNode,
}
impl Syntax for CodeBlockInfoString {
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl FromSyntax for CodeBlockInfoString {
    fn from_syntax(syntax: SyntaxNode) -> Self {
        Self { syntax }
    }
}
impl CodeBlockInfoString {
    pub fn len(&self) -> usize {
        self.syntax.len()
    }
    pub fn children(&self) -> SyntaxTokenChildren {
        SyntaxTokenChildren::new(self.syntax.children())
    }
    pub fn get(&self, index: usize) -> Option<&SyntaxToken> {
        self.syntax.get(index).map(|element| element.token())
    }
}
impl std::ops::Index<usize> for CodeBlockInfoString {
    type Output = SyntaxToken;
    fn index(&self, index: usize) -> &Self::Output {
        &self.syntax[index].token()
    }
}
impl std::fmt::Debug for CodeBlockInfoString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("CodeBlockInfoString")?;
        f.debug_list().entries(self.children()).finish()
    }
}
#[derive(Clone, Eq, PartialEq)]
pub enum AnyInlineNode {
    TextSpan(TextSpan),
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
impl Syntax for AnyInlineNode {
    fn syntax(&self) -> &SyntaxNode {
        match self {
            Self::TextSpan(node) => node.syntax(),
            Self::Emphasis(node) => node.syntax(),
            Self::Strong(node) => node.syntax(),
            Self::Link(node) => node.syntax(),
            Self::Image(node) => node.syntax(),
            Self::Autolink(node) => node.syntax(),
            Self::CodeSpan(node) => node.syntax(),
            Self::Hook(node) => node.syntax(),
            Self::Strikethrough(node) => node.syntax(),
            Self::Icu(node) => node.syntax(),
        }
    }
}
impl FromSyntax for AnyInlineNode {
    fn from_syntax(syntax: SyntaxNode) -> Self {
        match syntax.kind() {
            SyntaxKind::TEXT_SPAN => Self::TextSpan(TextSpan::from_syntax(syntax)),
            SyntaxKind::EMPHASIS => Self::Emphasis(Emphasis::from_syntax(syntax)),
            SyntaxKind::STRONG => Self::Strong(Strong::from_syntax(syntax)),
            SyntaxKind::LINK => Self::Link(Link::from_syntax(syntax)),
            SyntaxKind::IMAGE => Self::Image(Image::from_syntax(syntax)),
            SyntaxKind::AUTOLINK => Self::Autolink(Autolink::from_syntax(syntax)),
            SyntaxKind::CODE_SPAN => Self::CodeSpan(CodeSpan::from_syntax(syntax)),
            SyntaxKind::HOOK => Self::Hook(Hook::from_syntax(syntax)),
            SyntaxKind::STRIKETHROUGH => Self::Strikethrough(Strikethrough::from_syntax(syntax)),
            SyntaxKind::ICU => Self::Icu(Icu::from_syntax(syntax)),
            kind => unreachable!(
                "Invalid syntax kind {:?} encountered when constructing enum node {}",
                kind, "AnyInlineNode"
            ),
        }
    }
}
impl From<TextSpan> for AnyInlineNode {
    fn from(value: TextSpan) -> Self {
        Self::TextSpan(value)
    }
}
impl From<Emphasis> for AnyInlineNode {
    fn from(value: Emphasis) -> Self {
        Self::Emphasis(value)
    }
}
impl From<Strong> for AnyInlineNode {
    fn from(value: Strong) -> Self {
        Self::Strong(value)
    }
}
impl From<Link> for AnyInlineNode {
    fn from(value: Link) -> Self {
        Self::Link(value)
    }
}
impl From<Image> for AnyInlineNode {
    fn from(value: Image) -> Self {
        Self::Image(value)
    }
}
impl From<Autolink> for AnyInlineNode {
    fn from(value: Autolink) -> Self {
        Self::Autolink(value)
    }
}
impl From<CodeSpan> for AnyInlineNode {
    fn from(value: CodeSpan) -> Self {
        Self::CodeSpan(value)
    }
}
impl From<Hook> for AnyInlineNode {
    fn from(value: Hook) -> Self {
        Self::Hook(value)
    }
}
impl From<Strikethrough> for AnyInlineNode {
    fn from(value: Strikethrough) -> Self {
        Self::Strikethrough(value)
    }
}
impl From<Icu> for AnyInlineNode {
    fn from(value: Icu) -> Self {
        Self::Icu(value)
    }
}
impl std::fmt::Debug for AnyInlineNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut tuple = f.debug_tuple("AnyInlineNode");
        match self {
            Self::TextSpan(node) => tuple.field(node),
            Self::Emphasis(node) => tuple.field(node),
            Self::Strong(node) => tuple.field(node),
            Self::Link(node) => tuple.field(node),
            Self::Image(node) => tuple.field(node),
            Self::Autolink(node) => tuple.field(node),
            Self::CodeSpan(node) => tuple.field(node),
            Self::Hook(node) => tuple.field(node),
            Self::Strikethrough(node) => tuple.field(node),
            Self::Icu(node) => tuple.field(node),
        };
        tuple.finish()
    }
}
#[derive(Clone, Eq, PartialEq)]
#[repr(transparent)]
pub struct TextSpan {
    syntax: SyntaxNode,
}
impl Syntax for TextSpan {
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl FromSyntax for TextSpan {
    fn from_syntax(syntax: SyntaxNode) -> Self {
        Self { syntax }
    }
}
impl TextSpan {
    pub fn len(&self) -> usize {
        self.syntax.len()
    }
    pub fn children(&self) -> SyntaxTokenChildren {
        SyntaxTokenChildren::new(self.syntax.children())
    }
    pub fn get(&self, index: usize) -> Option<&SyntaxToken> {
        self.syntax.get(index).map(|element| element.token())
    }
}
impl std::ops::Index<usize> for TextSpan {
    type Output = SyntaxToken;
    fn index(&self, index: usize) -> &Self::Output {
        &self.syntax[index].token()
    }
}
impl std::fmt::Debug for TextSpan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("TextSpan")?;
        f.debug_list().entries(self.children()).finish()
    }
}
#[derive(Clone, Eq, PartialEq)]
#[repr(transparent)]
pub struct Emphasis {
    syntax: SyntaxNode,
}
impl Syntax for Emphasis {
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl FromSyntax for Emphasis {
    fn from_syntax(syntax: SyntaxNode) -> Self {
        Self { syntax }
    }
}
impl Emphasis {
    pub fn open_token(&self) -> SyntaxToken {
        support::required_token(&self.syntax, 0usize)
    }
    pub fn content(&self) -> InlineContent {
        support::required_node(&self.syntax, 1usize)
    }
    pub fn close_token(&self) -> SyntaxToken {
        support::required_token(&self.syntax, 2usize)
    }
}
impl std::fmt::Debug for Emphasis {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Emphasis")
            .field("[0] open_token", &self.open_token())
            .field("[1] content", &self.content())
            .field("[2] close_token", &self.close_token())
            .finish()
    }
}
#[derive(Clone, Eq, PartialEq)]
#[repr(transparent)]
pub struct Strong {
    syntax: SyntaxNode,
}
impl Syntax for Strong {
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl FromSyntax for Strong {
    fn from_syntax(syntax: SyntaxNode) -> Self {
        Self { syntax }
    }
}
impl Strong {
    pub fn open_token(&self) -> SyntaxToken {
        support::required_token(&self.syntax, 0usize)
    }
    pub fn open_two_token(&self) -> SyntaxToken {
        support::required_token(&self.syntax, 1usize)
    }
    pub fn content(&self) -> InlineContent {
        support::required_node(&self.syntax, 2usize)
    }
    pub fn close_token(&self) -> SyntaxToken {
        support::required_token(&self.syntax, 3usize)
    }
    pub fn close_two_token(&self) -> SyntaxToken {
        support::required_token(&self.syntax, 4usize)
    }
}
impl std::fmt::Debug for Strong {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Strong")
            .field("[0] open_token", &self.open_token())
            .field("[1] open_two_token", &self.open_two_token())
            .field("[2] content", &self.content())
            .field("[3] close_token", &self.close_token())
            .field("[4] close_two_token", &self.close_two_token())
            .finish()
    }
}
#[derive(Clone, Eq, PartialEq)]
#[repr(transparent)]
pub struct Link {
    syntax: SyntaxNode,
}
impl Syntax for Link {
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl FromSyntax for Link {
    fn from_syntax(syntax: SyntaxNode) -> Self {
        Self { syntax }
    }
}
impl Link {
    pub fn l_square_token(&self) -> SyntaxToken {
        support::required_token(&self.syntax, 0usize)
    }
    pub fn content(&self) -> InlineContent {
        support::required_node(&self.syntax, 1usize)
    }
    pub fn r_square_token(&self) -> SyntaxToken {
        support::required_token(&self.syntax, 2usize)
    }
    pub fn resource(&self) -> LinkResource {
        support::required_node(&self.syntax, 3usize)
    }
}
impl std::fmt::Debug for Link {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Link")
            .field("[0] l_square_token", &self.l_square_token())
            .field("[1] content", &self.content())
            .field("[2] r_square_token", &self.r_square_token())
            .field("[3] resource", &self.resource())
            .finish()
    }
}
#[derive(Clone, Eq, PartialEq)]
#[repr(transparent)]
pub struct Image {
    syntax: SyntaxNode,
}
impl Syntax for Image {
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl FromSyntax for Image {
    fn from_syntax(syntax: SyntaxNode) -> Self {
        Self { syntax }
    }
}
impl Image {
    pub fn exclaim_token(&self) -> SyntaxToken {
        support::required_token(&self.syntax, 0usize)
    }
    pub fn l_square_token(&self) -> SyntaxToken {
        support::required_token(&self.syntax, 1usize)
    }
    pub fn content(&self) -> InlineContent {
        support::required_node(&self.syntax, 2usize)
    }
    pub fn r_square_token(&self) -> SyntaxToken {
        support::required_token(&self.syntax, 3usize)
    }
    pub fn resource(&self) -> LinkResource {
        support::required_node(&self.syntax, 4usize)
    }
}
impl std::fmt::Debug for Image {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Image")
            .field("[0] exclaim_token", &self.exclaim_token())
            .field("[1] l_square_token", &self.l_square_token())
            .field("[2] content", &self.content())
            .field("[3] r_square_token", &self.r_square_token())
            .field("[4] resource", &self.resource())
            .finish()
    }
}
#[derive(Clone, Eq, PartialEq)]
#[repr(transparent)]
pub struct Autolink {
    syntax: SyntaxNode,
}
impl Syntax for Autolink {
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl FromSyntax for Autolink {
    fn from_syntax(syntax: SyntaxNode) -> Self {
        Self { syntax }
    }
}
impl Autolink {
    pub fn l_angle_token(&self) -> SyntaxToken {
        support::required_token(&self.syntax, 0usize)
    }
    pub fn uri_token(&self) -> SyntaxToken {
        support::required_token(&self.syntax, 1usize)
    }
    pub fn r_angle_token(&self) -> SyntaxToken {
        support::required_token(&self.syntax, 2usize)
    }
}
impl std::fmt::Debug for Autolink {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Autolink")
            .field("[0] l_angle_token", &self.l_angle_token())
            .field("[1] uri_token", &self.uri_token())
            .field("[2] r_angle_token", &self.r_angle_token())
            .finish()
    }
}
#[derive(Clone, Eq, PartialEq)]
#[repr(transparent)]
pub struct CodeSpan {
    syntax: SyntaxNode,
}
impl Syntax for CodeSpan {
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl FromSyntax for CodeSpan {
    fn from_syntax(syntax: SyntaxNode) -> Self {
        Self { syntax }
    }
}
impl CodeSpan {
    pub fn open_run_token(&self) -> SyntaxToken {
        support::required_token(&self.syntax, 0usize)
    }
    pub fn content(&self) -> CodeSpanContent {
        support::required_node(&self.syntax, 1usize)
    }
    pub fn close_run_token(&self) -> SyntaxToken {
        support::required_token(&self.syntax, 2usize)
    }
}
impl std::fmt::Debug for CodeSpan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CodeSpan")
            .field("[0] open_run_token", &self.open_run_token())
            .field("[1] content", &self.content())
            .field("[2] close_run_token", &self.close_run_token())
            .finish()
    }
}
#[derive(Clone, Eq, PartialEq)]
#[repr(transparent)]
pub struct Hook {
    syntax: SyntaxNode,
}
impl Syntax for Hook {
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl FromSyntax for Hook {
    fn from_syntax(syntax: SyntaxNode) -> Self {
        Self { syntax }
    }
}
impl Hook {
    pub fn _token(&self) -> SyntaxToken {
        support::required_token(&self.syntax, 0usize)
    }
    pub fn l_square_token(&self) -> SyntaxToken {
        support::required_token(&self.syntax, 1usize)
    }
    pub fn content(&self) -> InlineContent {
        support::required_node(&self.syntax, 2usize)
    }
    pub fn r_square_token(&self) -> SyntaxToken {
        support::required_token(&self.syntax, 3usize)
    }
    pub fn name(&self) -> HookName {
        support::required_node(&self.syntax, 4usize)
    }
}
impl std::fmt::Debug for Hook {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Hook")
            .field("[0] _token", &self._token())
            .field("[1] l_square_token", &self.l_square_token())
            .field("[2] content", &self.content())
            .field("[3] r_square_token", &self.r_square_token())
            .field("[4] name", &self.name())
            .finish()
    }
}
#[derive(Clone, Eq, PartialEq)]
#[repr(transparent)]
pub struct Strikethrough {
    syntax: SyntaxNode,
}
impl Syntax for Strikethrough {
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl FromSyntax for Strikethrough {
    fn from_syntax(syntax: SyntaxNode) -> Self {
        Self { syntax }
    }
}
impl Strikethrough {
    pub fn opening_run_token(&self) -> SyntaxToken {
        support::required_token(&self.syntax, 0usize)
    }
    pub fn content(&self) -> InlineContent {
        support::required_node(&self.syntax, 1usize)
    }
    pub fn closing_run_token(&self) -> SyntaxToken {
        support::required_token(&self.syntax, 2usize)
    }
}
impl std::fmt::Debug for Strikethrough {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Strikethrough")
            .field("[0] opening_run_token", &self.opening_run_token())
            .field("[1] content", &self.content())
            .field("[2] closing_run_token", &self.closing_run_token())
            .finish()
    }
}
#[derive(Clone, Eq, PartialEq)]
#[repr(transparent)]
pub struct Icu {
    syntax: SyntaxNode,
}
impl Syntax for Icu {
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl FromSyntax for Icu {
    fn from_syntax(syntax: SyntaxNode) -> Self {
        Self { syntax }
    }
}
impl Icu {
    pub fn l_curly_token(&self) -> SyntaxToken {
        support::required_token(&self.syntax, 0usize)
    }
    pub fn value(&self) -> AnyIcuPlaceholder {
        support::required_node(&self.syntax, 1usize)
    }
    pub fn r_curly_token(&self) -> SyntaxToken {
        support::required_token(&self.syntax, 2usize)
    }
}
impl std::fmt::Debug for Icu {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Icu")
            .field("[0] l_curly_token", &self.l_curly_token())
            .field("[1] value", &self.value())
            .field("[2] r_curly_token", &self.r_curly_token())
            .finish()
    }
}
#[derive(Clone, Eq, PartialEq)]
#[repr(transparent)]
pub struct LinkResource {
    syntax: SyntaxNode,
}
impl Syntax for LinkResource {
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl FromSyntax for LinkResource {
    fn from_syntax(syntax: SyntaxNode) -> Self {
        Self { syntax }
    }
}
impl LinkResource {
    pub fn l_paren_token(&self) -> SyntaxToken {
        support::required_token(&self.syntax, 0usize)
    }
    pub fn destination(&self) -> Option<AnyLinkDestination> {
        support::optional_node(&self.syntax, 1usize)
    }
    pub fn title(&self) -> Option<LinkTitle> {
        support::optional_node(&self.syntax, 2usize)
    }
    pub fn r_paren_token(&self) -> SyntaxToken {
        support::required_token(&self.syntax, 3usize)
    }
}
impl std::fmt::Debug for LinkResource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LinkResource")
            .field("[0] l_paren_token", &self.l_paren_token())
            .field("[1] destination", &self.destination())
            .field("[2] title", &self.title())
            .field("[3] r_paren_token", &self.r_paren_token())
            .finish()
    }
}
#[derive(Clone, Eq, PartialEq)]
pub enum AnyLinkDestination {
    StaticLinkDestination(StaticLinkDestination),
    DynamicLinkDestination(DynamicLinkDestination),
    ClickHandlerLinkDestination(ClickHandlerLinkDestination),
}
impl Syntax for AnyLinkDestination {
    fn syntax(&self) -> &SyntaxNode {
        match self {
            Self::StaticLinkDestination(node) => node.syntax(),
            Self::DynamicLinkDestination(node) => node.syntax(),
            Self::ClickHandlerLinkDestination(node) => node.syntax(),
        }
    }
}
impl FromSyntax for AnyLinkDestination {
    fn from_syntax(syntax: SyntaxNode) -> Self {
        match syntax.kind() {
            SyntaxKind::STATIC_LINK_DESTINATION => {
                Self::StaticLinkDestination(StaticLinkDestination::from_syntax(syntax))
            }
            SyntaxKind::DYNAMIC_LINK_DESTINATION => {
                Self::DynamicLinkDestination(DynamicLinkDestination::from_syntax(syntax))
            }
            SyntaxKind::CLICK_HANDLER_LINK_DESTINATION => {
                Self::ClickHandlerLinkDestination(ClickHandlerLinkDestination::from_syntax(syntax))
            }
            kind => unreachable!(
                "Invalid syntax kind {:?} encountered when constructing enum node {}",
                kind, "AnyLinkDestination"
            ),
        }
    }
}
impl From<StaticLinkDestination> for AnyLinkDestination {
    fn from(value: StaticLinkDestination) -> Self {
        Self::StaticLinkDestination(value)
    }
}
impl From<DynamicLinkDestination> for AnyLinkDestination {
    fn from(value: DynamicLinkDestination) -> Self {
        Self::DynamicLinkDestination(value)
    }
}
impl From<ClickHandlerLinkDestination> for AnyLinkDestination {
    fn from(value: ClickHandlerLinkDestination) -> Self {
        Self::ClickHandlerLinkDestination(value)
    }
}
impl std::fmt::Debug for AnyLinkDestination {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut tuple = f.debug_tuple("AnyLinkDestination");
        match self {
            Self::StaticLinkDestination(node) => tuple.field(node),
            Self::DynamicLinkDestination(node) => tuple.field(node),
            Self::ClickHandlerLinkDestination(node) => tuple.field(node),
        };
        tuple.finish()
    }
}
#[derive(Clone, Eq, PartialEq)]
#[repr(transparent)]
pub struct LinkTitle {
    syntax: SyntaxNode,
}
impl Syntax for LinkTitle {
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl FromSyntax for LinkTitle {
    fn from_syntax(syntax: SyntaxNode) -> Self {
        Self { syntax }
    }
}
impl LinkTitle {
    pub fn open_token(&self) -> SyntaxToken {
        support::required_token(&self.syntax, 0usize)
    }
    pub fn content(&self) -> LinkTitleContent {
        support::required_node(&self.syntax, 1usize)
    }
    pub fn close_token(&self) -> SyntaxToken {
        support::required_token(&self.syntax, 2usize)
    }
}
impl std::fmt::Debug for LinkTitle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LinkTitle")
            .field("[0] open_token", &self.open_token())
            .field("[1] content", &self.content())
            .field("[2] close_token", &self.close_token())
            .finish()
    }
}
#[derive(Clone, Eq, PartialEq)]
#[repr(transparent)]
pub struct StaticLinkDestination {
    syntax: SyntaxNode,
}
impl Syntax for StaticLinkDestination {
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl FromSyntax for StaticLinkDestination {
    fn from_syntax(syntax: SyntaxNode) -> Self {
        Self { syntax }
    }
}
impl StaticLinkDestination {
    pub fn len(&self) -> usize {
        self.syntax.len()
    }
    pub fn children(&self) -> SyntaxTokenChildren {
        SyntaxTokenChildren::new(self.syntax.children())
    }
    pub fn get(&self, index: usize) -> Option<&SyntaxToken> {
        self.syntax.get(index).map(|element| element.token())
    }
}
impl std::ops::Index<usize> for StaticLinkDestination {
    type Output = SyntaxToken;
    fn index(&self, index: usize) -> &Self::Output {
        &self.syntax[index].token()
    }
}
impl std::fmt::Debug for StaticLinkDestination {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("StaticLinkDestination")?;
        f.debug_list().entries(self.children()).finish()
    }
}
#[derive(Clone, Eq, PartialEq)]
#[repr(transparent)]
pub struct DynamicLinkDestination {
    syntax: SyntaxNode,
}
impl Syntax for DynamicLinkDestination {
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl FromSyntax for DynamicLinkDestination {
    fn from_syntax(syntax: SyntaxNode) -> Self {
        Self { syntax }
    }
}
impl DynamicLinkDestination {
    pub fn url(&self) -> Icu {
        support::required_node(&self.syntax, 0usize)
    }
}
impl std::fmt::Debug for DynamicLinkDestination {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DynamicLinkDestination")
            .field("[0] url", &self.url())
            .finish()
    }
}
#[derive(Clone, Eq, PartialEq)]
#[repr(transparent)]
pub struct ClickHandlerLinkDestination {
    syntax: SyntaxNode,
}
impl Syntax for ClickHandlerLinkDestination {
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl FromSyntax for ClickHandlerLinkDestination {
    fn from_syntax(syntax: SyntaxNode) -> Self {
        Self { syntax }
    }
}
impl ClickHandlerLinkDestination {
    pub fn name_token(&self) -> SyntaxToken {
        support::required_token(&self.syntax, 0usize)
    }
}
impl std::fmt::Debug for ClickHandlerLinkDestination {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ClickHandlerLinkDestination")
            .field("[0] name_token", &self.name_token())
            .finish()
    }
}
#[derive(Clone, Eq, PartialEq)]
#[repr(transparent)]
pub struct LinkTitleContent {
    syntax: SyntaxNode,
}
impl Syntax for LinkTitleContent {
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl FromSyntax for LinkTitleContent {
    fn from_syntax(syntax: SyntaxNode) -> Self {
        Self { syntax }
    }
}
impl LinkTitleContent {
    pub fn len(&self) -> usize {
        self.syntax.len()
    }
    pub fn children(&self) -> SyntaxTokenChildren {
        SyntaxTokenChildren::new(self.syntax.children())
    }
    pub fn get(&self, index: usize) -> Option<&SyntaxToken> {
        self.syntax.get(index).map(|element| element.token())
    }
}
impl std::ops::Index<usize> for LinkTitleContent {
    type Output = SyntaxToken;
    fn index(&self, index: usize) -> &Self::Output {
        &self.syntax[index].token()
    }
}
impl std::fmt::Debug for LinkTitleContent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("LinkTitleContent")?;
        f.debug_list().entries(self.children()).finish()
    }
}
#[derive(Clone, Eq, PartialEq)]
#[repr(transparent)]
pub struct CodeSpanContent {
    syntax: SyntaxNode,
}
impl Syntax for CodeSpanContent {
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl FromSyntax for CodeSpanContent {
    fn from_syntax(syntax: SyntaxNode) -> Self {
        Self { syntax }
    }
}
impl CodeSpanContent {
    pub fn len(&self) -> usize {
        self.syntax.len()
    }
    pub fn children(&self) -> SyntaxTokenChildren {
        SyntaxTokenChildren::new(self.syntax.children())
    }
    pub fn get(&self, index: usize) -> Option<&SyntaxToken> {
        self.syntax.get(index).map(|element| element.token())
    }
}
impl std::ops::Index<usize> for CodeSpanContent {
    type Output = SyntaxToken;
    fn index(&self, index: usize) -> &Self::Output {
        &self.syntax[index].token()
    }
}
impl std::fmt::Debug for CodeSpanContent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("CodeSpanContent")?;
        f.debug_list().entries(self.children()).finish()
    }
}
#[derive(Clone, Eq, PartialEq)]
#[repr(transparent)]
pub struct HookName {
    syntax: SyntaxNode,
}
impl Syntax for HookName {
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl FromSyntax for HookName {
    fn from_syntax(syntax: SyntaxNode) -> Self {
        Self { syntax }
    }
}
impl HookName {
    pub fn l_paren_token(&self) -> SyntaxToken {
        support::required_token(&self.syntax, 0usize)
    }
    pub fn name_token(&self) -> SyntaxToken {
        support::required_token(&self.syntax, 1usize)
    }
    pub fn r_paren_token(&self) -> SyntaxToken {
        support::required_token(&self.syntax, 2usize)
    }
}
impl std::fmt::Debug for HookName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HookName")
            .field("[0] l_paren_token", &self.l_paren_token())
            .field("[1] name_token", &self.name_token())
            .field("[2] r_paren_token", &self.r_paren_token())
            .finish()
    }
}
#[derive(Clone, Eq, PartialEq)]
pub enum AnyIcuPlaceholder {
    IcuVariable(IcuVariable),
    IcuPlural(IcuPlural),
    IcuSelectOrdinal(IcuSelectOrdinal),
    IcuSelect(IcuSelect),
    IcuDate(IcuDate),
    IcuTime(IcuTime),
    IcuNumber(IcuNumber),
}
impl Syntax for AnyIcuPlaceholder {
    fn syntax(&self) -> &SyntaxNode {
        match self {
            Self::IcuVariable(node) => node.syntax(),
            Self::IcuPlural(node) => node.syntax(),
            Self::IcuSelectOrdinal(node) => node.syntax(),
            Self::IcuSelect(node) => node.syntax(),
            Self::IcuDate(node) => node.syntax(),
            Self::IcuTime(node) => node.syntax(),
            Self::IcuNumber(node) => node.syntax(),
        }
    }
}
impl FromSyntax for AnyIcuPlaceholder {
    fn from_syntax(syntax: SyntaxNode) -> Self {
        match syntax.kind() {
            SyntaxKind::ICU_VARIABLE => Self::IcuVariable(IcuVariable::from_syntax(syntax)),
            SyntaxKind::ICU_PLURAL => Self::IcuPlural(IcuPlural::from_syntax(syntax)),
            SyntaxKind::ICU_SELECT_ORDINAL => {
                Self::IcuSelectOrdinal(IcuSelectOrdinal::from_syntax(syntax))
            }
            SyntaxKind::ICU_SELECT => Self::IcuSelect(IcuSelect::from_syntax(syntax)),
            SyntaxKind::ICU_DATE => Self::IcuDate(IcuDate::from_syntax(syntax)),
            SyntaxKind::ICU_TIME => Self::IcuTime(IcuTime::from_syntax(syntax)),
            SyntaxKind::ICU_NUMBER => Self::IcuNumber(IcuNumber::from_syntax(syntax)),
            kind => unreachable!(
                "Invalid syntax kind {:?} encountered when constructing enum node {}",
                kind, "AnyIcuPlaceholder"
            ),
        }
    }
}
impl From<IcuVariable> for AnyIcuPlaceholder {
    fn from(value: IcuVariable) -> Self {
        Self::IcuVariable(value)
    }
}
impl From<IcuPlural> for AnyIcuPlaceholder {
    fn from(value: IcuPlural) -> Self {
        Self::IcuPlural(value)
    }
}
impl From<IcuSelectOrdinal> for AnyIcuPlaceholder {
    fn from(value: IcuSelectOrdinal) -> Self {
        Self::IcuSelectOrdinal(value)
    }
}
impl From<IcuSelect> for AnyIcuPlaceholder {
    fn from(value: IcuSelect) -> Self {
        Self::IcuSelect(value)
    }
}
impl From<IcuDate> for AnyIcuPlaceholder {
    fn from(value: IcuDate) -> Self {
        Self::IcuDate(value)
    }
}
impl From<IcuTime> for AnyIcuPlaceholder {
    fn from(value: IcuTime) -> Self {
        Self::IcuTime(value)
    }
}
impl From<IcuNumber> for AnyIcuPlaceholder {
    fn from(value: IcuNumber) -> Self {
        Self::IcuNumber(value)
    }
}
impl std::fmt::Debug for AnyIcuPlaceholder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut tuple = f.debug_tuple("AnyIcuPlaceholder");
        match self {
            Self::IcuVariable(node) => tuple.field(node),
            Self::IcuPlural(node) => tuple.field(node),
            Self::IcuSelectOrdinal(node) => tuple.field(node),
            Self::IcuSelect(node) => tuple.field(node),
            Self::IcuDate(node) => tuple.field(node),
            Self::IcuTime(node) => tuple.field(node),
            Self::IcuNumber(node) => tuple.field(node),
        };
        tuple.finish()
    }
}
#[derive(Clone, Eq, PartialEq)]
#[repr(transparent)]
pub struct IcuVariable {
    syntax: SyntaxNode,
}
impl Syntax for IcuVariable {
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl FromSyntax for IcuVariable {
    fn from_syntax(syntax: SyntaxNode) -> Self {
        Self { syntax }
    }
}
impl IcuVariable {
    pub fn ident_token(&self) -> SyntaxToken {
        support::required_token(&self.syntax, 0usize)
    }
}
impl std::fmt::Debug for IcuVariable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IcuVariable")
            .field("[0] ident_token", &self.ident_token())
            .finish()
    }
}
#[derive(Clone, Eq, PartialEq)]
#[repr(transparent)]
pub struct IcuPlural {
    syntax: SyntaxNode,
}
impl Syntax for IcuPlural {
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl FromSyntax for IcuPlural {
    fn from_syntax(syntax: SyntaxNode) -> Self {
        Self { syntax }
    }
}
impl IcuPlural {
    pub fn variable(&self) -> IcuVariable {
        support::required_node(&self.syntax, 0usize)
    }
    pub fn variable_comma_token(&self) -> SyntaxToken {
        support::required_token(&self.syntax, 1usize)
    }
    pub fn format_token(&self) -> SyntaxToken {
        support::required_token(&self.syntax, 2usize)
    }
    pub fn format_comma_token(&self) -> SyntaxToken {
        support::required_token(&self.syntax, 3usize)
    }
    pub fn arms(&self) -> IcuPluralArms {
        support::required_node(&self.syntax, 4usize)
    }
}
impl std::fmt::Debug for IcuPlural {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IcuPlural")
            .field("[0] variable", &self.variable())
            .field("[1] variable_comma_token", &self.variable_comma_token())
            .field("[2] format_token", &self.format_token())
            .field("[3] format_comma_token", &self.format_comma_token())
            .field("[4] arms", &self.arms())
            .finish()
    }
}
#[derive(Clone, Eq, PartialEq)]
#[repr(transparent)]
pub struct IcuSelectOrdinal {
    syntax: SyntaxNode,
}
impl Syntax for IcuSelectOrdinal {
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl FromSyntax for IcuSelectOrdinal {
    fn from_syntax(syntax: SyntaxNode) -> Self {
        Self { syntax }
    }
}
impl IcuSelectOrdinal {
    pub fn variable(&self) -> IcuVariable {
        support::required_node(&self.syntax, 0usize)
    }
    pub fn variable_comma_token(&self) -> SyntaxToken {
        support::required_token(&self.syntax, 1usize)
    }
    pub fn format_token(&self) -> SyntaxToken {
        support::required_token(&self.syntax, 2usize)
    }
    pub fn format_comma_token(&self) -> SyntaxToken {
        support::required_token(&self.syntax, 3usize)
    }
    pub fn arms(&self) -> IcuPluralArms {
        support::required_node(&self.syntax, 4usize)
    }
}
impl std::fmt::Debug for IcuSelectOrdinal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IcuSelectOrdinal")
            .field("[0] variable", &self.variable())
            .field("[1] variable_comma_token", &self.variable_comma_token())
            .field("[2] format_token", &self.format_token())
            .field("[3] format_comma_token", &self.format_comma_token())
            .field("[4] arms", &self.arms())
            .finish()
    }
}
#[derive(Clone, Eq, PartialEq)]
#[repr(transparent)]
pub struct IcuSelect {
    syntax: SyntaxNode,
}
impl Syntax for IcuSelect {
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl FromSyntax for IcuSelect {
    fn from_syntax(syntax: SyntaxNode) -> Self {
        Self { syntax }
    }
}
impl IcuSelect {
    pub fn variable(&self) -> IcuVariable {
        support::required_node(&self.syntax, 0usize)
    }
    pub fn variable_comma_token(&self) -> SyntaxToken {
        support::required_token(&self.syntax, 1usize)
    }
    pub fn format_token(&self) -> SyntaxToken {
        support::required_token(&self.syntax, 2usize)
    }
    pub fn format_comma_token(&self) -> SyntaxToken {
        support::required_token(&self.syntax, 3usize)
    }
    pub fn arms(&self) -> IcuPluralArms {
        support::required_node(&self.syntax, 4usize)
    }
}
impl std::fmt::Debug for IcuSelect {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IcuSelect")
            .field("[0] variable", &self.variable())
            .field("[1] variable_comma_token", &self.variable_comma_token())
            .field("[2] format_token", &self.format_token())
            .field("[3] format_comma_token", &self.format_comma_token())
            .field("[4] arms", &self.arms())
            .finish()
    }
}
#[derive(Clone, Eq, PartialEq)]
#[repr(transparent)]
pub struct IcuDate {
    syntax: SyntaxNode,
}
impl Syntax for IcuDate {
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl FromSyntax for IcuDate {
    fn from_syntax(syntax: SyntaxNode) -> Self {
        Self { syntax }
    }
}
impl IcuDate {
    pub fn variable(&self) -> IcuVariable {
        support::required_node(&self.syntax, 0usize)
    }
    pub fn variable_comma_token(&self) -> SyntaxToken {
        support::required_token(&self.syntax, 1usize)
    }
    pub fn format_token(&self) -> SyntaxToken {
        support::required_token(&self.syntax, 2usize)
    }
    pub fn style(&self) -> Option<IcuDateTimeStyle> {
        support::optional_node(&self.syntax, 3usize)
    }
}
impl std::fmt::Debug for IcuDate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IcuDate")
            .field("[0] variable", &self.variable())
            .field("[1] variable_comma_token", &self.variable_comma_token())
            .field("[2] format_token", &self.format_token())
            .field("[3] style", &self.style())
            .finish()
    }
}
#[derive(Clone, Eq, PartialEq)]
#[repr(transparent)]
pub struct IcuTime {
    syntax: SyntaxNode,
}
impl Syntax for IcuTime {
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl FromSyntax for IcuTime {
    fn from_syntax(syntax: SyntaxNode) -> Self {
        Self { syntax }
    }
}
impl IcuTime {
    pub fn variable(&self) -> IcuVariable {
        support::required_node(&self.syntax, 0usize)
    }
    pub fn variable_comma_token(&self) -> SyntaxToken {
        support::required_token(&self.syntax, 1usize)
    }
    pub fn format_token(&self) -> SyntaxToken {
        support::required_token(&self.syntax, 2usize)
    }
    pub fn style(&self) -> Option<IcuDateTimeStyle> {
        support::optional_node(&self.syntax, 3usize)
    }
}
impl std::fmt::Debug for IcuTime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IcuTime")
            .field("[0] variable", &self.variable())
            .field("[1] variable_comma_token", &self.variable_comma_token())
            .field("[2] format_token", &self.format_token())
            .field("[3] style", &self.style())
            .finish()
    }
}
#[derive(Clone, Eq, PartialEq)]
#[repr(transparent)]
pub struct IcuNumber {
    syntax: SyntaxNode,
}
impl Syntax for IcuNumber {
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl FromSyntax for IcuNumber {
    fn from_syntax(syntax: SyntaxNode) -> Self {
        Self { syntax }
    }
}
impl IcuNumber {
    pub fn variable(&self) -> IcuVariable {
        support::required_node(&self.syntax, 0usize)
    }
    pub fn variable_comma_token(&self) -> SyntaxToken {
        support::required_token(&self.syntax, 1usize)
    }
    pub fn format_token(&self) -> SyntaxToken {
        support::required_token(&self.syntax, 2usize)
    }
    pub fn style(&self) -> Option<IcuNumberStyle> {
        support::optional_node(&self.syntax, 3usize)
    }
}
impl std::fmt::Debug for IcuNumber {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IcuNumber")
            .field("[0] variable", &self.variable())
            .field("[1] variable_comma_token", &self.variable_comma_token())
            .field("[2] format_token", &self.format_token())
            .field("[3] style", &self.style())
            .finish()
    }
}
#[derive(Clone, Eq, PartialEq)]
#[repr(transparent)]
pub struct IcuPluralArms {
    syntax: SyntaxNode,
}
impl Syntax for IcuPluralArms {
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl FromSyntax for IcuPluralArms {
    fn from_syntax(syntax: SyntaxNode) -> Self {
        Self { syntax }
    }
}
impl IcuPluralArms {
    pub fn len(&self) -> usize {
        self.syntax.len()
    }
    pub fn children(&self) -> TypedNodeChildren<IcuPluralArm> {
        TypedNodeChildren::new(SyntaxNodeChildren::new(self.syntax.children()))
    }
    pub fn get(&self, index: usize) -> Option<IcuPluralArm> {
        self.syntax
            .get(index)
            .map(|node| IcuPluralArm::from_syntax_element(node.clone()))
    }
}
impl std::ops::Index<usize> for IcuPluralArms {
    type Output = SyntaxToken;
    fn index(&self, index: usize) -> &Self::Output {
        &self.syntax[index].token()
    }
}
impl std::fmt::Debug for IcuPluralArms {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("IcuPluralArms")?;
        f.debug_list().entries(self.children()).finish()
    }
}
#[derive(Clone, Eq, PartialEq)]
#[repr(transparent)]
pub struct IcuPluralArm {
    syntax: SyntaxNode,
}
impl Syntax for IcuPluralArm {
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl FromSyntax for IcuPluralArm {
    fn from_syntax(syntax: SyntaxNode) -> Self {
        Self { syntax }
    }
}
impl IcuPluralArm {
    pub fn selector_token(&self) -> SyntaxToken {
        support::required_token(&self.syntax, 0usize)
    }
    pub fn l_curly_token(&self) -> SyntaxToken {
        support::required_token(&self.syntax, 1usize)
    }
    pub fn value(&self) -> IcuPluralValue {
        support::required_node(&self.syntax, 2usize)
    }
    pub fn r_curly_token(&self) -> SyntaxToken {
        support::required_token(&self.syntax, 3usize)
    }
}
impl std::fmt::Debug for IcuPluralArm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IcuPluralArm")
            .field("[0] selector_token", &self.selector_token())
            .field("[1] l_curly_token", &self.l_curly_token())
            .field("[2] value", &self.value())
            .field("[3] r_curly_token", &self.r_curly_token())
            .finish()
    }
}
#[derive(Clone, Eq, PartialEq)]
#[repr(transparent)]
pub struct IcuPluralValue {
    syntax: SyntaxNode,
}
impl Syntax for IcuPluralValue {
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl FromSyntax for IcuPluralValue {
    fn from_syntax(syntax: SyntaxNode) -> Self {
        Self { syntax }
    }
}
impl IcuPluralValue {
    pub fn content(&self) -> InlineContent {
        support::required_node(&self.syntax, 0usize)
    }
}
impl std::fmt::Debug for IcuPluralValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IcuPluralValue")
            .field("[0] content", &self.content())
            .finish()
    }
}
#[derive(Clone, Eq, PartialEq)]
#[repr(transparent)]
pub struct IcuDateTimeStyle {
    syntax: SyntaxNode,
}
impl Syntax for IcuDateTimeStyle {
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl FromSyntax for IcuDateTimeStyle {
    fn from_syntax(syntax: SyntaxNode) -> Self {
        Self { syntax }
    }
}
impl IcuDateTimeStyle {
    pub fn style_comma_token(&self) -> SyntaxToken {
        support::required_token(&self.syntax, 0usize)
    }
    pub fn style_text_token(&self) -> SyntaxToken {
        support::required_token(&self.syntax, 1usize)
    }
}
impl std::fmt::Debug for IcuDateTimeStyle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IcuDateTimeStyle")
            .field("[0] style_comma_token", &self.style_comma_token())
            .field("[1] style_text_token", &self.style_text_token())
            .finish()
    }
}
#[derive(Clone, Eq, PartialEq)]
#[repr(transparent)]
pub struct IcuNumberStyle {
    syntax: SyntaxNode,
}
impl Syntax for IcuNumberStyle {
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl FromSyntax for IcuNumberStyle {
    fn from_syntax(syntax: SyntaxNode) -> Self {
        Self { syntax }
    }
}
impl IcuNumberStyle {
    pub fn style_comma_token(&self) -> SyntaxToken {
        support::required_token(&self.syntax, 0usize)
    }
    pub fn style_text_token(&self) -> SyntaxToken {
        support::required_token(&self.syntax, 1usize)
    }
}
impl std::fmt::Debug for IcuNumberStyle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IcuNumberStyle")
            .field("[0] style_comma_token", &self.style_comma_token())
            .field("[1] style_text_token", &self.style_text_token())
            .finish()
    }
}
