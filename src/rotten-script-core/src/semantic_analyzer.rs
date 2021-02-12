use std::collections::HashMap;

use project::Project;

use crate::{builder::Builder, parser::ast::Ast};

mod dependency_graph;
mod file_map;
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
        let mut builder = Builder::new(&self, logger);
        if is_debug {
            builder.set_debug_mode();
        }
        builder.unparse()
    }
}
