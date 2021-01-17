use std::collections::HashMap;

use semantic_tree::SemanticTree;

use crate::{builder::Builder, parser::ast::Ast};

mod member_map;
pub(crate) mod semantic_tree;

pub fn analyze(ast_list: Vec<(String, &'_ Ast)>) -> SemanticTree {
    let mut tree = SemanticTree::new(ast_list);
    tree.analyze();
    tree
}

impl SemanticTree<'_> {
    pub fn call_builder(
        &self,
        is_debug: bool,
        logger: &'static dyn Fn(&str),
    ) -> HashMap<String, String> {
        let mut results = HashMap::new();
        self.ast_list.iter().for_each(|x| {
            let mut builder = Builder::new(self, x.1, &x.0, logger);
            if is_debug {
                builder.set_debug_mode();
            }
            builder.unparse();

            results.insert(x.0.clone(), builder.get_result());
        });
        results
    }
}
