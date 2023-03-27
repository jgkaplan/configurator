use pest::Parser;
use std::collections::HashMap;
use pest::iterators::Pair;
use crate::ast::Expr;
use crate::ast::Expr::*;
use crate::ast::Ident;
use crate::ast::Binop;
use crate::ast::Unop;

#[derive(Parser)]
#[grammar = "grammar.pest"]
pub struct GrammarParser;

// pub fn debug(){
//     println!("{:?}", Rule);
// }

pub fn parse(source: &str) -> std::result::Result<Expr, pest::error::Error<Rule>> {
    let file = GrammarParser::parse(Rule::file, source)?.next().unwrap();
    Ok(parse_expr(file))
}

// TODO: add the rules for parsing types

fn parse_ident(pair: Pair<Rule>) -> Ident {
    pair.as_str().to_string()
}

fn parse_term(pair: Pair<Rule>) -> Expr {
    match pair.as_rule() {
        Rule::list => {
            List(pair.into_inner().map(parse_expr).collect())
        },
        Rule::record => {
            let i = pair.into_inner();
            let mut hashmap: HashMap<Ident, Box<Expr>>= HashMap::new();
            for record_pair in i {
                let mut inner_rules = record_pair.into_inner();
                let k = parse_ident(inner_rules.next().unwrap());
                let v = parse_expr(inner_rules.next().unwrap());
                hashmap.insert(k, Box::new(v));
            }
            Record(hashmap)
        },
        Rule::string => {
            Text(pair.into_inner().next().unwrap().as_str().to_string())
        },
        // Rule::version,
        Rule::number => {
            Number(pair.as_str().parse().unwrap())
        },
        Rule::bool => {
            Boolean(pair.as_str().parse().unwrap())
        },
        // Rule::color,
        Rule::null => Null,
        Rule::ident => {
            Ident(parse_ident(pair))
        },
        Rule::lambda => {
            let mut it = pair.into_inner();
            let x = parse_ident(it.next().unwrap());
            let e = parse_expr(it.next().unwrap());
            Lambda(x, Box::new(e))
        },
        Rule::paren_expr => {
            parse_expr(pair.into_inner().next().unwrap())
        }
        _ => unreachable!()
    }
}

fn parse_expr(pair: Pair<Rule>) -> Expr {
    match pair.as_rule() {
        Rule::let_expr => {
            let mut it = pair.into_inner();
            let ident = it.next().unwrap();
            let ident: Ident = ident.to_string();
            let e1 = it.next().unwrap();
            let e1 = parse_term(e1);
            let e2 = it.next().unwrap();
            let e2 = parse_expr(e2);
            Let(ident, Box::new(e1), Box::new(e2))
        }
        Rule::if_expr => {
            let mut it = pair.into_inner();
            let b = it.next().unwrap();
            let b = parse_term(b);
            let e1 = it.next().unwrap();
            let e1 = parse_term(e1);
            let e2 = it.next().unwrap();
            let e2 = parse_term(e2);
            If(Box::new(b), Box::new(e1), Box::new(e2))
        },
        Rule::function_application => {
            let mut it = pair.into_inner();
            let e1 = it.next().unwrap();
            let e1 = parse_term(e1);
            let e2 = it.next().unwrap();
            let e2 = parse_term(e2);
            App(Box::new(e1), Box::new(e2))
        },
        Rule::binop_expr => {
            let mut it = pair.into_inner();
            let e1 = it.next().unwrap();
            let e1 = parse_term(e1);
            let bop = it.next().unwrap();
            let bop: Binop = match bop.as_str() {
                "+" => Binop::Plus,
                "**" => Binop::Pow,
                "*" => Binop::Times,
                "-" => Binop::Minus,
                "/" => Binop::Div,
                "." => Binop::Access,
                ">" => Binop::Gt,
                "<" => Binop::Lt,
                ">=" => Binop::Gte,
                "<=" => Binop::Lte,
                "==" => Binop::Eq,
                "!=" => Binop::Neq,
                "&&" => Binop::And,
                "||" => Binop::Or,
                "^" => Binop::Xor,
                _ => unreachable!()
            };
            let e2 = it.next().unwrap();
            let e2 = parse_term(e2);
            Binop(Box::new(e1), bop, Box::new(e2))
        },
        Rule::unop_expr => {
            let mut it = pair.into_inner();
            let unop = it.next().unwrap();
            let unop: Unop = match unop.as_str() {
                "-" => Unop::Neg,
                "!" => Unop::Not,
                _ => unreachable!()
            };
            let e = it.next().unwrap();
            let e = parse_term(e);
            Unop(unop, Box::new(e))
        },
        Rule::term => parse_term(pair),
        _ => {
            println!("{:#?}", pair);
            unreachable!()
        }
    }
}