use std::collections::HashMap;
use std::cmp::Ordering;
use std::cmp::PartialEq;
use std::mem::discriminant;

pub type Ident = String;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Bop {
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

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Uop {
    Neg,
    Not
}

#[derive(Eq, Clone, Debug)]
pub enum Type {
    Null,
    List(Box<Type>),
    Natural,
    Integer,
    Real,
    Number,
    Function(Box<Type>, Box<Type>),
    Bool,
    Record(HashMap<Ident, Type>),
    Text,
    Alternative(Box<Type>, Box<Type>),
    Any,
    Type,
    Ident(Ident)
}

pub trait JoinSemiLattice : PartialOrd {
    fn lub(&self, other: &Self) -> Self;
}

impl JoinSemiLattice for Type {
    fn lub(&self, other: &Type) -> Type {
        match (self, other){
            (a,b) if a >= b => a.clone(),
            (a,b) if a < b => b.clone(),
            (Type::Record(hm1), Type::Record(hm2)) => {
                let mut joined_hashmap = hm1.clone();
                for (k, v) in hm2 {
                    // might need clone for key or value
                    joined_hashmap.entry(k.clone()).and_modify(|existing| {*existing = v.lub(existing)}).or_insert(v.clone());
                }
                Type::Record(joined_hashmap)
            },
            (Type::List(a), Type::List(b)) => Type::List(Box::new(a.lub(b))), //*a.lub(&*b)
            (Type::Alternative(a,b),Type::Alternative(c,d)) => {
                let alt1 = a.lub(b);
                let alt2 = c.lub(d);
                match alt2 {
                    Type::Alternative(e,f) => {
                        // TODO: I'm not sure this actually is what I want, but we'll see
                        alt1.lub(&e).lub(&f)
                    },
                    _ => alt1.lub(&alt2)
                }
            },
            (Type::Alternative(a,b),c) 
            | (c, Type::Alternative(a,b)) => {
                if c <= a || c <= b {
                    self.clone()
                } else if c >= a && c >= b {
                    c.clone()
                } else if c >= a {
                    Type::Alternative(Box::new(c.clone()), b.clone())
                } else if c >= b {
                    Type::Alternative(a.clone(), Box::new(c.clone()))
                } else {
                    Type::Alternative(Box::new(Type::Alternative(a.clone(),b.clone())), Box::new(c.clone()))
                }
            },
            (a,b) => Type::Alternative(Box::new(a.clone()), Box::new(b.clone()))
        }
    }
}

/*
Any > other
List[Number] > List[Integer]
Number > Integer
if expecting a, can use b with a > b
 */
impl PartialOrd for Type {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        use Ordering::*;
        match (self, other) {
            //any
            (Type::Any, Type::Any) => Some(Equal),
            (_, Type::Any) => Some(Less),
            (Type::Any, _) => Some(Greater),
            //numbers
            (Type::Number, Type::Natural) | (Type::Number, Type::Real) | (Type::Number, Type::Integer) => Some(Greater),
            (Type::Natural, Type::Number) | (Type::Real, Type::Number) | (Type::Integer, Type::Number) => Some(Less),
            (Type::Integer, Type::Natural) | (Type::Real, Type::Integer) | (Type::Real, Type::Natural) => Some(Greater),
            (Type::Natural, Type::Integer) | (Type::Integer, Type::Real) | (Type::Natural, Type::Real) => Some(Less),
            //list
            (Type::List(t1), Type::List(t2)) => (*t1).partial_cmp(&*t2),
            //record
            (Type::Record(map1), Type::Record(map2)) if map1 == map2 => Some(Equal),
            (Type::Record(map1), Type::Record(map2)) => {
                let smaller;
                let larger;
                let result;
                if map1.len() < map2.len() {
                    smaller = map1;
                    larger = map2;
                    result = Some(Greater);
                } else {
                    smaller = map2;
                    larger = map1;
                    result = Some(Less);
                }
                let large_has_small = smaller.iter().all(|(key, val)| larger.get(key).map_or(false, |v| val == v));
                if large_has_small {
                    result
                } else{
                    None
                }
            }
            //alternative: todo
            (t, Type::Alternative(t1, t2)) if t <= &*t1  || t <= &*t2 => Some(Less),
            (t, Type::Alternative(t1, t2)) if t >= &*t1 && t >= &*t2 => Some(Greater),
            (Type::Alternative(t1, t2), t) if t <= &*t1  || t <= &*t2 => Some(Greater),
            (Type::Alternative(t1, t2), t) if t >= &*t1  && t >= &*t2 => Some(Less),
            (Type::Alternative(t1,t2), Type::Alternative(t3,t4)) => {
                let c13 = (*t1).partial_cmp(&*t3);
                let c14 = (*t1).partial_cmp(&*t4);
                let c23 = (*t2).partial_cmp(&*t3);
                let c24 = (*t2).partial_cmp(&*t4);
                // TODO: I think this is wrong
                match (c13, c14, c23, c24) {
                    (Some(x), _, _, Some(y)) if x == y => Some(x),
                    (_, Some(x), Some(y),_) if x == y => Some(x),
                    _ => None
                }
                // t1 | t2 < t3 | t4
                // Number | Bool 
            },
            //unknown: todo
            //equal type
            (t1, t2) if t1 == t2 => Some(Ordering::Equal),
            _ => None
        }
    }
}

