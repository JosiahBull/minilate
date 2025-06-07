pub type MinilateResult<T> = std::result::Result<T, MinilateError>;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ParseErrorKind {
    UnexpectedToken {
        expected: String,
        found: String,
    },
    UnexpectedEOF {
        /// Describes what was expected, e.g., "(expected '}}')"
        expected_what: String,
    },
    InvalidIdentifier {
        at_char: String,
    },
    UnknownKeyword {
        keyword: String,
    },
    Expected {
        description: String,
    },
    Message(String),
}

impl std::fmt::Display for ParseErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnexpectedToken { expected, found } => {
                write!(f, "Expected {}, found {}", expected, found)
            }
            Self::UnexpectedEOF { expected_what } => {
                write!(f, "Unexpected EOF{}", expected_what)
            }
            Self::InvalidIdentifier { at_char } => {
                write!(f, "Invalid identifier starting with '{}'", at_char)
            }
            Self::UnknownKeyword { keyword } => {
                write!(f, "Unknown keyword '{}'", keyword)
            }
            Self::Expected { description } => {
                write!(f, "Expected {}", description)
            }
            Self::Message(msg) => {
                write!(f, "Parser error: {}", msg)
            }
        }
    }
}

impl std::error::Error for ParseErrorKind {}

impl ParseErrorKind {
    pub fn unexpected_eof(expected: Option<String>) -> Self {
        Self::UnexpectedEOF {
            expected_what: expected.map_or_else(String::new, |e| format!(" (expected '{}')", e)),
        }
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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

impl std::error::Error for ParseError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.kind)
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MinilateError {
    TemplateExists {
        template_name: String,
    },
    MissingTemplate {
        template_name: String,
    },
    MissingVariable {
        variable_name: String,
    },
    MissingVariableData {
        variable_name: String,
    },
    TypeMismatch {
        variable_name: String,
        expected: crate::interface::VariableTy,
        found: crate::interface::VariableTy,
    },
    RenderError {
        message: String,
    },
    Parse(ParseError),
}

impl std::fmt::Display for MinilateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TemplateExists { template_name } => {
                write!(f, "Template already exists: {}", template_name)
            }
            Self::MissingTemplate { template_name } => {
                write!(f, "Template not found: {}", template_name)
            }
            Self::MissingVariable { variable_name } => {
                write!(f, "Variable not found: {}", variable_name)
            }
            Self::MissingVariableData { variable_name } => {
                write!(f, "Variable data missing: {}", variable_name)
            }
            Self::TypeMismatch {
                variable_name,
                expected,
                found,
            } => {
                write!(
                    f,
                    "Type mismatch for variable {}: expected {:?}, found {:?}",
                    variable_name, expected, found
                )
            }
            Self::RenderError { message } => {
                write!(f, "Rendering error: {}", message)
            }
            Self::Parse(parse_error) => {
                write!(f, "{}", parse_error)
            }
        }
    }
}

impl std::error::Error for MinilateError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Parse(parse_error) => Some(parse_error),
            Self::TemplateExists { .. }
            | Self::MissingTemplate { .. }
            | Self::MissingVariable { .. }
            | Self::MissingVariableData { .. }
            | Self::TypeMismatch { .. }
            | Self::RenderError { .. } => None,
        }
    }
}

impl From<ParseError> for MinilateError {
    fn from(error: ParseError) -> Self {
        Self::Parse(error)
    }
}
