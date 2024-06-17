use super::class::{Bytecode, Code, ConstantPool};
use super::value::Value;

#[derive(Debug)]
pub struct CallFrame<'a> {
    pub(super) constants: &'a ConstantPool,
    pub(super) bytecode: Bytecode<'a>,

    pub(super) locals: Locals,
    pub(super) stack: Stack,
}

impl<'a> CallFrame<'a> {
    pub fn new(constants: &'a ConstantPool, code: &'a Code) -> Self {
        Self {
            constants,
            bytecode: code.bytecode(),

            locals: Locals::new(code.max_locals as usize),
            stack: Stack::new(code.max_stack as usize),
        }
    }
}

#[derive(Debug)]
pub struct Stack {
    stack: Vec<Slot>,
}

impl Stack {
    pub fn new(n: usize) -> Self {
        Self {
            stack: Vec::with_capacity(n),
        }
    }

    pub fn push(&mut self, val: Value) {
        match val {
            Value::Long(_) | Value::Double(_) => {
                self.stack.push(Slot::Entry(val));
                self.stack.push(Slot::Marker);
            }
            _ => {
                self.stack.push(Slot::Entry(val));
            }
        }
    }

    pub fn push_int(&mut self, val: i32) {
        self.push(Value::Int(val))
    }

    pub fn push_long(&mut self, val: i64) {
        self.push(Value::Long(val))
    }

    pub fn push_float(&mut self, val: f32) {
        self.push(Value::Float(val))
    }

    pub fn push_double(&mut self, val: f64) {
        self.push(Value::Double(val))
    }

    pub fn push_ret_addr(&mut self, val: u32) {
        self.push(Value::RetAddr(val))
    }

    pub fn pop(&mut self) -> Value {
        match self.stack.pop() {
            Some(Slot::Entry(val)) => val,
            Some(Slot::Marker) => match self.stack.pop() {
                Some(Slot::Entry(val)) => val,
                Some(Slot::Marker) => panic!("invalid stack state"),
                None => panic!("cannot pop from empty stack"),
            },
            None => panic!("cannot pop from empty stack"),
        }
    }

    pub fn pop_int(&mut self) -> i32 {
        match self.pop() {
            Value::Int(val) => val,
            _ => panic!("invalid stack state"),
        }
    }

    pub fn pop_long(&mut self) -> i64 {
        match self.pop() {
            Value::Long(val) => val,
            _ => panic!("invalid stack state"),
        }
    }

    pub fn pop_float(&mut self) -> f32 {
        match self.pop() {
            Value::Float(val) => val,
            _ => panic!("invalid stack state"),
        }
    }

    pub fn pop_double(&mut self) -> f64 {
        match self.pop() {
            Value::Double(val) => val,
            _ => panic!("invalid stack state"),
        }
    }

    pub fn pop_ret_addr(&mut self) -> u32 {
        match self.pop() {
            Value::RetAddr(val) => val,
            _ => panic!("invalid stack state"),
        }
    }

    pub fn inst_pop(&mut self) {
        match self.stack.pop() {
            Some(Slot::Entry(_)) => (),
            Some(Slot::Marker) => {
                panic!("found category 2 type but expected category 1 type")
            }
            None => panic!("cannot pop from empty stack"),
        }
    }

    pub fn inst_pop2(&mut self) {
        match self.stack.pop() {
            Some(_) => (),
            None => panic!("cannot pop from empty stack"),
        };
        match self.stack.pop() {
            Some(Slot::Entry(_)) => (),
            Some(Slot::Marker) => panic!("invalid stack state"),
            None => panic!("cannot pop from empty stack"),
        }
    }

    pub fn inst_dup(&mut self) {
        self.dup_cat1(0);
    }

    pub fn inst_dup_x1(&mut self) {
        self.dup_cat1(1);
    }

    pub fn inst_dup_x2(&mut self) {
        self.dup_cat1(2);
    }

