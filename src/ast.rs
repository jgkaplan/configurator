use std::collections::HashMap;

pub type Ident = String;

#[derive(Debug)]
pub enum Binop {
    Eq,
    Neq,
    Lt,
    Gt,
    Lte,
    Gte,
    Access,
    Plus,
    Minus,
    Times,
    Div,
    And,
    Or,
    Xor,
    Pow
}

#[derive(Debug)]
pub enum Unop {
    Neg,
    Not
}

#[derive(Debug)]
pub enum Expr {
    Let(Ident, Box<Expr>, Box<Expr>),
    If(Box<Expr>, Box<Expr>, Box<Expr>),
    App(Box<Expr>, Box<Expr>),
    Binop(Box<Expr>, Binop, Box<Expr>),
    Unop(Unop, Box<Expr>),
    Ident(Ident),
    Record(HashMap<Ident, Box<Expr>>),
    List(Vec<Expr>),
    Text(String),
    Number(f64),
    Boolean(bool),
    Lambda(Ident,Box<Expr>),
    Null,
    //Color, Version, Path
}

// pub enum Value {
    
// }

// do we consider a function to be a value
