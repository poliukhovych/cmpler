use cmpler_core::lexer::{lex, TokenKind};

#[test]
fn test_keywords() {
    let input = "int if else return while";
    let kinds: Vec<_> = lex(input).into_iter().map(|t| t.kind).collect();
    assert_eq!(
        kinds,
        vec![
            TokenKind::Int,
            TokenKind::If,
            TokenKind::Else,
            TokenKind::Return,
            TokenKind::While,
        ]
    );
}

#[test]
fn test_identifiers_and_literals() {
    let input = "x y1 _z 123 0";
    let kinds: Vec<_> = lex(input).into_iter().map(|t| t.kind).collect();
    assert_eq!(
        kinds,
        vec![
            TokenKind::Identifier,
            TokenKind::Identifier,
            TokenKind::Identifier,
            TokenKind::IntegerLiteral,
            TokenKind::IntegerLiteral,
        ]
    );
}

#[test]
fn test_operators_and_punctuation() {
    let input = "+ - * / = == != < <= > >= ; , ( ) { }";
    let kinds: Vec<_> = lex(input).into_iter().map(|t| t.kind).collect();
    assert_eq!(
        kinds,
        vec![
            TokenKind::Plus,
            TokenKind::Minus,
            TokenKind::Star,
            TokenKind::Slash,
            TokenKind::Assign,
            TokenKind::Equal,
            TokenKind::NotEqual,
            TokenKind::Less,
            TokenKind::LessEqual,
            TokenKind::Greater,
            TokenKind::GreaterEqual,
            TokenKind::Semicolon,
            TokenKind::Comma,
            TokenKind::LParen,
            TokenKind::RParen,
            TokenKind::LBrace,
            TokenKind::RBrace,
        ]
    );
}

#[test]
fn test_mixed_spacing() {
    let input = "int   x= 42 ;";
    let kinds: Vec<_> = lex(input).into_iter().map(|t| t.kind).collect();
    assert_eq!(
        kinds,
        vec![
            TokenKind::Int,
            TokenKind::Identifier,
            TokenKind::Assign,
            TokenKind::IntegerLiteral,
            TokenKind::Semicolon,
        ]
    );
}

#[test]
fn test_lexical_error_token() {
    let input = "int $foo = 10;";
    let tokens = lex(input);

    let error_token = tokens.iter().find(|t| matches!(t.kind, TokenKind::Error));
    assert!(error_token.is_some(), "Expected a TokenKind::Error for invalid token");
}

#[test]
fn test_full_smallc_lexing() {
    let input = r#"
        int main() {
            int x = 10;
            if (x > 5) {
                return x + 1;
            } else {
                return x - 1;
            }
        }
    "#;

    let kinds: Vec<_> = lex(input).into_iter().map(|t| t.kind).collect();

    assert_eq!(
        kinds,
        vec![
            TokenKind::Int,
            TokenKind::Identifier, // main
            TokenKind::LParen,
            TokenKind::RParen,
            TokenKind::LBrace,
            TokenKind::Int,
            TokenKind::Identifier,
            TokenKind::Assign,
            TokenKind::IntegerLiteral,
            TokenKind::Semicolon,
            TokenKind::If,
            TokenKind::LParen,
            TokenKind::Identifier,
            TokenKind::Greater,
            TokenKind::IntegerLiteral,
            TokenKind::RParen,
            TokenKind::LBrace,
            TokenKind::Return,
            TokenKind::Identifier,
            TokenKind::Plus,
            TokenKind::IntegerLiteral,
            TokenKind::Semicolon,
            TokenKind::RBrace,
            TokenKind::Else,
            TokenKind::LBrace,
            TokenKind::Return,
            TokenKind::Identifier,
            TokenKind::Minus,
            TokenKind::IntegerLiteral,
            TokenKind::Semicolon,
            TokenKind::RBrace,
            TokenKind::RBrace,
        ]
    );
}
