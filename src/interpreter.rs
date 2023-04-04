use crate::ast::{TypedExpr, Ident, Value, ExprKind, Bop};
use std::collections::HashMap;

// reduce the AST to its simplest form
pub fn normalize(expr: &TypedExpr, bindings: &HashMap<Ident,Value>) -> Result<Value, String> {
    match expr.expr {
        ExprKind::App(e1, e2) => {
            // TODO: call by value? call by name? call by something else?
            let ne1 = normalize(&*e1, bindings)?;
            if let Value::Lambda(id, e) = ne1 {
                let ne2 = normalize(&*e2, bindings)?;
                let new_bindings = bindings.clone();
                new_bindings.insert(id, ne2);
                normalize(&e, &new_bindings)
            } else {
                Err(format!("Expression {ne1:#?} is not a lambda"))
            }
        },
        ExprKind::Lambda(id, _, e) => Ok(Value::Lambda(id, *e.clone())),
        ExprKind::Let(id, _, e1, e2) => {
            let newe1 = normalize(&*e1, bindings)?;
            let new_bindings = bindings.clone();
            new_bindings.insert(id, newe1);
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
                (Uop::Neg, Value::Number(n)) => Ok(Value::Number(-n)),
                (Uop::Not, Value::Boolean(b)) => Ok(Value::Boolean(!b)),
                _ => unreachable!()
            }
        }
        ExprKind::Record(hm) => {
            let reduced_hm = HashMap::new();
            for (k,v) in &hm {
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
            match bindings.get(&id) {
                Some(v) => Ok(v.clone()),
                None => Err(format!("Identifier '{id}' is not bound"))
            }
        },
        ExprKind::Text(t) => Ok(Value::Text(t)),
        ExprKind::Number(num) => Ok(Value::Number(num)),
        ExprKind::Boolean(b) => Ok(Value::Boolean(b)),
        ExprKind::Null => Ok(Value::Null),
        ExprKind::Binop(e1, bop, e2) => {
            let ne1 = normalize(&*e1, bindings)?;
            let ne2 = normalize(&*e2, bindings)?;
            eval_binop(bop, &ne1, &ne2)
        }
            
    }
}

fn eval_binop(bop: Bop, ne1: &Value, ne2: &Value) -> Result<Value, String>{
    use crate::ast::Value::*;
    match (bop, ne1, ne2) {
        // numbers
        (Bop::Pow, Number(a), Number(b)) => Ok(Number(a.powf(*b))),
        (Bop::Plus, Number(a), Number(b)) => Ok(Number(a + b)),
        (Bop::Minus, Number(a), Number(b)) => Ok(Number(a - b)),
        (Bop::Times, Number(a), Number(b)) => Ok(Number(a * b)),
        (Bop::Div, Number(a), Number(b)) => Ok(Number(a / b)),
        (Bop::And, Number(a), Number(b)) => {
            let a1: i64 = unsafe {a.to_int_unchecked()};
            let b1: i64 = unsafe {b.to_int_unchecked()};
            Ok(Number((a1 & b1) as f64))
        },
        (Bop::Or, Number(a), Number(b)) => {
            let a1: i64 = unsafe {a.to_int_unchecked()};
            let b1: i64 = unsafe {b.to_int_unchecked()};
            Ok(Number((a1 | b1) as f64))
        },
        (Bop::Xor, Number(a), Number(b)) => {
            let a1: i64 = unsafe {a.to_int_unchecked()};
            let b1: i64 = unsafe {b.to_int_unchecked()};
            Ok(Number((a1 ^ b1) as f64))
        },
        (Bop::Eq, Number(a), Number(b)) => Ok(Boolean(a == b)),
        (Bop::Neq, Number(a), Number(b)) => Ok(Boolean(a != b)),
        (Bop::Lt, Number(a), Number(b)) => Ok(Boolean(a < b)),
        (Bop::Gt, Number(a), Number(b)) => Ok(Boolean(a > b)),
        (Bop::Lte, Number(a), Number(b)) => Ok(Boolean(a <= b)),
        (Bop::Gte, Number(a), Number(b)) => Ok(Boolean(a >= b)),
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
        (Bop::Times, Text(t), Number(n)) => {
            let reps = unsafe {n.to_int_unchecked()};
            Ok(Text(t.repeat(reps)))
        },
        (Bop::Access, Text(t), Number(n)) => {
            let index = unsafe {n.to_int_unchecked()};
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
        (Bop::Neq, Record(hm1), Record(hm2)) => {
            match eval_binop(Bop::Eq, ne1, ne2)? {
                Boolean(b) => Ok(Boolean(!b)),
                _ => unreachable!()
            }
        },
        (Bop::Plus, Record(hm1), Record(hm2)) => {
            //When joining records, prefer the variable in the second one if there's overlap
            let mut joined_hashmap = hm1.clone();
            joined_hashmap.extend(*hm2);
            Ok(Record(joined_hashmap))
        },
        //list
        (Bop::Access, List(v), Number(n)) => {
            let index: usize = unsafe {n.to_int_unchecked()};
            match v.get(index) {
                None => Err(format!("Index {index} out of range")),
                Some(v) => Ok(v.clone())
            }
        },
        (Bop::Plus, List(v1), List(v2)) => {
            let mut joined = v1.clone();
            joined.extend(*v2);
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
        (Bop::Neq, List(v1), List(v2)) => {
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
    }
}

// takes in a normalized ast