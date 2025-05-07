use logos::Logos;
use crate::lexer::token::{TokenKind, Token};
use crate::utils::span::Span;

pub fn lex(source: &str) -> Vec<Token> {
    let mut lexer = TokenKind::lexer(source);
    let mut tokens = Vec::new();

    while let Some(result) = lexer.next() {
        let span = Span {
            start: lexer.span().start,
            end:   lexer.span().end,
        };

        let text = &source[span.start..span.end];

        let kind = result.unwrap_or(TokenKind::Error);

        tokens.push(Token {
            kind,
            span,
            text: text.to_string(),
        });
    }

    tokens
}
