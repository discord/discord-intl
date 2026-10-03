use std::fmt::Formatter;

use serde::{Serialize, Serializer};

#[derive(Debug, Clone, Copy)]
pub enum DiagnosticCategory {
    /// Diagnostics that indicate source content that will not function as intended in all cases,
    /// such as behavior across different language plural rules, or duplicated syntax that is
    /// allowed but overrides other parts of content unintentionally.
    Correctness,
    /// Diagnostics that indicate ambiguity or situations that are likely unintentional and should
    /// be reviewed for correctness.
    Suspicious,
    /// Diagnostics that refer to best practices or readability of the source text, generally
    /// without any effect on the rendered content.
    Style,
}

impl DiagnosticCategory {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Correctness => "correctness",
            Self::Suspicious => "suspicious",
            Self::Style => "style",
        }
    }
}

impl Serialize for DiagnosticCategory {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.as_str())
    }
}

impl std::fmt::Display for DiagnosticCategory {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}
