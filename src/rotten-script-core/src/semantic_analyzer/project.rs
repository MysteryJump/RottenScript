use std::collections::HashMap;

use crate::parser::ast::Ast;

use super::{
    dependency_graph::DependencyGraph,
    func_info::FuncInfo,
    member_map::{FileMap, MemberMap},
};

pub struct Project<'a> {
    pub member_map: MemberMap,
    pub file_maps: HashMap<String, FileMap<'a>>,
    project_dependency: Option<DependencyGraph>,
    pub ast_list: Vec<(String, &'a Ast)>,
    entry_point_id: Option<u32>,
    func_id_count: u32,
    project_name: String,
}

impl<'a> Project<'a> {
    pub fn new(ast_list: Vec<(String, &'a Ast)>) -> Self {
        Self {
            ast_list,
            file_maps: HashMap::new(),
            member_map: MemberMap::new(),
            project_dependency: None,
            entry_point_id: None,
            func_id_count: 0,
            project_name: "".to_string(),
        }
    }

    pub fn analyze(&mut self) {
        for (path, tunit) in &self.ast_list.clone() {
            let map = FileMap::new(path.clone(), &tunit, self.func_id_count);
            map.members
                .iter()
                .for_each(|(_, f)| self.member_map.insert(f.clone()).unwrap());
            self.func_id_count += map.func_count;
            self.file_maps.insert(map.path.clone(), map);
        }
    }

    pub fn get_entrypoint_func(&self) -> Option<&FuncInfo> {
        if self.entry_point_id.is_none() {
            None
        } else {
            let value = &self.member_map[self.entry_point_id.as_ref().unwrap()];
            Some(value)
        }
    }

    pub(crate) fn print_project_tree(&self) {
        println!(
            "entry point: {}",
            if self.entry_point_id.is_some() {
                &self.member_map[&self.entry_point_id.unwrap()].name
            } else {
                "[none]"
            }
        );
        println!("members: \n{:?}", self.member_map);
    }
}
