//! Defines the public interface for interacting with the Minilate templating system.
//!
//! This module provides the core traits and data structures that users of the
//! Minilate library will primarily work with. It establishes a contract for how
//! template engines should behave and how data (context) is supplied for rendering.
//!
//! # Key Components:
//!
//! - [`MinilateInterface`]: A trait that defines the essential operations for a
//!   templating engine, such as adding templates, rendering them, and querying
//!   for required context variables. The main implementation is [`crate::engine::MinilateEngine`].
//! - [`Context`]: A struct used to hold the data (variables) that will be available
//!   during template rendering. It's essentially a map of variable names to their
//!   values and types.
//! - [`Variable<'a>`]: Represents a single variable within a [`Context`], holding its
//!   type ([`VariableTy`]) and an optional string representation of its data.
//! - [`VariableTy`]: An enum specifying the type of a variable (e.g., `String`,
//!   `Boolean`, `Iterable`). This helps the engine understand how to use the
//!   variable in different template constructs (e.g., in conditions or loops).
//!
//! # Example: Defining Context and using the Interface
//!
//! ```rust
//! use minilate::{Context, Variable, VariableTy, MinilateInterface, MinilateEngine, MinilateResult};
//!
//! # fn main() -> MinilateResult<()> {
//! // 1. Create a context
//! let mut ctx = Context::new();
//! ctx.insert("username", VariableTy::String.with_data("Alice"));
//! ctx.insert("is_active", VariableTy::Boolean.with_data("true"));
//!
//! // 2. Use an engine implementing MinilateInterface
//! let mut engine = MinilateEngine::new(); // Example engine
//! engine.add_template("greeting", "Hello, {{ username }}! {{% if is_active %}}You are active.{{% endif %}}")?;
//!
//! // 3. Render
//! let output = engine.render("greeting", Some(&ctx))?;
//! assert_eq!(output, "Hello, Alice! You are active.");
//!
//! // 4. Check required context (for a different template)
//! engine.add_template("profile", "User: {{ user.name }}, Age: {{ user.age }}")?;
//! let empty_ctx = Context::new();
//! let required_vars = engine.context("profile", &empty_ctx);
//! // required_vars would likely contain ("user.name", VariableTy::String)
//! // and ("user.age", VariableTy::String).
//!
//! // Example showing how VariableTy helps:
//! engine.add_template("loop_example", "{{% for item in item_list %}}* {{ item }}{{% endfor %}}")?;
//! let required_for_loop = engine.context("loop_example", &empty_ctx);
//! assert!(required_for_loop.iter().any(|(name, ty)| *name == "item_list" && *ty == VariableTy::Iterable));
//! # Ok(())
//! # }
//! ```
//!
//! This interface is designed to be simple yet flexible enough to support the
//! core features of Minilate.

use std::{borrow::Cow, collections::BTreeMap};

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
/// Specifies the type of a variable within a [`Context`].
///
/// `VariableTy` is used by the Minilate engine to determine how a variable
/// should be treated during template rendering, especially in conditional
/// statements (`{{% if %}}`) and loops (`{{% for %}}`).
///
/// For example, a variable used in a `for` loop is expected to be `Iterable`.
/// A variable used in an `if` condition might be evaluated based on its
/// boolean interpretation or string emptiness.
pub enum VariableTy {
    /// Represents a simple string value.
    /// In conditions, an empty string is typically falsy, and a non-empty string is truthy.
    String,
    /// Represents a boolean value.
    /// The string data associated with this type (e.g., "true", "false") is parsed
    /// by the rendering logic.
    Boolean,
    /// Represents a collection of items that can be iterated over in a `{{% for %}}` loop.
    /// The string data for an iterable is typically a comma-separated list of values.
    Iterable,
}

impl VariableTy {
    /// Associates this `VariableTy` with string data, creating a [`Variable`].
    ///
    /// This is a convenience method to quickly construct a `Variable` instance.
    /// The provided `data` is converted into a `Cow<'a, str>`, allowing for both
    /// borrowed and owned string data.
    ///
    /// # Arguments
    ///
    /// * `data`: The string data for the variable. This can be any type that
    ///   implements `Into<Cow<'a, str>>`, such as `String` or `&'a str`.
    ///
    /// # Returns
    ///
    /// A new [`Variable<'a>`] instance with the current type and the provided data.
    ///
    /// # Example
    ///
    /// ```
    /// use minilate::{VariableTy, Variable};
    /// use std::borrow::Cow;
    ///
    /// let name_var = VariableTy::String.with_data("Alice");
    /// assert_eq!(name_var.ty(), VariableTy::String);
    /// assert_eq!(name_var.data(), Some("Alice"));
    ///
    /// let active_var = VariableTy::Boolean.with_data("true".to_string());
    /// assert_eq!(active_var.ty(), VariableTy::Boolean);
    /// assert_eq!(active_var.data(), Some("true"));
    /// ```
    pub fn with_data<'a, T: Into<Cow<'a, str>>>(self, data: T) -> Variable<'a> {
        Variable {
            ty: self,
            data: Some(data.into()),
        }
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Variable<'a> {
    ty: VariableTy,
    data: Option<Cow<'a, str>>,
}

