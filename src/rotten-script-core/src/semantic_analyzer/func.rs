use std::{collections::HashMap, rc::Rc};

use crate::{
    lexer::token::TokenBase,
    parser::{ast::Ast, ast_type::AstType, non_terminal::NonTerminal},
};

use super::{
    func_info::{FuncInfo, PrimitiveType, Type},
    interface_info::InterfaceInfo,
};

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
        let compound_expr = func_compound.children.as_ref().unwrap().last().unwrap();
        let block = self.create_block_from_compound_ast(compound_expr);
        self.tree = Some(block);
        Ok(())
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
            match &child_ast.ast_type {
                // analysis of expression statements needs `Math.sin(2) + Math.sin(2) to Math.sin(2); Math.sin(2);`
                // because Math.sin(2) maybe has side effects (in fact, Math.sin has no side effect.)
                AstType::NonTerminal(NonTerminal::Expression) => {
                    expressions.push(self.create_block_and_extract_from_expression(child_ast));
                }
                AstType::NonTerminal(NonTerminal::ExpressionStatement) => {
                    let expr = &child_ast.children.as_ref().unwrap()[0];
                    expressions.push(self.create_block_and_extract_from_expression(expr));
                }
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
                            None => Type::Unknown,
                        },
                        binded_expression: Box::new(self.create_block_from_expression(exp)),
                        binded_type: Type::Unknown,
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
                    let assignment_op = children[assignment_op_ind]
                        .token
                        .as_ref()
                        .unwrap()
                        .get_token()
                        .as_ref()
                        .unwrap()
                        .into();

                    expressions.push(Statement::Assignment(Assignment {
                        left_identifier: ident_str,
                        right_expression: Box::new(
                            self.create_block_from_expression(&children[assignment_op_ind + 1]),
                        ),
                        left_type: Type::Unknown,
                        right_type: Type::Unknown,
                        assignment_type: assignment_op,
                    }))
                }
                _ => {
                    panic!()
                }
            }
        }
        Block {
            statements: expressions,
            expression: None,
        }
    }

    fn create_block_from_expression(&self, expression_ast: &Ast) -> Statement {
        match &expression_ast.ast_type {
            AstType::Terminal => {
                let tb = expression_ast
                    .token
                    .as_ref()
                    .unwrap()
                    .get_token()
                    .as_ref()
                    .unwrap();
                match tb {
                    TokenBase::String(s) => Statement::Literal(Literal::String(s.clone())),
                    TokenBase::Number(n) => Statement::Literal(Literal::Number(n.parse().unwrap())),
                    TokenBase::Identifier(ident) => Statement::Identifier(ident.clone()),
                    TBR!("true") => Statement::Literal(Literal::Boolean(true)),
                    TBR!("false") => Statement::Literal(Literal::Boolean(false)),
                    TokenBase::Reserved(_) => panic!(),
                }
            }
            AstType::NonTerminal(NonTerminal::PrimaryExpression) => {
                let children = expression_ast.children.as_ref().unwrap();
                let len = children.len();
                let first = &children[0];
                let is_pathed = len > 1
                    && children[1..].iter().any(|x| {
                        x.token.is_some() && {
                            matches!(
                                x.token.as_ref().unwrap().get_token(),
                                Some(TokenBase::Identifier(_))
                            )
                        }
                    });
                if is_pathed {
                    Statement::PathedExpression(
                        self.analyze_pathed_expression(first, &children[1..]),
                    )
                } else {
                    match &first.ast_type {
                        AstType::Terminal => self.create_block_from_expression(first),
                        AstType::NonTerminal(NonTerminal::FunctionExpression) => {
                            panic!("This feature is currently unimplemented.")
                        }
                        AstType::NonTerminal(NonTerminal::CompoundExpression) => {
                            Statement::Block(Box::new(self.create_block_from_compound_ast(first)))
                        }
                        AstType::NonTerminal(NonTerminal::ParenthesizedExpression) => {
                            self.create_block_from_expression(&first.children.as_ref().unwrap()[0])
                        }
                        _ => {
                            panic!()
                        }
                    }
                }
            }
            // already ast distinct the each operator priority
            // binary expression section
            AstType::NonTerminal(NonTerminal::AdditiveExpression)
            | AstType::NonTerminal(NonTerminal::MultiplicativeExpression)
            | AstType::NonTerminal(NonTerminal::BitwiseAndExpression)
            | AstType::NonTerminal(NonTerminal::BitwiseOrExpression)
            | AstType::NonTerminal(NonTerminal::BitwiseXorExpression)
            | AstType::NonTerminal(NonTerminal::EqualityExpression)
            | AstType::NonTerminal(NonTerminal::ExponentiationExpression)
            | AstType::NonTerminal(NonTerminal::RelationalExpression)
            | AstType::NonTerminal(NonTerminal::ShiftExpression) => {
                let children = expression_ast.children.as_ref().unwrap();
                let mut bin_exp = BinaryExpressionTree::new();
                for item in children {
                    match item.ast_type {
                        AstType::NonTerminal(_) => {
                            bin_exp.add_expression(self.create_block_from_expression(item));
                        }
                        AstType::Terminal => {
                            let tk = item.token.as_ref().unwrap().get_token().as_ref().unwrap();
                            bin_exp.add_operator(tk.into());
                        }
                    }
                }
                Statement::Expression(Expression::BinaryExpression(bin_exp))
            }
            AstType::NonTerminal(NonTerminal::UnaryExpression) => {
                let children = expression_ast.children.as_ref().unwrap();
                let last_ast = children.last().unwrap();
                let last_expr = self.create_block_from_expression(last_ast);
                let mut una_exp = UnaryExpressionTree::new(last_expr);
                for item in &children[..children.len() - 1] {
                    let tk = item
                        .token
                        .as_ref()
                        .unwrap()
                        .get_token()
                        .as_ref()
                        .unwrap()
                        .into();
                    una_exp.add_opearator(tk);
                }
                Statement::Expression(Expression::UnaryExpression(una_exp))
            }
            _ => panic!(),
        }
    }

    fn analyze_pathed_expression(&self, first: &Ast, rest: &[Ast]) -> PathedExpression {
        let first_stmt = self.create_block_from_expression(first);
        let mut pathed_stmt = PathedExpression::new(first_stmt);
        for item in rest {
            match &item.ast_type {
                AstType::Terminal => {
                    let tk = item.token.as_ref().unwrap().get_token().as_ref().unwrap();
                    if let TokenBase::Identifier(id) = tk {
                        pathed_stmt.push_ident(id.to_string());
                    } else {
                        panic!();
                    }
                }
                AstType::NonTerminal(NonTerminal::Args) => {
                    let mut expressions = Vec::new();
                    let children = item.children.as_ref().unwrap();
                    for item in children {
                        expressions.push(self.create_block_from_expression(item));
                    }
                    pathed_stmt.push_args(expressions);
                }
                _ => panic!(),
            }
        }
        pathed_stmt
    }

    // TODO
    fn create_block_and_extract_from_expression(&self, expression_ast: &Ast) -> Statement {
        self.create_block_from_expression(expression_ast)
    }
}

