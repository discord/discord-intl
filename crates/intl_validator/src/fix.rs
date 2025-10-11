use crate::TextRange;
use intl_markdown_syntax::SyntaxToken;

#[derive(Debug, Clone)]
pub struct DiagnosticFix {
    pub message: Option<String>,
    pub source_span: TextRange,
    pub replacement: String,
}

impl DiagnosticFix {
    pub fn remove_text(source_span: TextRange) -> Self {
        DiagnosticFix {
            message: None,
            source_span,
            replacement: "".into(),
        }
    }

    pub fn replace_text(source_span: TextRange, replacement: &str) -> Self {
        DiagnosticFix {
            message: None,
            source_span,
            replacement: replacement.into(),
        }
    }

    pub fn insert_text(start: usize, new_text: &str) -> Self {
        DiagnosticFix {
            message: None,
            source_span: (start, start),
            replacement: new_text.into(),
        }
    }

    /// Replaces the given `token`'s text with the `replacement`, preserving all leading and
    /// trailing trivia on the token.
    pub fn replace_token(token: &SyntaxToken, replacement: &str) -> Self {
        let token_text_start = (token.text_offset() + token.text_start()) as usize;
        let token_text_end = token_text_start + (token.text_len() as usize);

        DiagnosticFix {
            message: None,
            source_span: (token_text_start, token_text_end),
            replacement: replacement.into(),
        }
    }

    pub fn with_message(mut self, message: &str) -> Self {
        self.message = Some(message.into());
        self
    }
}
