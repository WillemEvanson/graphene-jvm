use crate::reader::Reader;

use super::{parse_instruction, ConstantIdx};

/// An array of bytes which make up Java's bytecode. This gives utilities to
/// iterate over each bytecode and set the program counter.
///
/// [`Code`]: super::attribute::Code
#[derive(Clone)]
pub struct Bytecode<'a> {
    slice: &'a [u8],
    pc: u32,
}

impl<'a> Bytecode<'a> {
    pub fn new(slice: &'a [u8]) -> Self {
        Self { slice, pc: 0 }
    }

    pub fn pc(&self) -> u32 {
        self.pc
    }

    pub fn set_pc(&mut self, pc: u32) {
        self.pc = pc;
    }
}

impl<'a> Iterator for Bytecode<'a> {
    type Item = (u32, Instruction<'a>);

    fn next(&mut self) -> Option<Self::Item> {
        if self.pc < self.slice.len() as u32 {
            let pc = self.pc as usize;

            let mut reader = Reader::new(&self.slice[pc..]);
            let instruction = parse_instruction(&mut reader, pc).unwrap();
            self.pc = (self.slice.len() - reader.remaining()) as u32;

            Some((pc as u32, instruction))
        } else {
            None
        }
    }
}

impl std::fmt::Debug for Bytecode<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut list = f.debug_list();

        let mut code = self.clone();
        code.pc = 0;

        for (pc, instruction) in code {
            list.entry_with(|f| {
                if f.alternate() {
                    write!(f, "{pc}: ")?;
                }

                match instruction {
                    Instruction::lookupswitch(switch) => switch.fmt(f, pc),
                    Instruction::tableswitch(switch) => switch.fmt(f, pc),
                    _ => write!(f, "{instruction:?}"),
                }
            });
        }

