use std::borrow::Cow;
use std::collections::HashMap;

use crate::ast::AstNode;
use crate::error::{MinilateError, MinilateResult};
use crate::interface::{Context, MinilateInterface};
use crate::template::Template;

/// `MinilateEngine` is the primary implementation of the `MinilateInterface` trait,
/// providing a complete templating engine for the Minilate system
///
/// This engine manages a collection of named templates that can be added, rendered,
/// and analyzed for required context variables.
///
/// # Examples
///
/// ```
/// use minilate::{MinilateEngine, MinilateInterface, Context, VariableTy};
///
/// // Create a new engine
/// let mut engine = MinilateEngine::new();
///
/// // Add a template
/// engine.add_template("greeting", "Hello, {{ name }}!").unwrap();
///
/// // Setup context
/// let mut context = Context::new();
/// context.insert("name", VariableTy::String.with_data("World"));
///
/// // Render template
/// let output = engine.render("greeting", Some(&context)).unwrap();
/// assert_eq!(output, "Hello, World!");
/// ```
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MinilateEngine<'a> {
    templates: HashMap<String, Template<'a>>,
}

impl<'a> MinilateEngine<'a> {
    // Helper method to traverse the AST and collect variables from template inclusions
    fn collect_inclusion_variables<'b>(
        &'b self,
        node: &'b AstNode<'a>,
        variables: &mut Vec<(&'b str, crate::interface::VariableTy)>,
        context: &Context<'_>,
        visited: &mut Vec<&'b str>,
    ) {
        match node {
            AstNode::Root(children) => {
                // Process all children nodes
                for child in children {
                    self.collect_inclusion_variables(child, variables, context, visited);
                }
            }
            AstNode::If {
                condition,
                body,
                else_branch,
            } => {
                // Evaluate the condition with the current context
                let condition_result =
                    crate::template::evaluate_condition(condition, context).unwrap_or(false);

                if condition_result {
                    // Process the if body only if condition is true
                    for child in body {
                        self.collect_inclusion_variables(child, variables, context, visited);
                    }
                } else if let Some(else_node) = else_branch {
                    // Process the else branch only if condition is false
                    self.collect_inclusion_variables(else_node, variables, context, visited);
                }
            }
            AstNode::For {
                variable,
                iterable,
                body,
            } => {
                // Check if the iterable exists and has data
                let has_items = context
                    .get(iterable)
                    .and_then(|v| v.data())
                    .is_some_and(|d| !d.is_empty());

                // When the iterable has items in the context, we should process the body
                // and no longer need to collect the iterable variable (since it's available)
                if has_items {
                    // When the loop has items, we create a simulated context
                    // This creates a simulated context that assumes the loop variable exists
                    let mut loop_context = context.clone();
                    loop_context.insert(
                        variable,
                        crate::interface::VariableTy::String.with_data("dummy"),
                    );

                    // Process the body with the augmented context
                    for child in body {
                        self.collect_inclusion_variables(child, variables, &loop_context, visited);
                    }
                }
            }
            AstNode::TemplateInclude { template_name } => {
                // Skip if already visited to prevent infinite recursion
                // XXX: What about if the context changes between two different includes (e.g. one in a loop)?
                if visited.contains(template_name) {
                    return;
                }

                // Mark as visited
                visited.push(template_name);

                // If template exists, collect variables from it recursively
                if let Some(included_template) = self.templates.get(*template_name) {
                    // First collect variables from this template
                    included_template.collect_variables(variables, context);

                    // Collect template inclusions through a proper AST traversal
                    self.collect_inclusion_variables(
                        &included_template.ast,
                        variables,
                        context,
                        visited,
                    );
                }
            }
            // Other node types don't contribute to template inclusion
            AstNode::Constant { .. }
            | AstNode::Variable { .. }
            | AstNode::Not { .. }
            | AstNode::And { .. }
            | AstNode::Or { .. } => {}
        }
    }
}

impl MinilateEngine<'_> {
    /// Creates a new empty `MinilateEngine` instance.
    ///
    /// # Returns
    ///
    /// A new engine with no templates.
    pub fn new() -> Self {
        Self {
            templates: HashMap::new(),
        }
    }
}

impl Default for MinilateEngine<'_> {
    /// Creates a default `MinilateEngine` instance by calling `new()`.
    fn default() -> Self {
        Self::new()
    }
}

