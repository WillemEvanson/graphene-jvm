use crate::reader::Reader;

use super::parse_instruction;

/// An array of bytes which make up Java's bytecode. This gives utilities to
/// iterate over each bytecode and set the program counter.
///
/// [`Code`]: super::attribute::Code
#[derive(Clone)]
pub struct Bytecode<'a> {
    slice: &'a [u8],
    pc: usize,
}

impl<'a> Bytecode<'a> {
    pub fn new(slice: &'a [u8]) -> Self {
        Self { slice, pc: 0 }
    }

    pub fn set_pc(&mut self, pc: usize) {
        self.pc = pc;
    }
}

impl<'a> Iterator for Bytecode<'a> {
    type Item = (usize, Instruction<'a>);

    fn next(&mut self) -> Option<Self::Item> {
        if self.pc < self.slice.len() {
            let pc = self.pc;

            let mut reader = Reader::new(&self.slice[pc..]);
            let instruction = parse_instruction(&mut reader, pc).unwrap();
            self.pc = self.slice.len() - reader.remaining();

            Some((pc, instruction))
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
    aaload,
    aastore,
    aconst_null,
    aload(u8),
    anewarray(u16),
    areturn,
    arraylength,
    astore(u8),
    athrow,
    baload,
    bastore,
    bipush(i8),
    caload,
    castore,
    checkcast(u16),
    d2f,
    d2i,
    d2l,
    dadd,
    daload,
    dastore,
    dcmp(bool),
    dconst(f64),
    ddiv,
    dload(u8),
    dmul,
    dneg,
    drem,
    dreturn,
    dstore(u8),
    dsub,
    dup,
    dup_x1,
    dup_x2,
    dup2,
    dup2_x1,
    dup2_x2,
    f2d,
    f2i,
    f2l,
    fadd,
    faload,
    fastore,
    fcmp(bool),
    fconst(f32),
    fdiv,
    fload(u8),
    fmul,
    fneg,
    frem,
    freturn,
    fstore(u8),
    fsub,
    getfield(u16),
    getstatic(u16),
    goto(i16),
    goto_w(i32),
    i2b,
    i2c,
    i2d,
    i2f,
    i2l,
    i2s,
    iadd,
    iaload,
    iand,
    iastore,
    iconst(i8),
    idiv,
    if_acmp_eq(i16),
    if_acmp_ne(i16),
    if_icmp_eq(i16),
    if_icmp_ne(i16),
    if_icmp_lt(i16),
    if_icmp_ge(i16),
    if_icmp_gt(i16),
    if_icmp_le(i16),
    if_eq(i16),
    if_ne(i16),
    if_lt(i16),
    if_ge(i16),
    if_gt(i16),
    if_le(i16),
    ifnonnull(i16),
    ifnull(i16),
    iinc(u8, i8),
    iload(u8),
    imul,
    ineg,
    instanceof(u16),
    invokedynamic(u16),
    invokeinterface(u16, u8),
    invokespecial(u16),
    invokestatic(u16),
    invokevirtual(u16),
    ior,
    irem,
    ireturn,
    ishl,
    ishr,
    istore(u8),
    isub,
    iushr,
    ixor,
    jsr(i16),
    jsr_w(i32),
    l2d,
    l2f,
    l2i,
    ladd,
    laload,
    land,
    lastore,
    lcmp,
    lconst(u8),
    ldc(u8),
    ldc_w(u16),
    ldc2_w(u16),
    ldiv,
    lload(u8),
    lmul,
    lneg,
    lookupswitch(LookupSwitch<'a>),
    lor,
    lrem,
    lreturn,
    lshl,
    lshr,
    lstore(u8),
    lsub,
    lushr,
    lxor,
    monitorenter,
    monitorexit,
    multianewarray(u16, u8),
    new(u16),
    newarray(u8),
    nop,
    pop,
    pop2,
    putfield(u16),
    putstatic(u16),
    ret(u8),
    ret_void,
    saload,
    sastore,
    sipush(i16),
    swap,
    tableswitch(TableSwitch<'a>),
    wide_iload(u16),
    wide_fload(u16),
    wide_aload(u16),
    wide_lload(u16),
    wide_dload(u16),
    wide_istore(u16),
    wide_fstore(u16),
    wide_astore(u16),
    wide_lstore(u16),
    wide_dstore(u16),
    wide_ret(u16),
    wide_iinc(u16, i16),
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

    fn fmt(&self, f: &mut std::fmt::Formatter, pc: usize) -> std::fmt::Result {
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

    fn fmt(&self, f: &mut std::fmt::Formatter, pc: usize) -> std::fmt::Result {
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
