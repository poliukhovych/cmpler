use cmpler_core::parser::Parser;
use cmpler_core::parser::ParseError;
use cmpler_core::lexer::lex;
use cmpler_core::ast::{Decl, Stmt, Expr};
use cmpler_core::lexer::TokenKind;

#[test]
fn parse_simple_function() {
    let src = "int main() { return 42; }";
    let mut parser = Parser::new(lex(src));
    let program = parser.parse_program().expect("Failed to parse function");
    assert_eq!(program.decls.len(), 1);
    match &program.decls[0] {
        Decl::Function { name, params, body, .. } => {
            assert_eq!(name, "main");
            assert!(params.is_empty());
            assert_eq!(body.len(), 1);
            match &body[0] {
                Stmt::Return(expr) => match **expr {
                    Expr::IntLiteral { value, .. } => assert_eq!(value, 42),
                    _ => panic!("Expected integer literal in return"),
                },
                _ => panic!("Expected return statement"),
            }
        }
        _ => panic!("Expected a function declaration"),
    }
}

#[test]
fn parse_variable_declaration() {
    let src = "int x = 5;";
    let mut parser = Parser::new(lex(src));
    let program = parser.parse_program().expect("Failed to parse var decl");
    assert_eq!(program.decls.len(), 1);
    match &program.decls[0] {
        Decl::Var { name, init, .. } => {
            assert_eq!(name, "x");
            match **init {
                Expr::IntLiteral { value, .. } => assert_eq!(value, 5),
                _ => panic!("Expected integer literal as initializer"),
            }
        }
        _ => panic!("Expected a variable declaration"),
    }
}

#[test]
fn parse_if_else_statement() {
    let src = "int main() { if (1) { return 1; } else { return 0; } }";
    let mut parser = Parser::new(lex(src));
    let program = parser.parse_program().expect("Failed to parse if-else");
    assert_eq!(program.decls.len(), 1);
    if let Decl::Function { body, .. } = &program.decls[0] {
        assert_eq!(body.len(), 1);
        if let Stmt::If { cond, then_block, else_block } = &body[0] {
            match **cond {
                Expr::IntLiteral { value, .. } => assert_eq!(value, 1),
                _ => panic!("Expected int literal in condition"),
            }
            assert_eq!(then_block.len(), 1);
            let else_blk = else_block.as_ref().expect("Expected else block");
            assert_eq!(else_blk.len(), 1);
        } else {
            panic!("Expected if statement");
        }
    } else {
        panic!("Expected function declaration");
    }
}

#[test]
fn parse_while_loop() {
    let src = "int main() { while (0) { x = x + 1; } }";
    let mut parser = Parser::new(lex(src));
    let program = parser.parse_program().expect("Failed to parse while");
    assert_eq!(program.decls.len(), 1);
    if let Decl::Function { body, .. } = &program.decls[0] {
        assert_eq!(body.len(), 1);
        if let Stmt::While(cond, stmts) = &body[0] {
            match **cond {
                Expr::IntLiteral { value, .. } => assert_eq!(value, 0),
                _ => panic!("Expected int literal in while condition"),
            }
            assert_eq!(stmts.len(), 1);
        } else {
            panic!("Expected while statement");
        }
    }
}

#[test]
fn parse_for_loop() {
    let src = "int main() { for (i=0; i<10; i=i+1) { ; } }";
    let mut parser = Parser::new(lex(src));
    let program = parser.parse_program().expect("Failed to parse for");
    assert_eq!(program.decls.len(), 1);
    if let Decl::Function { body, .. } = &program.decls[0] {
        assert_eq!(body.len(), 1);
        if let Stmt::For { init, cond, inc, body: stmts } = &body[0] {
            assert!(init.is_some());
            assert!(cond.is_some());
            assert!(inc.is_some());
            assert_eq!(stmts.len(), 1);
        } else {
            panic!("Expected for statement");
        }
    }
}

#[test]
fn parse_expression_precedence() {
    let src = "int main() { return 1 + 2 * 3; }";
    let mut parser = Parser::new(lex(src));
    let program = parser.parse_program().expect("Failed to parse expr");
    if let Decl::Function { body, .. } = &program.decls[0] {
        if let Stmt::Return(expr) = &body[0] {
            if let Expr::Binary { op, left, right, .. } = &**expr {
                assert_eq!(*op, TokenKind::Plus);
                if let Expr::IntLiteral { value: l, .. } = **left {
                    assert_eq!(l, 1);
                } else { panic!("Left side not literal"); }
                if let Expr::Binary { op: inner_op, left: il, right: ir, .. } = &**right {
                    assert_eq!(*inner_op, TokenKind::Star);
                    if let Expr::IntLiteral { value: v, .. } = **il {
                        assert_eq!(v, 2);
                    }
                    if let Expr::IntLiteral { value: v, .. } = **ir {
                        assert_eq!(v, 3);
                    }
                } else { panic!("Right side not binary"); }
            } else {
                panic!("Expected binary expression");
            }
        }
    }
}

#[test]
fn parse_return_without_semicolon_error() {
    let src = "int main() { return 1 }";
    let mut parser = Parser::new(lex(src));
    let err = parser.parse_program().unwrap_err();
    match err {
        ParseError::Expected { expected, found, span: _ } => {
            assert!(expected.contains("Semicolon"));
            assert_eq!(found, TokenKind::RBrace);
        }
        _ => panic!("Expected semicolon error"),
    }
}
