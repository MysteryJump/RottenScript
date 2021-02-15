use crate::lexer::token::Token;

use super::{ast_type::AstType, non_terminal::NonTerminal};

pub struct Ast {
    pub children: Option<Vec<Ast>>,
    pub token: Option<Token>,
    pub ast_type: AstType,
    invalid_ast: bool,
    invalid_root_ast: bool,
}

impl Ast {
    pub fn new_leaf(token: Token) -> Ast {
        Ast {
            children: None,
            token: Some(token),
            ast_type: AstType::Terminal,
            invalid_ast: false,
            invalid_root_ast: false,
        }
    }

    pub fn new_node_with_leaves(node_type: NonTerminal, children: Vec<Ast>) -> Ast {
        let has_invalid_ast = children.iter().any(|x| x.invalid_ast);
        Ast {
            children: Some(children),
            token: None,
            ast_type: AstType::NonTerminal(node_type),
            invalid_ast: has_invalid_ast,
            invalid_root_ast: false,
        }
    }

    pub fn add_child(&mut self, ast: Ast) {
        if let Some(c) = &mut self.children {
            c.push(ast);
        } else {
            panic!();
        }
    }

    pub fn add_children(&mut self, ast: Vec<Ast>) {
        for item in ast {
            self.add_child(item);
        }
    }

    pub fn is_invalid(&self) -> bool {
        self.invalid_ast
    }

    pub fn set_invalid(&mut self) {
        self.invalid_ast = true;
        self.invalid_root_ast = true;
    }

    pub fn unparse(&self) -> String {
        self.unparse_with_depth(self, 0)
    }

    fn unparse_with_depth(&self, ast: &Ast, depth: usize) -> String {
        let mut unparsed = String::new();
        let mut space = String::new();
        let space = std::iter::repeat(' ')
            .take(depth)
            .fold(&mut space, |current, next| {
                current.push(next);
                current
            });
        if ast.ast_type == AstType::Terminal {
            let tk = ast.token.clone();
            unparsed.push_str(&space);
            unparsed.push_str(&format!("{}\n", tk.unwrap().to_string()));
        } else {
            unparsed.push_str(&format!("{}{:?}\n", space, ast.ast_type));
            if let Some(c) = &ast.children {
                for item in c {
                    unparsed.push_str(&self.unparse_with_depth(item, depth + 2));
                }
            }
        }
        unparsed
    }
}
