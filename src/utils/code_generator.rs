use std::iter::repeat_with;
use std::{collections::HashSet, iter};

const MATH_OPERATORS: &[&str] = &["+", "-", "*", "/"];
const COMPARE_OPERATORS: &[&str] = &["==", "!=", "<", ">", "<=", ">="];
const LOGIC_OPERATORS: &[&str] = &["&&", "||"];

pub fn generate_random_program() -> String {
    let mut result = String::new();

    let mut used_vars: HashSet<String> = HashSet::new();

    for _ in 0..fastrand::usize(1..5) {
        result.push_str(&generate_block(&mut used_vars, 0));
    }
    result.push('\n');

    result
}

fn generate_block(used_vars: &mut HashSet<String>, indent_level: usize) -> String {
    let indent = " ".repeat(indent_level * 4);
    let mut block = String::new();

    for _ in 1..3 {
        // Выбираем тип следующей инструкции
        // 0: Объявление переменной (var x = ...)
        // 1: Присваивание (x = ...)
        // 2: Print (print ...)
        // 3: If-Else
        // 4: While

        // ограничение вложенности - 3
        let statement_type = if indent_level <= 2 {
            fastrand::i32(0..5)
        } else {
            fastrand::i32(0..3)
        };

        match statement_type {
            0 => {
                let line = generate_var_declaration(used_vars);
                block.push_str(&indent);
                block.push_str(&line);
            }
            1 => {
                let line = if !used_vars.is_empty() {
                    format!(
                        "{} = {};",
                        get_random_var(used_vars).unwrap(),
                        generate_expression(used_vars)
                    )
                } else {
                    generate_var_declaration(used_vars)
                };
                block.push_str(&indent);
                block.push_str(&line);
            }
            2 => {
                let line = format!("{indent}print {}", generate_expression(used_vars));
                block.push_str(&line);
            }
            3 => {
                let line = format!("{indent}if ({}) {{\n", generate_condition(used_vars));
                block.push_str(&line);
                block.push_str(&generate_block(used_vars, indent_level + 1));

                if fastrand::f32() > 0.5 {
                    block.push_str(&format!("{indent}}} else {{\n"));
                    block.push_str(&generate_block(used_vars, indent_level + 1));
                }

                block.push_str(&format!("{indent}}}"));
            }
            4 => {
                let line = format!("{indent}while ({}) {{\n", generate_condition(used_vars));
                block.push_str(&line);
                block.push_str(&generate_block(used_vars, indent_level + 1));
                block.push_str(&format!("{indent}}}"));
            }
            _ => panic!(),
        }
        block.push('\n');
    }

    block
}

fn generate_line(used_vars: &mut HashSet<String>) -> String {
    let mut line = String::new();

    for _ in 0..fastrand::usize(2..5) {
        let var: String = generate_var(used_vars, None);
        used_vars.insert(var.clone());
        line.push_str(&var);
        line.push(' ');
    }
    line.push(';');
    line.push('\n');
    line
}

fn generate_expression(used_vars: &mut HashSet<String>) -> String {
    if fastrand::f32() > 0.6 || used_vars.is_empty() {
        return fastrand::i32(-100..100).to_string();
    }
    if fastrand::f32() > 0.5 {
        return get_random_var(used_vars).unwrap();
    }

    let left = if fastrand::f32() > 0.5 {
        get_random_var(used_vars).unwrap()
    } else {
        fastrand::i32(-100..100).to_string()
    };

    let right = if fastrand::f32() > 0.5 {
        get_random_var(used_vars).unwrap()
    } else {
        fastrand::i32(-100..100).to_string()
    };

    let op = *fastrand::choice(MATH_OPERATORS).unwrap();

    format!("{left} {op} {right}")
}

fn generate_condition(used_vars: &mut HashSet<String>) -> String {
    let left = generate_random_var_or_number(used_vars);
    let right = generate_random_var_or_number(used_vars);
    let op = *fastrand::choice(COMPARE_OPERATORS).unwrap();

    let res = format!("{left} {op} {right}");

    if fastrand::f32() > 0.7 {
        let logic_op = *fastrand::choice(LOGIC_OPERATORS).unwrap();
        let left2 = generate_random_var_or_number(used_vars);
        let right2 = generate_random_var_or_number(used_vars);
        let op2 = *fastrand::choice(COMPARE_OPERATORS).unwrap();

        return format!("({res}) {logic_op} ({left2} {op2} {right2})");
    }

    res
}

fn generate_var_declaration(used_vars: &mut HashSet<String>) -> String {
    format!(
        "var {} = {};",
        generate_var(used_vars, None),
        generate_expression(used_vars)
    )
}

fn generate_var(used_vars: &mut HashSet<String>, length: Option<usize>) -> String {
    let mut var: String;
    loop {
        var = generate_word(length);
        if !used_vars.contains(&var) {
            break;
        }
    }
    used_vars.insert(var.clone());
    var
}

fn generate_word(length: Option<usize>) -> String {
    iter::once(fastrand::alphabetic())
        .chain(repeat_with(fastrand::alphanumeric).take(length.unwrap_or(5)))
        .collect()
}

fn get_random_var(used_vars: &mut HashSet<String>) -> Option<String> {
    fastrand::choice(used_vars.iter()).cloned()
}

fn generate_random_var_or_number(used_vars: &mut HashSet<String>) -> String {
    if !used_vars.is_empty() && fastrand::f32() > 0.5 {
        get_random_var(used_vars).unwrap_or(fastrand::i32(..).to_string())
    } else {
        fastrand::i32(..).to_string()
    }
}
