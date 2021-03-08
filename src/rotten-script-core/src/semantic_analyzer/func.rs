use std::{collections::HashMap, rc::Rc};

use crate::parser::ast::Ast;

use super::{func_info::FuncInfo, interface_info::InterfaceInfo};

pub struct Func<'a> {
    ast: &'a Ast,
    func_info: Rc<FuncInfo>,
    tree: Option<Block>,
}

impl<'a> Func<'a> {
    // NOTE: currently aim to handle func with no nested function.
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

    pub fn analyze(
        &mut self,
        imported_members: HashMap<String, FuncInfo>,
        imported_interfaces: HashMap<&str, InterfaceInfo>,
    ) -> Result<(), ()> {
        let mut glob_sym_map = HashMap::new();
        for (identifier, info) in &imported_members {
            glob_sym_map.insert(identifier.clone(), info.return_type);
        }
        for (interface_name, info) in &imported_interfaces {
            info.members.iter().for_each(|(func_name, ty)| {
                glob_sym_map.insert(format!("{}.{}", interface_name, func_name), *ty);
            });
        }

        let func_compound = self.get_compound_ast();
        let block = self.create_block_from_compound_ast(func_compound);
        Err(())
    }

    fn get_compound_ast(&self) -> &Ast {
        let primary_exp = self.ast.children.as_ref().unwrap().last().unwrap();
        let compound_exp = &primary_exp.children.as_ref().unwrap()[0];
        compound_exp
    }

    fn create_block_from_compound_ast(&self, compound_ast: &Ast) -> Block {
        todo!()
    }
}

#[derive(Debug)]
pub struct Block {
    statements: Vec<Expression>,
    expression: Option<Expression>,
}

#[derive(Debug)]
pub enum Expression {
    // a = 3;
    Assignment(Assignment),
    // console.log("Hello, world!");
    Caller(Caller),
    // declaration
    Declaration(Declaration),
}

#[derive(Debug)]
pub struct Assignment {
    left_identifier: String,  // TODO: replace path
    right_identifier: String, // TODO: above,
    left_type: super::func_info::Type,
    right_type: super::func_info::Type,
}

#[derive(Debug)]
pub struct Caller {
    target_identifier: String, // TODO: replace path,
    arguments_pair: Vec<(Expression, super::func_info::Type)>,
}

#[derive(Debug)]
pub struct Declaration {
    declaration_type: DeclarationType,
    declared_identifier: String,
    declared_type: super::func_info::Type,
    binded_expression: Box<Expression>,
    binded_type: super::func_info::Type,
}

#[derive(Debug, Clone, Copy)]
pub enum DeclarationType {
    Let,
    Const,
}
