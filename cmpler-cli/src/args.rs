use std::path::PathBuf;
use clap::{Parser, Subcommand, ValueEnum};
use inkwell::OptimizationLevel;

#[derive(Debug, Parser)]
#[command(name = "cmpler", version, about = "Small-C compiler frontend")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    Build(BuildArgs),
    Run(RunArgs),
}

#[derive(Debug, Clone, PartialEq, ValueEnum)]
pub enum OptLevel {
    None,
    Less,
    Default,
    Aggressive,
}

impl From<OptLevel> for OptimizationLevel {
    fn from(level: OptLevel) -> Self {
        match level {
            OptLevel::None       => OptimizationLevel::None,
            OptLevel::Less       => OptimizationLevel::Less,
            OptLevel::Default    => OptimizationLevel::Default,
            OptLevel::Aggressive => OptimizationLevel::Aggressive,
        }
    }
}

#[derive(Debug, Parser)]
pub struct BuildArgs {
    #[arg(value_name = "FILE")]
    pub input: PathBuf,

    #[arg(short, long)]
    pub output: Option<PathBuf>,

    #[arg(long)]
    pub emit_ir: bool,

    #[arg(long)]
    pub emit_obj: bool,

    #[arg(short, long, value_enum, default_value_t = OptLevel::Default)]
    pub opt_level: OptLevel,
}

#[derive(Debug, Parser)]
pub struct RunArgs {
    #[arg(value_name = "FILE")]
    pub input: PathBuf,

    // TODO: implement JIT
    #[arg(long)]
    pub jit: bool,

    #[arg(last = true)]
    pub args: Vec<String>,
}
