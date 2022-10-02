/*
extern i32::add

start scope_global
    create a
    push 42
    assign a

    create b
    push 69
    assign b

    create res
    call i32::add
    assign res
    return
end scope_global
 */

use std::collections::btree_map::BTreeMap;
use std::collections::HashMap;
use std::time::{Duration, SystemTime};
use memmap2::Mmap;
use crate::variable::{Type, Value};

#[repr(u8)]
#[derive(Debug, Clone)]
pub(crate) enum Word {
    Noop,
    SetVar,
    Delete,
    Push,
    PushVar,
    Pop,
    Call,
    Frame,
    Return,
    Extern,
    Jump,
    JumpIf,
    JumpUnless,
    Marker
}

impl Word {
    pub(crate) fn u8(self) -> u8{
        self as u8
    }

    pub(crate) fn from_u8(id: u8) -> Self {
        match id {
            0 => Word::Noop,
            1 => Word::SetVar,
            2 => Word::Delete,
            3 => Word::Push,
            4 => Word::PushVar,
            5 => Word::Pop,
            6 => Word::Call,
            7 => Word::Frame,
            8 => Word::Return,
            9 => Word::Extern,
            10 => Word::Jump,
            11 => Word::JumpIf,
            12 => Word::JumpUnless,
            13 => Word::Marker,
            _ => panic!("invalid word: {}", id)
        }
    }
}

//pub(crate) type HashDict<K, V> = HashMap<K, V>;
//pub(crate) type HashDict<K, V> = BTreeMap<K, V>;
//pub(crate) type HashDict<K, V> = HashMap<K, V, nohash_hasher::BuildNoHashHasher<K>>;
pub(crate) type HashDict<K, V> = ahash::AHashMap<K, V>;

#[derive(Debug)]
pub(crate) struct Frame {
    pub(crate) locals: HashDict<usize, Value>,
    pub(crate) prog_ptr: usize
}

impl Frame {
    fn new(ptr: usize) -> Self{
        Frame {
            locals: Default::default(),
            prog_ptr: ptr
        }
    }
}

#[derive(Debug)]
pub(crate) struct Executor{
    pub(crate) stack_frames: Vec<Frame>,
    pub(crate) stack: Vec<Value>,
    pub(crate) program: Mmap,
    pub(crate) externs: HashMap<String, Value>, // has to be loaded only once, so Hashmap suffices
    pub(crate) current_marker: usize
}

