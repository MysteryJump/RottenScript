use std::collections::HashMap;

use project::Project;
// use semantic_tree::SemanticTree;

use crate::{builder::Builder, parser::ast::Ast};

mod dependency_graph;
mod func;
pub(crate) mod func_info;
mod member_map;
pub(crate) mod project;

pub fn analyze(ast_list: Vec<(String, &'_ Ast)>) -> Project {
    let mut tree = Project::new(ast_list);
    tree.analyze();
    tree
}

impl Project<'_> {
    pub fn call_builder(
        &self,
        is_debug: bool,
        logger: &'static dyn Fn(&str),
    ) -> HashMap<String, String> {
        let mut results = HashMap::new();
        let mut is_first = true;
        self.file_maps.iter().for_each(|x| {
            let mut builder = Builder::new(&self, x.1.ast, &x.0, logger);
            if is_debug && is_first {
                builder.set_debug_mode();
                is_first = false;
            }
            builder.unparse();

            results.insert(x.0.clone(), builder.get_result());
        });
        results
    }
}
