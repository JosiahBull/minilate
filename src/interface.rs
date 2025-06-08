use std::{borrow::Cow, collections::BTreeMap};

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum VariableTy {
    String,
    Boolean,
    Iterable,
}

impl VariableTy {
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
pub struct Context<'a> {
    data: BTreeMap<String, Variable<'a>>,
}

impl Context<'_> {
    pub fn new() -> Self {
        Self::default()
    }
}

impl<'a> Context<'a> {
    pub fn insert<T: AsRef<str>>(&mut self, name: T, variable: Variable<'a>) -> &mut Self {
        self.data.insert(name.as_ref().to_string(), variable);
        self
    }

    pub fn get<T: AsRef<str>>(&self, name: T) -> Option<&Variable<'a>> {
        self.data.get(name.as_ref())
    }

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
