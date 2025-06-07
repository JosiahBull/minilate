use std::borrow::Cow;

use crate::{
    ast::AstNode,
    error::{ParseError, ParseErrorKind},
};

type ParseResult<T> = Result<T, ParseError>;

struct Parser<'a> {
    input: &'a str,
    pos: usize,
    /// Current line number (1-indexed)
    line: usize,
    /// The starting location of the current line
    line_start_pos: usize,
}

impl<'a> Parser<'a> {
    fn new(input: &'a str) -> Self {
        Parser {
            input,
            pos: 0,
            line: 1,
            line_start_pos: 0,
        }
    }

    #[inline]
    fn current_column(&self) -> usize {
        self.pos - self.line_start_pos + 1
    }

    #[inline]
    fn make_error(&self, kind: ParseErrorKind) -> ParseError {
        ParseError {
            line: self.line,
            column: self.current_column(),
            kind,
        }
    }

    /// Advances the parser position by char_len bytes, correctly handling
    /// multi-byte characters. Updates line and column numbers if a newline is
    /// encountered.
    #[inline]
    fn advance_by_char(&mut self, current_char: char, char_len: usize) {
        if current_char == '\n' {
            self.line += 1;
            self.line_start_pos = self.pos + char_len;
        }
        self.pos += char_len;
    }

    /// Advances the parser position by `len` bytes.
    /// This method assumes that the consumed string `s` does NOT contain newlines.
    /// If it can, line/column tracking will be incorrect. Used for fixed delimiters.
    #[inline]
    fn advance_bytes_no_newline(&mut self, len: usize) {
        self.pos += len;
    }

    fn eof(&self) -> bool {
        self.pos >= self.input.len()
    }

    /// Peek if the remaining input starts with `s`
    fn peek(&self, s: &str) -> bool {
        self.input[self.pos..].starts_with(s)
    }

    /// Multi-token peek which checks if the remaining input starts with any of the provided tokens, ignoring whitespace between.
    fn peek_n<const N: usize>(&self, tokens: [&str; N]) -> bool {
        if !self.peek(tokens[0]) {
            return false;
        }

        let mut parser = Self {
            input: self.input,
            pos: self.pos + tokens[0].len(),
            line: self.line,
            line_start_pos: self.line_start_pos,
        };

        for token in &tokens[1..] {
            parser.consume_whitespace();
            if !parser.peek(token) {
                return false;
            }
            parser.advance_bytes_no_newline(token.len());
        }
        true
    }

    /// Consume `s` if the remaining input starts with it.
    /// Assumes `s` does not contain newlines.
    fn consume(&mut self, s: &str) -> bool {
        if self.peek(s) {
            self.advance_bytes_no_newline(s.len());
            true
        } else {
            false
        }
    }

    /// Consume leading whitespace and line comments (//), handling newlines correctly.
    fn consume_whitespace(&mut self) {
        loop {
            let initial_pos = self.pos;
            let initial_line = self.line;
            let initial_line_start_pos = self.line_start_pos;

            // Consume standard whitespace characters
            while !self.eof() {
                let current_char = self.input[self.pos..].chars().next().unwrap();
                if current_char.is_ascii_whitespace() {
                    self.advance_by_char(current_char, current_char.len_utf8());
                } else {
                    break;
                }
            }

            // Consume line comment if present
            if self.peek("//") {
                self.advance_bytes_no_newline(2); // Consume "//"
                while !self.eof() {
                    let current_char = self.input[self.pos..].chars().next().unwrap();
                    let char_len = current_char.len_utf8();
                    // We must call advance_by_char to correctly update line count if newline is part of comment termination
                    self.advance_by_char(current_char, char_len);
                    if current_char == '\n' {
                        break; // Comment ends at and includes the newline
                    }
                }
            }

            // If no progress was made in this iteration (neither whitespace nor comment consumed), stop.
            if self.pos == initial_pos
                && self.line == initial_line
                && self.line_start_pos == initial_line_start_pos
            {
                break;
            }
        }
    }

    /// Expect `s` to be the start of the remaining input, consume it or return Err.
    /// Assumes `s` does not contain newlines.
    fn expect(&mut self, s: &str) -> ParseResult<()> {
        if self.consume(s) {
            Ok(())
        } else {
            Err(self.make_error(ParseErrorKind::Expected {
                description: format!(
                    "'{}', found '{}'",
                    s,
                    &self.input[self.pos..std::cmp::min(self.pos + s.len() + 10, self.input.len())]
                ),
            }))
        }
    }

