use pest::Parser;
use std::collections::HashMap;
use pest::iterators::Pair;
use crate::ast::{Expr, ExprKind::*, Type, Ident, Bop, Uop};

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
            let id = parse_ident(pair);
            Type::Ident(id)
        },
        Rule::builtin_type => {
            match pair.as_str() {
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
        _ => {
            unreachable!()
        }
    }
}

fn parse_expr(pair: Pair<Rule>) -> Expr {
    match pair.as_rule() {
        Rule::let_expr => {
            let l = pair.into_inner().next().unwrap();
            let istyped = l.as_rule() == Rule::typed_let;
            let mut it = l.into_inner();
            let ident = parse_ident(it.next().unwrap());
            let t = istyped.then(|| parse_type(it.next().unwrap()));
            let e1 = it.next().unwrap();
            let e1 = parse_expr(e1);
            let e2 = it.next().unwrap();
            let e2 = parse_expr(e2);
            Expr {
                t: None,
                expr: Let(ident, t, Box::new(e1), Box::new(e2))
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
            let bop: Bop = match bop.as_str() {
                "+" => Bop::Plus,
                "**" => Bop::Pow,
                "*" => Bop::Times,
                "-" => Bop::Minus,
                "/" => Bop::Div,
                ">" => Bop::Gt,
                "<" => Bop::Lt,
                ">=" => Bop::Gte,
                "<=" => Bop::Lte,
                "==" => Bop::Eq,
                "!=" => Bop::Neq,
                "&&" => Bop::And,
                "||" => Bop::Or,
                "^" => Bop::Xor,
                _ => unreachable!()
            };
            let e2 = it.next().unwrap();
            let e2 = parse_expr(e2);
            Expr {
                t: None,
                expr: Binop(Box::new(e1), bop, Box::new(e2))
            }
        },
        Rule::access => {
            let it = pair.into_inner().next().unwrap();
            match it.as_rule() {
                Rule::dot_access => {
                    let mut it = it.into_inner();
                    let e1 = it.next().unwrap();
                    let e1 = parse_expr(e1);
        
                    let id = it.next().unwrap();
                    let id = parse_ident(id);
                    Expr {
                        t: None,
                        expr: Binop(Box::new(e1), Bop::Access, Box::new(Expr{t: None, expr: Ident(id)}))
                    }
                },
                Rule::arr_access => {
                    let mut it = it.into_inner();
                    let e1 = it.next().unwrap();
                    let e1 = parse_expr(e1);
        
                    let e2 = it.next().unwrap();
                    let e2 = parse_expr(e2);
                    Expr {
                        t: None,
                        expr: Binop(Box::new(e1), Bop::Access, Box::new(e2))
                    }
                },
                _ => unreachable!()
            }
            
        },
        Rule::unop_expr => {
            let mut it = pair.into_inner();
            let unop = it.next().unwrap();
            let unop: Uop = match unop.as_str() {
                "-" => Uop::Neg,
                "!" => Uop::Not,
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
        Rule::number => {
            let inner = pair.into_inner().next().unwrap();
            match inner.as_rule() {
                Rule::float_n => Expr {
                    t: Some(Type::Number),
                    expr: Float(inner.as_str().parse().unwrap())
                },
                Rule::integer_n => Expr {
                    t: Some(Type::Integer),
                    expr: Int(inner.as_str().parse().unwrap())
                },
                _ => unreachable!()
            }
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
            let istyped = lam.as_rule() == Rule::typed_lambda;
            let mut it = lam.into_inner();
            let x = parse_ident(it.next().unwrap());
            let t = istyped.then(|| parse_type(it.next().unwrap()));
            let e = parse_expr(it.next().unwrap());
            Expr {
                t: None,
                expr: Lambda(x, t, Box::new(e))
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