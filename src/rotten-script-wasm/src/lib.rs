mod utils;

use once_cell::sync::Lazy;
// use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::{collections::HashMap, sync::Mutex};

use rotten_script_core::{
    lexer::Lexer,
    parser::{token_stack::TokenStack, Parser},
    semantic_analyzer::analyze,
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
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn log_string(s: String);
}

#[wasm_bindgen]
pub fn process(file_str: &str) {
    rotten_script_core::LOGGER.lock().unwrap().logger = Some(Box::new(log_string));
    let mut lexer = rotten_script_core::lexer::Lexer::new(file_str, "");
    if lexer.lex().is_err() {
        log("some err from lexer");
    };

    let mut str3 = String::from("[");
    for item in &lexer.tokens {
        str3.push_str(&format!("{}, ", &item));
    }
    // log(&format!("{}]", str.trim_end().trim_matches(',')));

    let token_stack = &mut TokenStack::new(&lexer.tokens);
    let mut parser = Parser::new(token_stack);
    if parser.parse().is_err() {
        log("some err from parser");
    }
    let ast = parser.ast;
    let tree = analyze(vec![("sample1.rots".to_string(), &ast)]);
    let result = tree.call_builder(false);

    for item in result {
        log(&format!("// {}\n", item.0));
        log(&format!("{}\n", item.1));
    }
}

#[wasm_bindgen]
pub fn execute_processing() -> bool {
    rotten_script_core::LOGGER.lock().unwrap().logger = Some(Box::new(log_string));
    let files = &SOURCES.lock().unwrap().file_pairs;
    let mut has_error = false;
    let asts = files
        .iter()
        .map(|x| {
            let mut lexer = Lexer::new(&x.1, &x.0);
            match lexer.lex() {
                Ok(_) => {}
                Err(e) => {
                    log_string(format!("{}", e));
                    has_error = true;
                }
            }
            let token_stack = &mut TokenStack::new(&lexer.tokens);
            let mut parser = Parser::new(token_stack);
            match parser.parse() {
                Ok(_) => {}
                Err(e) => {
                    log_string(format!("{}", e));
                    has_error = true;
                }
            }
            (x.0.clone(), parser.ast)
        })
        .collect::<Vec<_>>();
    if has_error {
        false
    } else {
        let tree = analyze(asts.iter().map(|x| (x.0.clone(), &x.1)).collect());
        RESULTS.lock().unwrap().file_pairs = Some(tree.call_builder(false));
        true
    }
}

#[wasm_bindgen]
pub fn eject_sourcecode(file_path: &str) -> String {
    let ts = RESULTS.lock().unwrap();
    let map = ts.file_pairs.as_ref().unwrap();
    if let Some(t) = map.get(file_path) {
        t.clone()
    } else {
        "".to_string()
    }
}

#[wasm_bindgen]
pub fn add_file(full_path: &str, file_strs: &str) {
    SOURCES
        .lock()
        .unwrap()
        .file_pairs
        .push((String::from(full_path), String::from(file_strs)));
}

#[derive(Debug)]
pub struct SourceFiles {
    pub file_pairs: Vec<(String, String)>,
}

#[derive(Debug)]
pub struct TranspiledSources {
    pub file_pairs: Option<HashMap<String, String>>,
}

static SOURCES: Lazy<Mutex<SourceFiles>> = Lazy::new(|| {
    Mutex::new(SourceFiles {
        file_pairs: Vec::new(),
    })
});

static RESULTS: Lazy<Mutex<TranspiledSources>> =
    Lazy::new(|| Mutex::new(TranspiledSources { file_pairs: None }));
