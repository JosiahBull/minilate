use std::{
    borrow::Cow,
    collections::{BTreeMap, HashMap},
};

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

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Variable<'a> {
    ty: VariableTy,
    data: Option<Cow<'a, str>>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
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
        if let Some(data) = self.data.insert(name.as_ref().to_string(), variable) {
            // TODO (@JO)
            panic!("duplicate context: {:?}", data);
        }

        self
    }
}

impl Default for Context<'_> {
    fn default() -> Self {
        Self {
            data: BTreeMap::new(),
        }
    }
}

/// `MinilateEngine` is a trait for the Minilate templating engine, an
/// opinionated and minamalistic templating engine designed for use in offline,
/// static, single-threaded environments.
pub trait MinilateEngine {
    /// `add_template` tries to make a new template available in the engine.
    ///
    /// # Errors
    /// - If the template name is a duplicate.
    fn add_template<'a, N: AsRef<str>, C: Into<Cow<'a, str>>>(
        &mut self,
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

pub struct ExampleEngine {
    templates: std::sync::Mutex<HashMap<String, String>>,
}

impl ExampleEngine {
    pub fn new() -> Self {
        Self {
            templates: std::sync::Mutex::new(HashMap::new()),
        }
    }
}

impl MinilateEngine for ExampleEngine {
    fn add_template<'a, N: AsRef<str>, C: Into<Cow<'a, str>>>(
        &mut self,
        name: N,
        content: C,
    ) -> crate::MinilateResult<()> {
        let name = name.as_ref();

        if self.templates.lock().unwrap().contains_key(name) {
            return Err(crate::MinilateError::TemplateExists {
                template_name: name.to_string(),
            });
        }

        let data: String = content.into().to_string();
        let mut lock = self.templates.lock().unwrap();
        lock.insert(name.to_string(), data);

        Ok(())
    }

    fn render<'a, N: AsRef<str>>(
        &self,
        template_name: N,
        context: Option<&'a Context<'a>>,
    ) -> crate::MinilateResult<String> {
        let name = template_name.as_ref();
        let mut lock = self.templates.lock().unwrap();
        let data = lock
            .get(name)
            .ok_or(crate::MinilateError::MissingTemplate {
                template_name: name.to_string(),
            })?;

        Ok(data.clone())
    }

    fn context<'a, 'b, T: AsRef<str>>(
        &'b self,
        template_name: T,
        context: &'a Context<'a>,
    ) -> Vec<(&'b str, VariableTy)> {
        vec![]
    }
}
