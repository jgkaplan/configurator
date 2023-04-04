use crate::ast::{Expr, TypedExpr, ExprKind, Type, Bop, JoinSemiLattice, Ident};
use std::collections::HashMap;

// TODO: account for type aliases

pub fn typecheck(expr: &Expr, bindings: &HashMap<Ident, Type>) -> Result<TypedExpr, String> {
    use ExprKind::*;
    match &expr.expr {
        // TODO this won't work if let isn't typed
        Let(id, op_t, e1, e2) => {
            let te1 = typecheck(&*e1, bindings)?;
            match (op_t, te1.t) {
                (None, _) => (),
                (Some(t1), t2) if t1 == t2 => (),
                (Some(t1), t2) => return Err(format!("Declared type {t1:#?} of {id} does not match actual type of expression, {t2:#?}"))
            }
            let new_defs = bindings.clone();
            new_defs.insert(id, te1.t);
            let te2 = typecheck(&*e2, &new_defs)?;
            Ok(TypedExpr {
                t: te2.t,
                expr: Let(id, op_t, Box::new(te1), Box::new(te2))
            })
        },
        If(b, iftrue, iffalse) => {
            let tb = typecheck(&*b, bindings)?;
            if tb.t != Type::Bool {
                return Err(format!("Boolean expected in condition of if expression. Instead got {:#?}", tb.t));
            }
            let t_true = typecheck(&*iftrue, bindings)?;
            let t_false = typecheck(&*iffalse, bindings)?;
            if t_true.t != t_false.t {
                //TODO: this might not be what we want. For instance, if we have record types, we might want to use an alternative type. {x}|{x,y} instead of {x}
                if t_true.t < t_false.t{
                    // types are not equal, but we can use the more specific one.
                    return Ok(TypedExpr{
                        t: t_true.t,
                        expr: If(Box::new(tb), Box::new(t_true), Box::new(t_false))
                    });
                } else if t_true.t > t_false.t {
                    // types are not equal, but we can use the more specific one.
                    return Ok(TypedExpr{
                        t: t_false.t,
                        expr: If(Box::new(tb), Box::new(t_true), Box::new(t_false))
                    });
                } else {
                    // Todo: should this be an error, or should we use the alternative type?
                    // return Err("Branches of if expression do not have the same type. True branch had type {t_true.t} and False branch had type {t_false.t}");
                    return Ok(TypedExpr{
                        t: Type::Alternative(Box::new(t_true.t), Box::new(t_false.t)),
                        expr: If(Box::new(tb), Box::new(t_true), Box::new(t_false))
                    });
                }
            } else {
                //ok
                return Ok(TypedExpr{
                    t: t_true.t,
                    expr: If(Box::new(tb), Box::new(t_true), Box::new(t_false))
                });
            }
        },
        App(e1, e2) => {
            let t1 = typecheck(&*e1, bindings)?;
            let t2 = typecheck(&*e2, bindings)?;
            if let Type::Function(x,outtype) = t1.t {
                if *x == t2.t {
                    //ok
                    return Ok(TypedExpr { 
                        t: *outtype,
                        expr: App(Box::new(t1), Box::new(t2))
                    })
                } else {
                    return Err(format!("Argument to function application does not have expected type {:#?}. It instead has type {:#?}",x,t2.t));
                }
            } else {
                return Err(format!("First expression in function application is not a function. It has type {:#?}",t1.t));
            }
        },
        Binop(e1, bop, e2) => {
            let t1 = typecheck(&*e1, bindings)?;
            let t2 = typecheck(&*e2, bindings)?;
            let new_ex = Binop(Box::new(t1), bop, Box::new(t2));
            let new_type = match (bop, t1.t, t2.t) {
                //TODO: can't compare all types. restrict this
                (Bop::Eq, a, b) if a <= b || a >= b => Some(Type::Bool),
                (Bop::Neq, a, b) if a <= b || a >= b => Some(Type::Bool),
                (Bop::Lt, a, b) 
                | (Bop::Gt, a, b) 
                | (Bop::Lte, a, b)
                | (Bop::Gte, a, b) if a <= Type::Number && b <= Type::Number 
                    || a == Type::Text && b == Type::Text => Some(Type::Bool),
                // can only say that it's an Integer, not a natural
                (Bop::Minus, a, b) if a <= Type::Number && b <= Type::Number => {
                    if Type::Integer >= a && Type::Integer >= b {
                        Some(Type::Integer)
                    } else if a >= b {
                        Some(a)
                    } else {
                        Some(b)
                    }
                },
                // Pow is only definitely a natural if both arguments are natural
                (Bop::Pow, Type::Natural, Type::Natural) => Some(Type::Natural),
                // Pow with any other numbers could be just a real
                (Bop::Pow, a, b) if a <= Type::Number && b <= Type::Number => Some(Type::Number),
                // Div has no guarantees. TODO: decide if this should be option type for div by 0
                (Bop::Div, a, b) if a <= Type::Number && b <= Type::Number => Some(Type::Number),
                (Bop::Times, a, b)
                | (Bop::Plus, a, b) if a <= Type::Number && b <= Type::Number => {
                    if a >= b {
                        Some(a)
                    } else {
                        Some(b)
                    }
                },
                // Bitwise operations on integers or naturals
                (Bop::And, a, b)
                | (Bop::Or, a, b)
                | (Bop::Xor, a, b) if a <= Type::Integer && b <= Type::Integer => {
                    if a >= b {
                        Some(a)
                    } else {
                        Some(b)
                    }
                },
                (Bop::And, Type::Bool, Type::Bool)
                | (Bop::Or, Type::Bool, Type::Bool)
                | (Bop::Xor, Type::Bool, Type::Bool)  => Some(Type::Bool),
                //Joining lists
                (Bop::Plus, Type::List(a), Type::List(b)) if *a <= *b => Some(*b),
                (Bop::Plus, Type::List(a), Type::List(b)) if *a >= *b => Some(*a),
                (Bop::Plus, Type::List(a), Type::List(b)) => Some(Type::Alternative(a,b)),
                //Joining text
                (Bop::Plus, Type::Text, Type::Text) => Some(Type::Text),
                //multiplying text
                (Bop::Times, Type::Text, Type::Natural) => Some(Type::Text),
                //When joining records, prefer the variable in the second one if there's overlap
                (Bop::Plus, Type::Record(hm1), Type::Record(hm2)) => {
                    let mut joined_hashmap = hm1.clone();
                    joined_hashmap.extend(hm2);
                    Some(Type::Record(joined_hashmap))
                }
                // TODO: Access for text, lists, records
                // list access. We would want this to be a natural, but maybe can't guarantee it
                (Bop::Access, Type::List(a), Type::Integer) => Some(*a),
                // String access
                (Bop::Access, Type::Text, Type::Integer) => Some(Type::Text),
                // Record access
                (Bop::Access, Type::Record(hm), _) => {
                    //TODO: What is the type of a record key?
                    // want to allow numbers
                    // how do we typecheck this? The expr needs to be known before typechecking works.
                    // Some(Type::Text) //TODO: REMOVE
                }
                _ => None
            };
            match new_type {
                Some(t) => Ok(TypedExpr{t, expr: new_ex}),
                None => Err(format!("Type of binary operation for {:#?} doesn't work. Type of e1 is {:#?} and type of e2 is {:#?}.", bop, t1.t, t2.t))
            }
        },
        Unop(uop, e) => {
            let t1 = typecheck(&*e, bindings)?;
            match (uop, t1.t) {
                (Neg, t) if t <= Type::Number => Ok(TypedExpr{t, expr:Unop(uop,Box::new(t1))}),
                (Not, Type::Bool) => Ok(TypedExpr{t: Type::Bool, expr:Unop(uop,Box::new(t1))}),
                _ => Err(format!("Type of unary operation for {:#?} does not work. Type of e is {:#?}", uop, t1.t))
            }
        },
        Lambda(id,op_t,e) => {
            //TODO: what do we do if type isn't specified in the lambda?
            //how do we infer the type?
            // Ok(TypedExpr{t: Type::Text, expr: Lambda(id, op_t, Box::new(typecheck(&*e, bindings)?))}) // TODO: REMOVE
        },
        Record(hm) => {
            let record_type = HashMap::new();
            let typed_record = HashMap::new();
            for (key, val) in hm.iter() {
                let tval = typecheck(&*val, bindings)?;
                record_type.insert(key.clone(),tval.t);
                typed_record.insert(key.clone(), tval);
            }
            Ok(TypedExpr { t: Type::Record(record_type), expr: Record(typed_record) })
        },
        List(vec) => {
            let typed_vec : Result<Vec<TypedExpr>,String> = vec.iter().map(|e| typecheck(e, &bindings)).collect();
            let typed_vec = typed_vec?;
            // TODO: I don't think this is what we want.
            if vec.len() == 0 {
                return Ok(TypedExpr {
                    t: Type::List(Box::new(Type::Any)),
                    expr: List(typed_vec)
                });
            }
            let lub_type = typed_vec.iter().map(|typed_expr| typed_expr.t).reduce(|acc, t| acc.lub(&t));
            match lub_type {
                None => unreachable!(),
                Some(t) => Ok(TypedExpr {
                    t,
                    expr: List(typed_vec)
                })
            }
        },
        Ident(id) => {
            match bindings.get(&id) {
                None => Err(format!("identifier {id} does not have a type")),
                Some(t) => Ok(
                    TypedExpr{
                        t: t.clone(),
                        expr: Ident(id)
                    }
                )
            }
        },        
        Text(s) => {
            Ok(TypedExpr{
                t: Type::Text,
                expr: Text(s)
            })
        },
        Number(num) => {
            Ok(TypedExpr{
                t: Type::Number,
                expr: Number(num)
            })
        },
        Boolean(b) => {
            Ok(TypedExpr{
                t: Type::Bool,
                expr: Boolean(b)
            })
        },
        
        Null => {
            Ok(TypedExpr{
                t: Type::Null,
                expr: Null
            })
        },
    }
}

fn check() {

}

fn infer() {

}