mod args;
mod commands;

use crate::args::{Cli, Commands};
use crate::commands::{build, run};
use cmpler_core::logger::init_logger;
use clap::Parser;
use cmpler_core::error::CompilerError;

fn main() {
    init_logger();

    if let Err(e) = run_cli() {
        eprintln!("error: {}", e);
        std::process::exit(1);
    }
}

fn run_cli() -> Result<(), CompilerError> {
    let cfg = cmpler_core::config::Config::load()?;

    let cli = Cli::parse();
    match cli.command {
        Commands::Build(mut args) => {
            if !args.emit_ir {
                args.emit_ir = cfg.emit_ir;
            }
            if !args.emit_obj {
                args.emit_obj = cfg.emit_obj;
            }
            if args.output.is_none() {
                args.output = cfg.output.clone();
            }
            build::launch_build(&args)
        }
        Commands::Run(mut args) => {
            if !args.jit && cfg.emit_ir {
                args.jit = true;
            }
            run::launch_run(&args)
        }
    }
}