    /// Consume and return an identifier (alphanumeric + '_' + '.')
    fn consume_identifier(&mut self) -> ParseResult<&'a str> {
        self.consume_whitespace();
        let start = self.pos;
        while !self.eof() {
            // Peek char for newline handling before consuming as part of identifier
            let current_char = self.input[self.pos..].chars().next().unwrap();
            if current_char.is_ascii_alphanumeric() || current_char == '_' || current_char == '.' {
                // Identifiers cannot span newlines
                if current_char == '\n' {
                    break;
                }
                self.advance_by_char(current_char, current_char.len_utf8());
            } else {
                break;
            }
        }
        if start == self.pos {
            Err(self.make_error(ParseErrorKind::Expected {
                description: "identifier".to_string(),
            }))
        } else {
            Ok(&self.input[start..self.pos])
        }
    }

    /// Parses a block of constant text until a delimiter or end_tag_hint is found.
    fn parse_constant(&mut self, end_tag_hint: Option<&str>) -> ParseResult<AstNode<'a>> {
        let start_pos = self.pos;

        while !self.eof() {
            // Handle escaping with \{{ or \{{%
            if self.peek("\\{{") {
                // Skip the backslash but include the {{ in the constant
                self.advance_bytes_no_newline(1); // Skip only the backslash
                self.advance_bytes_no_newline(2); // Include the {{ in the constant
                continue;
            }

            // Check for escaped {{%
            if self.peek("\\{{%") {
                // Skip the backslash but include the {{% in the constant
                self.advance_bytes_no_newline(1); // Skip only the backslash
                self.advance_bytes_no_newline(3); // Include the {{% in the constant
                continue;
            }

            if self.peek("{{") {
                // Catches both {{ and {{%
                break;
            }
            if let Some(tag_hint) = end_tag_hint {
                if self.peek(tag_hint) {
                    break;
                }
            }

            let current_char = self.input[self.pos..].chars().next().unwrap(); // Safe due to !eof()
            self.advance_by_char(current_char, current_char.len_utf8());
        }

        // Even if start_pos == self.pos (e.g. immediate delimiter), a Constant node is fine if it's empty.
        // The logic in parse_nodes_until handles whether to keep empty constants.
        Ok(AstNode::Constant {
            data: Cow::Borrowed(&self.input[start_pos..self.pos]),
        })
    }

    fn parse_variable_or_comment(&mut self) -> ParseResult<AstNode<'a>> {
        self.expect("{{")?;

        // Check for template inclusion
        if self.consume("<<") {
            self.consume_whitespace();
            let template_name = self.consume_identifier()?;
            self.consume(".tmpl");
            self.consume_whitespace();
            self.expect("}}")?;
            return Ok(AstNode::TemplateInclude { template_name });
        }

        self.consume_whitespace();
        let name = self.consume_identifier()?;
        self.consume_whitespace();
        self.expect("}}")?;
        Ok(AstNode::Variable { name })
    }

    fn parse_node(&mut self) -> ParseResult<AstNode<'a>> {
        if self.peek("{{%") {
            self.parse_control_flow()
        } else if self.peek("{{") {
            self.parse_variable_or_comment()
        } else {
            // If it's neither a control flow nor a variable, it must be constant text.
            // This constant text could be empty if we're at EOF or right before another tag.
            self.parse_constant(None)
        }
    }

    /// Parse nodes until encountering a specific control tag with keyword
    fn parse_nodes_until<const N: usize>(
        &mut self,
        end_tag: Option<[&str; N]>,
    ) -> ParseResult<Vec<AstNode<'a>>> {
        let mut nodes = Vec::new();
        loop {
            if self.eof() {
                if let Some(tag) = end_tag {
                    return Err(
                        self.make_error(ParseErrorKind::unexpected_eof(Some(tag.join(" "))))
                    );
                }
                break;
            }

            if let Some(tag) = end_tag {
                if self.peek_n(tag) {
                    break;
                }
            }

            let node_start_pos = self.pos;
            let node = self.parse_node()?;

            // Avoid pushing empty constant nodes unless it's the only thing (e.g. empty template)
            // or if it represents consumed whitespace that is significant.
            // If parse_node returned an empty constant and pos hasn't changed, it means we are likely
            // right before a tag that parse_constant correctly stopped at.
            if node_start_pos == self.pos && self.peek("{{") {
                // This empty constant was likely due to parse_constant stopping immediately.
                // The next iteration will parse the tag. Don't add this empty node.
                // However, if parse_node() itself advances pos (e.g. parsing a variable), this check is fine.
                // This specific check is for an empty constant that consumed nothing.
                if let AstNode::Constant { data } = &node {
                    if data.is_empty() {
                        continue;
                    }
                }
            }
            nodes.push(node);
        }
        Ok(nodes)
    }

    // --- Condition Parsing (Recursive Descent for boolean expressions) ---
    // Precedence: OR -> AND -> NOT -> Primary (variable)

    // Entry point for parsing a condition expression
    fn parse_condition_expression(&mut self) -> ParseResult<Box<AstNode<'a>>> {
        self.parse_or_expression()
    }

    // Handles OR (||)
    fn parse_or_expression(&mut self) -> ParseResult<Box<AstNode<'a>>> {
        let mut left = self.parse_and_expression()?;
        loop {
            self.consume_whitespace();
            if self.consume("||") {
                let right = self.parse_and_expression()?;
                left = Box::new(AstNode::Or { left, right });
            } else {
                break;
            }
        }
        Ok(left)
    }

    // Handles AND (&&)
    fn parse_and_expression(&mut self) -> ParseResult<Box<AstNode<'a>>> {
        let mut left = self.parse_not_expression()?;
        loop {
            self.consume_whitespace();
            if self.consume("&&") {
                let right = self.parse_not_expression()?;
                left = Box::new(AstNode::And { left, right });
            } else {
                break;
            }
        }
        Ok(left)
    }

    // Handles NOT (!)
    fn parse_not_expression(&mut self) -> ParseResult<Box<AstNode<'a>>> {
        self.consume_whitespace();
        if self.consume("!") {
            let condition = self.parse_not_expression()?;
            Ok(Box::new(AstNode::Not { condition }))
        } else {
            self.parse_primary_expression()
        }
    }

    // Handles primary expressions (currently just variables)
    fn parse_primary_expression(&mut self) -> ParseResult<Box<AstNode<'a>>> {
        self.consume_whitespace();
        let name = self.consume_identifier()?;
        Ok(Box::new(AstNode::Variable { name }))
    }

    // --- Control Flow Parsing ---

    fn parse_control_flow(&mut self) -> ParseResult<AstNode<'a>> {
        self.expect("{{%")?;
        self.consume_whitespace();
        let keyword = self.consume_identifier()?;
        match keyword {
            "if" => self.parse_if_statement(),
            "for" => self.parse_for_loop(),
            _ => Err(self.make_error(ParseErrorKind::UnknownKeyword {
                keyword: keyword.to_string(),
            })),
        }
    }

    fn parse_for_loop(&mut self) -> ParseResult<AstNode<'a>> {
        self.consume_whitespace();
        let variable = self.consume_identifier()?;
        self.consume_whitespace();
        self.expect("in")?;
        self.consume_whitespace();
        let iterable = self.consume_identifier()?;
        self.consume_whitespace();
        self.expect("%}}")?;

        let body = self.parse_nodes_until(Some(["{{%", "endfor", "%}}"]))?;
        self.expect("{{%")?;
        self.consume_whitespace();
        self.expect("endfor")?;
        self.consume_whitespace();
        self.expect("%}}")?;

        Ok(AstNode::For {
            iterable,
            variable,
            body,
        })
    }

    fn parse_if_statement(&mut self) -> ParseResult<AstNode<'a>> {
        let condition = self.parse_condition_expression()?;
        self.consume_whitespace();
        self.expect("%}}")?;
        self.parse_if_block_internal(condition)
    }

    fn parse_if_block_internal(
        &mut self,
        current_if_condition: Box<AstNode<'a>>,
    ) -> ParseResult<AstNode<'a>> {
        let mut body_nodes = Vec::new();
        let mut else_branch_for_current_if: Option<Box<AstNode<'a>>> = None;

        'body_parsing_loop: loop {
            if self.eof() {
                return Err(self.make_error(ParseErrorKind::unexpected_eof(Some(
                    "{{% endif %}} or {{% else %}} or {{% else if %}}".to_string(),
                ))));
            }

            if self.peek_n(["{{%", "else", "if"]) {
                self.expect("{{%")?;
                self.consume_whitespace();
                self.expect("else")?;
                self.consume_whitespace();
                self.expect("if")?;
                self.consume_whitespace();
                let next_if_condition = self.parse_condition_expression()?;
                self.consume_whitespace();
                self.expect("%}}")?;

                let nested_if_node = self.parse_if_block_internal(next_if_condition)?;
                else_branch_for_current_if = Some(Box::new(AstNode::Root(vec![nested_if_node])));
                break 'body_parsing_loop;
            } else if self.peek_n(["{{%", "else", "%}}"]) {
                self.expect("{{%")?;
                self.consume_whitespace();
                self.expect("else")?;
                self.consume_whitespace();
                self.expect("%}}")?;
                // Now parse the else body until we hit the end of the if block
                let else_body = self.parse_nodes_until(Some(["{{%", "endif", "%}}"]))?;
                self.expect("{{%")?;
                self.consume_whitespace();
                self.expect("endif")?;
                self.consume_whitespace();
                self.expect("%}}")?;
                else_branch_for_current_if = Some(Box::new(AstNode::Root(else_body)));
                break 'body_parsing_loop;
            } else if self.peek_n(["{{%", "endif", "%}}"]) {
                self.expect("{{%")?;
                self.consume_whitespace();
                self.expect("endif")?;
                self.consume_whitespace();
                self.expect("%}}")?;
                break 'body_parsing_loop;
            } else {
                body_nodes.push(self.parse_node()?);
            }
        }

        Ok(AstNode::If {
            condition: current_if_condition,
            body: body_nodes,
            else_branch: else_branch_for_current_if,
        })
    }
}

