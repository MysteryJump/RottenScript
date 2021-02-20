use std::{env, fs, io};

use rotten_script_core::{
    lexer::Lexer,
    parser::{token_stack::TokenStack, Parser},
    semantic_analyzer::analyze,
};

const HELP_TEXT: &str = "Usage: rotc [PROJECT-PATH]";

fn main() {
    let args = env::args().collect::<Vec<_>>();
    let path = if args.len() >= 2 {
        args[1].clone()
    } else {
        println!("{}", HELP_TEXT);
        return;
    };

    let files = get_directory_files_recursive(&path).unwrap();

    let content_file_pair = files
        .iter()
        .map(|x| (x.to_string(), fs::read_to_string(x).unwrap()))
        .collect::<Vec<_>>();

    let mut has_error = false;
    // TODO: Replace for stmt
    let ast_pairs = content_file_pair
        .iter()
        .map(|x| {
            let mut lexer = Lexer::new(&x.1, &x.0);
            let lexer_result = lexer.lex();
            match lexer_result {
                Ok(_) => {}
                Err(e) => {
                    println!("{}", e);
                    has_error = true;
                }
            }
            // println!("{:?}", lexer.tokens);
            let token_stack = &mut TokenStack::new(&lexer.tokens);
            let mut parser = Parser::new(token_stack);
            match parser.parse() {
                Ok(_) => {}
                Err(e) => {
                    println!("{}", e);
                    has_error = true;
                }
            }
            println!("{}", parser.ast.unparse());
            (x.0.clone(), parser.ast)
        })
        .collect::<Vec<_>>();
    if has_error {
        return;
    }

    let project = analyze(ast_pairs.iter().map(|x| (x.0.clone(), &x.1)).collect());
    let result = project.call_builder(true);

    for item in result {
        println!("// {}", item.0);
        println!("{}", item.1);
    }
}

fn get_directory_files_recursive(path: &str) -> io::Result<Vec<String>> {
    let mut files = Vec::new();
    let dir = fs::read_dir(path)?;
    for entry in dir {
        let entry = entry?;
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            files.append(&mut get_directory_files_recursive(&format!(
                "{}/{}",
                path,
                entry.file_name().to_str().unwrap()
            ))?);
        } else if file_type.is_file() && entry.file_name().to_str().unwrap().ends_with(".rots") {
            files.push(format!(
                "{}/{}",
                path,
                entry.file_name().to_str().as_ref().unwrap().to_string()
            ));
        }
    }
    Ok(files)
}
