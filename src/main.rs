extern crate pest;
#[macro_use]
extern crate pest_derive;
// use clap::Parser;
use std::fs;
mod cli;
mod parser;
mod ast;

fn main() {
    // let cli = cli::Cli::parse();
    // println!("Hello, world!");
    test();
}

fn test(){
    let unparsed_file = fs::read_to_string("testfile").expect("cannot read file");
    let e = parser::parse(&unparsed_file).expect("unsuccessful parse");
    println!("{:#?}", e);
}