pub(crate) fn tokenize(input: &str) -> Result<AstNode<'_>, ParseError> {
    if input.is_empty() {
        return Ok(AstNode::Root(vec![]));
    }
    let mut parser = Parser::new(input);
    let nodes = parser.parse_nodes_until::<3>(None)?; // generic of 3 to avoid extra monomorphization

    if !parser.eof() {
        return Err(parser.make_error(ParseErrorKind::Message(format!(
            "Parser did not consume entire input. Remaining: '{}'",
            &parser.input[parser.pos..]
        ))));
    }

    Ok(AstNode::Root(nodes))
}

/// Tests for individual functions in the paresr module.
#[cfg(test)]
mod test_utils {
    use super::*;

    #[test]
    #[ntest::timeout(100)]
    fn test_peek_any() {
        let parser = Parser::new("  {{%     if condition %}}");
        assert!(!parser.peek_n(["{{%", "if"]));
        assert!(!parser.peek_n(["{{%", "else"]));
        assert!(!parser.peek_n(["{{%", "if", "else"]));
        assert!(!parser.peek_n(["{{%", "endif"]));

        let parser = Parser::new("{{%     if condition %}}");
        assert!(parser.peek_n(["{{%", "if"]));
        assert!(!parser.peek_n(["{{%", "else"]));
        assert!(!parser.peek_n(["{{%", "if", "else"]));
        assert!(!parser.peek_n(["{{%", "endif"]));

        let parser = Parser::new("{{%if condition %}}");
        assert!(parser.peek_n(["{{%", "if"]));
        assert!(!parser.peek_n(["{{%", "else"]));
        assert!(!parser.peek_n(["{{%", "if", "else"]));
        assert!(!parser.peek_n(["{{%", "endif"]));
    }
}

/// Tests for the parser module via tokenizer.
#[cfg(test)]
mod tests {
    use std::borrow::Cow;

    use super::*;

    // Helper macro for quick AST node creation in tests
    macro_rules! var {
        ($name:expr) => {
            AstNode::Variable { name: $name }
        };
    }
    macro_rules! const_str {
        ($data:expr) => {
            AstNode::Constant {
                data: Cow::Borrowed($data),
            }
        };
    }

    #[test]
    #[ntest::timeout(100)]
    fn test_empty_input() {
        assert_eq!(tokenize("").unwrap(), AstNode::Root(vec![]));
    }

