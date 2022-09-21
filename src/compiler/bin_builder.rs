use std::collections::HashMap;
use std::hash::Hash;
use crate::Word;
use crate::variable::{Type, Value};

pub(crate) struct VarId(pub(crate) usize);
pub(crate) struct Marker(pub(crate) usize);

impl VarId {
    pub(crate) fn u8(&self) -> Vec<u8>{
        Type::uint_u8(self.0 as u128, 4)
    }
}

struct VarProvider {
    highest_free_var: usize,
    lower_var_pool: Vec<usize>
}

impl VarProvider {
    fn gen_var_id(&mut self) -> VarId {
        if let Some(id) = self.lower_var_pool.pop() {
            VarId(id)
        }
        else {
            let id = VarId(self.highest_free_var);
            self.highest_free_var += 1;
            id
        }
    }

    fn destroy_var_id(&mut self, var: VarId) {
        self.lower_var_pool.push(var.0);
    }
}

struct MarkerProvider {
    marker_id: usize,
    unresolved_markers: HashMap<usize, Vec<usize>>,
    resolved_markers: HashMap<usize, usize>
}

pub(crate) struct BinBuilder {
    code: Vec<u8>,
    v_provider: VarProvider,
    m_provider: MarkerProvider
}

impl BinBuilder {
    pub(crate) fn new() -> Self {
        BinBuilder {
            code: vec![],
            v_provider: VarProvider {
                highest_free_var: 0,
                lower_var_pool: vec![]
            },
            m_provider: MarkerProvider {
                marker_id: 0,
                unresolved_markers: HashMap::new(),
                resolved_markers: HashMap::new()
            }
        }
    }

    pub(crate) fn load_extern(&mut self, name: &str, var_id: &VarId, signature: Type){
        self.code.push(Word::Extern.u8());
        self.code.append(&mut var_id.u8());
        self.code.append(&mut Type::str_u8(name));
        self.code.append(&mut signature.u8());
    }

    pub(crate) fn set_var(&mut self, var_id: &VarId, value_creator: impl Fn(&mut BinBuilder)){
        value_creator.call((self, ));
        self.code.push(Word::SetVar.u8());
        self.code.append(&mut var_id.u8());
    }

    pub(crate) fn add_code(&mut self, code_adder: impl Fn(&mut Vec<u8>)){
        code_adder.call((&mut self.code, ));
    }

    pub(crate) fn push_value(&mut self, val: Value){
        self.code.push(Word::Push.u8());
        self.code.append(&mut val.u8());
    }

    pub(crate) fn push_variable(&mut self, var: &VarId){
        self.code.push(Word::PushVar.u8());
        self.code.append(&mut var.u8());
    }

    pub(crate) fn call_function(&mut self, var: &VarId, arguments: impl Fn(&mut BinBuilder)){
        arguments.call((self, ));
        self.code.push(Word::Call.u8());
        self.code.append(&mut var.u8());
    }

    pub(crate) fn return_scope(&mut self, return_val: impl Fn(&mut BinBuilder)){
        return_val.call((self, ));
        self.code.push(Word::Return.u8());
    }

    pub(crate) fn debug_marker(&mut self, marker: usize){
        self.code.push(Word::Marker.u8());
        self.code.append(&mut Type::uint_u8(marker as u128, 8));
    }

    pub(crate) fn gen_marker(&mut self) -> Marker{
        Marker({
            let id = self.m_provider.marker_id;
            self.m_provider.marker_id += 1;
            self.m_provider.unresolved_markers.insert(id, vec![]);
            id
        })
    }

    pub(crate) fn set_marker(&mut self, marker: &Marker){
        if self.m_provider.resolved_markers.contains_key(&marker.0) {
            panic!("Marker {} was already set", marker.0)
        }
        let usages = self.m_provider.unresolved_markers.remove(&marker.0).expect("Marker {} does not exist (create with gen_marker)");
        let pos = self.code.len();
        self.m_provider.resolved_markers.insert(marker.0, pos);
        if let [p0, p1, p2, p3] = Type::uint_u8(pos as u128, 4)[..] {
            for usage in usages {
                for _ in 0..4 {
                    self.code.remove(usage);
                }
                self.code.insert(usage, p0);
                self.code.insert(usage + 1, p1);
                self.code.insert(usage + 2, p2);
                self.code.insert(usage + 3, p3);
            }
        }
        else {
            unreachable!()
        }
    }

    pub(crate) fn jump(&mut self, jmp: JmpType, marker: &Marker){
        self.code.push(match jmp {
            JmpType::Jmp => Word::Jump,
            JmpType::If => Word::JumpIf,
            JmpType::Unless => Word::JumpUnless
        }.u8());
        if let Some(&pos) = self.m_provider.resolved_markers.get(&marker.0) {
            self.code.append(&mut Type::uint_u8(pos as u128, 4)); // actual position
        }
        else {
            self.code.extend(vec![ 0, 0, 0, 0 ]); // placeholder
            let usages = self.m_provider.unresolved_markers.get_mut(&marker.0).expect("Marker {} does not exist (create with gen_marker)");
            usages.push(self.code.len() - 4);
        }
    }

    pub(crate) fn build(self) -> Vec<u8>{
        if self.m_provider.unresolved_markers.len() > 0 {
            panic!("{} unresolved markers were still left!", self.m_provider.unresolved_markers.len())
        }
        self.code
    }

    pub(crate) fn gen_var_id(&mut self) -> VarId {
        self.v_provider.gen_var_id()
    }

    pub(crate) fn destroy_var_id(&mut self, var: VarId) {
        self.v_provider.destroy_var_id(var)
    }
}

pub(crate) enum JmpType {
    Jmp,
    If,
    Unless
}