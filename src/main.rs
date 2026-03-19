use compiler::lexer::Lexer;
use compiler::parser::Parser;
use compiler::{code_generator, token::Token};

fn main() {
    _lab2();
}

fn _lab1() {
    let _code_example = "var x = 123;
print x + 5;";

    let code_example = code_generator::generate_random_program(3);
    println!("code example:\n\n{}\n", code_example);

    let lexer = Lexer::new(&code_example);

    for i in lexer {
        println!("{:?}", i);
    }
}

fn _lab2() {
    // let code_example = code_generator::generate_random_program();
    let code_example = "var x = 123;
if (x == 123) {
    print x + 5;
    x = x + 1;
}";
    println!("code example:\n\n{}\n", code_example);

    let lexer = Lexer::new(code_example);
    let tokens: Vec<Token> = Vec::from_iter(lexer);

    let parser = Parser::new(tokens.into_iter());

    for i in parser {
        println!("{:?}", i);
    }
}
