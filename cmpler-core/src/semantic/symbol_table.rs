use std::collections::HashMap;
use crate::utils::span::Span;
use crate::ast::nodes::Decl;
use crate::semantic::error::SemanticError;

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Int,
    Void,
}

#[derive(Debug, Clone)]
pub struct Symbol {
    pub name: String,
    pub ty: Type,
    pub span: Span,
}

#[derive(Debug)]
pub struct SymbolTable {
    scopes: Vec<HashMap<String, Symbol>>,
}

impl SymbolTable {
    pub fn new() -> Self {
        SymbolTable { scopes: vec![HashMap::new()] }
    }

    pub fn enter_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    pub fn exit_scope(&mut self) {
        self.scopes.pop();
    }

    pub fn insert(&mut self, decl: &Decl) -> Result<(), SemanticError> {
        let (name, ty, span) = match decl {
            Decl::Function { name, .. } => (name.clone(), Type::Int, decl.span()),
            Decl::Var { name, .. }      => (name.clone(), Type::Int, decl.span()),
        };

        let scope = self.scopes.last_mut().unwrap();
        if scope.contains_key(&name) {
            Err(SemanticError::DuplicateSymbol(name, span))
        } else {
            scope.insert(name.clone(), Symbol { name, ty, span });
            Ok(())
        }
    }

    pub fn insert_symbol(&mut self, name: String, ty: Type, span: Span) -> Result<(), SemanticError> {
        let scope = self.scopes.last_mut().unwrap();
        if scope.contains_key(&name) {
            Err(SemanticError::DuplicateSymbol(name, span))
        } else {
            scope.insert(name.clone(), Symbol { name, ty, span });
            Ok(())
        }
    }

    pub fn lookup(&self, name: &str) -> Option<&Symbol> {
        for scope in self.scopes.iter().rev() {
            if let Some(sym) = scope.get(name) {
                return Some(sym);
            }
        }
        None
    }
}
