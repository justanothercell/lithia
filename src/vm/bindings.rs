use std::collections::HashMap;
use crate::variable::Value;

pub mod operators;
pub mod io;
pub mod misc;

pub(crate) fn standard_bindings() -> HashMap<String, Value>{
    let mut bindings = HashMap::new();
    bindings.extend(operators::bindings());
    bindings.extend(io::bindings());
    bindings.extend(misc::bindings());
    bindings
}