impl PartialEq for Type {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Type::Alternative(a,b), Type::Alternative(c,d)) => {
                *a == *c && *b == *d || *a == *d && *b == *c
            },
            (Type::Alternative(a,b), t) => *t == **a && *t == **b,
            (t, Type::Alternative(a,b)) => *t == **a && *t == **b,
            (Type::List(a), Type::List(b)) => **a == **b,
            (Type::Function(a,b), Type::Function(c,d)) => **a == **c && **b == **d,
            (Type::Record(a), Type::Record(b)) => {
                a == b
                // if self.len() != other.len() {
                //     return false;
                // }
                // for (key, val) in a {
                //     let v = b.get(key);
                //     match v {
                //         None => return false,
                //         Some(v1) => {
                //             if *val != *v1{
                //                 return false;
                //             }
                //         }
                //     }
                // }
                // a.iter().all(|(key, val)| other.get(key).map_or(false, |v| *val == *v));
                // true
            },
            (t1, t2) => discriminant(t1) == discriminant(t2)
        }
    }
}

#[derive(Debug, Clone)]
pub struct Expr {
    pub t: Option<Type>,
    pub expr: ExprKind<Self>
}

#[derive(Debug, Clone)]
pub struct TypedExpr {
    pub t: Type,
    pub expr: ExprKind<Self>
}


#[derive(Debug, Clone)]
pub enum ExprKind<Wrapper> {
    Let(Ident, Option<Type>, Box<Wrapper>, Box<Wrapper>),
    If(Box<Wrapper>, Box<Wrapper>, Box<Wrapper>),
    App(Box<Wrapper>, Box<Wrapper>),
    Binop(Box<Wrapper>, Bop, Box<Wrapper>),
    Unop(Uop, Box<Wrapper>),
    Ident(Ident),
    Record(HashMap<Ident, Wrapper>),
    List(Vec<Wrapper>),
    Text(String),
    Int(i64),
    Float(f64),
    Boolean(bool),
    Lambda(Ident,Option<Type>,Box<Wrapper>),
    Null,
    //Color, Version, Path
}


#[derive(Debug, Clone)]
pub enum Value {
    Record(HashMap<Ident, Value>),
    List(Vec<Value>),
    Text(String),
    Int(i64),
    Float(f64),
    Boolean(bool),
    Lambda(Ident,Expr),
    Null,
    //Color, Version, Path
}

// do we consider a function to be a value
