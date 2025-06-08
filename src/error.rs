//! Defines the error types used throughout the Minilate library.
//!
//! This module provides a comprehensive set of errors that can occur during
//! template parsing, rendering, or engine operations. The primary error type
//! is [`MinilateError`], which encompasses various specific error kinds.
//!
//! For parsing-specific issues, [`ParseError`] and its associated [`ParseErrorKind`]
//! provide detailed information about syntax errors, including line and column numbers.
//!
//! # Key Error Types:
//!
//! - [`MinilateResult<T>`]: A type alias for `Result<T, MinilateError>`, used for functions
//!   that can return Minilate-specific errors.
//! - [`MinilateError`]: The main enum representing all possible errors from the Minilate engine.
//!   This includes issues like missing templates, missing variables, type mismatches,
//!   rendering problems, and parsing errors.
//! - [`ParseError`]: A struct detailing errors specifically from the template parsing process
//!   (see [`crate::parser`]). It includes:
//!     - `line`: The line number where the error occurred.
//!     - `column`: The column number where the error occurred.
//!     - `kind`: A [`ParseErrorKind`] enum specifying the nature of the parsing error.
//! - [`ParseErrorKind`]: An enum that categorizes different parsing failures, such as
//!   unexpected tokens, premature end-of-file, invalid identifiers, or unknown keywords.
//!
//! # Example: Handling a MinilateError
//!
//! ```rust
//! use minilate::{MinilateEngine, MinilateInterface, Context, MinilateError};
//!
//! let mut engine = MinilateEngine::new();
//! // Attempt to render a template that doesn't exist
//! match engine.render("nonexistent_template", None) {
//!     Ok(output) => println!("Rendered: {}", output),
//!     Err(MinilateError::MissingTemplate { template_name }) => {
//!         eprintln!("Error: Template '{}' not found.", template_name);
//!     }
//!     Err(e) => {
//!         eprintln!("An unexpected error occurred: {}", e);
//!     }
//! }
//! ```
//!
//! Understanding these error types is crucial for robust error handling when
//! integrating Minilate into an application.

pub type MinilateResult<T> = std::result::Result<T, MinilateError>;

/// Represents errors that can occur during parsing of templates into the internal AST structure.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ParseErrorKind {
    /// Found an unexpected token in the template, provides a hint about what was expected.
    UnexpectedToken {
        expected: String,
        found: String,
    },
    /// The parser reached the end of the input unexpectedly, possibly missing a closing brace or similar
    UnexpectedEOF {
        /// Describes what was expected, e.g., "(expected '}}')"
        expected_what: String,
    },
    /// An identifier was expected, but we weren't able to parse it for some reason.
    InvalidIdentifier {
        at_char: String,
    },
    /// An unknown keyword was encountered in the template.
    UnknownKeyword {
        keyword: String,
    },
    /// A generic expected error, used for cases where the parser expects something specific.
    Expected {
        description: String,
    },
    /// A generic message for parser errors that don't fit into the other categories.
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

/// A parsing error containing the line and column where the error occurred, along with the [`ParseErrorKind`].
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

/// Represents errors that can occur during the operation of the Minilate template engine.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MinilateError {
    /// Adding this template would overwrite an existing one.
    TemplateExists {
        template_name: String,
    },
    /// The requested template was not found.
    MissingTemplate {
        template_name: String,
    },
    /// A variable was referenced but not found in the context.
    MissingVariable {
        variable_name: String,
    },
    /// A variable was referenced but its data was not provided.
    MissingVariableData {
        variable_name: String,
    },
    /// A variable was found, but its type did not match the expected type.
    TypeMismatch {
        variable_name: String,
        expected: crate::interface::VariableTy,
        found: crate::interface::VariableTy,
    },
    /// An error occurred during the rendering process, with a message describing the issue.
    RenderError {
        message: String,
    },
    /// A parsing error occurred, containing the details of the error.
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
