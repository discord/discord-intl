use std::fmt::Formatter;

use serde::{Serialize, Serializer};

#[derive(Debug, Clone, Copy)]
pub enum DiagnosticCategory {
    Info,
    Style,
    Correctness,
    Complexity,
    Suspicious,
}

impl DiagnosticCategory {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Info => "info",
            Self::Style => "style",
            Self::Correctness => "correctness",
            Self::Complexity => "complexity",
            Self::Suspicious => "suspicious",
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
