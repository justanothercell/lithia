use std::collections::HashMap;
use crate::variable::{Type, Value, VarObject, VMObject};

pub(crate) fn bindings() -> HashMap<String, Value> {
    HashMap::from([
        ("to_dbg_string".to_string(), Value::Fn(|args| {
            if args.len() != 1 { panic!("Invalid number of args {:?} for println, expected 0", args) }
            return vec![Value::String(format!("{:?}", args[0]))]
        }, vec![Type::Empty], vec![Type::String]))
    ])
}

impl<T: VMObject> VMObject for Option<T> {
    fn box_clone(&self) -> Box<dyn VMObject> {
        let cloned = match self {
            Some(t) => {
                let c: T = VarObject::vmo_to::<T>(t.box_clone());
                Some(c)
            },
            None => None
        };
        Box::from(cloned)
    }

    fn debug(&self) -> String {
        "Option<T>".to_string()
    }
}

impl VMObject for Value {
    fn box_clone(&self) -> Box<dyn VMObject> {
        Box::from(self.clone())
    }

    fn debug(&self) -> String {
        "Value".to_string()
    }
}