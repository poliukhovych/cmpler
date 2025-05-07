# CMPLER

Cmpler is a compiler for a subset of the C language (Small-C), built on top of LLVM and implemented in Rust. It consists of:

* **cmpler-core**: the compiler core library providing lexer, parser, semantic analysis, custom IR, and LLVM code generation.
* **cmpler-cli**: a command-line interface to compile, JIT‑execute, and link binaries.
* **cmpler-tests**: unit and integration tests covering all components.

## Overview

* **Lexical and syntactic analysis** for Small-C syntax.
* **Semantic checks**: undefined variables, duplicates, and scope rules.
* **Custom intermediate representation** (IR) with lowering from AST.
* **LLVM IR generation** and object code emission using the `inkwell` crate.
* **Built-in optimizations** (O0…O3) via LLVM pass manager.
* **JIT execution** through LLVM ExecutionEngine and AOT compilation with system linker.
* **Configuration** via `cmpler.toml` and CLI flags.

## Getting Started

### Requirements

* Rust toolchain (stable 1.x or later)
* LLVM development libraries
* `cc` (system linker)

### Building

This repository is a Cargo workspace:

```bash
git clone https://github.com/yourusername/cmpler.git
cd cmpler
cargo build --release
```

The CLI binary will be at `target/release/cmpler-cli`.

### Usage

#### Build

Compile a Small-C source file (`.c`) to IR, object, or executable:

```bash
# Generate executable (a.out)
cmpler-cli build program.c

# Emit LLVM IR (.ll)
cmpler-cli build program.c --emit-ir

# Emit object code (.o)
cmpler-cli build program.c --emit-obj

# Specify output filename
cmpler-cli build program.c -o my_program

# Control optimization level: none, less, default, aggressive
cmpler-cli build program.c --opt-level aggressive
```

#### Run

Execute via JIT or compile+run:

```bash
# JIT execution
cmpler-cli run program.c --jit

# Compile, link, and run
cmpler-cli run program.c
```

## Configuration

CMPLER can load settings from a `cmpler.toml` file located in the working directory or any parent directory. Example `cmpler.toml`:

```toml
emit_ir = true
emit_obj = false
target = "x86_64-unknown-linux-gnu"
output = "build/myprog"
verbose = true
```

CLI flags override configuration file values.

## Project Structure

```text
cmpler/
├── cmpler-core/     # Compiler core library
├── cmpler-cli/      # Command-line tool
├── cmpler-tests/    # Unit and integration tests
└── Cargo.toml       # Workspace manifest
```

* **cmpler-core/src** contains modules: `lexer`, `parser`, `ast`, `semantic`, `ir`, `codegen`, `config`, `error`, `logger`, `utils`, `driver`.
* **cmpler-cli/src** contains: `main.rs`, `args.rs`, and `commands/{build.rs,run.rs}`.
* **cmpler-tests/src** contains automated tests for each compiler stage.

## Contributing

Contributions are welcome. To contribute:

1. Fork the repository.
2. Create a feature branch (`git checkout -b feature/XYZ`).
3. Implement tests and functionality.
4. Submit a pull request for review.

## License

This project is licensed under the MIT license. See `LICENSE` for details.
