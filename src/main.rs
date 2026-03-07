use compiler::code_generator;
use compiler::lexer::Lexer;
use compiler::parser::Parser;

fn main() {
    lab1();
}

fn lab1() {
    let _code_example = "var x = 123;
print x + 5;";

    let code_example = code_generator::generate_random_program();
    println!("{}", code_example);

    let lexer = Lexer::new(&code_example);

    for i in lexer {
        println!("{:?}", i);
    }
}

fn _lab2() {
    let code_example = code_generator::generate_random_program();
    println!("{}", code_example);

    let lexer = Lexer::new(&code_example);

    let parser = Parser::new(lexer);

    for i in parser {
        println!("{:?}", i);
    }
}
