pub type MinilateResult<T> = std::result::Result<T, MinilateError>;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash, thiserror::Error)]
pub enum ParseErrorKind {
    #[error("Expected {expected}, found {found}")]
    UnexpectedToken { expected: String, found: String },
    #[error("Unexpected EOF {expected_what}")]
    UnexpectedEOF {
        /// Describes what was expected, e.g., "(expected '}}')"
        expected_what: String,
    },
    #[error("Invalid identifier starting with '{at_char}'")]
    InvalidIdentifier { at_char: String },
    #[error("Unknown keyword '{keyword}'")]
    UnknownKeyword { keyword: String },
    #[error("Expected {description}")]
    Expected { description: String },
    #[error("Parser error: {0}")]
    Message(String),
}

impl ParseErrorKind {
    pub fn unexpected_eof(expected: Option<String>) -> Self {
        ParseErrorKind::UnexpectedEOF {
            expected_what: expected.map_or_else(String::new, |e| format!(" (expected '{}')", e)),
        }
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash, thiserror::Error)]
pub struct ParseError {
    pub line: usize,
    pub column: usize,
    pub kind: ParseErrorKind,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Parse error at line {}, column {}: {}",
            self.line, self.column, self.kind
        )
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash, thiserror::Error)]
pub enum MinilateError {
    #[error("Template already exists: {template_name}")]
    TemplateExists { template_name: String },
    #[error("Template not found: {template_name}")]
    MissingTemplate { template_name: String },
    #[error("Variable not found: {variable_name}")]
    MissingVariable { variable_name: String },
    #[error("Variable data missing: {variable_name}")]
    MissingVariableData { variable_name: String },
    #[error("Type mismatch for variable {variable_name}: expected {expected:?}, found {found:?}")]
    TypeMismatch {
        variable_name: String,
        expected: crate::interface::VariableTy,
        found: crate::interface::VariableTy,
    },
    #[error("Rendering error: {message}")]
    RenderError { message: String },
    #[error(transparent)]
    Parse(#[from] ParseError),
}
