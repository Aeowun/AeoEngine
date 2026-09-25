pub mod parser_declarations;
pub mod parser_expressions;
pub mod parser_statements;
pub mod parser_tests;

use crate::scripting::ast::*;
use crate::scripting::source::SourceSpan;
use crate::scripting::token::{Token, TokenKind};

#[derive(Clone, Debug, PartialEq)]
pub struct ParserError {
    pub message: String,
    pub span: SourceSpan,
    pub line: usize,
    pub column: usize,
}

impl ParserError {
    pub(super) fn new(message: impl Into<String>, token: &Token) -> Self {
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
        let mut statements = Vec::new();

        self.skip_newlines();

        while !self.is_at_end() {
            match self.peek_kind() {
                TokenKind::Entity | TokenKind::Fn | TokenKind::Import | TokenKind::On => {
                    declarations.push(self.parse_declaration()?);
                }
                _ => {
                    statements.push(self.parse_statement()?);
                }
            }
            self.skip_newlines();
        }

        let mut start = self.tokens.first().map(|t| t.span.start).unwrap_or(0);
        let mut end = self.tokens.last().map(|t| t.span.end).unwrap_or(0);

        if let Some(first) = declarations.first() {
            start = start.min(first.span().start);
            end = end.max(declarations.last().unwrap().span().end);
        }

        if let Some(first) = statements.first() {
            start = start.min(first.span.start);
            end = end.max(statements.last().unwrap().span.end);
        }

        Ok(Program {
            span: SourceSpan::new(start, end),
            declarations,
            statements,
        })
    }

    pub(super) fn consume_identifier(&mut self, message: &str) -> Result<(String, SourceSpan), ParserError> {
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

    pub(super) fn consume_simple(&mut self, expected: TokenKind, message: &str) -> Result<Token, ParserError> {
        if self.check_simple(&expected) {
            Ok(self.advance().clone())
        } else {
            Err(self.error_current(message))
        }
    }

    pub(super) fn match_simple(&mut self, expected: &TokenKind) -> bool {
        self.match_simple_return(expected).is_some()
    }

    pub(super) fn match_simple_return(&mut self, expected: &TokenKind) -> Option<Token> {
        if self.check_simple(expected) {
            Some(self.advance().clone())
        } else {
            None
        }
    }

    pub(super) fn check_simple(&self, expected: &TokenKind) -> bool {
        std::mem::discriminant(self.peek_kind()) == std::mem::discriminant(expected)
    }

    pub(super) fn peek_next_simple(&self, expected: &TokenKind) -> bool {
        if self.current + 1 >= self.tokens.len() {
            return false;
        }

        std::mem::discriminant(&self.tokens[self.current + 1].kind)
            == std::mem::discriminant(expected)
    }

    pub(super) fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    pub(super) fn peek_kind(&self) -> &TokenKind {
        &self.peek().kind
    }

    pub(super) fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }

        &self.tokens[self.current.saturating_sub(1)]
    }

    pub(super) fn is_at_end(&self) -> bool {
        matches!(self.peek_kind(), TokenKind::Eof)
    }

    pub(super) fn is_method_call_peek(&self) -> bool {
        if self.current + 3 >= self.tokens.len() {
            return false;
        }

        matches!(self.tokens[self.current].kind, TokenKind::Identifier(_))
            && matches!(self.tokens[self.current + 1].kind, TokenKind::Colon)
            && matches!(self.tokens[self.current + 2].kind, TokenKind::Identifier(_))
            && matches!(self.tokens[self.current + 3].kind, TokenKind::LeftParen)
    }

    pub(super) fn skip_newlines(&mut self) {
        while self.check_simple(&TokenKind::Newline) {
            self.advance();
        }
    }

    pub(super) fn consume_statement_end(&mut self) {
        if self.check_simple(&TokenKind::Newline) {
            self.skip_newlines();
        }
    }

    pub(super) fn error_current(&self, message: impl Into<String>) -> ParserError {
        ParserError::new(message, self.peek())
    }
}