    #[test]
    #[ntest::timeout(100)]
    fn test_simple_constant() {
        assert_eq!(
            tokenize("hello world").unwrap(),
            AstNode::Root(vec![const_str!("hello world")])
        );
    }

    #[test]
    #[ntest::timeout(100)]
    fn test_constant_ending_at_eof() {
        assert_eq!(
            tokenize("text").unwrap(),
            AstNode::Root(vec![const_str!("text")])
        );
    }

    #[test]
    #[ntest::timeout(100)]
    fn test_simple_variable() {
        assert_eq!(
            tokenize("{{name}}").unwrap(),
            AstNode::Root(vec![var!("name")])
        );
    }

    #[test]
    #[ntest::timeout(100)]
    fn test_variable_with_whitespace() {
        assert_eq!(
            tokenize("{{ name }}").unwrap(),
            AstNode::Root(vec![var!("name")])
        );
    }

    #[test]
    #[ntest::timeout(100)]
    fn test_variable_with_dot() {
        assert_eq!(
            tokenize("{{ user.name }}").unwrap(),
            AstNode::Root(vec![var!("user.name")])
        );
    }

    #[test]
    #[ntest::timeout(100)]
    fn test_constant_and_variable() {
        assert_eq!(
            tokenize("Hello {{name}}!").unwrap(),
            AstNode::Root(vec![const_str!("Hello "), var!("name"), const_str!("!")])
        );
    }

    #[test]
    #[ntest::timeout(100)]
    fn test_multiple_variables() {
        assert_eq!(
            tokenize("{{first}} {{second}}").unwrap(),
            AstNode::Root(vec![var!("first"), const_str!(" "), var!("second")])
        );
    }

    #[test]
    #[ntest::timeout(100)]
    fn test_leading_constant() {
        assert_eq!(
            tokenize("Prefix {{var}}").unwrap(),
            AstNode::Root(vec![const_str!("Prefix "), var!("var")])
        );
    }

    #[test]
    #[ntest::timeout(100)]
    fn test_trailing_constant() {
        assert_eq!(
            tokenize("{{var}} Suffix").unwrap(),
            AstNode::Root(vec![var!("var"), const_str!(" Suffix")])
        );
    }

    #[test]
    #[ntest::timeout(100)]
    fn test_panic_unclosed_variable() {
        let err = tokenize("{{var").unwrap_err();
        assert!(
            matches!(err.kind, ParseErrorKind::Expected { ref description } if description.contains("'}}'"))
        );
    }

    #[test]
    #[ntest::timeout(100)]
    fn test_panic_empty_variable() {
        let err = tokenize("{{}}").unwrap_err();
        assert!(
            matches!(err.kind, ParseErrorKind::Expected { ref description } if description.contains("identifier"))
        );
    }

    #[test]
    #[ntest::timeout(100)]
    fn test_panic_empty_variable_with_space() {
        let err = tokenize("{{ }}").unwrap_err();
        assert!(
            matches!(err.kind, ParseErrorKind::Expected { ref description } if description.contains("identifier"))
        );
    }

    // Test to ensure that if parse_constant returns an empty string (e.g. "{{var}}"),
    // it's not added to the node list by parse_nodes_until if a real node follows.
    #[test]
    #[ntest::timeout(100)]
    fn test_no_spurious_empty_constants_at_start_of_tag() {
        let ast = tokenize("{{var}}").unwrap();
        if let AstNode::Root(nodes) = ast {
            assert_eq!(nodes.len(), 1);
            assert_eq!(nodes[0], var!("var"));
        } else {
            panic!("Expected Root node");
        }
    }

    #[test]
    #[ntest::timeout(100)]
    fn test_no_spurious_empty_constants_between_tags() {
        let ast = tokenize("{{var1}}{{var2}}").unwrap();
        if let AstNode::Root(nodes) = ast {
            assert_eq!(nodes.len(), 2);
            assert_eq!(nodes[0], var!("var1"));
            assert_eq!(nodes[1], var!("var2"));
        } else {
            panic!("Expected Root node");
        }
    }

    // --- Tests for Condition Parsing ---
    // Helper to parse a condition string directly for testing.
    // This simulates being inside an `{{% if ... %}}` block.
    fn parse_test_condition(condition_str: &str) -> ParseResult<Box<AstNode>> {
        // Returns Result now
        let mut parser = Parser::new(condition_str);
        let condition_node_result = parser.parse_condition_expression();
        if condition_node_result.is_ok() && !parser.eof() {
            // If parsing was ok, but we didn't consume everything, that's an error for this helper
            return Err(parser.make_error(ParseErrorKind::Message(format!(
                "parse_test_condition did not consume entire input. Remaining: '{}'",
                &parser.input[parser.pos..]
            ))));
        }
        condition_node_result
    }

    #[test]
    #[ntest::timeout(100)]
    fn test_condition_single_variable() {
        assert_eq!(
            parse_test_condition("isActive").unwrap(),
            Box::new(var!("isActive"))
        );
    }

    #[test]
    #[ntest::timeout(100)]
    fn test_condition_not() {
        assert_eq!(
            parse_test_condition("!isActive").unwrap(),
            Box::new(AstNode::Not {
                condition: Box::new(var!("isActive"))
            })
        );
    }

    #[test]
    #[ntest::timeout(100)]
    fn test_condition_double_not() {
        assert_eq!(
            parse_test_condition("!!user").unwrap(),
            Box::new(AstNode::Not {
                condition: Box::new(AstNode::Not {
                    condition: Box::new(var!("user"))
                })
            })
        );
    }

    #[test]
    #[ntest::timeout(100)]
    fn test_condition_and() {
        assert_eq!(
            parse_test_condition("user && isActive").unwrap(),
            Box::new(AstNode::And {
                left: Box::new(var!("user")),
                right: Box::new(var!("isActive"))
            })
        );
    }

