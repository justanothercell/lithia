use std::collections::HashMap;
use crate::variable::{Type, Value};

pub mod operators;
pub mod io;
pub mod misc;

#[derive(Debug, Clone)]
pub(crate) struct Function(pub(crate) fn(Vec<Value>) -> Vec<Value>, pub(crate) Vec<Type>, pub(crate) Vec<Type>);

pub(crate) fn standard_bindings() -> HashMap<String, Function>{
    let mut bindings = HashMap::new();
    bindings.extend(operators::bindings());
    bindings.extend(io::bindings());
    bindings.extend(misc::bindings());
    bindings
}