impl MinilateInterface for MinilateEngine<'_> {
    /// Adds a new template to the engine with the given name and content.
    ///
    /// # Arguments
    ///
    /// * `name` - The name to identify this template by
    /// * `content` - The template content string
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the template was successfully added
    /// * `Err(MinilateError::TemplateExists)` if a template with the given name already exists
    /// * `Err(MinilateError::Parse)` if the template content contains syntax errors
    ///
    /// # Examples
    ///
    /// ```
    /// use minilate::{MinilateEngine, MinilateInterface};
    ///
    /// let mut engine = MinilateEngine::new();
    /// engine.add_template("greeting", "Hello, {{ name }}!").unwrap();
    /// ```
    fn add_template<'a, N: AsRef<str>, C: Into<Cow<'a, str>>>(
        &mut self,
        name: N,
        content: C,
    ) -> MinilateResult<()> {
        let name = name.as_ref();

        if self.templates.contains_key(name) {
            return Err(MinilateError::TemplateExists {
                template_name: name.to_string(),
            });
        }

        let content_str: String = content.into().to_string();

        // Parse the template content into an AST using the Template implementation
        let mut template = Template::new(content_str)?;
        template.name = Some(name.to_string());

        self.templates.insert(name.to_string(), template);

        Ok(())
    }

    /// Renders a template with the given name using the provided context.
    ///
    /// # Arguments
    ///
    /// * `template_name` - The name of the template to render
    /// * `context` - Optional context with variables for template rendering
    ///
    /// # Returns
    ///
    /// * `Ok(String)` containing the rendered template content
    /// * `Err(MinilateError::MissingTemplate)` if no template with the given name exists
    /// * Other errors may be returned from the rendering process (missing variables, type mismatches, etc.)
    ///
    /// # Examples
    ///
    /// ```
    /// use minilate::{MinilateEngine, MinilateInterface, Context, VariableTy};
    ///
    /// let mut engine = MinilateEngine::new();
    /// engine.add_template("greeting", "Hello, {{ name }}!").unwrap();
    ///
    /// let mut context = Context::new();
    /// context.insert("name", VariableTy::String.with_data("World"));
    ///
    /// let output = engine.render("greeting", Some(&context)).unwrap();
    /// assert_eq!(output, "Hello, World!");
    /// ```
    fn render<'a, N: AsRef<str>>(
        &self,
        template_name: N,
        context: Option<&'a Context<'a>>,
    ) -> MinilateResult<String> {
        let name = template_name.as_ref();
        let template = self
            .templates
            .get(name)
            .ok_or_else(|| MinilateError::MissingTemplate {
                template_name: name.to_string(),
            })?;

        let default_context = Context::default();
        let context = context.unwrap_or(&default_context);

        template.render(context, Some(self))
    }

    /// Analyzes a template and returns a list of required variables that aren't already in the context.
    ///
    /// This method identifies all variables used in the template and their expected types,
    /// excluding any that are already present in the provided context.
    ///
    /// # Arguments
    ///
    /// * `template_name` - The name of the template to analyze
    /// * `context` - The current context to check against
    ///
    /// # Returns
    ///
    /// A vector of tuples containing variable name and expected type for each variable
    /// required by the template that's not already in the context.
    ///
    /// Returns an empty vector if the template doesn't exist.
    ///
    /// # Examples
    ///
    /// ```
    /// use minilate::{MinilateEngine, MinilateInterface, Context};
    ///
    /// let mut engine = MinilateEngine::new();
    /// engine.add_template("greeting", "Hello, {{ name }}!").unwrap();
    ///
    /// let empty_context = Context::new();
    /// let variables = engine.context("greeting", &empty_context);
    ///
    /// assert_eq!(variables.len(), 1);
    /// assert_eq!(variables[0].0, "name");
    /// ```
    fn context<'a, 'b, T: AsRef<str>>(
        &'b self,
        template_name: T,
        context: &'a Context<'a>,
    ) -> Vec<(&'b str, crate::interface::VariableTy)> {
        let name = template_name.as_ref();

        // If template doesn't exist, return empty vec
        let template = match self.templates.get(name) {
            Some(t) => t,
            None => return vec![],
        };

        // Collect variables
        let mut variables = Vec::new();
        let mut visited = Vec::new();

        // First collect variables from this template
        template.collect_variables(&mut variables, context);

        // Collect template inclusions through a proper AST traversal
        self.collect_inclusion_variables(&template.ast, &mut variables, context, &mut visited);

        // Remove duplicates from the variables list
        variables.sort_by(|(a, _), (b, _)| a.cmp(b));
        variables.dedup_by(|(a, _), (b, _)| a == b);

        variables
    }
}