#[derive(Debug)]
pub struct Block {
    statements: Vec<Statement>,
    expression: Option<Statement>,
}

impl GetType for Block {
    fn get_type(&self) -> Type {
        match &self.expression {
            Some(s) => s.get_type(),
            None => Type::Primitive(PrimitiveType::Void),
        }
    }
}

#[derive(Debug)]
pub enum Statement {
    // a = 3;
    Assignment(Assignment),
    // declaration
    Declaration(Declaration),
    Expression(Expression),
    PathedExpression(PathedExpression),
    Literal(Literal),
    Block(Box<Block>),
    Identifier(String),
}

impl GetType for Statement {
    fn get_type(&self) -> Type {
        match self {
            Statement::Expression(expr) => expr.get_type(),
            Statement::Literal(lit) => lit.get_type(),
            Statement::PathedExpression(p) => p.get_type(),
            Statement::Block(b) => b.get_type(),
            _ => panic!(),
        }
    }
}

#[derive(Debug)]
pub struct Assignment {
    left_identifier: String,          // TODO: replace path
    right_expression: Box<Statement>, // Only allowed Caller and Expression
    left_type: Type,
    right_type: Type,
    assignment_type: AssignmentType,
}

#[derive(Debug)]
pub struct Declaration {
    declaration_type: DeclarationType,
    declared_identifier: String,
    declared_type: Type,
    binded_expression: Box<Statement>,
    binded_type: Type,
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
    UnaryExpression(UnaryExpressionTree),
}

impl GetType for Expression {
    fn get_type(&self) -> Type {
        match &self {
            Expression::BinaryExpression(b) => b.get_type(),
            Expression::UnaryExpression(u) => u.get_type(),
        }
    }
}

#[derive(Debug, Default)]
pub struct BinaryExpressionTree {
    expressions_vec: Vec<(Statement, Type)>,
    operators_vec: Vec<BinaryOperator>,
}

impl BinaryExpressionTree {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_expression(&mut self, stmt: Statement) {
        let ty = stmt.get_type();
        if self.expressions_vec.len() != self.operators_vec.len() {
            panic!()
        }
        self.expressions_vec.push((stmt, ty));
    }

    pub fn add_operator(&mut self, op: BinaryOperator) {
        if self.operators_vec.len() + 1 != self.expressions_vec.len() {
            panic!()
        }
        self.operators_vec.push(op);
    }

