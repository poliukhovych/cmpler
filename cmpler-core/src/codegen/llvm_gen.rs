use inkwell::context::Context;
use inkwell::builder::Builder;
use inkwell::module::Module;
use inkwell::types::IntType;
use inkwell::values::{BasicValueEnum, FunctionValue, PointerValue};
use inkwell::passes::PassManager;
use inkwell::OptimizationLevel;
use std::collections::HashMap;
use crate::ast::nodes::{Program, Decl, Stmt, Expr};
use crate::lexer::TokenKind;

pub struct LLVMCodeGen<'ctx> {
    context: &'ctx Context,
    module: Module<'ctx>,
    builder: Builder<'ctx>,
    i32_type: IntType<'ctx>,
    function: Option<FunctionValue<'ctx>>,
    variables: HashMap<String, PointerValue<'ctx>>,
    pass_manager: PassManager<Module<'ctx>>,
}

impl<'ctx> LLVMCodeGen<'ctx> {
    pub fn new(
        context: &'ctx Context,
        module_name: &str,
        opt_level: OptimizationLevel,
    ) -> Self {
        let module = context.create_module(module_name);
        let builder = context.create_builder();
        let i32_type = context.i32_type();
        let pass_manager = PassManager::create(());

        if opt_level != OptimizationLevel::None {
            pass_manager.add_instruction_combining_pass();
            pass_manager.add_reassociate_pass();
            pass_manager.add_gvn_pass();
            pass_manager.add_cfg_simplification_pass();
        }

        LLVMCodeGen {
            context,
            module,
            builder,
            i32_type,
            function: None,
            variables: HashMap::new(),
            pass_manager,
        }
    }

