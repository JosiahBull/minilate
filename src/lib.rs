mod ast;
mod engine;
mod error;
mod interface;
mod parser;
mod template;

// Crate-level imports to make convienent imports for the rest of the library.
// Public exports.
pub use engine::MinilateEngine;
pub use error::MinilateError;
pub(crate) use error::MinilateResult;
pub use interface::{Context, MinilateInterface, VariableTy};
pub use template::Template;
