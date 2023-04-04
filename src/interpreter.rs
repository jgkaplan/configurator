use crate::ast::{TypedExpr, Expr, Ident, Value, ExprKind, Bop};
use std::collections::HashMap;

// reduce the AST to its simplest form
pub fn normalize(expr: &Expr, bindings: &HashMap<Ident,Value>) -> Result<Value, String> {
    match &expr.expr {
        ExprKind::App(e1, e2) => {
            // TODO: call by value? call by name? call by something else?
            let ne1 = normalize(&*e1, bindings)?;
            if let Value::Lambda(id, e) = ne1 {
                let ne2 = normalize(&*e2, bindings)?;
                let mut new_bindings = bindings.clone();
                new_bindings.insert(id, ne2);
                normalize(&e, &new_bindings)
            } else {
                Err(format!("Expression {ne1:#?} is not a lambda"))
            }
        },
        ExprKind::Lambda(id, _, e) => Ok(Value::Lambda(id.clone(), *e.clone())),
        ExprKind::Let(id, _, e1, e2) => {
            let newe1 = normalize(&*e1, bindings)?;
            let mut new_bindings = bindings.clone();
            new_bindings.insert(id.clone(), newe1);
            normalize(&*e2, &new_bindings)
        }
        ExprKind::If(b, e1, e2) => {
            let newb = normalize(&*b, bindings)?;
            if let Value::Boolean(bo) = newb {
                if bo {
                    normalize(&*e1, bindings)
                } else {
                    normalize(&*e2, bindings)
                }
            } else {
                Err(format!("Value {newb:#?} is not a boolean"))
            }
        },
        ExprKind::Unop(uop, e) => {
            use crate::ast::Uop;
            let v = normalize(&*e, bindings)?;
            match (uop, v) {
                (Uop::Neg, Value::Float(n)) => Ok(Value::Float(-n)),
                (Uop::Neg, Value::Int(n)) => Ok(Value::Int(-n)),
                (Uop::Not, Value::Boolean(b)) => Ok(Value::Boolean(!b)),
                _ => unreachable!()
            }
        }
        ExprKind::Record(hm) => {
            let mut reduced_hm = HashMap::new();
            for (k,v) in hm {
                let newv = normalize(v, bindings)?;
                reduced_hm.insert(k.clone(), newv);
            }
            Ok(Value::Record(reduced_hm))
        },
        ExprKind::List(l) => {
            let newl : Result<Vec<Value>, String> = l.iter().map(|e| normalize(e, bindings)).collect();
            Ok(Value::List(newl?))
        },
        ExprKind::Ident(id) => {
            match bindings.get(id) {
                Some(v) => Ok(v.clone()),
                None => Err(format!("Identifier '{id}' is not bound"))
            }
        },
        ExprKind::Text(t) => Ok(Value::Text(t.clone())),
        ExprKind::Float(num) => Ok(Value::Float(num.clone())),
        ExprKind::Int(num) => Ok(Value::Int(num.clone())),
        ExprKind::Boolean(b) => Ok(Value::Boolean(b.clone())),
        ExprKind::Null => Ok(Value::Null),
        ExprKind::Binop(e1, bop, e2) => {
            let ne1 = normalize(&*e1, bindings)?;
            let ne2 = normalize(&*e2, bindings)?;
            eval_binop(*bop, &ne1, &ne2)
        }
            
    }
}

