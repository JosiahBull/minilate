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
            ParseErrorKind::UnexpectedToken { expected, found } => {
                write!(f, "Expected {}, found {}", expected, found)
            }
            ParseErrorKind::UnexpectedEOF { expected_what } => {
                write!(f, "Unexpected EOF{}", expected_what)
            }
            ParseErrorKind::InvalidIdentifier { at_char } => {
                write!(f, "Invalid identifier starting with '{}'", at_char)
            }
            ParseErrorKind::UnknownKeyword { keyword } => {
                write!(f, "Unknown keyword '{}'", keyword)
            }
            ParseErrorKind::Expected { description } => {
                write!(f, "Expected {}", description)
            }
            ParseErrorKind::Message(msg) => {
                write!(f, "Parser error: {}", msg)
            }
        }
    }
}

impl std::error::Error for ParseErrorKind {}

impl ParseErrorKind {
    pub fn unexpected_eof(expected: Option<String>) -> Self {
        ParseErrorKind::UnexpectedEOF {
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
            MinilateError::TemplateExists { template_name } => {
                write!(f, "Template already exists: {}", template_name)
            }
            MinilateError::MissingTemplate { template_name } => {
                write!(f, "Template not found: {}", template_name)
            }
            MinilateError::MissingVariable { variable_name } => {
                write!(f, "Variable not found: {}", variable_name)
            }
            MinilateError::MissingVariableData { variable_name } => {
                write!(f, "Variable data missing: {}", variable_name)
            }
            MinilateError::TypeMismatch {
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
            MinilateError::RenderError { message } => {
                write!(f, "Rendering error: {}", message)
            }
            MinilateError::Parse(parse_error) => {
                write!(f, "{}", parse_error)
            }
        }
    }
}

impl std::error::Error for MinilateError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            MinilateError::Parse(parse_error) => Some(parse_error),
            _ => None,
        }
    }
}

impl From<ParseError> for MinilateError {
    fn from(error: ParseError) -> Self {
        MinilateError::Parse(error)
    }
}
