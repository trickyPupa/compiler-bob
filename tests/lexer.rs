use compiler::lexer::Lexer;
use compiler::token::TokenType;

#[test]
fn lexer_types_test() {
    let code_example = "var x = 123;
    print x + 5;";

    let required_types = vec![
        TokenType::VAR,
        TokenType::ID,
        TokenType::EQ,
        TokenType::NUMBER,
        TokenType::SEMICOLON,
        TokenType::PRINT,
        TokenType::ID,
        TokenType::PLUS,
        TokenType::NUMBER,
        TokenType::SEMICOLON,
        TokenType::EOF,
    ];

    let lexer = Lexer::new(code_example);

    for (i, token) in lexer.enumerate() {
        assert_eq!(required_types[i], token.ttype);
    }
}

#[test]
fn lexer_array_tokens() {
    let code_example = "var xs = [1, 2];";

    let required_types = vec![
        TokenType::VAR,
        TokenType::ID,
        TokenType::EQ,
        TokenType::LBRACKET,
        TokenType::NUMBER,
        TokenType::COMMA,
        TokenType::NUMBER,
        TokenType::RBRACKET,
        TokenType::SEMICOLON,
        TokenType::EOF,
    ];

    let lexer = Lexer::new(code_example);

    for (i, token) in lexer.enumerate() {
        assert_eq!(required_types[i], token.ttype);
    }
}
