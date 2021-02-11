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

    println!(
        "{}",
        env::current_dir().unwrap().as_path().to_str().unwrap()
    );
    println!("{}", path);

    let files = get_directory_files_recursive(&path).unwrap();

    let content_file_pair = files
        .iter()
        .map(|x| (x.to_string(), fs::read_to_string(x).unwrap()))
        .collect::<Vec<_>>();

    let ast_pairs = content_file_pair
        .iter()
        .map(|x| {
            let mut lexer = Lexer::new(&x.1, &logger);
            lexer.lex().unwrap();
            let token_stack = &mut TokenStack::new(&lexer.tokens);
            let mut parser = Parser::new(token_stack, &logger);
            parser.parse().unwrap();
            (x.0.clone(), parser.ast)
        })
        .collect::<Vec<_>>();

    let project = analyze(ast_pairs.iter().map(|x| (x.0.clone(), &x.1)).collect());
    let result = project.call_builder(true, &logger);

    for item in result {
        println!("// {}\n", item.0);
        println!("{}\n", item.1);
    }
}

fn logger(line: &str) {
    println!("{}", line);
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