    pub fn inst_dup2(&mut self) {
        self.dup_cat2(0);
    }

    pub fn inst_dup2_x1(&mut self) {
        self.dup_cat2(1);
    }

    pub fn inst_dup2_x2(&mut self) {
        self.dup_cat2(2);
    }

    pub fn inst_swap(&mut self) {
        match self.stack.last_chunk_mut() {
            Some([Slot::Entry(lower), Slot::Entry(upper)]) => {
                std::mem::swap(lower, upper);
            }
            Some(_) => panic!("invalid stack state"),
            None => panic!("cannot swap with stack len of {}", self.stack.len()),
        }
    }

    #[track_caller]
    fn dup_cat1(&mut self, n: usize) {
        let entry = match self.stack.last() {
            Some(entry @ Slot::Entry(_)) => *entry,
            Some(Slot::Marker) => panic!("invalid stack state"),
            None => panic!("cannot pop from empty stack"),
        };
        self.stack.insert(self.stack.len() - n - 1, entry);
    }

    #[track_caller]
    fn dup_cat2(&mut self, n: usize) {
        let chunk = match self.stack.last_chunk() {
            Some(chunk @ [Slot::Entry(_), Slot::Entry(_) | Slot::Marker]) => *chunk,
            Some(_) => panic!("invalid stack state"),
            None => panic!("cannot pop from empty stack"),
        };
        self.stack.insert(self.stack.len() - n - 2, chunk[0]);
        self.stack.insert(self.stack.len() - n - 2, chunk[1]);
    }
}

#[derive(Debug)]
pub struct Locals {
    slots: Box<[Slot]>,
}

impl Locals {
    pub fn new(n: usize) -> Self {
        Self {
            slots: vec![Slot::Marker; n].into_boxed_slice(),
        }
    }

    pub fn get(&self, i: usize) -> Value {
        match self.slots.get(i) {
            Some(Slot::Entry(val)) => *val,
            _ => panic!("invalid local index: {i}"),
        }
    }

    pub fn get_int(&self, i: usize) -> i32 {
        match self.get(i) {
            Value::Int(val) => val,
            _ => panic!("invalid locals state"),
        }
    }

    pub fn get_long(&self, i: usize) -> i64 {
        match self.get(i) {
            Value::Long(val) => val,
            _ => panic!("invalid locals state"),
        }
    }

    pub fn get_float(&self, i: usize) -> f32 {
        match self.get(i) {
            Value::Float(val) => val,
            _ => panic!("invalid locals state"),
        }
    }

    pub fn get_double(&self, i: usize) -> f64 {
        match self.get(i) {
            Value::Double(val) => val,
            _ => panic!("invalid locals state"),
        }
    }

    pub fn get_ret_addr(&self, i: usize) -> u32 {
        match self.get(i) {
            Value::RetAddr(val) => val,
            _ => panic!("invalid locals state"),
        }
    }

    pub fn set(&mut self, i: usize, val: Value) {
        match val {
            Value::Long(_) | Value::Double(_) => {
                self.slots[i] = Slot::Entry(val);
                self.slots[i + 1] = Slot::Marker;
            }
            _ => self.slots[i] = Slot::Entry(val),
        }
    }

    pub fn set_int(&mut self, i: usize, val: i32) {
        self.set(i, Value::Int(val));
    }

    pub fn set_long(&mut self, i: usize, val: i64) {
        self.set(i, Value::Long(val));
    }

    pub fn set_float(&mut self, i: usize, val: f32) {
        self.set(i, Value::Float(val));
    }

    pub fn set_double(&mut self, i: usize, val: f64) {
        self.set(i, Value::Double(val));
    }

    pub fn set_ret_addr(&mut self, i: usize, val: u32) {
        self.set(i, Value::RetAddr(val));
    }
}

#[derive(Debug, Clone, Copy)]
enum Slot {
    Entry(Value),
    Marker,
}