impl Variable<'_> {
    pub const fn ty(&self) -> VariableTy {
        self.ty
    }

    pub fn data(&self) -> Option<&str> {
        self.data.as_ref().map(|s| s.as_ref())
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
/// Holds the data (variables) available during template rendering.
///
/// A `Context` is essentially a map where keys are variable names (strings)
/// and values are [`Variable<'a>`] instances, which include both the variable's
/// data (as a string) and its [`VariableTy`].
///
/// The Minilate engine uses the `Context` to look up values for `{{ variable }}`
/// substitutions, to evaluate conditions in `{{% if %}}` blocks, and to iterate
/// over collections in `{{% for %}}` loops.
///
/// `Context` uses a `BTreeMap` internally to store variables, ensuring that
/// variable iteration (if ever needed directly) is ordered by name.
///
/// # Lifetimes
///
/// The lifetime `'a` is associated with the string data within the [`Variable<'a>`]
/// instances. This allows the `Context` to hold variables that borrow string slices.
///
/// # Examples
///
/// ```
/// use minilate::{Context, VariableTy};
///
/// let mut ctx = Context::new();
///
/// // Insert some variables
/// ctx.insert("username", VariableTy::String.with_data("Alice"));
/// ctx.insert("is_active", VariableTy::Boolean.with_data("true"));
/// ctx.insert("items", VariableTy::Iterable.with_data("apple,banana,cherry"));
///
/// // Check if a variable exists
/// assert!(ctx.contains("username"));
///
/// // Get a variable
/// if let Some(user_var) = ctx.get("username") {
///     assert_eq!(user_var.ty(), VariableTy::String);
///     assert_eq!(user_var.data(), Some("Alice"));
/// }
/// ```
pub struct Context<'a> {
    data: BTreeMap<String, Variable<'a>>,
}

impl Context<'_> {
    /// Creates a new, empty `Context`.
    ///
    /// This is equivalent to `Context::default()`.
    ///
    /// # Returns
    ///
    /// A new `Context` instance with no variables.
    pub fn new() -> Self {
        Self::default()
    }
}

impl<'a> Context<'a> {
    /// Inserts a variable into the context.
    ///
    /// If the context did not have this key present, `None` is returned.
    /// If the context did have this key present, the value is updated, and the old
    /// value is returned. The key is not updated, though; this matters for
    /// types that can be `==` without being identical.
    ///
    /// The `name` can be any type that implements `AsRef<str>`, allowing for
    /// flexibility (e.g., `String` or `&str`).
    ///
    /// # Arguments
    ///
    /// * `name`: The name of the variable (e.g., `"username"`).
    /// * `variable`: The [`Variable<'a>`] to insert.
    ///
    /// # Returns
    ///
    /// A mutable reference to `self` for method chaining.
    ///
    /// # Example
    ///
    /// ```
    /// use minilate::{Context, VariableTy};
    ///
    /// let mut ctx = Context::new();
    /// ctx.insert("name", VariableTy::String.with_data("Bob"))
    ///    .insert("age", VariableTy::String.with_data("30"));
    ///
    /// assert!(ctx.contains("name"));
    /// assert!(ctx.contains("age"));
    /// ```
    pub fn insert<T: AsRef<str>>(&mut self, name: T, variable: Variable<'a>) -> &mut Self {
        self.data.insert(name.as_ref().to_string(), variable);
        self
    }

    /// Retrieves a reference to a variable from the context.
    ///
    /// # Arguments
    ///
    /// * `name`: The name of the variable to retrieve.
    ///
    /// # Returns
    ///
    /// An `Option<&Variable<'a>>`. Returns `Some(&Variable)` if the variable
    /// exists, and `None` otherwise.
    ///
    /// # Example
    ///
    /// ```
    /// use minilate::{Context, VariableTy};
    ///
    /// let mut ctx = Context::new();
    /// ctx.insert("city", VariableTy::String.with_data("Paris"));
    ///
    /// let city_var = ctx.get("city").unwrap();
    /// assert_eq!(city_var.data(), Some("Paris"));
    ///
    /// assert!(ctx.get("country").is_none());
    /// ```
    pub fn get<T: AsRef<str>>(&self, name: T) -> Option<&Variable<'a>> {
        self.data.get(name.as_ref())
    }

    /// Checks if the context contains a variable with the given name.
    ///
    /// # Arguments
    ///
    /// * `name`: The name of the variable to check for.
    ///
    /// # Returns
    ///
    /// `true` if a variable with the specified name exists in the context,
    /// `false` otherwise.
    ///
    /// # Example
    ///
    /// ```
    /// use minilate::{Context, VariableTy};
    ///
    /// let mut ctx = Context::new();
    /// ctx.insert("is_admin", VariableTy::Boolean.with_data("false"));
    ///
    /// assert!(ctx.contains("is_admin"));
    /// assert!(!ctx.contains("is_moderator"));
    /// ```
    pub fn contains<T: AsRef<str>>(&self, name: T) -> bool {
        self.data.contains_key(name.as_ref())
    }
}

/// `MinilateEngine` is a trait for the Minilate templating engine, an
/// opinionated and minamalistic templating engine designed for use in offline,
/// static, single-threaded environments.
pub trait MinilateInterface {
    /// `add_template` tries to make a new template available in the engine.
    ///
    /// # Errors
    /// - If the template name is a duplicate.
    fn add_template<'a, N: AsRef<str>, C: Into<Cow<'a, str>>>(
        &'a mut self,
        name: N,
        content: C,
    ) -> crate::MinilateResult<()>;

    /// `render` tries to render a template with the given context.
    ///
    /// # Errors
    /// - If the template name is not found.
    /// - If a required context variable is missing.
    fn render<'a, N: AsRef<str>>(
        &self,
        template_name: N,
        context: Option<&'a Context<'a>>,
    ) -> crate::MinilateResult<String>;

    /// `context` will return a Vec<()> of all missing context objects required
    /// to succesffully render the selected template.
    ///
    /// This will intelligently skip inaccessible variables, so an optional
    /// context may be provided to show current user selections.
    fn context<'a, 'b, T: AsRef<str>>(
        &'b self,
        template_name: T,
        context: &'a Context<'a>,
    ) -> Vec<(&'b str, VariableTy)>;
}

// ExampleEngine is moved to engine.rs and replaced with MinilateEngine
