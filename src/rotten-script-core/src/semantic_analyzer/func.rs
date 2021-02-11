use std::rc::Rc;

use crate::parser::ast::Ast;

use super::func_info::FuncInfo;

pub struct Func<'a> {
    ast: &'a Ast,
    func_info: Rc<FuncInfo>,
    tree: Option<AnalyzedFuncTree>,
}

impl<'a> Func<'a> {
    pub fn new(ast: &'a Ast, info: Rc<FuncInfo>) -> Self {
        Func {
            ast,
            func_info: info,
            tree: None,
        }
    }
    pub fn get_ast(&self) -> &'a Ast {
        self.ast
    }
    pub fn get_func_info(&self) -> Rc<FuncInfo> {
        self.func_info.clone()
    }
}

pub struct AnalyzedFuncTree {}
