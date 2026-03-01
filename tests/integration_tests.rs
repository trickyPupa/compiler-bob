use compiler::lexer::Lexer;
use compiler::token::TokenType;

#[test]
fn lexer_types_test() {
    let code_example = "var x = 123;
    print x + 5;";

    let required_types = vec![
        TokenType::ID,
        TokenType::ID,
        TokenType::EQ,
        TokenType::NUMBER,
        TokenType::SEMICOLON,
        TokenType::PRINT,
        TokenType::ID,
        TokenType::PLUS,
        TokenType::NUMBER,
        TokenType::SEMICOLON
    ];

    let lexer = Lexer::new(code_example);

    for (i, token) in lexer.enumerate() {
        // println!("{:?}", i);
        assert_eq!(required_types[i], token.ttype);
    }
}