        list.finish()
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy)]
pub enum Instruction<'a> {
    // Constant
    nop,
    aconst_null,
    iconst(i32),
    lconst(i64),
    fconst(f32),
    dconst(f64),
    bipush(i8),
    sipush(i16),
    ldc(ConstantIdx),

    // Load
    iload(u16),
    lload(u16),
    fload(u16),
    dload(u16),
    aload(u16),
    iaload,
    laload,
    faload,
    daload,
    aaload,
    baload,
    caload,
    saload,

    // Store
    istore(u16),
    lstore(u16),
    fstore(u16),
    dstore(u16),
    astore(u16),
    iastore,
    lastore,
    fastore,
    dastore,
    aastore,
    bastore,
    castore,
    sastore,

    // Stack
    pop,
    pop2,
    dup,
    dup_x1,
    dup_x2,
    dup2,
    dup2_x1,
    dup2_x2,
    swap,

    // Math
    iadd,
    ladd,
    fadd,
    dadd,
    isub,
    lsub,
    fsub,
    dsub,
    imul,
    lmul,
    fmul,
    dmul,
    idiv,
    ldiv,
    fdiv,
    ddiv,
    irem,
    lrem,
    frem,
    drem,
    ineg,
    lneg,
    fneg,
    dneg,
    ishl,
    lshl,
    ishr,
    lshr,
    iushr,
    lushr,
    iand,
    land,
    ior,
    lor,
    ixor,
    lxor,

    // Conversion
    i2l,
    i2f,
    i2d,
    l2i,
    l2f,
    l2d,
    f2i,
    f2l,
    f2d,
    d2i,
    d2l,
    d2f,
    i2b,
    i2c,
    i2s,

    // Comparison
    lcmp,
    fcmp(bool),
    dcmp(bool),
    if_eq(i16),
    if_ne(i16),
    if_lt(i16),
    if_ge(i16),
    if_gt(i16),
    if_le(i16),
    if_icmp_eq(i16),
    if_icmp_ne(i16),
    if_icmp_lt(i16),
    if_icmp_ge(i16),
    if_icmp_gt(i16),
    if_icmp_le(i16),
    if_acmp_eq(i16),
    if_acmp_ne(i16),

    // Control
    goto(i32),
    jsr(i32),
    ret(u16),
    tableswitch(TableSwitch<'a>),
    lookupswitch(LookupSwitch<'a>),
    ireturn,
    lreturn,
    freturn,
    dreturn,
    areturn,
    ret_void,

    // Reference
    getstatic(ConstantIdx),
    putstatic(ConstantIdx),
    getfield(ConstantIdx),
    putfield(ConstantIdx),
    invokevirtual(ConstantIdx),
    invokespecial(ConstantIdx),
    invokestatic(ConstantIdx),
    invokeinterface(ConstantIdx, u8),
    invokedynamic(ConstantIdx),
    new(ConstantIdx),
    newarray(ArrayKind),
    anewarray(ConstantIdx),
    arraylength,
    athrow,
    checkcast(ConstantIdx),
    instanceof(ConstantIdx),
    monitorenter,
    monitorexit,

    // Extended
    multianewarray(ConstantIdx, u8),
    ifnonnull(i16),
    ifnull(i16),
    iinc(u16, i16),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArrayKind {
    Bool,
    Char,
    Float,
    Double,
    Byte,
    Short,
    Int,
    Long,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct LookupSwitch<'a> {
    default: i32,
    pairs: &'a [u8],
}

impl<'a> LookupSwitch<'a> {
    pub fn new(default: i32, pairs: &'a [u8]) -> Self {
        assert!(pairs.len() % (4 * 2) == 0);
        Self { default, pairs }
    }

    pub fn lookup(&self, key: i32) -> i32 {
        let mut reader = Reader::new(self.pairs);

        while !reader.is_empty() {
            let to_match = reader.read_u32().unwrap() as i32;
            let offset = reader.read_u32().unwrap() as i32;

            if to_match == key {
                return offset;
            }
        }

        self.default
    }

    pub fn pairs(&self) -> impl Iterator<Item = (i32, i32)> + 'a {
        let mut reader = Reader::new(self.pairs);

        std::iter::from_fn(move || {
            if reader.is_empty() {
                None
            } else {
                Some((
                    reader.read_u32().unwrap() as i32,
                    reader.read_u32().unwrap() as i32,
                ))
            }
        })
    }

    fn fmt(&self, f: &mut std::fmt::Formatter, pc: u32) -> std::fmt::Result {
        write!(f, "lookupswitch: ")?;

        let mut debug_list = f.debug_list();
        for (key, offset) in self.pairs() {
            debug_list.entry_with(|f| write!(f, "{}: {}", key, pc as i32 + offset));
        }
        debug_list.entry_with(|f| write!(f, "default: {}", pc as i32 + self.default));
        debug_list.finish()
    }
}

impl std::fmt::Debug for LookupSwitch<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt(f, 0)
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct TableSwitch<'a> {
    default: i32,
    low: i32,
    high: i32,
    offsets: &'a [u8],
}

impl<'a> TableSwitch<'a> {
    pub fn new(default: i32, low: i32, high: i32, offsets: &'a [u8]) -> Self {
        assert!(offsets.len() % 4 == 0);
        Self {
            default,
            low,
            high,
            offsets,
        }
    }

    pub fn lookup(&self, key: i32) -> i32 {
        if key < self.low || self.high < key {
            return self.default;
        }

        let position = key - self.low;

        let mut reader = Reader::new(self.offsets);
        reader.skip(position as usize * 4).unwrap();
        reader.read_u32().unwrap() as i32
    }

    pub fn offsets(&self) -> impl Iterator<Item = i32> + 'a {
        let mut reader = Reader::new(self.offsets);
        std::iter::from_fn(move || reader.read_u32().map(|value| value as i32).ok())
    }

    fn fmt(&self, f: &mut std::fmt::Formatter, pc: u32) -> std::fmt::Result {
        write!(f, "tableswitch: ")?;

        let mut debug_list = f.debug_list();
        for (i, offset) in self.offsets().enumerate() {
            debug_list.entry_with(|f| write!(f, "{}: {}", self.low + i as i32, pc as i32 + offset));
        }
        debug_list.entry_with(|f| write!(f, "default: {}", pc as i32 + self.default));
        debug_list.finish()
    }
}

impl std::fmt::Debug for TableSwitch<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt(f, 0)
    }
}