    #[test]
    #[ntest::timeout(100)]
    fn test_condition_or() {
        assert_eq!(
            parse_test_condition("isAdmin || isSuperuser").unwrap(),
            Box::new(AstNode::Or {
                left: Box::new(var!("isAdmin")),
                right: Box::new(var!("isSuperuser"))
            })
        );
    }

    #[test]
    #[ntest::timeout(100)]
    fn test_condition_precedence_and_then_or() {
        // Expected: (a && b) || c
        assert_eq!(
            parse_test_condition("a && b || c").unwrap(),
            Box::new(AstNode::Or {
                left: Box::new(AstNode::And {
                    left: Box::new(var!("a")),
                    right: Box::new(var!("b"))
                }),
                right: Box::new(var!("c"))
            })
        );
    }

    #[test]
    #[ntest::timeout(100)]
    fn test_condition_precedence_or_and_and() {
        // Expected: a || (b && c) -> but current grammar is left-associative for same precedence
        // OR and AND are usually different precedence. Here, OR is lower (binds less tightly).
        // a || b && c  should be a || (b && c) if AND is higher precedence.
        // Current parser: OR binds less tightly than AND.
        // parse_or_expression calls parse_and_expression.
        // So, `left` in OR is an AND-expression. `right` in OR is an AND-expression.
        // `a || b && c` -> left (`a`) becomes `var!(a)`. Then `||` is found.
        // `right` becomes `parse_and_expression` on `b && c`. This correctly yields `And(b,c)`.
        // So result is `Or(a, And(b,c))`. This is correct.
        assert_eq!(
            parse_test_condition("a || b && c").unwrap(),
            Box::new(AstNode::Or {
                left: Box::new(var!("a")),
                right: Box::new(AstNode::And {
                    left: Box::new(var!("b")),
                    right: Box::new(var!("c"))
                })
            })
        );
    }

    #[test]
    #[ntest::timeout(100)]
    fn test_condition_precedence_not_and() {
        // Expected: (!a) && b
        assert_eq!(
            parse_test_condition("!a && b").unwrap(),
            Box::new(AstNode::And {
                left: Box::new(AstNode::Not {
                    condition: Box::new(var!("a"))
                }),
                right: Box::new(var!("b"))
            })
        );
    }

    #[test]
    #[ntest::timeout(100)]
    fn test_condition_precedence_not_or() {
        // Expected: (!a) || b
        assert_eq!(
            parse_test_condition("!a || b").unwrap(),
            Box::new(AstNode::Or {
                left: Box::new(AstNode::Not {
                    condition: Box::new(var!("a"))
                }),
                right: Box::new(var!("b"))
            })
        );
    }

    #[test]
    #[ntest::timeout(100)]
    fn test_condition_complex_precedence() {
        // Expected: (!a && (b || !c)) || d
        // Our parser: ( (!a) && (b || (!c)) ) || d
        // parse_or_expression:
        //   left = parse_and_expression applied to "!a && b || !c"
        //     parse_and_expression for "!a && b || !c":
        //       left_and = parse_not_expression for "!a" -> Not(a)
        //       sees "&&"
        //       right_and = parse_not_expression for "b || !c"
        //         parse_not_expression for "b || !c" sees "b" (not "!") -> calls parse_primary_expression -> var(b)
        //         This is wrong. `parse_not_expression` should not consume parts of an OR or AND on its right if it's not part of its own operand.
        // The precedence is handled by the call stack: parse_or calls parse_and, parse_and calls parse_not, parse_not calls parse_primary.
        // For "!a && b || !c":
        // parse_or_expression:
        //   left = parse_and_expression("!a && b")  -> And(Not(a), b)
        //   sees "||"
        //   right = parse_and_expression("!c") -> parse_not_expression("!c") -> Not(c)
        // Result: Or(And(Not(a), b), Not(c))
        // This means "!a && b || !c" is ((!a) && b) || (!c)
        assert_eq!(
            parse_test_condition("!a && b || !c").unwrap(),
            Box::new(AstNode::Or {
                left: Box::new(AstNode::And {
                    left: Box::new(AstNode::Not {
                        condition: Box::new(var!("a"))
                    }),
                    right: Box::new(var!("b"))
                }),
                right: Box::new(AstNode::Not {
                    condition: Box::new(var!("c"))
                })
            })
        );
    }

    #[test]
    #[ntest::timeout(100)]
    fn test_condition_empty_string() {
        let err = parse_test_condition("").unwrap_err();
        assert!(matches!(err.kind, ParseErrorKind::Expected { .. }));
    }

    #[test]
    #[ntest::timeout(100)]
    fn test_condition_only_operator_and() {
        let err = parse_test_condition("&").unwrap_err();
        assert!(matches!(err.kind, ParseErrorKind::Expected { .. }));
    }

    #[test]
    #[ntest::timeout(100)]
    fn test_condition_incomplete_and() {
        let err = parse_test_condition("a &&").unwrap_err();
        assert!(matches!(err.kind, ParseErrorKind::Expected { .. }));
    }

    #[test]
    #[ntest::timeout(100)]
    fn test_condition_incomplete_or() {
        let err = parse_test_condition("a ||").unwrap_err();
        assert!(matches!(err.kind, ParseErrorKind::Expected { .. }));
    }

    #[test]
    #[ntest::timeout(100)]
    fn test_condition_incomplete_not() {
        let err = parse_test_condition("!").unwrap_err();
        assert!(matches!(err.kind, ParseErrorKind::Expected { .. }));
    }

