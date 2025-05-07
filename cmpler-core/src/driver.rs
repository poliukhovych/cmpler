use tracing::{instrument, info};
use std::path::Path;
use crate::ast::Program;
use crate::error::CompilerError;
use crate::lexer::lex;
use crate::parser::Parser;
use crate::semantic::SemanticAnalyzer;

use inkwell::context::Context;
use inkwell::OptimizationLevel;
use crate::codegen::llvm_gen::LLVMCodeGen;
use inkwell::targets::{InitializationConfig, RelocMode, CodeModel, FileType, Target, TargetMachine};

#[instrument(level = "info", skip(source))]
pub fn compile(source: &str) -> Result<Program, CompilerError> {
    info!("Starting compilation");

    let tokens = lex(source);
    info!(token_count = tokens.len(), "Lexing complete");

    let mut parser = Parser::new(tokens);
    let program = parser.parse_program()?;
    info!(decls = program.decls.len(), "Parsing complete");

    let mut analyzer = SemanticAnalyzer::new();
    analyzer.analyze(&program)?;
    info!("Semantic analysis complete");

    Ok(program)
}

pub fn compile_to_llvm_ir(
    source: &str,
    opt_level: OptimizationLevel,
) -> Result<String, CompilerError> {
    let program = compile(source)?;
    let context = Context::create();
    let module = LLVMCodeGen::compile_program(&context, &program, opt_level);
    Ok(module.print_to_string().to_string())
}

pub fn compile_to_object(
    source: &str,
    opt_level: OptimizationLevel,
    output_path: &Path,
) -> Result<std::path::PathBuf, CompilerError> {
    let program = compile(source)?;
    let context = Context::create();
    let module = LLVMCodeGen::compile_program(&context, &program, opt_level);

    Target::initialize_all(&InitializationConfig::default());
    let triple = TargetMachine::get_default_triple();
    let target = Target::from_triple(&triple)
        .map_err(|e| CompilerError::Codegen(format!("Failed to get target: {}", e)))?;
    let cpu = "generic";
    let features = "";
    let tm = target.create_target_machine(
        &triple,
        cpu,
        features,
        opt_level,
        RelocMode::Default,
        CodeModel::Default,
    ).ok_or_else(|| CompilerError::Codegen("Failed to create target machine".into()))?;

    tm.write_to_file(&module, FileType::Object, output_path)
        .map_err(|e| CompilerError::Codegen(format!("Failed to write object file: {}", e)))?;
    Ok(output_path.to_path_buf())
}

pub fn link_executable(
    object_file: &Path,
    output_exe: &Path,
) -> Result<(), CompilerError> {
    let status = std::process::Command::new("cc")
        .arg(object_file)
        .arg("-o")
        .arg(output_exe)
        .status()
        .map_err(CompilerError::Io)?;
    if !status.success() {
        return Err(CompilerError::Codegen(format!("Linker failed: {}", status)));
    }
    Ok(())
}