impl Executor {
    pub(crate) fn run(&mut self, mut initial_heap: Vec<Value>) -> (Vec<Value>, Duration){
        self.stack_frames.push(Frame::new(0));
        self.stack.append(&mut initial_heap);

        let start = SystemTime::now();

        macro_rules! log_step {
            ($($arg:tt)*) => {
                ()//println!("[{:?}] {}", SystemTime::now().duration_since(start).unwrap(), format!($($arg)*))
            };
        }

        macro_rules! perf {
            ($statement: stmt) => {
                $statement
            };
        }

        // used if perf is on
        let mut timings = HashMap::<u8, (u32, u128)>::new();
        let mut perf_start: SystemTime;

        let u8_id: fn(&[u8]) -> (_, _) = |p|Type::u8_uint(p, 4);

        loop {
            perf! {
                perf_start = SystemTime::now()
            }
            let p = self.frame().prog_ptr;
            let next = self.program[p];
            self.frame().prog_ptr += 1;
            match next {
                //Noop
                0 => {
                    log_step!("no-op instruction")
                }
                //SetVar
                1 => {
                    let id = self.next_in(u8_id) as usize;
                    let val = self.stack.pop().expect("Stack was empty");
                    self.frame().locals.insert(id, val);
                    log_step!("set variable {}", id)
                }
                //Delete
                2 => {
                    let id = self.next_in(u8_id) as usize;
                    let _ = self.frame().locals.remove(&id);
                    log_step!("deleted variable {}", id)
                }
                //Push
                3 => {
                    let val = self.next_in(Value::from_u8);
                    log_step!("pushed val {:?}", val);
                    self.stack.push(val);
                }
                //PushVar
                4 => {
                    let id = self.next_in(u8_id) as usize;
                    let val = self.frame().locals.get(&id).expect(&format!("Unable to find local variable with id {}", id)).clone();
                    self.stack.push(val);
                    log_step!("pushed var {}", id)
                }
                //Pop
                5 => {
                    let _v = self.stack.pop().expect("Stack was empty");
                    log_step!("popped val {:?}", _v);
                }
                //Call
                6 => {
                    let id = self.next_in(u8_id) as usize;
                    let func = self.frame().locals.get(&id).expect(&format!("Unable to find local (function) variable with id {}", id)).clone();
                    if let Value::Fn(fun, t_args, _t_ret) = func {
                        let args = self.stack.split_off(self.stack.len() - t_args.len());
                        self.stack.append(&mut fun.call((args,)));
                    }
                    else {
                        panic!("Variable {} is not a function: {:?}", id, func)
                    }
                    log_step!("called function {}", id)
                }
                //Frame
                7 => {
                    let p = self.frame().prog_ptr;
                    self.stack_frames.push(Frame::new(p));
                    log_step!("pushed stack frame")
                }
                //Return
                8 => {
                    self.stack_frames.pop().expect("Unable to pop frame");
                    if self.stack_frames.len() > 0 {
                        log_step!("popped stack frame")
                    }
                    else {
                        log_step!("popped stack frame, finishing execution and returning {} values", self.stack.len());
                        perf! {{
                            println!("| word       | calls   | total          | time/call |");
                            println!("|------------|---------|----------------|-----------|");
                            for (word, (calls, total_time)) in timings {
                                println!("| {:10} | {:7} | {:12}ns | {:7}ns |", format!("{:?}", Word::from_u8(word)), calls, total_time, total_time / calls as u128)
                            }
                        }}
                        return (self.stack.to_owned(), SystemTime::now().duration_since(start).unwrap());
                    }
                }
                //Extern
                9 => {
                    let id = self.next_in(|p|Type::u8_uint(p, 4)) as usize;
                    let name = self.next_in(Type::u8_str);
                    let _signature = self.next_in(Type::from_u8);
                    let ex = self.externs.get(&name).expect(&format!("Could not find extern function {}", name)).clone();
                    self.frame().locals.insert(id, ex);
                    log_step!("loaded extern {} {:?} as {}", name, signature, id);
                }
                //Jump
                10 => {
                    let dest = self.next_in(|p|Type::u8_uint(p, 4)) as usize;
                    self.frame().prog_ptr = dest;
                    log_step!("jumped to {}", dest)
                }
                //JumpIf
                11 => {
                    let v = self.stack.pop().expect("Stack was empty");
                    if let Value::Bool(b) = v {
                        if b {
                            let dest = self.next_in(|p|Type::u8_uint(p, 4)) as usize;
                            self.frame().prog_ptr = dest;
                            log_step!("jumped conditionally (if) to {}", dest)
                        }
                        else {
                            self.frame().prog_ptr += 4;
                            log_step!("did not jump conditionally (if)")
                        }
                    }
                    else {
                        panic!("Value on stack was not of type Bool: {:?}", v)
                    }
                }
                //JumpUnless
                12 => {
                    let v = self.stack.pop().expect("Stack was empty");
                    if let Value::Bool(b) = v {
                        if !b {

                            let dest = self.next_in(|p|Type::u8_uint(p, 4)) as usize;
                            self.frame().prog_ptr = dest;
                            log_step!("jumped conditionally (unless) to {}", dest)
                        }
                        else{
                            self.frame().prog_ptr += 4;
                            log_step!("did not jump conditionally (unless)")
                        }
                    }
                    else {
                        panic!("Value on stack was not of type Bool: {:?}", v)
                    }
                }
                //Marker
                13 => {
                    self.current_marker = self.next_in(|p|Type::u8_uint(p, 8)) as usize;
                    log_step!("set marker to {}", self.current_marker)
                }
                invalid => unimplemented!("This word is not implemented: {}", invalid)
            }
            perf! {{
                if !timings.contains_key(&next){
                    timings.insert(next, (0, 0));
                }
                let f = timings.get_mut(&next).unwrap();
                f.0 += 1;
                f.1 += SystemTime::now().duration_since(perf_start).unwrap().as_nanos();
            }}
        }
    }

    fn frame(&mut self) -> &mut Frame {
        self.stack_frames.last_mut().expect("Frame stack is empty!")
    }

    fn next_in<T>(&mut self, consumer: fn(&[u8]) -> (T, usize)) -> T{
        let p = self.frame().prog_ptr;
        let (r, i) = consumer.call((&self.program[p..],));
        self.frame().prog_ptr += i;
        r
    }
}