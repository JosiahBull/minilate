use std::borrow::Cow;

pub(crate) enum AstNode<'a> {
    /// A constant block of text from the template, with all escapes processed.
    ///
    /// If there were no escapes in the given text this will be Borrowed -
    /// otherwise we are forced to allocate.
    Constant { data: Cow<'a, str> },
    /// A variable.
    Variable { name: &'a str },
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
    },
    /// A Else statement.
    Else { body: Vec<AstNode<'a>> },
    /// Conditional NOT
    Not { condition: Box<AstNode<'a>> },
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
}
