#[cfg(test)]
mod tests {
    use super::super::*;
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
    return nil
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
        light = get_light(10, 5, 3)

        if light {
            light:set_enabled(true)
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

    #[test]
    fn test_top_level_statements() {
        let source = r#"
debug.log("hello")
const x = 10
if x > 0 {
    debug.log("positive")
}
"#;
        let program = parse(source);
        assert_eq!(program.statements.len(), 3);
        assert_eq!(program.declarations.len(), 0);

        match &program.statements[0].kind {
            StatementKind::Expression(expr) => {
                assert!(matches!(expr.kind, ExpressionKind::Call { .. }));
            }
            _ => panic!("expected expression statement"),
        }
    }

    #[test]
    fn test_mixed_top_level() {
        let source = r#"
debug.log("start")
fn helper() {}
debug.log("end")
entity Test {}
"#;
        let program = parse(source);
        assert_eq!(program.statements.len(), 2);
        assert_eq!(program.declarations.len(), 2);
    }
}
