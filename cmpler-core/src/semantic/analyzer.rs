use crate::ast::nodes::{Program, Decl, Stmt, Expr};
use crate::semantic::symbol_table::{SymbolTable, Type};
use crate::semantic::error::SemanticError;
use tracing::instrument;

pub struct SemanticAnalyzer {
    symbols: SymbolTable,
}

impl SemanticAnalyzer {
    pub fn new() -> Self {
        SemanticAnalyzer { symbols: SymbolTable::new() }
    }

    #[instrument(level = "info", skip(self, program))]
    pub fn analyze(&mut self, program: &Program) -> Result<(), SemanticError> {
        for decl in &program.decls {
            self.symbols.insert(decl)?;
        }
        for decl in &program.decls {
            self.check_decl(decl)?;
        }
        Ok(())
    }

    #[instrument(level = "debug", skip(self, decl))]
    fn check_decl(&mut self, decl: &Decl) -> Result<(), SemanticError> {
        match decl {
            Decl::Function { name: _, body, .. } => {
                self.symbols.enter_scope();
                for stmt in body {
                    self.check_stmt(stmt)?;
                }
                self.symbols.exit_scope();
                Ok(())
            }
            Decl::Var { name: _, init, .. } => {
                let ty = self.check_expr(init)?;
                if ty != Type::Int {
                    return Err(SemanticError::TypeMismatch {
                        expected: "Int".into(),
                        found: format!("{:?}", ty),
                        span: init.span(),
                    });
                }
                Ok(())
            }
        }
    }

    #[instrument(level = "debug", skip(self, stmt))]
    fn check_stmt(&mut self, stmt: &Stmt) -> Result<(), SemanticError> {
        match stmt {
            Stmt::Empty => Ok(()),
            Stmt::Return(expr) => {
                self.check_expr(expr)?;
                Ok(())
            }
            Stmt::Expr(expr) => {
                self.check_expr(expr)?;
                Ok(())
            }
            Stmt::If { cond, then_block, else_block, .. } => {
                let cond_ty = self.check_expr(cond)?;
                if cond_ty != Type::Int {
                    return Err(SemanticError::TypeMismatch {
                        expected: "Int".into(),
                        found: format!("{:?}", cond_ty),
                        span: cond.span(),
                    });
                }
                self.symbols.enter_scope();
                for s in then_block {
                    self.check_stmt(s)?;
                }
                self.symbols.exit_scope();
                if let Some(else_blk) = else_block {
                    self.symbols.enter_scope();
                    for s in else_blk {
                        self.check_stmt(s)?;
                    }
                    self.symbols.exit_scope();
                }
                Ok(())
            }
            Stmt::While(cond, body) => {
                let ty = self.check_expr(cond)?;
                if ty != Type::Int {
                    return Err(SemanticError::TypeMismatch {
                        expected: "Int".into(),
                        found: format!("{:?}", ty),
                        span: cond.span(),
                    });
                }
                self.symbols.enter_scope();
                for s in body {
                    self.check_stmt(s)?;
                }
                self.symbols.exit_scope();
                Ok(())
            }
            Stmt::For { init, cond, inc, body, .. } => {
                self.symbols.enter_scope();
                if let Some(expr) = init {
                    let _ = self.check_expr(expr)?;
                }
                if let Some(expr) = cond {
                    let ty = self.check_expr(expr)?;
                    if ty != Type::Int {
                        return Err(SemanticError::TypeMismatch {
                            expected: "Int".into(),
                            found: format!("{:?}", ty),
                            span: expr.span(),
                        });
                    }
                }
                if let Some(expr) = inc {
                    let _ = self.check_expr(expr)?;
                }
                for s in body {
                    self.check_stmt(s)?;
                }
                self.symbols.exit_scope();
                Ok(())
            }
            Stmt::LocalVar { name, init, span } => {
                let ty = self.check_expr(init)?;
                if ty != Type::Int {
                    return Err(SemanticError::TypeMismatch {
                        expected: "Int".into(),
                        found: format!("{:?}", ty),
                        span: *span,
                    });
                }
                self.symbols.insert_symbol(name.clone(), Type::Int, *span)?;
                Ok(())
            }
            Stmt::Block(stmts) => {
                self.symbols.enter_scope();
                for s in stmts {
                    self.check_stmt(s)?;
                }
                self.symbols.exit_scope();
                Ok(())
            }
        }
    }

    #[instrument(level = "debug", skip(self, expr))]
    fn check_expr(&mut self, expr: &Expr) -> Result<Type, SemanticError> {
        match expr {
            Expr::IntLiteral { .. } => Ok(Type::Int),
            Expr::Var { name, span } => {
                if let Some(sym) = self.symbols.lookup(name) {
                    Ok(sym.ty.clone())
                } else {
                    Err(SemanticError::UndefinedVariable(name.clone(), *span))
                }
            }
            Expr::Binary { left, right, span, .. } => {
                let lt = self.check_expr(left)?;
                let rt = self.check_expr(right)?;
                if lt != Type::Int || rt != Type::Int {
                    return Err(SemanticError::TypeMismatch {
                        expected: "Int".into(),
                        found: format!("{:?}", if lt != Type::Int { lt } else { rt }),
                        span: *span,
                    });
                }
                Ok(Type::Int)
            }
        }
    }
}
