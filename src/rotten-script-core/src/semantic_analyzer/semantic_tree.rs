use std::collections::HashMap;

use crate::parser::{ast::Ast, ast_type::AstType, non_terminal::NonTerminal};
#[allow(dead_code)]
pub struct SemanticTree<'a> {
    pub ast_list: Vec<(String, &'a Ast)>,
    ir_tree: String,
    members: HashMap<i32, FuncInfo>,
    entry_point_id: Option<i32>,
    func_id_count: i32,
}

#[derive(Debug)]
pub struct FuncInfo {
    pub name: String,
    pub full_path: String,
    exported_type: ExportedType,
    args: Arguments,
    return_type: Type,
    pub func_id: i32,
    is_entry: bool,
    attributes: Vec<String>,
}

impl FuncInfo {
    pub fn new(
        name: String,
        path: String,
        id: i32,
        attributes: Vec<String>,
        exported_type: ExportedType,
    ) -> FuncInfo {
        let is_entry = attributes.iter().any(|x| x == &String::from("EntryPoint"));
        FuncInfo {
            name,
            full_path: path,
            exported_type,
            args: Arguments {
                arguments: Vec::new(),
            },
            return_type: Type::Primitive(PrimitiveType::Void),
            func_id: id,
            is_entry,
            attributes,
        }
    }
}

#[derive(Debug)]
pub enum ExportedType {
    Export,
    DefaultExport,
    None,
}

#[derive(Debug)]
struct Arguments {
    arguments: Vec<(String, Type)>,
}
#[allow(dead_code)]
#[derive(Debug)]
enum Type {
    Primitive(PrimitiveType),
    Object,
}
#[allow(dead_code)]
#[derive(Debug)]
enum PrimitiveType {
    Number,
    String,
    Boolean,
    Void,
}

impl SemanticTree<'_> {
    pub fn new(ast_list: Vec<(String, &'_ Ast)>) -> SemanticTree<'_> {
        SemanticTree {
            ast_list,
            ir_tree: String::new(),
            members: HashMap::new(),
            entry_point_id: None,
            func_id_count: 0,
        }
    }

    pub fn analyze(&mut self) {
        for (path, tunit) in self.ast_list.clone() {
            // let translation_unit = &tunit.children.as_ref().unwrap()[0];
            self.construct_func_info_map(tunit, &path);
        }
    }

    fn construct_func_info_map<'a>(
        &mut self,
        translation_unit: &'a Ast,
        path: &str,
        // map: &'a mut HashMap<i32, FuncInfo>,
    ) {
        let mut count = 0;
        let mut attributes = Vec::new();
        // let mut map = HashMap::new();
        for ast in translation_unit.children.as_ref().unwrap() {
            if let AstType::NonTerminal(nt) = &ast.ast_type {
                match nt {
                    NonTerminal::ImportDeclaration => {}
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
                        let func_info = FuncInfo::new(
                            func_name,
                            path.to_string(),
                            count + self.func_id_count,
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
                            if self.entry_point_id.is_none() {
                                self.entry_point_id = Some(func_info.func_id);
                            } else {
                                panic!("found multiple entrypoint")
                            }
                        }
                        self.members.insert(func_info.func_id, func_info);
                    }
                    _ => panic!(),
                }
            } else {
                panic!()
            }
        }
        self.func_id_count += count;
    }

    pub fn get_entrypoint_func_name(&self) -> Option<String> {
        if self.entry_point_id.is_none() {
            None
        } else {
            let value = &self.members[self.entry_point_id.as_ref().unwrap()];
            Some(value.name.clone())
        }
    }

    pub(crate) fn print_semantic_tree(&self) {
        println!(
            "entry point: {}",
            if self.entry_point_id.is_some() {
                &self.members[&self.entry_point_id.unwrap()].name
            } else {
                "[none]"
            }
        );
        println!("members: \n{:?}", self.members);
    }
}