    pub fn check(&self) -> Result<(), ()> {
        if self.expressions_vec.len() + 1 == self.operators_vec.len() {
            Ok(())
        } else {
            Err(())
        }
    }
}

impl GetType for BinaryExpressionTree {
    fn get_type(&self) -> Type {
        self.expressions_vec.first().unwrap().1
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
    LogAnd,
    LogOr,
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

impl From<&TokenBase> for BinaryOperator {
    fn from(tb: &TokenBase) -> Self {
        match tb {
            TBR!("+") => BinaryOperator::Add,
            TBR!("/") => BinaryOperator::Div,
            TBR!("*") => BinaryOperator::Mul,
            TBR!("%") => BinaryOperator::Mod,
            TBR!("-") => BinaryOperator::Sub,
            TBR!("^") => BinaryOperator::Xor,
            TBR!("|") => BinaryOperator::Or,
            TBR!("&") => BinaryOperator::And,
            TBR!("&&") => BinaryOperator::LogAnd,
            TBR!("||") => BinaryOperator::LogOr,
            TBR!("**") => BinaryOperator::Exp,
            TBR!(">>") => BinaryOperator::RShift,
            TBR!(">>>") => BinaryOperator::UnRShift,
            TBR!("<<") => BinaryOperator::LShift,
            TBR!("==") => BinaryOperator::Eq,
            TBR!("!=") => BinaryOperator::NotEq,
            TBR!("<") => BinaryOperator::Less,
            TBR!(">") => BinaryOperator::Greater,
            TBR!("<=") => BinaryOperator::LessEq,
            TBR!(">=") => BinaryOperator::GreaterEq,
            _ => panic!(),
        }
    }
}

impl From<TokenBase> for BinaryOperator {
    fn from(tb: TokenBase) -> Self {
        BinaryOperator::from(&tb)
    }
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
    LShift,
    RShift,
    UnRShift,
    LogAnd,
    LogXor,
    LogOr,
}

impl From<&TokenBase> for AssignmentType {
    fn from(tb: &TokenBase) -> Self {
        match tb {
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
        }
    }
}

impl From<TokenBase> for AssignmentType {
    fn from(tb: TokenBase) -> Self {
        AssignmentType::from(&tb)
    }
}

#[derive(Debug)]
pub struct UnaryExpressionTree {
    expression: Box<Statement>,
    expression_type: Type,
    operators_vec: Vec<UnaryOperator>,
}

impl UnaryExpressionTree {
    pub fn new(expr: Statement) -> Self {
        Self {
            expression_type: expr.get_type(),
            expression: Box::new(expr),
            operators_vec: Vec::new(),
        }
    }

    pub fn add_opearator(&mut self, op: UnaryOperator) {
        self.operators_vec.push(op);
    }
}

impl GetType for UnaryExpressionTree {
    fn get_type(&self) -> Type {
        self.expression_type
    }
}

#[derive(Debug)]
pub enum UnaryOperator {
    Plus,
    Minus,
    Not,
    LogNot,
}

impl From<&TokenBase> for UnaryOperator {
    fn from(tb: &TokenBase) -> Self {
        match tb {
            TBR!("+") => UnaryOperator::Plus,
            TBR!("-") => UnaryOperator::Minus,
            TBR!("!") => UnaryOperator::Not,
            TBR!("~") => UnaryOperator::LogNot,
            _ => panic!(),
        }
    }
}

#[derive(Debug)]
pub enum Literal {
    Boolean(bool),
    String(String),
    Number(f64),
}

impl GetType for Literal {
    fn get_type(&self) -> Type {
        match self {
            Literal::Boolean(_) => Type::Primitive(PrimitiveType::Boolean),
            Literal::String(_) => Type::Primitive(PrimitiveType::String),
            Literal::Number(_) => Type::Primitive(PrimitiveType::Number),
        }
    }
}

#[derive(Debug)]
pub struct PathedExpression {
    first: Box<Statement>,
    rests: Vec<ArgsOrIdent>,
    ty: Type,
}

impl PathedExpression {
    pub fn new(first: Statement) -> Self {
        Self {
            first: Box::new(first),
            rests: Vec::new(),
            ty: Type::Unknown,
        }
    }

    pub fn push_args(&mut self, args: Vec<Statement>) {
        self.rests.push(ArgsOrIdent::Args(args));
    }

    pub fn push_ident(&mut self, ident: String) {
        self.rests.push(ArgsOrIdent::Ident(ident));
    }
}

impl GetType for PathedExpression {
    fn get_type(&self) -> Type {
        self.ty
    }
}

#[derive(Debug)]
pub enum ArgsOrIdent {
    Args(Vec<Statement>),
    Ident(String),
}

trait GetType {
    fn get_type(&self) -> Type;
}
