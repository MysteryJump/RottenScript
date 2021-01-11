use semantic_tree::SemanticTree;

use crate::parser::ast::Ast;

pub(crate) mod semantic_tree;

pub fn analyze(ast_list: Vec<(String, &'_ Ast)>) -> SemanticTree {
    let mut tree = SemanticTree::new(ast_list);
    tree.analyze();
    tree
}
