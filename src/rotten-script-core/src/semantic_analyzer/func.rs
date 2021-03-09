use std::{collections::HashMap, rc::Rc};

use crate::parser::{ast::Ast, ast_type::AstType, non_terminal::NonTerminal};

use super::{func_info::FuncInfo, interface_info::InterfaceInfo};

use crate::TBR;

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
        let children = compound_ast.children.as_ref().unwrap();
        let mut expressions = Vec::new();
        for child_ast in children {
            match child_ast.ast_type {
                // analysis of expression statements needs `Math.sin(2) + Math.sin(2) to Math.sin(2); Math.sin(2);`
                // because Math.sin(2) maybe has side effects (in fact, Math.sin has no side effect.)
                AstType::NonTerminal(NonTerminal::ExpressionStatement) => {}
                AstType::NonTerminal(NonTerminal::ConstDeclaration)
                | AstType::NonTerminal(NonTerminal::LetDeclaration) => {
                    let declaration_type = match child_ast.ast_type {
                        AstType::NonTerminal(NonTerminal::LetDeclaration) => DeclarationType::Let,
                        AstType::NonTerminal(NonTerminal::ConstDeclaration) => {
                            DeclarationType::Const
                        }
                        _ => panic!(),
                    };
                    let declar_body = &child_ast.children.as_ref().unwrap()[0];
                    let declar_body_children = declar_body.children.as_ref().unwrap();

                    let identifier = declar_body_children[0].token.as_ref().unwrap();
                    let type_ident = if declar_body_children.len() == 2 {
                        None
                    } else {
                        Some(declar_body_children[1].token.as_ref().unwrap())
                    };
                    let exp = &declar_body_children[if type_ident.is_some() { 2 } else { 1 }];
                    let declar = Declaration {
                        declaration_type,
                        declared_identifier: identifier.to_string(),
                        declared_type: match type_ident {
                            Some(t) => t.to_string().into(),
                            None => super::func_info::Type::Unknown,
                        },
                        binded_expression: Box::new(self.create_block_from_expression(exp)),
                        binded_type: super::func_info::Type::Unknown,
                    };
                    expressions.push(Statement::Declaration(declar));
                }
                AstType::NonTerminal(NonTerminal::AssignmentStatement) => {
                    let children = child_ast.children.as_ref().unwrap();
                    let assignment_op_ind = children.len() - 2;
                    let idents = &children[..assignment_op_ind];
                    let ident_str = idents
                        .iter()
                        .map(|x| {
                            let tk = x.token.as_ref().unwrap();
                            tk.to_string()
                        })
                        .collect::<Vec<_>>()
                        .join(".");
                    let assignment_op = match &children[assignment_op_ind]
                        .token
                        .as_ref()
                        .unwrap()
                        .get_token()
                        .as_ref()
                        .unwrap()
                    {
                        TBR!("=") => AssignmentType::Assign,
                        TBR!("*=") => AssignmentType::Mul,
                        TBR!("/=") => AssignmentType::Div,
                        TBR!("%=") => AssignmentType::Mod,
                        TBR!("+=") => AssignmentType::Add,
                        TBR!("-=") => AssignmentType::Sub,
                        TBR!("<<=") => AssignmentType::LShift,
                        TBR!(">>=") => AssignmentType::RShift,
                        TBR!(">>>=") => AssignmentType::UnRShift,
                        TBR!("&=") => AssignmentType::LogAnd,
                        TBR!("^=") => AssignmentType::LogXor,
                        TBR!("|=") => AssignmentType::LogOr,
                        TBR!("**=") => AssignmentType::Exp,
                        _ => panic!(),
                    };
                    expressions.push(Statement::Assignment(Assignment {
                        left_identifier: ident_str,
                        right_expression: Box::new(
                            self.create_block_from_expression(&children[assignment_op_ind]),
                        ),
                        left_type: super::func_info::Type::Unknown,
                        right_type: super::func_info::Type::Unknown,
                        assignment_type: assignment_op,
                    }))
                }
                _ => {}
            }
        }
        Block {
            statements: expressions,
            expression: None,
        }
    }

    fn create_block_from_expression(&self, expression_ast: &Ast) -> Statement {
        todo!()
    }

    fn create_block_and_extract_from_expression(&self, expression_ast: &Ast) -> Statement {
        todo!()
    }
}

#[derive(Debug)]
pub struct Block {
    statements: Vec<Statement>,
    expression: Option<Statement>,
}

#[derive(Debug)]
pub enum Statement {
    // a = 3;
    Assignment(Assignment),
    // console.log("Hello, world!");
    Caller(Caller),
    // declaration
    Declaration(Declaration),
    Expression(Expression),
}

#[derive(Debug)]
pub struct Assignment {
    left_identifier: String,          // TODO: replace path
    right_expression: Box<Statement>, // Only allowed Caller and Expression
    left_type: super::func_info::Type,
    right_type: super::func_info::Type,
    assignment_type: AssignmentType,
}

#[derive(Debug)]
pub struct Caller {
    target_identifier: String, // TODO: replace path,
    arguments_pair: Vec<(Statement, super::func_info::Type)>,
}

#[derive(Debug)]
pub struct Declaration {
    declaration_type: DeclarationType,
    declared_identifier: String,
    declared_type: super::func_info::Type,
    binded_expression: Box<Statement>,
    binded_type: super::func_info::Type,
}

#[derive(Debug, Clone, Copy)]
pub enum DeclarationType {
    Let,
    Const,
}

#[derive(Debug)]
pub enum Expression {
    BinaryExpression(BinaryExpressionTree),
    // Caller or expression statement is only allowed
    UnaryExpression(Box<Statement>),
}

#[derive(Debug, Default)]
pub struct BinaryExpressionTree {
    expressions_vec: Vec<Statement>,
    operators_vec: Vec<BinaryOperator>,
}

impl BinaryExpressionTree {
    pub fn new() -> Self {
        Self::default()
    }
}

#[derive(Debug)]
pub enum BinaryOperator {
    Add,
    Div,
    Mul,
    Mod,
    Sub,
    Xor,
    Or,
    And,
    Not,
    LogAnd,
    LogOr,
    LogNot,
    Exp,
    RShift,
    UnRShift,
    LShift,
    Eq,
    NotEq,
    Less,
    Greater,
    GreaterEq,
    LessEq,
}

#[derive(Debug)]
pub enum AssignmentType {
    Assign,
    Add,
    Div,
    Mod,
    Sub,
    Mul,
    Exp,
    Or,
    LShift,
    RShift,
    UnRShift,
    LogAnd,
    LogXor,
    LogOr,
}
