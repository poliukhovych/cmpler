use std::fs;
use std::path::PathBuf;
use crate::args::BuildArgs;
use cmpler_core::driver::{compile_to_llvm_ir, compile_to_object, link_executable};
use cmpler_core::error::CompilerError;
use inkwell::OptimizationLevel;

pub fn launch_build(args: &BuildArgs) -> Result<(), CompilerError> {
    let source = fs::read_to_string(&args.input)
        .map_err(CompilerError::Io)?;
    let opt_level: OptimizationLevel = args.opt_level.clone().into();

    if args.emit_ir {
        let ir = compile_to_llvm_ir(&source, opt_level)?;
        let out = args.output.clone().unwrap_or_else(|| args.input.with_extension("ll"));
        fs::write(&out, ir).map_err(CompilerError::Io)?;
        println!("[cmpler] Wrote LLVM IR to {}", out.display());
    }

    if args.emit_obj {
        let out = args.output.clone().unwrap_or_else(|| args.input.with_extension("o"));
        compile_to_object(&source, opt_level, &out)?;
        println!("[cmpler] Wrote object file to {}", out.display());
    }

    if !args.emit_ir && !args.emit_obj {
        let obj_path = args.input.with_extension("o");
        compile_to_object(&source, opt_level, &obj_path)?;

        let exe_path = args.output.clone().unwrap_or_else(|| {
            if cfg!(windows) {
                args.input.with_extension("exe")
            } else {
                PathBuf::from("a.out")
            }
        });
        link_executable(&obj_path, &exe_path)?;
        println!("[cmpler] Generated executable {}", exe_path.display());
    }

    Ok(())
}
