use pest::Parser;
use std::collections::HashMap;
use pest::iterators::Pair;
use crate::ast::Expr;
use crate::ast::ExprKind::*;
use crate::ast::Type;
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

fn parse_type(pair: Pair<Rule>) -> Type {
    match pair.as_rule() {
        Rule::function_type => {
            let mut it = pair.into_inner();
            let t1 = parse_type(it.next().unwrap());
            let t2 = parse_type(it.next().unwrap());
            Type::Function(Box::new(t1), Box::new(t2))
        },
        Rule::list_type => {
            let t = parse_type(pair.into_inner().next().unwrap());
            Type::List(Box::new(t))
        },
        Rule::record_type => {
            let i = pair.into_inner();
            let mut hashmap: HashMap<Ident, Type>= HashMap::new();
            for record_pair in i {
                let mut inner_rules = record_pair.into_inner();
                let k = parse_ident(inner_rules.next().unwrap());
                let v = parse_type(inner_rules.next().unwrap());
                hashmap.insert(k, v);
            }
            Type::Record(hashmap)
        },
        Rule::alternative_type => {
            let mut it = pair.into_inner();
            let t1 = parse_type(it.next().unwrap());
            let t2 = parse_type(it.next().unwrap());
            Type::Alternative(Box::new(t1), Box::new(t2))
        },
        Rule::user_type => {
            let id = parse_ident(pair.into_inner().next().unwrap());
            Type::Ident(id)
        },
        Rule::builtin_type => {
            match pair.into_inner().next().unwrap().as_str() {
                "Bool" => Type::Bool,
                "Text" => Type::Text,
                "Number" => Type::Number,
                "Natural" => Type::Natural,
                "Integer" => Type::Integer,
                "Real" => Type::Real,
                // "Color" => Type::Color,
                // "Path" => Type::Path,
                "Null" => Type::Null,
                // "Version" => Type::Version, // semver
                "Any" => Type::Any,
                "Type" => Type::Type,
                _ => unreachable!()
            }
        },
        Rule::paren_type => {
            parse_type(pair.into_inner().next().unwrap())
        },
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
            let e1 = parse_expr(e1);
            let e2 = it.next().unwrap();
            let e2 = parse_expr(e2);
            Expr {
                t: None,
                expr: Let(ident, Box::new(e1), Box::new(e2))
            }
            
        }
        Rule::if_expr => {
            let mut it = pair.into_inner();
            let b = it.next().unwrap();
            let b = parse_expr(b);
            let e1 = it.next().unwrap();
            let e1 = parse_expr(e1);
            let e2 = it.next().unwrap();
            let e2 = parse_expr(e2);
            Expr {
                t: None,
                expr: If(Box::new(b), Box::new(e1), Box::new(e2))
            }
        },
        Rule::function_application => {
            let mut it = pair.into_inner();
            let e1 = it.next().unwrap();
            let e1 = parse_expr(e1);
            let e2 = it.next().unwrap();
            let e2 = parse_expr(e2);
            Expr {
                t: None,
                expr: App(Box::new(e1), Box::new(e2))
            }
        },
        Rule::binop_expr => {
            let mut it = pair.into_inner();
            let e1 = it.next().unwrap();
            let e1 = parse_expr(e1);
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
            let e2 = parse_expr(e2);
            Expr {
                t: None,
                expr: Binop(Box::new(e1), bop, Box::new(e2))
            }
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
            let e = parse_expr(e);
            Expr {
                t: None,
                expr: Unop(unop, Box::new(e))
            }
        },
        Rule::term => parse_expr(pair),
        Rule::list => {
            Expr {
                t: None,
                expr: List(pair.into_inner().map(parse_expr).collect())
            }
        },
        Rule::record => {
            let i = pair.into_inner();
            let mut hashmap: HashMap<Ident, Expr>= HashMap::new();
            for record_pair in i {
                let mut inner_rules = record_pair.into_inner();
                let k = parse_ident(inner_rules.next().unwrap());
                let v = parse_expr(inner_rules.next().unwrap());
                hashmap.insert(k, v);
            }
            Expr {
                t: None,
                expr: Record(hashmap)
            }
        },
        Rule::string => Expr {
            t: Some(Type::Text),
            expr: Text(pair.into_inner().next().unwrap().as_str().to_string())
        },
        // Rule::version,
        //TODO: maybe split this parsing so that we can get the type better
        Rule::number => Expr {
            t: Some(Type::Number),
            expr: Number(pair.as_str().parse().unwrap())
        },
        Rule::bool => Expr {
            t: Some(Type::Bool),
            expr: Boolean(pair.as_str().parse().unwrap())
        },
        // Rule::color,
        Rule::null => Expr {
            t: Some(Type::Null),
            expr: Null
        },
        Rule::ident => Expr {
            t: None,
            expr: Ident(parse_ident(pair))
        },
        Rule::lambda => {
            let lam = pair.into_inner().next().unwrap();
            match lam.as_rule() {
                // TODO: these rules don't do anything
                Rule::untyped_lambda => {
                    let mut it = lam.into_inner();
                    let x = parse_ident(it.next().unwrap());
                    let e = parse_expr(it.next().unwrap());
                    Expr {
                        t: None,
                        expr: Lambda(x, Box::new(e))
                    }
                },
                Rule::typed_lambda => {
                    let mut it = lam.into_inner();
                    let x = parse_ident(it.next().unwrap());
                    let t = parse_type(it.next().unwrap());
                    let e = parse_expr(it.next().unwrap());
                    Expr {
                        t: None,
                        expr: Lambda(x, Box::new(e))
                    }
                },
                _ => unreachable!()
            }
        },
        Rule::paren_expr => {
            parse_expr(pair.into_inner().next().unwrap())
        },
        _ => {
            println!("{:#?}", pair); //TODO: remove this
            unreachable!()
        }
    }
}