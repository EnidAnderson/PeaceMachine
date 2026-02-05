use std::fmt;

/// Stable, tagged errors: `pm/<tag>: <details>`
#[derive(Debug, Clone)]
pub struct PmError {
    pub tag: &'static str,
    pub details: String,
}

impl PmError {
    pub fn new(tag: &'static str, details: impl Into<String>) -> Self {
        Self { tag, details: details.into() }
    }

    pub fn render(&self) -> String {
        format!("{}: {}", self.tag, self.details)
    }
}

impl fmt::Display for PmError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.render())
    }
}

impl std::error::Error for PmError {}
