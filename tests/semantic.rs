use compiler::lexer::Lexer;
use compiler::parser::Parser;
use compiler::semantic::analyzer::Analyzer;

fn analyze_source(source: &str) -> Vec<String> {
    let lexer = Lexer::new(source);
    let parser = Parser::new(lexer);
    let mut analyzer = Analyzer::new(parser);
    analyzer.analyze();
    analyzer.errors().to_vec()
}

#[test]
fn detects_undeclared_variable_usage() {
    let errors = analyze_source("print x;");

    assert!(
        errors
            .iter()
            .any(|e| e.contains("Undeclared variable used x"))
    );
}

#[test]
fn detects_uninitialized_variable_usage() {
    let errors = analyze_source("var x; print x;");

    assert!(
        errors
            .iter()
            .any(|e| e.contains("Uninitialized variable used x"))
    );
}

#[test]
fn reports_duplicate_definition_with_position() {
    let source = "var x = 1;\nvar x = 2;\n";
    let errors = analyze_source(source);

    assert!(
        errors.iter().any(|e| {
            e.contains("[Line 2, Col 1]") && e.contains("already defined in this scope")
        })
    );
}

#[test]
fn warns_about_unused_local_variable_in_block() {
    let errors = analyze_source("{ var x = 1; }");

    assert!(errors.iter().any(|e| {
        e.contains("[Semantic Warning]")
            && e.contains("declared, but has not used")
            && e.contains("[Line 1, Col 1]")
    }));
}

#[test]
fn handles_nested_scopes_with_shadowing_without_errors() {
    let source = "var x = 1; { var x = 2; print x; } print x;";
    let errors = analyze_source(source);

    assert!(errors.is_empty(), "unexpected errors: {errors:?}");
}

#[test]
fn allows_access_to_parent_scope_variable() {
    let source = "var x = 1; { print x; }";
    let errors = analyze_source(source);

    assert!(errors.is_empty(), "unexpected errors: {errors:?}");
}
