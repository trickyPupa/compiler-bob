use compiler::code_generator;
use compiler::lexer::Lexer;

fn main() {
    lab1();
}

fn lab1() {
    let code_example = "var x = 123;
print x + 5;";

    let code_example = code_generator::generate_random_program();
    println!("{}", code_example);

    let lexer = Lexer::new(&code_example);

    for i in lexer {
        println!("{:?}", i);
    }
}
