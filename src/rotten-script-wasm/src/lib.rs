mod utils;

use rotten_script_core::{
    builder::Builder,
    parser::{token_stack::TokenStack, Parser},
};
use wasm_bindgen::prelude::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[wasm_bindgen]
pub fn process(file_str: &str) {
    let mut lexer = rotten_script_core::lexer::Lexer::new(file_str, &log);
    if lexer.lex().is_err() {
        log("some err");
    }
    let mut str3 = String::from("[");
    for item in &lexer.tokens {
        str3.push_str(&format!("{}, ", &item));
    }
    // log(&format!("{}]", str.trim_end().trim_matches(',')));

    let token_stack = &mut TokenStack::new(&lexer.tokens);
    let mut parser = Parser::new(token_stack, &log);
    if parser.parse().is_err() {
        log("some err");
    }
    let ast = parser.ast;
    let mut builder = Builder::new(&ast);
    builder.unparse();
    log(&builder.get_result());
}
