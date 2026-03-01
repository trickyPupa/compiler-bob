use std::{collections::HashSet, iter};
use std::iter::repeat_with;

pub fn generate_random_program() -> String {
    let mut result = String::new();

    let mut used_vars: HashSet<String> = HashSet::new();

    for _ in 0..fastrand::usize(1..20) {
        result.push_str(&generate_line(&mut used_vars));
    }

    result
}

fn generate_block(used_vars: &mut HashSet<String>, indent_level: usize) -> String {
    let indent = " ".repeat(indent_level * 4);
    let mut block = String::new();

    for i in 1..4 {

        // Выбираем тип следующей инструкции
        // 0: Объявление переменной (var x = ...)
        // 1: Присваивание (x = ...)
        // 2: Print (print ...)
        // 3: If-Else
        // 4: While

        // ограничение вложенности - 3
        let statement_type = if indent_level <= 2 { fastrand::i32(0..5) } else { fastrand::i32(0..3) };
        
        // test
        let statement_type = 0;

        match statement_type {
            0 => {
                let line = generate_line(used_vars);
                block.push_str(&indent);
                block.push_str(&line);
            },
            1 => {

            },
            2 => {

            },
            3 => {

            },
            4 => {

            },
            _ => panic!(),
        }
        block.push('\n');
    }

    block
}

fn generate_line(used_vars: &mut HashSet<String>) -> String {
    let mut line = String::new();

    for _ in 0..fastrand::usize(3..10) {
        let mut var: String;
        loop {
            var = generate_var(None);
            if !used_vars.contains(&var) {
                break;
            }
        }
        used_vars.insert(var.clone());
        line.push_str(&var);
        line.push(' ');
    }
    line.push(';');
    line
}

fn generate_var(length: Option<usize>) -> String {
    iter::once(fastrand::alphabetic())
        .chain(repeat_with(fastrand::alphanumeric)
        .take(length.unwrap_or(10)))
        .collect()
}

fn generate_number() -> String {
    fastrand::i32(..).to_string()
}
