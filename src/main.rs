use compiler::{interpreter::RuntimeInterpreter, parser::Parser};
use compiler::{code_generator, token::Token};
use compiler::{lexer::Lexer, semantic::analyzer::Analyzer};

fn main() {
    _interpreter();
    // let code_example = code_generator::generate_random_program(5);
    // println!("{}", code_example);
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

fn _lab3() {
    // let code_example = code_generator::generate_random_program();
    let code_example = "var y = 123;
if (x == 123) {
    print f + 5;
    x = count + 1;
}";
    println!("code example:\n\n{}\n", code_example);

    let lexer = Lexer::new(code_example);
    let tokens: Vec<Token> = Vec::from_iter(lexer);

    let parser = Parser::new(tokens.into_iter());

    let mut semantic = Analyzer::new(parser);
    semantic.analyze();

    for i in semantic.errors() {
        println!("{}", i);
    }
}

fn _interpreter() {
        let code_example = "var y = 123;
if (y == 123) {
    print y + 5;
    var x = y + 1;
}
y = y * 2;
print y;
y = y + 123;
print y;";

    let lexer = Lexer::new(code_example);
    let tokens: Vec<Token> = Vec::from_iter(lexer);

    let parser = Parser::new(tokens.into_iter());

    // let mut semantic = Analyzer::new(parser);
    // semantic.analyze();

    let mut runtime = RuntimeInterpreter::new(parser);
    let _ = runtime.execute_program();

    for i in runtime.output() {
        println!("{i}");
    }
}