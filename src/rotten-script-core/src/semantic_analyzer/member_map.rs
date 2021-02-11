use crate::parser::{ast::Ast, ast_type::AstType, non_terminal::NonTerminal};

use super::{
    func::Func,
    func_info::{ExportedType, FuncInfo},
};
use std::{collections::HashMap, fmt::Debug, ops::Index, rc::Rc};
pub struct MemberMap {
    // key: func_id, value: FuncInfo
    members: HashMap<u32, Rc<FuncInfo>>,
    // key: func_name(full), value: func_id
    func_path_to_func_id_map: HashMap<String, u32>,
    // key: file_path, value: func_id
    func_ids_of_files: HashMap<String, Vec<u32>>,
    count: usize,
}

#[derive(Debug)]
pub struct Import {
    import_path: String,
    import_member: Vec<String>,
}

pub struct FileMap<'a> {
    imports: Vec<Import>,
    exports: Vec<String>,
    pub path: String,
    pub members: HashMap<String, Rc<FuncInfo>>,
    pub functions: HashMap<String, Rc<Func<'a>>>,
    file_name: String,
    pub func_count: u32,
    pub ast: &'a Ast,
}

impl<'a> FileMap<'a> {
    pub fn new(path: String, translation_unit: &'a Ast, cumulative_func_count: u32) -> Self {
        let mut count = 0;
        let mut attributes = Vec::new();
        let mut entry_point_id = None;
        let mut imports = Vec::new();
        let mut exports = Vec::new();
        let mut funcs = HashMap::new();

        let mut map = HashMap::new();
        for ast in translation_unit.children.as_ref().unwrap() {
            if let AstType::NonTerminal(nt) = &ast.ast_type {
                match nt {
                    NonTerminal::ImportDeclaration => {
                        match &ast.children.as_ref().unwrap()[0].ast_type {
                            AstType::NonTerminal(nt)
                                if nt == &NonTerminal::DefaultImportDeclaration =>
                            {
                                panic!("does not support default import currently")
                            }
                            AstType::NonTerminal(nt)
                                if nt == &NonTerminal::NamedImportDeclaration =>
                            {
                                let named = &ast.children.as_ref().unwrap()[0];
                                let named_children = named.children.as_ref().unwrap();
                                let import_members = named_children[..named_children.len() - 1]
                                    .iter()
                                    .map(|x| x.token.clone().unwrap().to_string())
                                    .collect::<Vec<_>>();
                                let from_file =
                                    named_children.last().unwrap().token.clone().unwrap();
                                imports.push(Import {
                                    import_member: import_members,
                                    import_path: from_file.to_string(),
                                });
                            }
                            AstType::NonTerminal(_) | AstType::Terminal => panic!(),
                        }
                    }
                    NonTerminal::Attribute => {
                        attributes.push(
                            ast.children.as_ref().unwrap()[0]
                                .token
                                .as_ref()
                                .unwrap()
                                .to_string(),
                        );
                    }
                    NonTerminal::ExportableConstDeclaration => {
                        let has_export;
                        let has_default;
                        let ast_len = ast.children.as_ref().unwrap().len();
                        if ast_len >= 2 {
                            has_export = true;
                            if ast_len == 3 {
                                has_default = true;
                            } else {
                                has_default = false;
                            }
                        } else {
                            has_export = false;
                            has_default = false;
                        }
                        let const_declar_body = &ast.children.as_ref().unwrap()[if has_default {
                            2
                        } else if has_export {
                            1
                        } else {
                            0
                        }];

                        let declar_body = &const_declar_body.children.as_ref().unwrap()[0];
                        let func_name = declar_body.children.as_ref().unwrap()[0]
                            .token
                            .as_ref()
                            .unwrap()
                            .to_string();
                        if has_export {
                            exports.push(func_name.clone());
                        }
                        let func_info = FuncInfo::new(
                            func_name,
                            path.to_string(),
                            count + cumulative_func_count,
                            attributes.clone(),
                            if has_default {
                                ExportedType::DefaultExport
                            } else if has_export {
                                ExportedType::Export
                            } else {
                                ExportedType::None
                            },
                        );
                        count += 1;
                        attributes.clear();
                        if func_info.is_entry {
                            if entry_point_id.is_none() {
                                entry_point_id = Some(func_info.func_id);
                            } else {
                                panic!("found multiple entrypoint")
                            }
                        }
                        let func_info_rc = Rc::new(func_info);
                        let func = Func::new(declar_body, func_info_rc.clone());

                        map.insert(func_info_rc.name.clone(), func_info_rc.clone());
                        funcs.insert(func_info_rc.clone().name.clone(), Rc::new(func));
                    }
                    _ => panic!(),
                }
            } else {
                panic!()
            }
        }
        let file_name = Self::extract_file_name_from_full_path(path.clone());

        Self {
            functions: funcs,
            imports,
            exports,
            path,
            members: map,
            file_name,
            func_count: count,
            ast: translation_unit,
        }
    }

    fn extract_file_name_from_full_path(full_path: String) -> String {
        let ind = full_path.rfind(|x| x == '/' || x == '\\').unwrap();
        full_path[ind + 1..].to_string()
    }
}

impl Index<&u32> for MemberMap {
    type Output = FuncInfo;

    fn index(&self, index: &u32) -> &Self::Output {
        &self.members[index]
    }
}

/// Index for func_full_path
impl Index<&String> for MemberMap {
    type Output = FuncInfo;

    fn index(&self, index: &String) -> &Self::Output {
        let func_id = &self.func_path_to_func_id_map[index];
        &self.members[func_id]
    }
}

impl Debug for MemberMap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(self.members.values()).finish()
    }
}

impl Default for MemberMap {
    fn default() -> Self {
        MemberMap {
            members: HashMap::new(),
            func_path_to_func_id_map: HashMap::new(),
            func_ids_of_files: HashMap::new(),
            count: 0,
        }
    }
}

impl<'a> MemberMap {
    pub fn new() -> MemberMap {
        MemberMap::default()
    }
    pub fn len(&self) -> usize {
        self.count
    }

    pub fn insert(&mut self, func: Rc<FuncInfo>) -> Result<(), ()> {
        let func_name = func.full_path.clone();

        self.func_path_to_func_id_map
            .insert(func_name, func.func_id);

        if self.func_path_to_func_id_map.contains_key(&func.file_name) {
            self.func_ids_of_files
                .insert(func.file_name.clone(), vec![func.func_id]);
        } else {
            match self.func_ids_of_files.get_mut(&func.file_name) {
                Some(x) => x.push(func.func_id),
                None => {
                    self.func_ids_of_files
                        .insert(func.file_name.clone(), vec![func.func_id]);
                }
            }
        }
        let result = self.members.insert(func.func_id, func);
        if result.is_some() {
            Err(())
        } else {
            self.count += 1;
            Ok(())
        }
    }

    pub fn get_from_file_name(&self, file_name: &str) -> Vec<Rc<FuncInfo>> {
        let ids = &self.func_ids_of_files[file_name];
        let infos = ids
            .iter()
            .map(|x| self.members[x].clone())
            .collect::<Vec<_>>();
        infos
    }
}
