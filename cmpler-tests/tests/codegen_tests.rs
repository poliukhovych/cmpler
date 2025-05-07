use cmpler_core::driver::compile_to_llvm_ir;
use inkwell::OptimizationLevel;

#[test]
fn codegen_simple_return() {
    let src = "int main() { return 42; }";
    let ir = compile_to_llvm_ir(src, OptimizationLevel::None).unwrap();
    assert!(ir.contains("define i32 @main()"), "Expected function definition, got: {}", ir);
    assert!(ir.contains("ret i32 42"), "Expected return 42, got: {}", ir);
}

#[test]
fn codegen_default_return() {
    let src = "int foo() {}";
    let ir = compile_to_llvm_ir(src, OptimizationLevel::None).unwrap();
    assert!(ir.contains("define i32 @foo()"));
    assert!(ir.contains("ret i32 0"));
}

#[test]
fn codegen_if_statement() {
    let src = "int main() { if (1) { return 10; } else { return 20; } }";
    let ir = compile_to_llvm_ir(src, OptimizationLevel::None).unwrap();
    assert!(ir.contains("br i1 true"), "Expected conditional branch with true, got: {}", ir);
    assert_eq!(ir.matches("ret i32 10").count(), 1, "Expected one return 10, got: {}", ir);
    assert_eq!(ir.matches("ret i32 20").count(), 1, "Expected one return 20, got: {}", ir);
}

#[test]
fn codegen_nested_if_no_else() {
    let src = "int main() { if (0) { return 1; } return 2; }";
    let ir = compile_to_llvm_ir(src, OptimizationLevel::None).unwrap();
    assert!(ir.contains("br i1 false"), "Expected conditional branch with false, got: {}", ir);
    assert_eq!(ir.matches("ret i32 1").count(), 1, "Expected one return 1, got: {}", ir);
    assert_eq!(ir.matches("ret i32 2").count(), 1, "Expected one return 2, got: {}", ir);
}

#[test]
fn codegen_arithmetic_constant() {
    let src = "int main() { int x = 1 + 2; return x; }";
    let ir = compile_to_llvm_ir(src, OptimizationLevel::None).unwrap();
    assert!(ir.contains("store i32 3"), "Expected store of constant 3, got: {}", ir);
    assert!(ir.contains("ret i32 %"), "Expected return of variable, got: {}", ir);
}

#[test]
fn codegen_precedence_return() {
    let src = "int main() { return (1 + 2) * 3; }";
    let ir = compile_to_llvm_ir(src, OptimizationLevel::None).unwrap();
    assert!(ir.contains("ret i32 9"), "Expected return constant 9, got: {}", ir);
}

#[test]
fn codegen_while_loop_basic() {
    let src = "int main() { int x = 0; while (x < 3) { x = x + 1; } return x; }";
    let ir = compile_to_llvm_ir(src, OptimizationLevel::None).unwrap();
    assert!(ir.contains("icmp slt"), "Expected compare slt, got: {}", ir);
    assert!(ir.contains("br i1"), "Expected conditional branch, got: {}", ir);
    assert!(ir.contains("addtmp") || ir.contains("store i32 1"), "Expected increment, got: {}", ir);
    assert!(ir.contains("ret i32 %"));
}

#[test]
fn codegen_block_scope_two_alloca() {
    let src = r#"int main() { int x = 1; { int y = x + 2; return y; } }"#;
    let ir = compile_to_llvm_ir(src, OptimizationLevel::None).unwrap();
    let allocas = ir.matches("alloca i32").count();
    assert!(allocas >= 2, "Expected at least two allocas, got: {}", allocas);
}

#[test]
fn codegen_constant_folding_return() {
    let src = "int main() { return 1 + 2; }";
    let ir = compile_to_llvm_ir(src, OptimizationLevel::Aggressive).unwrap();
    assert!(ir.contains("ret i32 3"), "Expected return constant 3, got: {}", ir);
}

#[test]
fn codegen_comparison_ops() {
    let src = r#"
        int main() {
            if (1 == 2) { return 3; }
            if (2 != 3) { return 4; }
            if (4 < 5) { return 5; }
            if (5 <= 5) { return 6; }
            if (6 > 5) { return 7; }
            if (5 >= 5) { return 8; }
            return 0;
        }
    "#;
    let ir = compile_to_llvm_ir(src, OptimizationLevel::None).unwrap();
    assert_eq!(ir.matches("ret i32 3").count(), 1, "Expected one return 3, got: {}", ir);
    assert_eq!(ir.matches("ret i32 4").count(), 1, "Expected one return 4, got: {}", ir);
    assert_eq!(ir.matches("ret i32 5").count(), 1, "Expected one return 5, got: {}", ir);
    assert_eq!(ir.matches("ret i32 6").count(), 1, "Expected one return 6, got: {}", ir);
    assert_eq!(ir.matches("ret i32 7").count(), 1, "Expected one return 7, got: {}", ir);
    assert_eq!(ir.matches("ret i32 8").count(), 1, "Expected one return 8, got: {}", ir);
}

#[test]
fn codegen_assignment_statement() {
    let src = "int main() { int x = 0; x = x + 1; return x; }";
    let ir = compile_to_llvm_ir(src, OptimizationLevel::None).unwrap();
    assert!(ir.matches("store i32").count() >= 2, "Expected at least two store instructions, got: {}", ir);
    assert!(ir.contains("addtmp"), "Expected add instruction for x + 1, got: {}", ir);
}

#[test]
fn codegen_nested_loops() {
    let src = r#"
        int main() {
            int i = 0;
            while (i < 2) {
                int j = 0;
                while (j < 3) {
                    j = j + 1;
                }
                i = i + 1;
            }
            return i;
        }
    "#;
    let ir = compile_to_llvm_ir(src, OptimizationLevel::None).unwrap();
    assert!(ir.matches("icmp slt").count() >= 2, "Expected at least two icmp slt, got: {}", ir);
    assert!(ir.contains("addtmp"), "Expected addtmp for increments, got: {}", ir);
}

#[test]
fn codegen_multiple_locals_and_add() {
    let src = "int main() { int a = 1; int b = 2; return a + b; }";
    let ir = compile_to_llvm_ir(src, OptimizationLevel::None).unwrap();
    assert!(ir.matches("alloca i32").count() >= 2, "Expected two allocas, got: {}", ir);
    assert!(ir.contains("addtmp"), "Expected addtmp for a + b, got: {}", ir);
}

#[test]
fn codegen_multiple_functions() {
    let src = "int f() {} int g() { return 1; }";
    let ir = compile_to_llvm_ir(src, OptimizationLevel::None).unwrap();
    assert!(ir.contains("define i32 @f()"), "Expected function f, got: {}", ir);
    assert!(ir.contains("define i32 @g()"), "Expected function g, got: {}", ir);
}

#[test]
fn codegen_nested_if_else() {
    let src = r#"
        int main() {
            int a = 0;
            if (1) {
                if (0) { a = 5; } else { a = 6; }
            } else {
                a = 7;
            }
            return a;
        }
    "#;
    let ir = compile_to_llvm_ir(src, OptimizationLevel::None).unwrap();
    assert!(ir.matches("br i1").count() >= 2, "Expected nested branches, got: {}", ir);
    assert_eq!(ir.matches("store i32 5").count(), 1, "Expected store 5, got: {}", ir);
    assert_eq!(ir.matches("store i32 6").count(), 1, "Expected store 6, got: {}", ir);
    assert_eq!(ir.matches("store i32 7").count(), 1, "Expected store 7, got: {}", ir);
}