    pub fn compile_program(
        context: &'ctx Context,
        program: &Program,
        opt_level: OptimizationLevel,
    ) -> Module<'ctx> {
        let mut gen = LLVMCodeGen::new(context, "cmpler_module", opt_level);
        gen.gen_program(program);
        if opt_level != OptimizationLevel::None {
            gen.pass_manager.run_on(&gen.module);
        }
        gen.module.clone()
    }

    fn gen_program(&mut self, program: &Program) {
        for decl in &program.decls {
            if let Decl::Function { .. } = decl {
                self.gen_function(decl);
            }
        }
    }

    fn gen_function(&mut self, decl: &Decl) {
        if let Decl::Function { name, body, .. } = decl {
            let fn_val = self.module.add_function(
                name,
                self.i32_type.fn_type(&[], false),
                None,
            );
            self.function = Some(fn_val);
            let entry = self.context.append_basic_block(fn_val, "entry");
            self.builder.position_at_end(entry);
            self.variables.clear();

            for stmt in body {
                self.gen_stmt(stmt);
            }

            if self.builder.get_insert_block().unwrap().get_terminator().is_none() {
                let _ = self.builder.build_return(Some(&self.i32_type.const_zero()));
            }
        }
    }

    fn gen_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Empty => {}
            Stmt::LocalVar { name, init, .. } => {
                let ptr = self.builder.build_alloca(self.i32_type, name).expect("alloca failed");
                let val = self.gen_expr(init);
                self.builder.build_store(ptr, val).expect("store failed");
                self.variables.insert(name.clone(), ptr);
            }
            Stmt::Expr(expr) => {
                if let Expr::Binary { op: TokenKind::Assign, left, right, .. } = expr.as_ref() {
                    if let Expr::Var { name, .. } = &**left {
                        let val = self.gen_expr(right);
                        let ptr = *self.variables.get(name).expect("undefined variable");
                        self.builder.build_store(ptr, val).expect("store failed");
                    }
                } else {
                    self.gen_expr(expr);
                }
            }
            Stmt::Return(expr) => {
                let val = self.gen_expr(expr);
                let _ = self.builder.build_return(Some(&val));
            }
            Stmt::If { cond, then_block, else_block } => {
                self.gen_if(cond, then_block, else_block);
            }
            Stmt::While(cond, body) => {
                self.gen_while(cond, body);
            }
            Stmt::For { init, cond, inc, body } => {
                self.gen_for(init, cond, inc, body);
            }
            Stmt::Block(stmts) => {
                for s in stmts {
                    self.gen_stmt(s);
                }
            }
        }
    }

    fn gen_if(&mut self, cond: &Expr, then_block: &[Stmt], else_block: &Option<Vec<Stmt>>) {
        let func = self.function.unwrap();
        let then_bb = self.context.append_basic_block(func, "then");
        let else_bb = self.context.append_basic_block(func, "else");
        let cont_bb = self.context.append_basic_block(func, "cont");

        let cond_val = self.gen_expr(cond).into_int_value();
        let cond_bool = if cond_val.get_type().get_bit_width() == 1 {
            cond_val
        } else {
            let zero = self.i32_type.const_zero();
            self.builder.build_int_compare(inkwell::IntPredicate::NE, cond_val, zero, "ifcond").expect("icmp")
        };

        let _ = self.builder.build_conditional_branch(cond_bool, then_bb, else_bb);

        self.builder.position_at_end(then_bb);
        for s in then_block { self.gen_stmt(s); }
        if self.builder.get_insert_block().unwrap().get_terminator().is_none() {
            let _ = self.builder.build_unconditional_branch(cont_bb);
        }

        self.builder.position_at_end(else_bb);
        if let Some(els) = else_block { for s in els { self.gen_stmt(s); } }
        if self.builder.get_insert_block().unwrap().get_terminator().is_none() {
            let _ = self.builder.build_unconditional_branch(cont_bb);
        }

        self.builder.position_at_end(cont_bb);
    }

    fn gen_while(&mut self, cond: &Expr, body: &[Stmt]) {
        let func = self.function.unwrap();
        let loop_bb = self.context.append_basic_block(func, "loop");
        let cont_bb = self.context.append_basic_block(func, "cont");

        let _ = self.builder.build_unconditional_branch(loop_bb);
        self.builder.position_at_end(loop_bb);

        let cond_val = self.gen_expr(cond).into_int_value();
        let cond_bool = if cond_val.get_type().get_bit_width() == 1 {
            cond_val
        } else {
            let zero = self.i32_type.const_zero();
            self.builder.build_int_compare(inkwell::IntPredicate::SLT, cond_val, zero, "whilecond").expect("icmp")
        };

        let body_bb = self.context.append_basic_block(func, "body");
        let _ = self.builder.build_conditional_branch(cond_bool, body_bb, cont_bb);

        self.builder.position_at_end(body_bb);
        for s in body { self.gen_stmt(s); }
        if self.builder.get_insert_block().unwrap().get_terminator().is_none() {
            let _ = self.builder.build_unconditional_branch(loop_bb);
        }

        self.builder.position_at_end(cont_bb);
    }

    fn gen_for(&mut self, init: &Option<Box<Expr>>, cond: &Option<Box<Expr>>, inc: &Option<Box<Expr>>, body: &[Stmt]) {
        if let Some(i) = init { let _ = self.gen_expr(i); }
        let func = self.function.unwrap();
        let loop_bb = self.context.append_basic_block(func, "loop");
        let cont_bb = self.context.append_basic_block(func, "cont");

        let _ = self.builder.build_unconditional_branch(loop_bb);
        self.builder.position_at_end(loop_bb);

        if let Some(c) = cond {
            let cond_val = self.gen_expr(c).into_int_value();
            let zero = self.i32_type.const_zero();
            let cond_bool = self.builder.build_int_compare(inkwell::IntPredicate::SLT, cond_val, zero, "forcond").expect("icmp");
            let _ = self.builder.build_conditional_branch(cond_bool, loop_bb, cont_bb);
        }

        for s in body { self.gen_stmt(s); }
        if let Some(i) = inc { let _ = self.gen_expr(i); }
        if self.builder.get_insert_block().unwrap().get_terminator().is_none() {
            let _ = self.builder.build_unconditional_branch(loop_bb);
        }

        self.builder.position_at_end(cont_bb);
    }

    fn gen_expr(&mut self, expr: &Expr) -> BasicValueEnum<'ctx> {
        match expr {
            Expr::IntLiteral { value, .. } => self.i32_type.const_int(*value as u64, false).into(),
            Expr::Var { name, .. } => {
                let ptr = *self.variables.get(name).unwrap();
                self.builder.build_load(self.i32_type, ptr, name).expect("load").into()
            }
            Expr::Binary { op, left, right, .. } => {
                let lhs = self.gen_expr(left).into_int_value();
                let rhs = self.gen_expr(right).into_int_value();
                let instr = match op {
                    TokenKind::Plus => self.builder.build_int_add(lhs, rhs, "addtmp").expect("add"),
                    TokenKind::Minus => self.builder.build_int_sub(lhs, rhs, "subtmp").expect("sub"),
                    TokenKind::Star => self.builder.build_int_mul(lhs, rhs, "multmp").expect("mul"),
                    TokenKind::Slash => self.builder.build_int_signed_div(lhs, rhs, "divtmp").expect("div"),
                    TokenKind::Less => self.builder.build_int_compare(inkwell::IntPredicate::SLT, lhs, rhs, "lttmp").expect("icmp"),
                    TokenKind::LessEqual => self.builder.build_int_compare(inkwell::IntPredicate::SLE, lhs, rhs, "letmp").expect("icmp"),
                    TokenKind::Greater => self.builder.build_int_compare(inkwell::IntPredicate::SGT, lhs, rhs, "gttmp").expect("icmp"),
                    TokenKind::GreaterEqual => self.builder.build_int_compare(inkwell::IntPredicate::SGE, lhs, rhs, "getmp").expect("icmp"),
                    TokenKind::Equal => self.builder.build_int_compare(inkwell::IntPredicate::EQ, lhs, rhs, "eqtmp").expect("icmp"),
                    TokenKind::NotEqual => self.builder.build_int_compare(inkwell::IntPredicate::NE, lhs, rhs, "netmp").expect("icmp"),
                    _ => unreachable!(),
                };
                instr.into()
            }
        }
    }
}
