use super::ast::*;
use super::source::SourceSpan;
use super::token::{Token, TokenKind};

#[derive(Clone, Debug, PartialEq)]
pub struct ParserError {
    pub message: String,
    pub span: SourceSpan,
    pub line: usize,
    pub column: usize,
}

impl ParserError {
    fn new(message: impl Into<String>, token: &Token) -> Self {
        Self {
            message: message.into(),
            span: token.span,
            line: token.line,
            column: token.column,
        }
    }
}

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }

    pub fn parse(mut self) -> Result<Program, ParserError> {
        let mut declarations = Vec::new();

        self.skip_newlines();

        while !self.is_at_end() {
            declarations.push(self.parse_declaration()?);
            self.skip_newlines();
        }

        let span = match (declarations.first(), declarations.last()) {
            (Some(first), Some(last)) => SourceSpan::new(first.span().start, last.span().end),
            _ => self.peek().span,
        };

        Ok(Program { span, declarations })
    }

    fn parse_declaration(&mut self) -> Result<Declaration, ParserError> {
        match self.peek_kind() {
            TokenKind::Entity => self.parse_entity().map(Declaration::Entity),

            TokenKind::Fn => self.parse_function().map(Declaration::Function),

            TokenKind::Import => self.parse_import().map(Declaration::Import),

            _ => Err(self.error_current("Expected a declaration.")),
        }
    }

    fn parse_entity(&mut self) -> Result<EntityDecl, ParserError> {
        let entity_token = self.consume_simple(TokenKind::Entity, "Expected 'entity'.")?;

        let start = entity_token.span.start;

        let (name, _) = self.consume_identifier("Expected entity name.")?;

        self.skip_newlines();

        self.consume_simple(TokenKind::LeftBrace, "Expected '{' after entity name.")?;

        let mut members = Vec::new();

        self.skip_newlines();

        while !self.check_simple(&TokenKind::RightBrace) && !self.is_at_end() {
            if self.check_simple(&TokenKind::Fn) {
                members.push(EntityMember::Function(self.parse_function()?));
            } else {
                members.push(EntityMember::Field(self.parse_field()?));
            }

            self.skip_newlines();
        }

        let right_brace =
            self.consume_simple(TokenKind::RightBrace, "Expected '}' after entity body.")?;

        Ok(EntityDecl {
            span: SourceSpan::new(start, right_brace.span.end),
            name,
            members,
        })
    }

    fn parse_field(&mut self) -> Result<FieldDecl, ParserError> {
        let (name, name_span) = self.consume_identifier("Expected field name.")?;

        let type_annotation = if self.match_simple(&TokenKind::Colon) {
            Some(self.parse_type()?)
        } else {
            None
        };

        let initializer = if self.match_simple(&TokenKind::Equal) {
            Some(self.parse_expression()?)
        } else {
            None
        };

        if type_annotation.is_none() && initializer.is_none() {
            return Err(self.error_current("A field needs a type, an initializer, or both."));
        }

        self.consume_statement_end();

        let end = initializer
            .as_ref()
            .map(|expression| expression.span.end)
            .or_else(|| {
                type_annotation
                    .as_ref()
                    .map(|type_annotation| type_annotation.span.end)
            })
            .unwrap_or(name_span.end);

        Ok(FieldDecl {
            span: SourceSpan::new(name_span.start, end),
            name,
            type_annotation,
            initializer,
        })
    }

    fn parse_function(&mut self) -> Result<FunctionDecl, ParserError> {
        let fn_token = self.consume_simple(TokenKind::Fn, "Expected 'fn'.")?;

        let start = fn_token.span.start;

        let (name, _) = self.consume_identifier("Expected function name.")?;

        self.consume_simple(TokenKind::LeftParen, "Expected '(' after function name.")?;

        let mut parameters = Vec::new();

        self.skip_newlines();

        if !self.check_simple(&TokenKind::RightParen) {
            loop {
                let (parameter_name, parameter_name_span) =
                    self.consume_identifier("Expected parameter name.")?;

                let type_annotation = if self.match_simple(&TokenKind::Colon) {
                    Some(self.parse_type()?)
                } else {
                    None
                };

                let parameter_end = type_annotation
                    .as_ref()
                    .map(|type_annotation| type_annotation.span.end)
                    .unwrap_or(parameter_name_span.end);

                parameters.push(Parameter {
                    span: SourceSpan::new(parameter_name_span.start, parameter_end),
                    name: parameter_name,
                    type_annotation,
                });

                self.skip_newlines();

                if !self.match_simple(&TokenKind::Comma) {
                    break;
                }

                self.skip_newlines();
            }
        }

        self.consume_simple(TokenKind::RightParen, "Expected ')' after parameters.")?;

        let return_type = if self.match_simple(&TokenKind::Colon) {
            Some(self.parse_type()?)
        } else {
            None
        };

        self.skip_newlines();

        let body = self.parse_block()?;

        Ok(FunctionDecl {
            span: SourceSpan::new(start, body.span.end),
            name,
            parameters,
            return_type,
            body,
        })
    }

    fn parse_import(&mut self) -> Result<ImportDecl, ParserError> {
        let import_token = self.consume_simple(TokenKind::Import, "Expected 'import'.")?;

        let start = import_token.span.start;

        let (first, first_span) = self.consume_identifier("Expected import name.")?;

        let mut path = vec![first];
        let mut end = first_span.end;

        while self.match_simple(&TokenKind::Dot) {
            let (part, part_span) = self.consume_identifier("Expected identifier after '.'.")?;

            end = part_span.end;
            path.push(part);
        }

        self.consume_statement_end();

        Ok(ImportDecl {
            span: SourceSpan::new(start, end),
            path,
        })
    }

    fn parse_type(&mut self) -> Result<Type, ParserError> {
        let (name, name_span) = self.consume_identifier("Expected type name.")?;

        let base = Type {
            span: name_span,
            kind: TypeKind::Named(name),
        };

        if let Some(question) = self.match_simple_return(&TokenKind::QuestionMark) {
            return Ok(Type {
                span: SourceSpan::new(name_span.start, question.span.end),
                kind: TypeKind::Optional(Box::new(base)),
            });
        }

        Ok(base)
    }

    fn parse_block(&mut self) -> Result<Block, ParserError> {
        self.skip_newlines();

        let left_brace = self.consume_simple(TokenKind::LeftBrace, "Expected '{'.")?;

        let start = left_brace.span.start;

        let mut statements = Vec::new();

        self.skip_newlines();

        while !self.check_simple(&TokenKind::RightBrace) && !self.is_at_end() {
            statements.push(self.parse_statement()?);

            self.skip_newlines();
        }

        let right_brace =
            self.consume_simple(TokenKind::RightBrace, "Expected '}' after block.")?;

        Ok(Block {
            span: SourceSpan::new(start, right_brace.span.end),
            statements,
        })
    }

    fn parse_statement(&mut self) -> Result<Statement, ParserError> {
        match self.peek_kind() {
            TokenKind::Const => self.parse_variable(true),

            TokenKind::Identifier(_) => {
                if self.peek_next_simple(&TokenKind::Colon) {
                    self.parse_variable(false)
                } else {
                    self.parse_expression_or_assignment()
                }
            }

            TokenKind::If => self.parse_if(),

            TokenKind::While => self.parse_while(),

            TokenKind::For => self.parse_for(),

            TokenKind::Return => self.parse_return(),

            _ => self.parse_expression_or_assignment(),
        }
    }

    fn parse_variable(&mut self, is_const: bool) -> Result<Statement, ParserError> {
        let start = if is_const {
            self.consume_simple(TokenKind::Const, "Expected 'const'.")?
                .span
                .start
        } else {
            self.peek().span.start
        };

        let (name, name_span) = self.consume_identifier("Expected variable name.")?;

        let type_annotation = if self.match_simple(&TokenKind::Colon) {
            Some(self.parse_type()?)
        } else {
            None
        };

        let initializer = if self.match_simple(&TokenKind::Equal) {
            Some(self.parse_expression()?)
        } else {
            None
        };

        if type_annotation.is_none() && initializer.is_none() {
            return Err(self.error_current("A variable needs a type, an initializer, or both."));
        }

        self.consume_statement_end();

        let end = initializer
            .as_ref()
            .map(|expression| expression.span.end)
            .or_else(|| {
                type_annotation
                    .as_ref()
                    .map(|type_annotation| type_annotation.span.end)
            })
            .unwrap_or(name_span.end);

        Ok(Statement {
            span: SourceSpan::new(start, end),
            kind: StatementKind::Variable {
                name,
                type_annotation,
                initializer,
                is_const,
            },
        })
    }

    fn parse_if(&mut self) -> Result<Statement, ParserError> {
        let if_token = self.consume_simple(TokenKind::If, "Expected 'if'.")?;

        let start = if_token.span.start;

        let condition = self.parse_expression()?;
        let then_block = self.parse_block()?;

        let mut else_if = Vec::new();
        let mut else_block = None;
        let mut end = then_block.span.end;

        loop {
            self.skip_newlines();

            if !self.match_simple(&TokenKind::Else) {
                break;
            }

            self.skip_newlines();

            if self.match_simple(&TokenKind::If) {
                let condition = self.parse_expression()?;

                let block = self.parse_block()?;
                end = block.span.end;

                else_if.push((condition, block));
            } else {
                let block = self.parse_block()?;
                end = block.span.end;
                else_block = Some(block);
                break;
            }
        }

        Ok(Statement {
            span: SourceSpan::new(start, end),
            kind: StatementKind::If {
                condition,
                then_block,
                else_if,
                else_block,
            },
        })
    }

    fn parse_while(&mut self) -> Result<Statement, ParserError> {
        let while_token = self.consume_simple(TokenKind::While, "Expected 'while'.")?;

        let condition = self.parse_expression()?;
        let body = self.parse_block()?;

        Ok(Statement {
            span: SourceSpan::new(while_token.span.start, body.span.end),
            kind: StatementKind::While { condition, body },
        })
    }

    fn parse_for(&mut self) -> Result<Statement, ParserError> {
        let for_token = self.consume_simple(TokenKind::For, "Expected 'for'.")?;

        let (name, _) = self.consume_identifier("Expected loop variable name.")?;

        self.consume_simple(TokenKind::In, "Expected 'in' after loop variable.")?;

        let iterable = self.parse_expression()?;
        let body = self.parse_block()?;

        Ok(Statement {
            span: SourceSpan::new(for_token.span.start, body.span.end),
            kind: StatementKind::For {
                name,
                iterable,
                body,
            },
        })
    }

    fn parse_return(&mut self) -> Result<Statement, ParserError> {
        let return_token = self.consume_simple(TokenKind::Return, "Expected 'return'.")?;

        let start = return_token.span.start;

        if self.check_simple(&TokenKind::Newline)
            || self.check_simple(&TokenKind::RightBrace)
            || self.check_simple(&TokenKind::Eof)
        {
            self.consume_statement_end();

            return Ok(Statement {
                span: return_token.span,
                kind: StatementKind::Return(None),
            });
        }

        let value = self.parse_expression()?;

        self.consume_statement_end();

        Ok(Statement {
            span: SourceSpan::new(start, value.span.end),
            kind: StatementKind::Return(Some(value)),
        })
    }

    fn parse_expression_or_assignment(&mut self) -> Result<Statement, ParserError> {
        let expression = self.parse_expression()?;

        let operator = match self.peek_kind() {
            TokenKind::Equal => Some(AssignmentOperator::Assign),

            TokenKind::PlusEqual => Some(AssignmentOperator::Add),

            TokenKind::MinusEqual => Some(AssignmentOperator::Subtract),

            TokenKind::StarEqual => Some(AssignmentOperator::Multiply),

            TokenKind::SlashEqual => Some(AssignmentOperator::Divide),

            _ => None,
        };

        if let Some(operator) = operator {
            self.advance();

            let value = self.parse_expression()?;
            self.consume_statement_end();

            let span = SourceSpan::new(expression.span.start, value.span.end);

            return Ok(Statement {
                span,
                kind: StatementKind::Assignment {
                    target: expression,
                    operator,
                    value,
                },
            });
        }

        self.consume_statement_end();

        let span = expression.span;

        Ok(Statement {
            span,
            kind: StatementKind::Expression(expression),
        })
    }

    fn parse_expression(&mut self) -> Result<Expression, ParserError> {
        self.parse_or()
    }

    fn parse_or(&mut self) -> Result<Expression, ParserError> {
        let mut expression = self.parse_and()?;

        while self.match_simple(&TokenKind::OrOr) {
            let right = self.parse_and()?;

            let span = SourceSpan::new(expression.span.start, right.span.end);

            expression = Expression {
                span,
                kind: ExpressionKind::Binary {
                    left: Box::new(expression),
                    operator: BinaryOperator::Or,
                    right: Box::new(right),
                },
            };
        }

        Ok(expression)
    }

    fn parse_and(&mut self) -> Result<Expression, ParserError> {
        let mut expression = self.parse_equality()?;

        while self.match_simple(&TokenKind::AndAnd) {
            let right = self.parse_equality()?;

            let span = SourceSpan::new(expression.span.start, right.span.end);

            expression = Expression {
                span,
                kind: ExpressionKind::Binary {
                    left: Box::new(expression),
                    operator: BinaryOperator::And,
                    right: Box::new(right),
                },
            };
        }

        Ok(expression)
    }

    fn parse_equality(&mut self) -> Result<Expression, ParserError> {
        let mut expression = self.parse_comparison()?;

        loop {
            let operator = if self.match_simple(&TokenKind::EqualEqual) {
                Some(BinaryOperator::Equal)
            } else if self.match_simple(&TokenKind::BangEqual) {
                Some(BinaryOperator::NotEqual)
            } else {
                None
            };

            let Some(operator) = operator else {
                break;
            };

            let right = self.parse_comparison()?;

            let span = SourceSpan::new(expression.span.start, right.span.end);

            expression = Expression {
                span,
                kind: ExpressionKind::Binary {
                    left: Box::new(expression),
                    operator,
                    right: Box::new(right),
                },
            };
        }

        Ok(expression)
    }

    fn parse_comparison(&mut self) -> Result<Expression, ParserError> {
        let mut expression = self.parse_term()?;

        loop {
            let operator = match self.peek_kind() {
                TokenKind::Less => Some(BinaryOperator::Less),

                TokenKind::LessEqual => Some(BinaryOperator::LessEqual),

                TokenKind::Greater => Some(BinaryOperator::Greater),

                TokenKind::GreaterEqual => Some(BinaryOperator::GreaterEqual),

                _ => None,
            };

            let Some(operator) = operator else {
                break;
            };

            self.advance();

            let right = self.parse_term()?;

            let span = SourceSpan::new(expression.span.start, right.span.end);

            expression = Expression {
                span,
                kind: ExpressionKind::Binary {
                    left: Box::new(expression),
                    operator,
                    right: Box::new(right),
                },
            };
        }

        Ok(expression)
    }

    fn parse_term(&mut self) -> Result<Expression, ParserError> {
        let mut expression = self.parse_factor()?;

        loop {
            let operator = match self.peek_kind() {
                TokenKind::Plus => Some(BinaryOperator::Add),

                TokenKind::Minus => Some(BinaryOperator::Subtract),

                _ => None,
            };

            let Some(operator) = operator else {
                break;
            };

            self.advance();

            let right = self.parse_factor()?;

            let span = SourceSpan::new(expression.span.start, right.span.end);

            expression = Expression {
                span,
                kind: ExpressionKind::Binary {
                    left: Box::new(expression),
                    operator,
                    right: Box::new(right),
                },
            };
        }

        Ok(expression)
    }

    fn parse_factor(&mut self) -> Result<Expression, ParserError> {
        let mut expression = self.parse_unary()?;

        loop {
            let operator = match self.peek_kind() {
                TokenKind::Star => Some(BinaryOperator::Multiply),

                TokenKind::Slash => Some(BinaryOperator::Divide),

                TokenKind::Percent => Some(BinaryOperator::Modulo),

                _ => None,
            };

            let Some(operator) = operator else {
                break;
            };

            self.advance();

            let right = self.parse_unary()?;

            let span = SourceSpan::new(expression.span.start, right.span.end);

            expression = Expression {
                span,
                kind: ExpressionKind::Binary {
                    left: Box::new(expression),
                    operator,
                    right: Box::new(right),
                },
            };
        }

        Ok(expression)
    }

    fn parse_unary(&mut self) -> Result<Expression, ParserError> {
        if let Some(token) = self.match_simple_return(&TokenKind::Minus) {
            let expression = self.parse_unary()?;

            return Ok(Expression {
                span: SourceSpan::new(token.span.start, expression.span.end),
                kind: ExpressionKind::Unary {
                    operator: UnaryOperator::Negate,
                    expression: Box::new(expression),
                },
            });
        }

        if let Some(token) = self.match_simple_return(&TokenKind::Bang) {
            let expression = self.parse_unary()?;

            return Ok(Expression {
                span: SourceSpan::new(token.span.start, expression.span.end),
                kind: ExpressionKind::Unary {
                    operator: UnaryOperator::Not,
                    expression: Box::new(expression),
                },
            });
        }

        self.parse_postfix()
    }

    fn parse_postfix(&mut self) -> Result<Expression, ParserError> {
        let mut expression = self.parse_primary()?;

        loop {
            if self.match_simple(&TokenKind::LeftParen) {
                let mut arguments = Vec::new();

                self.skip_newlines();

                if !self.check_simple(&TokenKind::RightParen) {
                    loop {
                        arguments.push(self.parse_expression()?);

                        self.skip_newlines();

                        if !self.match_simple(&TokenKind::Comma) {
                            break;
                        }

                        self.skip_newlines();
                    }
                }

                let right_paren =
                    self.consume_simple(TokenKind::RightParen, "Expected ')' after arguments.")?;

                let start = expression.span.start;

                expression = Expression {
                    span: SourceSpan::new(start, right_paren.span.end),
                    kind: ExpressionKind::Call {
                        callee: Box::new(expression),
                        arguments,
                    },
                };

                continue;
            }

            if self.match_simple(&TokenKind::Dot) {
                let (name, name_span) =
                    self.consume_identifier("Expected property name after '.'.")?;

                let start = expression.span.start;

                expression = Expression {
                    span: SourceSpan::new(start, name_span.end),
                    kind: ExpressionKind::Member {
                        object: Box::new(expression),
                        name,
                    },
                };

                continue;
            }

            if self.match_simple(&TokenKind::LeftBracket) {
                let index = self.parse_expression()?;

                let right_bracket =
                    self.consume_simple(TokenKind::RightBracket, "Expected ']' after index.")?;

                let start = expression.span.start;

                expression = Expression {
                    span: SourceSpan::new(start, right_bracket.span.end),
                    kind: ExpressionKind::Index {
                        object: Box::new(expression),
                        index: Box::new(index),
                    },
                };

                continue;
            }

            break;
        }

        Ok(expression)
    }

    fn parse_primary(&mut self) -> Result<Expression, ParserError> {
        let token = self.peek().clone();

        match token.kind {
            TokenKind::Number(value) => {
                self.advance();

                Ok(Expression {
                    span: token.span,
                    kind: ExpressionKind::Number(value),
                })
            }

            TokenKind::String(value) => {
                self.advance();

                Ok(Expression {
                    span: token.span,
                    kind: ExpressionKind::String(value),
                })
            }

            TokenKind::True => {
                self.advance();

                Ok(Expression {
                    span: token.span,
                    kind: ExpressionKind::Bool(true),
                })
            }

            TokenKind::False => {
                self.advance();

                Ok(Expression {
                    span: token.span,
                    kind: ExpressionKind::Bool(false),
                })
            }

            TokenKind::Null => {
                self.advance();

                Ok(Expression {
                    span: token.span,
                    kind: ExpressionKind::Null,
                })
            }

            TokenKind::Identifier(name) => {
                self.advance();

                Ok(Expression {
                    span: token.span,
                    kind: ExpressionKind::Identifier(name),
                })
            }

            TokenKind::LeftParen => {
                self.advance();

                let expression = self.parse_expression()?;

                let right_paren =
                    self.consume_simple(TokenKind::RightParen, "Expected ')' after expression.")?;

                let kind = expression.kind;

                Ok(Expression {
                    span: SourceSpan::new(token.span.start, right_paren.span.end),
                    kind,
                })
            }

            TokenKind::LeftBracket => self.parse_array(),

            TokenKind::LeftBrace => self.parse_map(),

            _ => Err(ParserError::new("Expected an expression.", &token)),
        }
    }

    fn parse_array(&mut self) -> Result<Expression, ParserError> {
        let left_bracket = self.consume_simple(TokenKind::LeftBracket, "Expected '['.")?;

        let mut values = Vec::new();

        self.skip_newlines();

        if !self.check_simple(&TokenKind::RightBracket) {
            loop {
                values.push(self.parse_expression()?);

                self.skip_newlines();

                if !self.match_simple(&TokenKind::Comma) {
                    break;
                }

                self.skip_newlines();
            }
        }

        let right_bracket =
            self.consume_simple(TokenKind::RightBracket, "Expected ']' after array.")?;

        Ok(Expression {
            span: SourceSpan::new(left_bracket.span.start, right_bracket.span.end),
            kind: ExpressionKind::Array(values),
        })
    }

    fn parse_map(&mut self) -> Result<Expression, ParserError> {
        let left_brace = self.consume_simple(TokenKind::LeftBrace, "Expected '{'.")?;

        let mut entries = Vec::new();

        self.skip_newlines();

        if !self.check_simple(&TokenKind::RightBrace) {
            loop {
                let (key, _) = self.consume_identifier("Expected map key.")?;

                self.consume_simple(TokenKind::Colon, "Expected ':' after map key.")?;

                let value = self.parse_expression()?;

                entries.push((key, value));

                self.skip_newlines();

                if !self.match_simple(&TokenKind::Comma) {
                    break;
                }

                self.skip_newlines();
            }
        }

        let right_brace = self.consume_simple(TokenKind::RightBrace, "Expected '}' after map.")?;

        Ok(Expression {
            span: SourceSpan::new(left_brace.span.start, right_brace.span.end),
            kind: ExpressionKind::Map(entries),
        })
    }

    fn consume_identifier(&mut self, message: &str) -> Result<(String, SourceSpan), ParserError> {
        match self.peek_kind() {
            TokenKind::Identifier(name) => {
                let name = name.clone();
                let span = self.peek().span;

                self.advance();

                Ok((name, span))
            }

            _ => Err(self.error_current(message)),
        }
    }

    fn consume_simple(&mut self, expected: TokenKind, message: &str) -> Result<Token, ParserError> {
        if self.check_simple(&expected) {
            Ok(self.advance().clone())
        } else {
            Err(self.error_current(message))
        }
    }

    fn match_simple(&mut self, expected: &TokenKind) -> bool {
        self.match_simple_return(expected).is_some()
    }

    fn match_simple_return(&mut self, expected: &TokenKind) -> Option<Token> {
        if self.check_simple(expected) {
            Some(self.advance().clone())
        } else {
            None
        }
    }

    fn check_simple(&self, expected: &TokenKind) -> bool {
        std::mem::discriminant(self.peek_kind()) == std::mem::discriminant(expected)
    }

    fn peek_next_simple(&self, expected: &TokenKind) -> bool {
        if self.current + 1 >= self.tokens.len() {
            return false;
        }

        std::mem::discriminant(&self.tokens[self.current + 1].kind)
            == std::mem::discriminant(expected)
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    fn peek_kind(&self) -> &TokenKind {
        &self.peek().kind
    }

    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }

        &self.tokens[self.current.saturating_sub(1)]
    }

    fn is_at_end(&self) -> bool {
        matches!(self.peek_kind(), TokenKind::Eof)
    }

    fn skip_newlines(&mut self) {
        while self.check_simple(&TokenKind::Newline) {
            self.advance();
        }
    }

    fn consume_statement_end(&mut self) {
        if self.check_simple(&TokenKind::Newline) {
            self.skip_newlines();
        }
    }

    fn error_current(&self, message: impl Into<String>) -> ParserError {
        ParserError::new(message, self.peek())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scripting::ast::{
        BinaryOperator, Declaration, EntityMember, ExpressionKind, StatementKind, TypeKind,
    };
    use crate::scripting::lexer::Lexer;

    fn parse(source: &str) -> Program {
        let tokens = Lexer::new(source).tokenize().expect("lexer should succeed");

        Parser::new(tokens).parse().expect("parser should succeed")
    }

    #[test]
    fn parses_basic_entity() {
        let program = parse(
            r#"
entity Door {

    open: bool = false

    fn update(dt: number) {

        if open {
            print("open")
        }
    }
}
"#,
        );

        assert_eq!(program.declarations.len(), 1);

        let Declaration::Entity(entity) = &program.declarations[0] else {
            panic!("expected entity");
        };

        assert_eq!(entity.name, "Door");
        assert_eq!(entity.members.len(), 2);
        assert!(!entity.span.is_empty());
        assert!(!program.span.is_empty());
    }

    #[test]
    fn parses_function_parameters_and_return_type() {
        let program = parse(
            r#"
fn add(a: number, b: number): number {
    return a + b
}
"#,
        );

        let Declaration::Function(function) = &program.declarations[0] else {
            panic!("expected function");
        };

        assert_eq!(function.name, "add");
        assert_eq!(function.parameters.len(), 2);

        assert_eq!(
            function
                .return_type
                .as_ref()
                .map(|return_type| &return_type.kind),
            Some(&TypeKind::Named("number".to_string()))
        );

        assert!(
            function
                .parameters
                .iter()
                .all(|parameter| !parameter.span.is_empty())
        );
    }

    #[test]
    fn parses_expression_precedence() {
        let program = parse(
            r#"
entity Test {

    value: number = 1 + 2 * 3
}
"#,
        );

        let Declaration::Entity(entity) = &program.declarations[0] else {
            panic!("expected entity");
        };

        let EntityMember::Field(field) = &entity.members[0] else {
            panic!("expected field");
        };

        let Some(expression) = &field.initializer else {
            panic!("expected initializer");
        };

        assert!(matches!(
            expression.kind,
            ExpressionKind::Binary {
                operator: BinaryOperator::Add,
                ..
            }
        ));

        assert!(field.span.start <= expression.span.start);

        assert!(expression.span.start < expression.span.end);

        assert_eq!(field.span.end, expression.span.end);
    }

    #[test]
    fn parses_calls_and_member_access() {
        let program = parse(
            r#"
entity Test {

    fn update(dt: number) {
        character.jump()
        transform.move_forward(dt)
    }
}
"#,
        );

        let Declaration::Entity(entity) = &program.declarations[0] else {
            panic!("expected entity");
        };

        let EntityMember::Function(function) = &entity.members[0] else {
            panic!("expected function");
        };

        assert_eq!(function.body.statements.len(), 2);

        assert!(
            function
                .body
                .statements
                .iter()
                .all(|statement| { !statement.span.is_empty() })
        );
    }

    #[test]
    fn parses_for_loop() {
        let program = parse(
            r#"
entity Test {

    fn update(dt: number) {

        for enemy in enemies {
            enemy.highlight()
        }
    }
}
"#,
        );

        let Declaration::Entity(entity) = &program.declarations[0] else {
            panic!("expected entity");
        };

        let EntityMember::Function(function) = &entity.members[0] else {
            panic!("expected function");
        };

        assert!(matches!(
            function.body.statements[0].kind,
            StatementKind::For { .. }
        ));

        assert!(!function.body.statements[0].span.is_empty());
    }

    #[test]
    fn parses_import() {
        let program = parse(
            r#"
import Shared.Math
"#,
        );

        let Declaration::Import(import) = &program.declarations[0] else {
            panic!("expected import");
        };

        assert_eq!(import.path, vec!["Shared".to_string(), "Math".to_string()]);

        assert!(!import.span.is_empty());
    }

    #[test]
    fn parser_errors_keep_source_spans() {
        let source = "entity Test {";
        let tokens = Lexer::new(source).tokenize().expect("lexer should succeed");

        let error = Parser::new(tokens).parse().expect_err("parser should fail");

        assert_eq!(error.line, 1);
        assert!(error.column > 0);
        assert!(error.span.start <= error.span.end);
    }

    #[test]
    fn optional_types_keep_their_full_span() {
        let source = r#"
fn find_player(): Entity? {
    return null
}
"#;

        let program = parse(source);

        let Declaration::Function(function) = &program.declarations[0] else {
            panic!("expected function");
        };

        let return_type = function.return_type.as_ref().expect("return type");

        assert!(matches!(return_type.kind, TypeKind::Optional(_)));

        assert_eq!(return_type.span.len(), "Entity?".len() as u32);
    }

    #[test]
    fn parses_mutable_local_variable() {
        let program = parse(
            r#"
fn test() {
    value: number = 10
    value += 5
}
"#,
        );

        let Declaration::Function(function) = &program.declarations[0] else {
            panic!("expected function");
        };

        let StatementKind::Variable {
            name,
            type_annotation,
            initializer,
            is_const,
        } = &function.body.statements[0].kind
        else {
            panic!("expected variable declaration");
        };

        assert_eq!(name, "value");
        assert!(type_annotation.is_some());
        assert!(initializer.is_some());
        assert!(!is_const);
    }

    #[test]
    fn parses_const_local_variable() {
        let program = parse(
            r#"
fn test() {
    const limit: number = 100
}
"#,
        );

        let Declaration::Function(function) = &program.declarations[0] else {
            panic!("expected function");
        };

        let StatementKind::Variable { name, is_const, .. } = &function.body.statements[0].kind
        else {
            panic!("expected variable declaration");
        };

        assert_eq!(name, "limit");
        assert!(is_const);
    }

    #[test]
    fn parses_light_style_script() {
        let program = parse(
            r#"
entity LightBlinker {

    light: Light?

    fn update(dt: number) {
        light = getLightFromPos(10, 5, 3)

        if light {
            light.on()
        }
    }
}
"#,
        );

        let Declaration::Entity(entity) = &program.declarations[0] else {
            panic!("expected entity");
        };

        assert_eq!(entity.members.len(), 2);

        let EntityMember::Field(field) = &entity.members[0] else {
            panic!("expected field");
        };

        assert!(matches!(
            field
                .type_annotation
                .as_ref()
                .map(|type_annotation| &type_annotation.kind),
            Some(TypeKind::Optional(_))
        ));

        let EntityMember::Function(function) = &entity.members[1] else {
            panic!("expected function");
        };

        assert_eq!(function.body.statements.len(), 2);
    }
}