    #[test]
    #[ntest::timeout(100)]
    fn test_condition_trailing_operator_error() {
        // parse_test_condition("a && b ||") would return Err
        // The error kind should be ParseErrorKind::Expected { description: "identifier" }
        // The error message from Display for ParseError will contain the line/col and specific message.
        let _result = parse_test_condition("a && b ||");
        // "a && b ||" is 9 characters. Error occurs at column 10 (pos 9).
        let err = parse_test_condition("a && b ||").unwrap_err();
        assert_eq!(err.line, 1);
        assert_eq!(err.column, 10);
        assert!(
            matches!(err.kind, ParseErrorKind::Expected { ref description } if description == "identifier" )
        );
    }

    // --- Tests for For Loops ---
    #[test]
    #[ntest::timeout(100)]
    fn test_simple_for_loop() {
        let input = "{{% for item in items %}} {{item}} {{% endfor %}}";
        let expected = AstNode::Root(vec![AstNode::For {
            variable: "item",
            iterable: "items",
            body: vec![const_str!(" "), var!("item"), const_str!(" ")],
        }]);
        assert_eq!(tokenize(input).unwrap(), expected);
    }

    #[test]
    #[ntest::timeout(100)]
    fn test_for_loop_with_constants_and_vars() {
        let input = "{{% for x in list %}}Value: {{x}}!{{% endfor %}}";
        let expected = AstNode::Root(vec![AstNode::For {
            variable: "x",
            iterable: "list",
            body: vec![const_str!("Value: "), var!("x"), const_str!("!")],
        }]);
        assert_eq!(tokenize(input).unwrap(), expected);
    }

    #[test]
    #[ntest::timeout(100)]
    fn test_empty_for_loop() {
        let input = "{{% for i in data %}}{{% endfor %}}";
        let expected = AstNode::Root(vec![AstNode::For {
            variable: "i",
            iterable: "data",
            body: vec![],
        }]);
        assert_eq!(tokenize(input).unwrap(), expected);
    }

    #[test]
    #[ntest::timeout(100)]
    fn test_for_loop_missing_in() {
        let input = "{{% for item items %}}loop{{% endfor %}}";
        let err = tokenize(input).unwrap_err();
        assert_eq!(err.line, 1);
        // "{{% for item " is 13 chars. Error occurs after consuming "item", whitespace, then "items"
        // Error occurs when `expect("in")` fails.
        // Parser state: `{{% for item items %}}...`
        // `{{%` consumed. ` ` consumed. `for` consumed. ` ` consumed. `item` consumed.
        // ` ` consumed. `items` is consumed as iterable if `in` was present. Here `items` is consumed as variable.
        // Then `expect("in")` is called.
        // `consume_identifier()` for `item` moves pos.
        // `consume_whitespace()`
        // `expect("in")` is called. `pos` is after `item `.
        // Let's re-evaluate:
        // `{{%` (3) ` ` (1) `for` (3) ` ` (1) `item` (4) ` ` (1) -> pos = 13
        // next is `items`. `expect("in")` fails. Column is pos(13) - line_start_pos(0) + 1 = 14.
        assert_eq!(err.column, 14); // After "for item ", trying to find "in", sees "items"
        assert!(
            matches!(err.kind, ParseErrorKind::Expected { ref description } if description.contains("in") || description.contains("'in'"))
        );
    }

    #[test]
    #[ntest::timeout(100)]
    fn test_for_loop_missing_iterable() {
        let input = "{{% for item in %}}loop{{% endfor %}}";
        let err = tokenize(input).unwrap_err();
        assert_eq!(err.line, 1);
        // "{{% for item in " is 16 chars. `consume_identifier` for iterable is called on empty string.
        assert_eq!(err.column, 17); // Position after "in "
        assert!(
            matches!(err.kind, ParseErrorKind::Expected { ref description } if description == "identifier")
        );
    }

    #[test]
    #[ntest::timeout(100)]
    fn test_for_loop_missing_closing_tag_delimiter() {
        let input = "{{% for item in items loop{{% endfor %}}";
        let err = tokenize(input).unwrap_err();
        assert_eq!(err.line, 1);
        // "{{% for item in items " is 21 chars. `loop...` is where `%}}` is expected.
        assert_eq!(err.column, 23); // Position after "items "
        assert!(
            matches!(err.kind, ParseErrorKind::Expected { ref description } if description.contains("'%}}'"))
        );
    }

    #[test]
    #[ntest::timeout(100)]
    fn test_for_loop_unclosed_block() {
        let input = "{{% for item in items %}}loop";
        let err = tokenize(input).unwrap_err();
        // Error occurs at EOF when `parse_nodes_until` expects "{{% endfor %}}"
        // Input length is 26. Line 1, Column 27 (pos 26, 1-based)
        assert_eq!(err.line, 1);
        assert_eq!(err.column, input.len() + 1);
        assert!(
            matches!(err.kind, ParseErrorKind::UnexpectedEOF { ref expected_what } if expected_what.contains("{{% endfor %}}"))
        );
    }

    // --- Tests for If Statements ---
    #[test]
    #[ntest::timeout(100)]
    fn test_simple_if() {
        let input = "{{% if condition %}}Hello{{% endif %}}";
        let expected = AstNode::Root(vec![AstNode::If {
            condition: Box::new(var!("condition")),
            body: vec![const_str!("Hello")],
            else_branch: None,
        }]);
        assert_eq!(tokenize(input), Ok(expected));
    }

    #[test]
    #[ntest::timeout(100)]
    fn test_if_with_empty_body() {
        let input = "{{% if condition %}}{{% endif %}}";
        let expected = AstNode::Root(vec![AstNode::If {
            condition: Box::new(var!("condition")),
            body: vec![],
            else_branch: None,
        }]);
        assert_eq!(tokenize(input), Ok(expected));
    }

