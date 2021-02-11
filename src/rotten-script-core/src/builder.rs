use crate::{
    lexer::{reserved_word::ReservedWord, token::Token},
    parser::{ast::Ast, ast_type::AstType, non_terminal::NonTerminal},
    semantic_analyzer::project::Project,
};

pub struct Builder<'a> {
    ast: &'a Ast,
    semantic_tree: &'a Project<'a>,
    result: String,
    debug_mode: bool,
    file_name: Option<String>,
    logger: Box<dyn Fn(&str)>,
}

// TODO: unparse using semantic-analyzed tree
impl Builder<'_> {
    // pub fn new(ast: &Ast) -> Builder {
    //     let tree = semantic_analyzer::analyze(vec![(String::from("sample.rots"), ast)]);
    //     Builder {
    //         ast,
    //         semantic_tree: &tree,
    //         result: String::new(),
    //         debug_mode: false,
    //         file_name: None,
    //     }
    // }

    pub fn new<'a>(
        tree: &'a Project<'a>,
        ast: &'a Ast,
        file_name: &str,
        logger: &'static dyn Fn(&str),
    ) -> Builder<'a> {
        Builder {
            semantic_tree: tree,
            ast,
            debug_mode: false,
            result: String::new(),
            file_name: Some(file_name.to_string()),
            logger: Box::new(logger),
        }
    }

    pub fn set_debug_mode(&mut self) {
        self.debug_mode = true;
    }

    pub fn unparse(&mut self) {
        let ast = self.ast; // translation_unit
        if ast.children.is_some() {
            ast.children.as_ref().unwrap().iter().for_each(|x| {
                self.unparse_rec(x, 0);
            });
        }

        let entry = self.semantic_tree.get_entrypoint_func();

        if entry.is_some()
            && &self.semantic_tree.member_map[&(entry.as_ref().unwrap().full_path)].file_name
                == self.file_name.as_ref().unwrap()
        {
            self.result
                .push_str(&format!("\n\n{}();\n", entry.unwrap().name));
        }

        if self.debug_mode {
            self.semantic_tree.print_project_tree();
        }
    }

    fn unparse_rec(&mut self, ast: &Ast, depth: u32) {
        if let AstType::NonTerminal(t) = &ast.ast_type {
            // if t != &NonTerminal::Attribute {
            //     ast.children.as_ref().unwrap().iter().for_each(|x| {
            //         self.unparse_rec(x);
            //     })
            // }
            match t {
                NonTerminal::ConstDeclaration => {
                    self.result.push_str("const ");
                    self.unparse_rec(&ast.children.as_ref().unwrap()[0], depth);
                }
                NonTerminal::LetDeclaration => {
                    self.result.push_str("let ");
                    self.unparse_rec(&ast.children.as_ref().unwrap()[0], depth);
                }
                NonTerminal::DeclarationBody => {
                    let children = ast.children.as_ref().unwrap();
                    self.unparse_rec(&children[0], depth);
                    self.result.push_str(" = ");
                    self.unparse_rec(&children[1], depth);
                    self.result.push(';');
                    self.add_lf_with_depth_space(depth);
                }
                NonTerminal::Expression => {
                    self.unparse_rec(&ast.children.as_ref().unwrap()[0], depth);
                }
                NonTerminal::CallExpression => {
                    let children = ast.children.as_ref().unwrap();
                    let len = children.len();
                    for (i, child) in children.iter().take(len - 1).enumerate() {
                        if i != 0 {
                            self.result.push('.');
                        }
                        self.unparse_rec(child, depth);
                    }
                    self.unparse_rec(&children[len - 1], depth);
                }
                NonTerminal::FunctionExpression => {
                    self.result.push_str("() => ");
                    self.unparse_rec(&ast.children.as_ref().unwrap()[0], depth);
                }
                NonTerminal::CompoundExpression => {
                    self.result.push('{');
                    self.add_lf_with_depth_space(depth + 1);
                    let children = ast.children.as_ref().unwrap();
                    let last_is_statement = children.last().unwrap().ast_type
                        != AstType::NonTerminal(NonTerminal::Expression);
                    for item in children {
                        self.unparse_rec(item, depth + 1);
                    }

                    if last_is_statement {
                        self.result = self.result[0..self.result.len() - 4].to_string();
                    } else {
                        self.add_lf_with_depth_space(depth);
                    }
                    self.result.push('}');
                }
                NonTerminal::Args => {
                    self.result.push('(');
                    let children = ast.children.as_ref().unwrap();
                    for (ind, item) in children.iter().enumerate() {
                        if ind != 0 {
                            self.result.push(',');
                        }
                        self.unparse_rec(item, depth);
                    }
                    self.result.push(')');
                }
                NonTerminal::ExpressionStatement => {
                    self.unparse_rec(&ast.children.as_ref().unwrap()[0], depth);
                    self.result.push(';');
                    self.add_lf_with_depth_space(depth);
                }
                NonTerminal::ExportableConstDeclaration => {
                    let ast_len = ast.children.as_ref().unwrap().len();
                    match ast_len {
                        1 => {
                            self.unparse_rec(&ast.children.as_ref().unwrap()[0], depth);
                        }
                        2 => {
                            self.result.push_str("export ");
                            self.unparse_rec(&ast.children.as_ref().unwrap()[1], depth);
                        }
                        3 => {
                            self.result.push_str("export ");
                            self.unparse_rec(&ast.children.as_ref().unwrap()[2], depth);
                        }
                        _ => panic!(),
                    }
                }
                NonTerminal::ImportDeclaration => {
                    self.unparse_rec(&ast.children.as_ref().unwrap()[0], depth);
                    self.result.push(';');
                    self.add_lf_with_depth_space(depth);
                }
                NonTerminal::DefaultImportDeclaration => {
                    self.result.push_str("import ");
                    self.unparse_rec(&ast.children.as_ref().unwrap()[0], depth);
                    self.result.push_str(" from ");
                    self.unparse_rec(&ast.children.as_ref().unwrap()[1], depth);
                }
                NonTerminal::NamedImportDeclaration => {
                    self.result.push_str("import {");
                    let len = ast.children.as_ref().unwrap().len();
                    for i in 0..len - 1 {
                        self.result.push(' ');
                        self.unparse_rec(&ast.children.as_ref().unwrap()[i], depth);
                        self.result.push(',');
                    }
                    self.result.push_str(" } from ");
                    self.unparse_rec(&ast.children.as_ref().unwrap()[len - 1], depth);
                }
                _ => {}
            }
        } else {
            match ast.token.as_ref().unwrap() {
                Token::String(s) => {
                    self.result.push_str(&format!("\"{}\"", s));
                }
                Token::Number(n) | Token::Identifier(n) => self.result.push_str(n),
                Token::Reserved(r) => {
                    self.result.push_str(&r.to_string());
                    if r == &ReservedWord::LeftCurly || r == &ReservedWord::SemiColon {
                        self.result.push('\n');
                    }
                }
            }
        }
    }

    pub fn get_result(&self) -> String {
        self.result.clone()
    }

    fn add_lf_with_depth_space(&mut self, depth: u32) {
        let space = (0..depth).fold(String::new(), |mut x, _| {
            x.push_str("    ");
            x
        });
        self.result.push('\n');
        self.result.push_str(&space);
    }
}
