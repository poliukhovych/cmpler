use cmpler_core::{compile, error::CompilerError};

#[test]
fn semantic_ok_simple_function() {
    let src = "int main() { int x = 5; return x; }";
    assert!(compile(src).is_ok());
}

#[test]
fn semantic_error_undefined_variable() {
    let src = "int main() { return y; }";
    match compile(src) {
        Err(CompilerError::Semantic(err)) => {
            let msg = err.to_string();
            assert!(msg.contains("Undefined variable 'y'"));
        }
        other => panic!("Expected undefined variable error, got {:?}", other),
    }
}

#[test]
fn semantic_error_duplicate_global_symbol() {
    let src = "int x = 1; int x = 2;";
    match compile(src) {
        Err(CompilerError::Semantic(err)) => {
            let msg = err.to_string();
            assert!(msg.contains("Duplicate symbol 'x'"));
        }
        other => panic!("Expected duplicate global symbol error, got {:?}", other),
    }
}

#[test]
fn semantic_error_duplicate_local_variable() {
    let src = "int main() { int x = 1; int x = 2; return 0; }";
    match compile(src) {
        Err(CompilerError::Semantic(err)) => {
            let msg = err.to_string();
            assert!(msg.contains("Duplicate symbol 'x'"));
        }
        other => panic!("Expected duplicate local variable error, got {:?}", other),
    }
}

#[test]
fn semantic_error_duplicate_function() {
    let src = "int f() {} int f() {}";
    match compile(src) {
        Err(CompilerError::Semantic(err)) => {
            let msg = err.to_string();
            assert!(msg.contains("Duplicate symbol 'f'"));
        }
        other => panic!("Expected duplicate function error, got {:?}", other),
    }
}

#[test]
fn semantic_ok_variable_shadowing() {
    let src = r#"
        int main() {
            int x = 1;
            {
                int x = 2;
                return x;
            }
        }
    "#;
    assert!(compile(src).is_ok());
}

#[test]
fn semantic_error_undefined_in_inner_block() {
    let src = r#"
        int main() {
            {
                int x = 1;
            }
            return x;
        }
    "#;
    match compile(src) {
        Err(CompilerError::Semantic(err)) => {
            let msg = err.to_string();
            assert!(msg.contains("Undefined variable 'x'"));
        }
        other => panic!("Expected undefined inner-block variable error, got {:?}", other),
    }
}

#[test]
fn semantic_ok_use_var_in_for_loop() {
    let src = "int main() { int x = 0; for (; x < 10; x = x + 1) {} return x; }";
    assert!(compile(src).is_ok());
}

#[test]
fn semantic_error_undefined_in_for_loop() {
    let src = "int main() { for (y = 0; y < 10; y = y + 1) {} }";
    match compile(src) {
        Err(CompilerError::Semantic(err)) => {
            let msg = err.to_string();
            assert!(msg.contains("Undefined variable 'y'"));
        }
        other => panic!("Expected undefined for-loop variable error, got {:?}", other),
    }
}
