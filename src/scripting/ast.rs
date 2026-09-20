use super::source::SourceSpan;

#[derive(Clone, Debug, PartialEq)]
pub struct Program {
    pub span: SourceSpan,
    pub declarations: Vec<Declaration>,
    pub statements: Vec<Statement>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Declaration {
    Entity(EntityDecl),
    Function(FunctionDecl),
    Import(ImportDecl),
    Event(EventDecl),
}

impl Declaration {
    pub fn span(&self) -> SourceSpan {
        match self {
            Self::Entity(entity) => entity.span,
            Self::Function(function) => function.span,
            Self::Import(import) => import.span,
            Self::Event(event) => event.span,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct EntityDecl {
    pub span: SourceSpan,
    pub name: String,
    pub members: Vec<EntityMember>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum EntityMember {
    Field(FieldDecl),
    Function(FunctionDecl),
}

impl EntityMember {
    pub fn span(&self) -> SourceSpan {
        match self {
            Self::Field(field) => field.span,
            Self::Function(function) => function.span,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct FieldDecl {
    pub span: SourceSpan,
    pub name: String,
    pub type_annotation: Option<Type>,
    pub initializer: Option<Expression>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct EventDecl {
    pub span: SourceSpan,
    pub name: String,
    pub parameters: Vec<Parameter>,
    pub body: Block,
}

#[derive(Clone, Debug, PartialEq)]
pub struct FunctionDecl {
    pub span: SourceSpan,
    pub name: String,
    pub parameters: Vec<Parameter>,
    pub return_type: Option<Type>,
    pub body: Block,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Parameter {
    pub span: SourceSpan,
    pub name: String,
    pub type_annotation: Option<Type>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ImportDecl {
    pub span: SourceSpan,
    pub path: Vec<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Type {
    pub span: SourceSpan,
    pub kind: TypeKind,
}

#[derive(Clone, Debug, PartialEq)]
pub enum TypeKind {
    Named(String),
    Optional(Box<Type>),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Block {
    pub span: SourceSpan,
    pub statements: Vec<Statement>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Statement {
    pub span: SourceSpan,
    pub kind: StatementKind,
}

#[derive(Clone, Debug, PartialEq)]
pub enum StatementKind {
    Variable {
        name: String,
        type_annotation: Option<Type>,
        initializer: Option<Expression>,
        is_const: bool,
    },

    Assignment {
        target: Expression,
        operator: AssignmentOperator,
        value: Expression,
    },

    If {
        condition: Expression,
        then_block: Block,
        else_if: Vec<(Expression, Block)>,
        else_block: Option<Block>,
    },

    While {
        condition: Expression,
        body: Block,
    },

    For {
        name: String,
        iterable: Expression,
        body: Block,
    },

    Return(Option<Expression>),

    Expression(Expression),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AssignmentOperator {
    Assign,
    Add,
    Subtract,
    Multiply,
    Divide,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Expression {
    pub span: SourceSpan,
    pub kind: ExpressionKind,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ExpressionKind {
    Number(f64),
    String(String),
    Bool(bool),
    Nil,

    Identifier(String),

    Unary {
        operator: UnaryOperator,
        expression: Box<Expression>,
    },

    Binary {
        left: Box<Expression>,
        operator: BinaryOperator,
        right: Box<Expression>,
    },

    Call {
        callee: Box<Expression>,
        arguments: Vec<Expression>,
    },

    MethodCall {
        object: Box<Expression>,
        method: String,
        arguments: Vec<Expression>,
    },

    Member {
        object: Box<Expression>,
        name: String,
    },

    Index {
        object: Box<Expression>,
        index: Box<Expression>,
    },

    Array(Vec<Expression>),

    Map(Vec<(Expression, Expression)>),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UnaryOperator {
    Negate,
    Not,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BinaryOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,

    Equal,
    NotEqual,

    Less,
    LessEqual,
    Greater,
    GreaterEqual,

    And,
    Or,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scripting::source::SourceSpan;

    fn span(start: u32, end: u32) -> SourceSpan {
        SourceSpan::new(start, end)
    }

    #[test]
    fn declarations_expose_their_spans() {
        let entity = EntityDecl {
            span: span(0, 20),
            name: "Test".to_string(),
            members: Vec::new(),
        };

        let declaration = Declaration::Entity(entity);

        assert_eq!(declaration.span(), span(0, 20));
    }

    #[test]
    fn expressions_store_only_one_compact_span() {
        let expression = Expression {
            span: span(4, 10),
            kind: ExpressionKind::Number(42.0),
        };

        assert_eq!(expression.span, span(4, 10));
    }

    #[test]
    fn variable_statement_can_be_const() {
        let statement = Statement {
            span: span(0, 20),
            kind: StatementKind::Variable {
                name: "limit".to_string(),
                type_annotation: None,
                initializer: None,
                is_const: true,
            },
        };

        let StatementKind::Variable { is_const, .. } = statement.kind else {
            panic!("expected variable");
        };

        assert!(is_const);
    }
}