fn eval_binop(bop: Bop, ne1: &Value, ne2: &Value) -> Result<Value, String>{
    use crate::ast::Value::*;
    match (bop, ne1, ne2) {
        // numbers
        (Bop::Pow, Int(a), Int(b)) => Ok(Float((*a as f64).powf(*b as f64))),
        (Bop::Pow, Float(a), Float(b)) => Ok(Float(a.powf(*b))),
        (Bop::Pow, Int(a), Float(b)) => Ok(Float((*a as f64).powf(*b))),
        (Bop::Pow, Float(a), Int(b)) => Ok(Float(a.powf(*b as f64))),
        (Bop::Plus, Int(a), Int(b)) => Ok(Int(a + b)),
        (Bop::Minus, Int(a), Int(b)) => Ok(Int(a - b)),
        (Bop::Times, Int(a), Int(b)) => Ok(Int(a * b)),
        (Bop::Plus, Float(a), Float(b)) => Ok(Float(a + b)),
        (Bop::Minus, Float(a), Float(b)) => Ok(Float(a - b)),
        (Bop::Times, Float(a), Float(b)) => Ok(Float(a * b)),
        (Bop::Plus, Int(a), Float(b)) => Ok(Float(*a as f64 + *b)),
        (Bop::Minus, Int(a), Float(b)) => Ok(Float(*a as f64 - *b)),
        (Bop::Times, Int(a), Float(b)) => Ok(Float(*a as f64 * *b)),
        (Bop::Plus, Float(a), Int(b)) => Ok(Float(*a + *b as f64)),
        (Bop::Minus, Float(a), Int(b)) => Ok(Float(*a - *b as f64)),
        (Bop::Times, Float(a), Int(b)) => Ok(Float(*a * *b as f64)),
        (Bop::Div, Int(a), Int(b)) => Ok(Float((*a as f64) / (*b as f64))),
        (Bop::Div, Int(a), Float(b)) => Ok(Float((*a as f64) / b)),
        (Bop::Div, Float(a), Int(b)) => Ok(Float(a / (*b as f64))),
        (Bop::Div, Float(a), Float(b)) => Ok(Float(a / b)),
        (Bop::And, Int(a), Int(b)) => {
            Ok(Int(a & b))
        },
        (Bop::Or, Int(a), Int(b)) => {
            Ok(Int(a | b))
        },
        (Bop::Xor, Int(a), Int(b)) => {
            Ok(Int(a ^ b))
        },
        (Bop::Eq, Int(a), Int(b)) => Ok(Boolean(a == b)),
        (Bop::Neq, Int(a), Int(b)) => Ok(Boolean(a != b)),
        (Bop::Lt, Int(a), Int(b)) => Ok(Boolean(a < b)),
        (Bop::Gt, Int(a), Int(b)) => Ok(Boolean(a > b)),
        (Bop::Lte, Int(a), Int(b)) => Ok(Boolean(a <= b)),
        (Bop::Gte, Int(a), Int(b)) => Ok(Boolean(a >= b)),
        (Bop::Eq, Float(a), Float(b)) => Ok(Boolean(a == b)),
        (Bop::Neq, Float(a), Float(b)) => Ok(Boolean(a != b)),
        (Bop::Lt, Float(a), Float(b)) => Ok(Boolean(a < b)),
        (Bop::Gt, Float(a), Float(b)) => Ok(Boolean(a > b)),
        (Bop::Lte, Float(a), Float(b)) => Ok(Boolean(a <= b)),
        (Bop::Gte, Float(a), Float(b)) => Ok(Boolean(a >= b)),
        (Bop::Eq, Int(a), Float(b)) => Ok(Boolean(*a as f64 == *b)),
        (Bop::Neq, Int(a), Float(b)) => Ok(Boolean(*a as f64 != *b)),
        (Bop::Lt, Int(a), Float(b)) => Ok(Boolean((*a as f64) < *b)),
        (Bop::Gt, Int(a), Float(b)) => Ok(Boolean(*a as f64 > *b)),
        (Bop::Lte, Int(a), Float(b)) => Ok(Boolean(*a as f64 <= *b)),
        (Bop::Gte, Int(a), Float(b)) => Ok(Boolean(*a as f64 >= *b)),
        (Bop::Eq, Float(a), Int(b)) => Ok(Boolean(*a == *b as f64)),
        (Bop::Neq, Float(a), Int(b)) => Ok(Boolean(*a != *b as f64)),
        (Bop::Lt, Float(a), Int(b)) => Ok(Boolean(*a < *b as f64)),
        (Bop::Gt, Float(a), Int(b)) => Ok(Boolean(*a > *b as f64)),
        (Bop::Lte, Float(a), Int(b)) => Ok(Boolean(*a <= *b as f64)),
        (Bop::Gte, Float(a), Int(b)) => Ok(Boolean(*a >= *b as f64)),
        
        // bools
        (Bop::And, Boolean(a), Boolean(b)) => Ok(Boolean(*a && *b)),
        (Bop::Or, Boolean(a), Boolean(b)) => Ok(Boolean(*a || *b)),
        (Bop::Xor, Boolean(a), Boolean(b)) => Ok(Boolean(a ^ b)),
        (Bop::Eq, Boolean(a), Boolean(b)) => Ok(Boolean(a == b)),
        (Bop::Neq, Boolean(a), Boolean(b)) => Ok(Boolean(a != b)),
        // null
        (Bop::Eq, Null, Null) => Ok(Boolean(true)),
        (Bop::Neq, Null, Null) => Ok(Boolean(false)),
        // text
        (Bop::Plus, Text(t1), Text(t2)) => Ok(Text(t1.clone() + &t2)),
        (Bop::Times, Text(t), Int(n)) => {
            let reps = match usize::try_from(*n){
                Ok(u) => u,
                Err(e) => return Err(e.to_string())
            };
            Ok(Text(t.repeat(reps)))
        },
        (Bop::Access, Text(t), Int(n)) => {
            let index = match usize::try_from(*n){
                Ok(u) => u,
                Err(e) => return Err(e.to_string())
            };
            match t.chars().nth(index) {
                None => Err(format!("Index {index} out of bounds")),
                Some(c) => Ok(Text(c.to_string()))
            }
        },
        (Bop::Eq, Text(t1), Text(t2)) => Ok(Boolean(t1 == t2)),
        (Bop::Neq, Text(t1), Text(t2)) => Ok(Boolean(t1 != t2)),
        (Bop::Lt, Text(t1), Text(t2)) => Ok(Boolean(t1 < t2)),
        (Bop::Gt, Text(t1), Text(t2)) => Ok(Boolean(t1 > t2)),
        (Bop::Lte, Text(t1), Text(t2)) => Ok(Boolean(t1 <= t2)),
        (Bop::Gte, Text(t1), Text(t2)) => Ok(Boolean(t1 >= t2)),
        //record
        (Bop::Access, Record(hm), Text(key)) => {
            match hm.get(key) {
                None => Err(format!("Key '{key}' not in record")),
                Some(v) => Ok(v.clone())
            }
        },
        (Bop::Eq, Record(hm1), Record(hm2)) => {
            if hm1.len() != hm2.len() {
                return Ok(Boolean(false));
            }
            for (k, v) in hm1.iter() {
                match hm2.get(k) {
                    None => return Ok(Boolean(false)),
                    Some(v2) => {
                        if let Boolean(false) = eval_binop(Bop::Eq, v, v2)?{
                            return Ok(Boolean(false))
                        }
                    }
                }
            }
            Ok(Boolean(true))
        },
        (Bop::Neq, Record(_), Record(_)) => {
            match eval_binop(Bop::Eq, ne1, ne2)? {
                Boolean(b) => Ok(Boolean(!b)),
                _ => unreachable!()
            }
        },
        (Bop::Plus, Record(hm1), Record(hm2)) => {
            //When joining records, prefer the variable in the second one if there's overlap
            let mut joined_hashmap = hm1.clone();
            joined_hashmap.extend(hm2.clone());
            Ok(Record(joined_hashmap))
        },
        //list
        (Bop::Access, List(v), Int(n)) => {
            let index = match usize::try_from(*n){
                Ok(u) => u,
                Err(e) => return Err(e.to_string())
            };
            match v.get(index) {
                None => Err(format!("Index {index} out of range")),
                Some(v) => Ok(v.clone())
            }
        },
        (Bop::Plus, List(v1), List(v2)) => {
            let mut joined = v1.clone();
            joined.extend(v2.clone());
            Ok(List(joined))
        },
        (Bop::Eq, List(v1), List(v2)) => {
            if v1.len() != v2.len() {
                return Ok(Boolean(false));
            }
            for (e1, e2) in v1.iter().zip(v2) {
               if let Boolean(false) = eval_binop(Bop::Eq, e1, e2)? {
                return Ok(Boolean(false));
               }
            }
            Ok(Boolean(true))
        },
        (Bop::Neq, List(_), List(_)) => {
            match eval_binop(Bop::Eq, ne1, ne2)? {
                Boolean(b) => Ok(Boolean(!b)),
                _ => unreachable!()
            }
        },

        // lambda has no binops
        // TODO: can we do structural equality for lambdas?
        // comparisons between any other types are not equal
        (Bop::Eq, _, _) => Ok(Boolean(false)),
        (Bop::Neq, _, _) => Ok(Boolean(true)),
        _ => Err(format!("This should have been caught by the typechecker"))
    }
}

// takes in a normalized ast