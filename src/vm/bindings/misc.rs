use std::collections::HashMap;
use crate::variable::{Type, Value, VarObject, VMObject};
use crate::vm::bindings::Function;

pub(crate) fn bindings() -> HashMap<String, Function> {
    HashMap::from([
        ("to_dbg_string".to_string(), Function(|args| {
            if args.len() != 1 { panic!("Invalid number of args {:?} for println, expected 0", args) }
            return vec![Value::String(format!("{:?}", args[0]))]
        }, vec![Type::Empty], vec![Type::String]))
    ])
}

impl VMObject for Value {
    fn box_clone(&self) -> Box<dyn VMObject> {
        Box::from(self.clone())
    }

    fn debug(&self) -> String {
        "Value".to_string()
    }
}