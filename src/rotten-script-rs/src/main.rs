use std::fs;

use rotten_script_core::{
    builder::Builder,
    lexer::Lexer,
    parser::{token_stack::TokenStack, Parser},
    semantic_analyzer::analyze,
};

fn main() {
    let content =
        fs::read_to_string("rotten-script-wasm/node-project/sample/sample1.rots").unwrap();
    let mut lexer = Lexer::new(&content, &logger);
    lexer.lex().unwrap();
    let token_stack = &mut TokenStack::new(&lexer.tokens);
    let mut parser = Parser::new(token_stack, &logger);
    parser.parse().unwrap();
    let ast = parser.ast;
    let tree = analyze(vec![("sample1.rots".to_string(), &ast)]);
    let result = tree.call_builder(false);

    for item in result {
        println!("// {}\n", item.0);
        println!("{}\n", item.1);
    }
}

fn logger(line: &str) {
    println!("{}", line);
}
