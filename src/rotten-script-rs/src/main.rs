use std::fs;

use rotten_script_core::{
    builder::Builder,
    lexer::Lexer,
    parser::{token_stack::TokenStack, Parser},
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
    let mut builder = Builder::new(&ast);
    builder.set_debug_mode();
    builder.unparse();
    println!("// above output is for debug\n\n");
    println!("{}", builder.get_result());
}

fn logger(line: &str) {
    println!("{}", line);
}
