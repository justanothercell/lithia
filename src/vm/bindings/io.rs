use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::fs::File;
use std::io::{stdin, stdout, Write};
use crate::variable::{VMObject, Type, Value, VarObject};

pub(crate) fn bindings() -> HashMap<String, Value> {
    HashMap::from([
        ("print".to_string(), Value::Fn(|args| {
            if args.len() != 1 { panic!("Invalid number of args {:?} for print, expected 1", args) }
            if let [Value::String(u1)] = &args[0..1] {
                print!("{}", u1);
                stdout().flush().expect("Error flushing stdout");
                return vec![];
            }
            panic!("Invalid args {:?} for print", args)
        }, vec![Type::String], vec![])),
        ("println".to_string(), Value::Fn(|args| {
            if args.len() != 1 { panic!("Invalid number of args {:?} for println, expected 1", args) }
            if let [Value::String(u1)] = &args[0..1] {
                println!("{}", u1);
                return vec![];
            }
            panic!("Invalid args {:?} for println", args)
        }, vec![Type::String], vec![])),
        ("input".to_string(), Value::Fn(|args| {
            if args.len() != 1 { panic!("Invalid number of args {:?} for println, expected 1", args) }
            if let [Value::String(u1)] = &args[0..1] {
                let mut s=String::new();
                print!("{}", u1);
                stdout().flush().expect("Error flushing stdout");
                stdin().read_line(&mut s).expect("Could not read stdin");
                if let Some('\n')=s.chars().next_back() {
                    s.pop();
                }
                if let Some('\r')=s.chars().next_back() {
                    s.pop();
                }
                return vec![Value::String(s)];
            }
            panic!("Invalid args {:?} for println", args)
        }, vec![Type::String], vec![Type::String])),
        ("File::create".to_string(), Value::Fn(|args|{
            if args.len() != 1 { panic!("Invalid number of args {:?} for File::create, expected 1", args) }
            if let [Value::String(u1)] = &args[0..1] {
                let file = File::create(u1);
                return vec![Value::Object(VarObject::new(file.unwrap()))]
            }
            panic!("Invalid args {:?} for File::create", args)
        }, vec![Type::String], vec![Type::Object])),
        ("File::open".to_string(), Value::Fn(|args|{
            if args.len() != 1 { panic!("Invalid number of args {:?} for File::open, expected 1", args) }
            if let [Value::String(u1)] = &args[0..1] {
                let file = File::open(u1);
                return vec![Value::Object(VarObject::new(file.unwrap()))]
            }
            panic!("Invalid args {:?} for File::create", args)
        }, vec![Type::String], vec![Type::Object])),
        ("File::write".to_string(), Value::Fn(|args|{
            if args.len() != 2 { panic!("Invalid number of args {:?} for File::write, expected 2", args) }
            if let [Value::String(u1), Value::Object(u2)] = &args[0..2] {
                let mut file = u2.clone().to::<File>();
                file.write(u1.as_bytes()).expect("Unable to write to file");
                return vec![]
            }
            panic!("Invalid args {:?} for File::write", args)
        }, vec![Type::String, Type::Object], vec![]))
    ])
}

impl VMObject for File {
    fn box_clone(&self) -> Box<dyn VMObject> {
        let f = self.try_clone().expect("Unable to clone file");
        Box::from(f)
    }

    fn debug(&self) -> String {
        "File".to_string()
    }
}