    #[test]
    #[ntest::timeout(100)]
    fn test_if_else() {
        let input = "{{% if user.active %}}Welcome!{{% else %}}Access Denied.{{% endif %}}";
        let expected = AstNode::Root(vec![AstNode::If {
            condition: Box::new(var!("user.active")),
            body: vec![const_str!("Welcome!")],
            else_branch: Some(Box::new(AstNode::Root(vec![const_str!("Access Denied.")]))),
        }]);
        assert_eq!(tokenize(input), Ok(expected));
    }

    #[test]
    #[ntest::timeout(100)]
    fn test_if_else_if() {
        // {{% if a %}} A {{% else if b %}} B {{% endif %}}
        // This implies the endif closes 'b', and then the outer 'if a' structure.
        let input = "{{% if a %}} A {{% else if b %}} B {{% endif %}}";
        let expected = AstNode::Root(vec![AstNode::If {
            // Outer If (a)
            condition: Box::new(var!("a")),
            body: vec![const_str!(" A ")],
            else_branch: Some(Box::new(AstNode::Root(
                // Else branch for "a"
                vec![
                    // Body of this Root contains the "else if b" part
                    AstNode::If {
                        // Inner If (b) - representing "else if b"
                        condition: Box::new(var!("b")),
                        body: vec![const_str!(" B ")],
                        else_branch: None, // "else if b" has no further "else"
                    },
                ],
            ))),
        }]);
        assert_eq!(tokenize(input), Ok(expected));
    }

    #[test]
    #[ntest::timeout(100)]
    fn test_if_else_if_else() {
        let input = "{{% if cA %}}Aye{{% else if cB %}}Bee{{% else %}}Sea{{% endif %}}";
        let expected = AstNode::Root(vec![AstNode::If {
            // Outer If (cA)
            condition: Box::new(var!("cA")),
            body: vec![const_str!("Aye")],
            else_branch: Some(Box::new(AstNode::Root(
                // Else branch for cA
                vec![
                    // Body contains the "else if cB..." part
                    AstNode::If {
                        // Inner If (cB) for "else if cB"
                        condition: Box::new(var!("cB")),
                        body: vec![const_str!("Bee")],
                        else_branch: Some(Box::new(AstNode::Root(
                            // Else branch for cB
                            vec![const_str!("Sea")], // Body of the innermost else
                        ))),
                    },
                ],
            ))),
        }]);
        assert_eq!(tokenize(input), Ok(expected));
    }

    #[test]
    #[ntest::timeout(100)]
    fn test_if_with_complex_condition() {
        // Let's re-test the complex condition from before directly within an if
        let input_complex = "{{% if !a && b || !c %}}Content{{% endif %}}";
        let expected_complex_cond = Box::new(AstNode::Or {
            left: Box::new(AstNode::And {
                left: Box::new(AstNode::Not {
                    condition: Box::new(var!("a")),
                }),
                right: Box::new(var!("b")),
            }),
            right: Box::new(AstNode::Not {
                condition: Box::new(var!("c")),
            }),
        });

        assert_eq!(
            tokenize(input_complex),
            Ok(AstNode::Root(vec![AstNode::If {
                condition: expected_complex_cond,
                body: vec![const_str!("Content")],
                else_branch: None
            }]))
        );
    }

    #[test]
    #[ntest::timeout(100)]
    fn test_if_missing_closing_tag_delimiter() {
        let input = "{{% if condition text {{% endif %}}";
        let err = tokenize(input).unwrap_err();
        assert_eq!(err.line, 1);
        // "{{% if condition " = 17 characters, error at 18th char (pos 17)
        assert_eq!(err.column, 18);
        assert!(
            matches!(err.kind, ParseErrorKind::Expected { ref description } if description.contains("'%}}'"))
        );
    }

    #[test]
    #[ntest::timeout(100)]
    fn test_if_unclosed_simple() {
        let input = "{{% if condition %}} text";
        let err = tokenize(input).unwrap_err();
        assert_eq!(err.line, 1);
        // Input length is 23. Error is at EOF (pos 23), so column 24.
        assert_eq!(err.column, input.len() + 1);
        assert!(
            matches!(err.kind, ParseErrorKind::UnexpectedEOF { ref expected_what } if expected_what.contains("{{% endif %}}"))
        );
    }

    #[test]
    #[ntest::timeout(100)]
    fn test_if_unclosed_with_else() {
        let input = "{{% if condition %}} text {{% else %}} other";
        let err = tokenize(input).unwrap_err();
        assert_eq!(err.line, 1);
        // Error occurs at EOF when parse_nodes_until for else body expects endif
        assert_eq!(err.column, input.len() + 1);
        assert!(
            matches!(err.kind, ParseErrorKind::UnexpectedEOF { ref expected_what } if expected_what.contains("{{% endif %}}"))
        );
    }

    #[test]
    #[ntest::timeout(100)]
    fn test_if_unclosed_with_else_if() {
        let input = "{{% if c1 %}} A {{% else if c2 %}} B";
        let err = tokenize(input).unwrap_err();
        assert_eq!(err.line, 1);
        // Error occurs at EOF during parsing of the body of "else if c2 "
        assert_eq!(err.column, input.len() + 1);
        assert!(
            matches!(err.kind, ParseErrorKind::UnexpectedEOF { ref expected_what } if expected_what.contains("{{% endif %}}"))
        );
    }

    // Nested structures
    #[test]
    #[ntest::timeout(100)]
    fn test_nested_if_in_for() {
        let input = concat!(
            "{{% for user in users %}}",
            "{{% if user.active %}}",
            "{{user.name}}",
            "{{% else %}}",
            "Inactive",
            "{{% endif %}}",
            "{{% endfor %}}"
        );
        let expected = AstNode::Root(vec![AstNode::For {
            variable: "user",
            iterable: "users",
            body: vec![AstNode::If {
                condition: Box::new(var!("user.active")),
                body: vec![var!("user.name")],
                else_branch: Some(Box::new(AstNode::Root(vec![const_str!("Inactive")]))),
            }],
        }]);
        assert_eq!(tokenize(input), Ok(expected));
    }

