use std::{fs, path::PathBuf, process::Command};
use tracing::info;
use inkwell::context::Context;
use inkwell::OptimizationLevel;
use inkwell::execution_engine::JitFunction;
use crate::args::RunArgs;
use cmpler_core::driver::{compile, compile_to_object, link_executable};
use cmpler_core::codegen::llvm_gen::LLVMCodeGen;
use cmpler_core::error::CompilerError;

// For now it just uses LLVM ExecutionEngine
pub fn launch_run(args: &RunArgs) -> Result<(), CompilerError> {
    let source = fs::read_to_string(&args.input)
        .map_err(CompilerError::Io)?;

    if args.jit {
        let program = compile(&source)?;
        let context = Context::create();
        let module = LLVMCodeGen::compile_program(&context, &program, OptimizationLevel::Default);
        let ee = module.create_jit_execution_engine(OptimizationLevel::Default)
            .map_err(|e| CompilerError::Codegen(format!("Failed to create JIT engine: {:?}", e)))?;

        unsafe {
            type MainFn = unsafe extern "C" fn() -> i32;
            let main_fn: JitFunction<MainFn> = ee.get_function("main")
                .map_err(|e| CompilerError::Codegen(format!("Failed to find 'main': {:?}", e)))?;
            let result = main_fn.call();
            println!("{}", result);
        }
    } else {
        let obj_path = args.input.with_extension("o");
        compile_to_object(&source, OptimizationLevel::Default, &obj_path)?;

        let exe_path = PathBuf::from("a.out");
        link_executable(&obj_path, &exe_path)?;
        info!("Running {}", exe_path.display());

        let mut cmd = Command::new(&exe_path);
        cmd.args(&args.args);
        let status = cmd.status().map_err(CompilerError::Io)?;
        if !status.success() {
            return Err(CompilerError::Codegen(format!("Execution failed: {}", status)));
        }
    }

    Ok(())
}
