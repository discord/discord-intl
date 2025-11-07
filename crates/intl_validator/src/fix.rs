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

    pub fn with_suggestion(mut self, message: &str) -> Self {
        self.message = Some(message.into());
        self
    }
}

pub fn apply_fixes(message: &str, fixes: &[DiagnosticFix]) -> String {
    let mut sorted_fixes = fixes.iter().collect::<Vec<_>>();
    let mut total_fix_length = 0;
    sorted_fixes.sort_by_key(|fix| {
        total_fix_length += fix.source_span.1 - fix.source_span.0;
        fix.source_span.1
    });

    let mut result = String::with_capacity(message.len() + total_fix_length);
    let mut current_offset = 0;
    for fix in sorted_fixes {
        result.push_str(&message[current_offset..fix.source_span.0]);
        if fix.replacement.len() > 0 {
            result.push_str(&fix.replacement);
        }
        current_offset = fix.source_span.1;
    }
    result.push_str(&message[current_offset..]);
    result
}
