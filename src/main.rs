extern crate pest;
#[macro_use]
extern crate pest_derive;
// use clap::Parser;
use std::fs;
use std::collections::HashMap;

mod cli;
mod parser;
mod ast;
// mod typechecker;
mod builtins;
mod interpreter;

fn main() {
    // let cli = cli::Cli::parse();
    // println!("Hello, world!");
    test();
}

fn test() -> Result<(), String>{
    let unparsed_file = fs::read_to_string("testfile").expect("cannot read file");
    let e = parser::parse(&unparsed_file).expect("unsuccessful parse");
    // let te: ast::TypedExpr = typechecker::typecheck(&e, &HashMap::new())?;
    let reduced: ast::Value = interpreter::normalize(&e, &HashMap::new())?;
    println!("{:#?}", reduced);
    Ok(())
}
