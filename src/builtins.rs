use std::collections::HashMap;
use crate::ast::{Ident, Type};

// Built in functions
//TODO: need to separate types of builtins (for typechecking) and definitions
pub const BUILTINS: HashMap<Ident, Type> = HashMap::new();