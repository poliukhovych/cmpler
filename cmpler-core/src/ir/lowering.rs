// TODO: own IR
/*use crate::ast::nodes::{Program, Decl, Stmt, Expr};
use crate::lexer::{ TokenKind };
use std::collections::HashMap;

#[derive(Debug)]
pub struct IRModule {
    pub functions: Vec<IRFunction>,
}

#[derive(Debug)]
pub struct IRFunction {
    pub name: String,
    pub params: Vec<String>,
    pub blocks: Vec<IRBlock>,
}

#[derive(Debug)]
pub struct IRBlock {
    pub name: String,
    pub instrs: Vec<IRInstr>,
}

#[derive(Debug)]
pub enum IRInstr {
    Alloca { var: String },
    Store { var: String, value: IROperand },
    Load { dest: String, var: String },
    Binary { op: IROp, dest: String, lhs: IROperand, rhs: IROperand },
    Return { value: Option<IROperand> },
    Jump { label: String },
    CondJump { cond: IROperand, then_label: String, else_label: String },
}

#[derive(Debug, Clone, Copy)]
pub enum IROp {
    Add, Sub, Mul, Div,
    Lt, Le, Gt, Ge,
    Eq, Ne,
}

#[derive(Debug)]
pub enum IROperand {
    Var(String),
    Const(i64),
}

struct LoweringContext {
    next_temp: usize,
    symbol_table: HashMap<String, String>,
    module: IRModule,
}

impl LoweringContext {
    pub fn new() -> Self {
        LoweringContext {
            next_temp: 0,
            symbol_table: HashMap::new(),
            module: IRModule { functions: Vec::new() },
        }
    }

    pub fn lower_program(program: &Program) -> IRModule {
        let mut ctx = LoweringContext::new();
        for decl in &program.decls {
            ctx.lower_function(decl);
        }
        ctx.module
    }

    fn lower_function(&mut self, decl: &Decl) {
        if let Decl::Function { name, body, span: _, .. } = decl { // TODO: fix
            let mut func = IRFunction {
                name: name.clone(),
                params: Vec::new(),
                blocks: Vec::new(),
            };
            let mut entry = IRBlock { name: "entry".to_string(), instrs: Vec::new() };
            for stmt in body {
                self.lower_stmt(stmt, &mut entry);
            }
            self.ensure_return(&mut entry);
            func.blocks.push(entry);
            self.module.functions.push(func);
        }
    }

    fn lower_stmt(&mut self, stmt: &Stmt, block: &mut IRBlock) {
        match stmt {
            Stmt::Empty => {}
            Stmt::LocalVar { name, init, span: _ } => {
                let var_ptr = name.clone();
                block.instrs.push(IRInstr::Alloca { var: var_ptr.clone() });
                let val = self.lower_expr(init, block);
                block.instrs.push(IRInstr::Store { var: var_ptr.clone(), value: val });
                self.symbol_table.insert(name.clone(), var_ptr);
            }
            Stmt::Expr(expr) => { self.lower_expr(expr, block); }
            Stmt::Return(expr) => {
                let val = self.lower_expr(expr, block);
                block.instrs.push(IRInstr::Return { value: Some(val) });
            }
            Stmt::If { cond, then_block, else_block } => {
                let cond_val = self.lower_expr(cond, block);
                let then_label = self.new_label();
                let else_label = self.new_label();
                let cont_label = self.new_label();
                block.instrs.push(IRInstr::CondJump { cond: cond_val, then_label: then_label.clone(), else_label: else_label.clone() });
                let mut then_bb = IRBlock { name: then_label.clone(), instrs: Vec::new() };
                for s in then_block { self.lower_stmt(s, &mut then_bb); }
                then_bb.instrs.push(IRInstr::Jump { label: cont_label.clone() });
                let mut else_bb = IRBlock { name: else_label.clone(), instrs: Vec::new() };
                if let Some(eb) = else_block {
                    for s in eb { self.lower_stmt(s, &mut else_bb); }
                }
                else_bb.instrs.push(IRInstr::Jump { label: cont_label.clone() });
                block.instrs.push(IRInstr::Jump { label: then_label });
                self.module.functions.last_mut().unwrap().blocks.push(then_bb);
                self.module.functions.last_mut().unwrap().blocks.push(else_bb);
                let cont_bb = IRBlock { name: cont_label, instrs: Vec::new() };
                self.module.functions.last_mut().unwrap().blocks.push(cont_bb);
            }
            Stmt::While(cond, body) => {
                let loop_label = self.new_label();
                let cont_label = self.new_label();
                block.instrs.push(IRInstr::Jump { label: loop_label.clone() });
                let mut loop_bb = IRBlock { name: loop_label.clone(), instrs: Vec::new() };
                let cond_val = self.lower_expr(cond, &mut loop_bb);
                loop_bb.instrs.push(IRInstr::CondJump { cond: cond_val, then_label: loop_label.clone(), else_label: cont_label.clone() });
                for s in body { self.lower_stmt(s, &mut loop_bb); }
                loop_bb.instrs.push(IRInstr::Jump { label: loop_label.clone() });
                let cont_bb = IRBlock { name: cont_label, instrs: Vec::new() };
                block.instrs.push(IRInstr::Jump { label: loop_label });
                self.module.functions.last_mut().unwrap().blocks.push(loop_bb);
                self.module.functions.last_mut().unwrap().blocks.push(cont_bb);
            }
            Stmt::For { init, cond, inc, body } => {
                if let Some(i) = init { self.lower_stmt(&Stmt::Expr(i.clone()), block); }
                let loop_label = self.new_label();
                let cont_label = self.new_label();
                block.instrs.push(IRInstr::Jump { label: loop_label.clone() });
                let mut loop_bb = IRBlock { name: loop_label.clone(), instrs: Vec::new() };
                if let Some(c) = cond {
                    let cv = self.lower_expr(c, &mut loop_bb);
                    loop_bb.instrs.push(IRInstr::CondJump { cond: cv, then_label: loop_label.clone(), else_label: cont_label.clone() });
                }
                for s in body { self.lower_stmt(s, &mut loop_bb); }
                if let Some(i) = inc { self.lower_stmt(&Stmt::Expr(i.clone()), &mut loop_bb); }
                loop_bb.instrs.push(IRInstr::Jump { label: loop_label.clone() });
                let cont_bb = IRBlock { name: cont_label, instrs: Vec::new() };
                self.module.functions.last_mut().unwrap().blocks.push(loop_bb);
                self.module.functions.last_mut().unwrap().blocks.push(cont_bb);
            }
            Stmt::Block(stmts) => {
                for s in stmts { self.lower_stmt(s, block); }
            }
        }
    }

    fn lower_expr(&mut self, expr: &Expr, block: &mut IRBlock) -> IROperand {
        match expr {
            Expr::IntLiteral { value, .. } => IROperand::Const(*value),
            Expr::Var { name, .. } => IROperand::Var(self.symbol_table[name].clone()),
            Expr::Binary { op, left, right, span: _ } => {
                let lhs = self.lower_expr(left, block);
                let rhs = self.lower_expr(right, block);
                let temp = self.fresh_temp();
                block.instrs.push(IRInstr::Binary { op: match op {
                    TokenKind::Plus => IROp::Add,
                    TokenKind::Minus => IROp::Sub,
                    TokenKind::Star => IROp::Mul,
                    TokenKind::Slash => IROp::Div,
                    TokenKind::Less => IROp::Lt,
                    TokenKind::LessEqual => IROp::Le,
                    TokenKind::Greater => IROp::Gt,
                    TokenKind::GreaterEqual => IROp::Ge,
                    TokenKind::Equal => IROp::Eq,
                    TokenKind::NotEqual => IROp::Ne,
                    _ => unreachable!(),
                }, dest: temp.clone(), lhs, rhs });
                IROperand::Var(temp)
            }
        }
    }

    fn ensure_return(&self, block: &mut IRBlock) {
        if let Some(IRInstr::Return { .. }) = block.instrs.last() { return; }
        block.instrs.push(IRInstr::Return { value: None });
    }

    fn fresh_temp(&mut self) -> String {
        let name = format!("t{}", self.next_temp);
        self.next_temp += 1;
        name
    }

    fn new_label(&mut self) -> String {
        let name = format!("L{}", self.next_temp);
        self.next_temp += 1;
        name
    }
}*/
