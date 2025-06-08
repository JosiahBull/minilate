//! Defines the Abstract Syntax Tree (AST) structures used by Minilate.
//!
//! The AST is the internal representation of a parsed template, capturing its
//! structure and components like constant text, variables, control flow (if/for),
//! and template inclusions.
//!
//! The [`crate::parser`] module is responsible for generating these AST nodes
//! from a template string. The [`crate::engine::MinilateEngine`] then uses this
//! AST to render the template with a given [`crate::interface::Context`].
//!
//! # Key Structures
//!
//! - [`AstNode`]: The core enum representing different types of nodes in the template.
//!   - `AstNode::Root`: The top-level node of a parsed template.
//!   - `AstNode::Constant`: Represents a block of static text.
//!   - `AstNode::Variable`: Represents a `{{ variable }}` substitution.
//!   - `AstNode::For`: Represents a `{{% for item in items %}}` loop.
//!   - `AstNode::If`: Represents an `{{% if condition %}}` block, potentially with `else` or `else if` branches.
//!   - `AstNode::Not`, `AstNode::And`, `AstNode::Or`: Represent logical operations within conditions.
//!   - `AstNode::TemplateInclude`: Represents a `{{<< sub_template.tmpl }}` inclusion.
//!
//! The structure of the AST allows for efficient traversal during rendering and
//! context analysis (e.g., determining required variables).

use std::borrow::Cow;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, PartialEq, Clone)] // Added PartialEq and Clone for easier testing
pub enum AstNode<'a> {
    Root(Vec<AstNode<'a>>),
    /// A constant block of text from the template, with all escapes processed.
    ///
    /// If there were no escapes in the given text this will be Borrowed -
    /// otherwise we are forced to allocate.
    Constant {
        data: Cow<'a, str>,
    },
    /// A variable.
    Variable {
        name: &'a str,
    },
    /// A For loop.
    For {
        iterable: &'a str,
        variable: &'a str,
        body: Vec<AstNode<'a>>,
    },
    /// A If statement.
    If {
        condition: Box<AstNode<'a>>,
        body: Vec<AstNode<'a>>,
        else_branch: Option<Box<AstNode<'a>>>, // This will typically be an AstNode::Root for else branches
    },
    /// Conditional NOT
    Not {
        condition: Box<AstNode<'a>>,
    },
    /// Conditional AND
    And {
        left: Box<AstNode<'a>>,
        right: Box<AstNode<'a>>,
    },
    /// Conditional OR
    Or {
        left: Box<AstNode<'a>>,
        right: Box<AstNode<'a>>,
    },
    /// Template inclusion
    TemplateInclude {
        template_name: &'a str,
    },
}
