mod ast;
mod error;
mod interface;

// Crate-level imports to make convienent imports for the rest of the library.
pub(crate) use error::MinilateResult;

// Public exports.
pub use error::MinilateError;
pub use interface::{Context, MinilateEngine, Variable, VariableTy};

// Temporary export.
pub use interface::ExampleEngine;
