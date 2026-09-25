use crate::scripting::ast::*;
use crate::scripting::source::SourceSpan;
use crate::scripting::token::TokenKind;

use super::{Parser, ParserError};

impl Parser {
    pub(super) fn parse_expression(&mut self) -> Result<Expression, ParserError> {
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

            if self.match_simple(&TokenKind::Colon) {
                let (method, _method_span) =
                    self.consume_identifier("Expected method name after ':'.")?;

                self.consume_simple(TokenKind::LeftParen, "Expected '(' after method name.")?;

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
                    kind: ExpressionKind::MethodCall {
                        object: Box::new(expression),
                        method,
                        arguments,
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

            TokenKind::Nil => {
                self.advance();

                Ok(Expression {
                    span: token.span,
                    kind: ExpressionKind::Nil,
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

            TokenKind::Fn => self.parse_anonymous_function(),

            _ => Err(ParserError::new("Expected an expression.", &token)),
        }
    }

    fn parse_anonymous_function(&mut self) -> Result<Expression, ParserError> {
        let fn_token = self.consume_simple(TokenKind::Fn, "Expected 'fn'.")?;
        let start = fn_token.span.start;

        self.consume_simple(TokenKind::LeftParen, "Expected '(' after 'fn'.")?;

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

        Ok(Expression {
            span: SourceSpan::new(start, body.span.end),
            kind: ExpressionKind::AnonymousFunction {
                parameters,
                return_type,
                body,
            },
        })
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
                let key_token = self.peek().clone();
                let key = match key_token.kind {
                    TokenKind::String(s) => {
                        self.advance();
                        Expression {
                            span: key_token.span,
                            kind: ExpressionKind::String(s),
                        }
                    }
                    TokenKind::Number(n) => {
                        self.advance();
                        Expression {
                            span: key_token.span,
                            kind: ExpressionKind::Number(n),
                        }
                    }
                    TokenKind::Identifier(s) => {
                        self.advance();
                        Expression {
                            span: key_token.span,
                            kind: ExpressionKind::String(s),
                        }
                    }
                    _ => {
                        return Err(ParserError::new(
                            "Expected map key (string, number, or identifier).",
                            &key_token,
                        ));
                    }
                };

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
}