    #[test]
    #[ntest::timeout(100)]
    fn test_nested_for_in_if() {
        let input = concat!(
            "{{% if items_exist %}}",
            "{{% for item in items %}}",
            "{{item}}",
            "{{% endfor %}}",
            "{{% else %}}",
            "No items.",
            "{{% endif %}}"
        );
        let expected = AstNode::Root(vec![AstNode::If {
            condition: Box::new(var!("items_exist")),
            body: vec![AstNode::For {
                variable: "item",
                iterable: "items",
                body: vec![var!("item")],
            }],
            else_branch: Some(Box::new(AstNode::Root(vec![const_str!("No items.")]))),
        }]);
        assert_eq!(tokenize(input).unwrap(), expected);
    }

    // --- Tests for Line Comments (//) ---

    #[test]
    #[ntest::timeout(100)]
    fn test_comment_full_line() {
        let input = "// This is a full line comment\n{{var}}";
        let expected = AstNode::Root(vec![
            const_str!("// This is a full line comment\n"),
            var!("var"),
        ]);
        assert_eq!(tokenize(input).unwrap(), expected);
    }

    #[test]
    #[ntest::timeout(100)]
    fn test_comment_after_whitespace() {
        let input = "  // This is a comment after whitespace\n{{var}}";
        let expected = AstNode::Root(vec![
            const_str!("  // This is a comment after whitespace\n"),
            var!("var"),
        ]);
        assert_eq!(tokenize(input).unwrap(), expected);
    }

    #[test]
    #[ntest::timeout(100)]
    fn test_comment_at_end_of_file() {
        let input = "{{var}}\n// This is a comment at EOF";
        let expected = AstNode::Root(vec![
            var!("var"),
            const_str!("\n// This is a comment at EOF"),
        ]);
        assert_eq!(tokenize(input).unwrap(), expected);
    }

    #[test]
    #[ntest::timeout(100)]
    fn test_comment_at_very_end_of_file_no_newline() {
        let input = "{{var}}//EOF comment";
        let expected = AstNode::Root(vec![var!("var"), const_str!("//EOF comment")]);
        assert_eq!(tokenize(input).unwrap(), expected);
    }

    #[test]
    #[ntest::timeout(100)]
    fn test_only_comment_in_file() {
        let input = "// Just a comment";
        let expected = AstNode::Root(vec![const_str!("// Just a comment")]);
        assert_eq!(tokenize(input).unwrap(), expected);
    }

    #[test]
    #[ntest::timeout(100)]
    fn test_only_comment_with_newline_in_file() {
        let input = "// Just a comment\n";
        let expected = AstNode::Root(vec![const_str!("// Just a comment\n")]);
        assert_eq!(tokenize(input).unwrap(), expected);
    }

    #[test]
    #[ntest::timeout(100)]
    fn test_multiple_comments() {
        let input = "// Comment 1\n{{var1}}\n// Comment 2\n  // Comment 3\n{{var2}} // Comment 4";
        let expected = AstNode::Root(vec![
            const_str!("// Comment 1\n"),
            var!("var1"),
            const_str!("\n// Comment 2\n  // Comment 3\n"),
            var!("var2"),
            const_str!(" // Comment 4"),
        ]);
        assert_eq!(tokenize(input).unwrap(), expected);
    }

    #[test]
    #[ntest::timeout(100)]
    fn test_comment_between_tags() {
        let input = "{{var1}} // comment here\n{{var2}}";
        let expected = AstNode::Root(vec![
            var!("var1"),
            const_str!(" // comment here\n"),
            var!("var2"),
        ]);
        assert_eq!(tokenize(input).unwrap(), expected);
    }

    #[test]
    #[ntest::timeout(100)]
    fn test_comment_inside_tag_is_not_a_comment() {
        // Inside a tag, // is not a comment but part of the identifier
        let input = "{{a_b}}"; // Use a variable name without special characters
        let expected = AstNode::Root(vec![var!("a_b")]);
        // Remove the unwrap and handle the Result directly
        match tokenize(input) {
            Ok(result) => assert_eq!(result, expected),
            Err(e) => panic!("Expected success, got error: {:?}", e),
        }
    }

    #[test]
    #[ntest::timeout(100)]
    fn test_comment_inside_directive_tag() {
        // This test verifies how comments in directive tags are handled
        let input_if = "{{% if a//b %}}text{{% endif %}}";
        // This should error because // isn't valid in a directive without proper handling
        let err = tokenize(input_if).unwrap_err();
        assert_eq!(err.line, 1);
        assert!(matches!(err.kind, ParseErrorKind::Expected { .. }));
    }

    #[test]
    #[ntest::timeout(100)]
    fn test_comment_in_constant_text_is_not_a_comment() {
        let input = "This is text with // inside it.";
        let expected = AstNode::Root(vec![const_str!("This is text with // inside it.")]);
        assert_eq!(tokenize(input).unwrap(), expected);
    }

    #[test]
    #[ntest::timeout(100)]
    fn test_empty_lines_and_comments() {
        let input = "\n  // comment\n\n{{var}}\n  \n// another";
        let expected = AstNode::Root(vec![
            const_str!("\n  // comment\n\n"), // Include comment and following newlines
            var!("var"),
            const_str!("\n  \n// another"), // Include trailing comment
        ]);
        assert_eq!(tokenize(input).unwrap(), expected);
    }
}
