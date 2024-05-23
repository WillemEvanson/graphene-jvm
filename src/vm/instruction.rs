use crate::reader::{Reader, ReaderError};

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

fn parse_instruction<'a>(
    reader: &mut Reader<'a>,
    current_offset: usize,
) -> Result<Instruction<'a>, ReaderError> {
    let instruction = match reader.read_u8().unwrap() {
        // Constants
        0x00 => Instruction::nop,
        0x01 => Instruction::aconst_null,
        0x02 => Instruction::iconst(-1),
        0x03 => Instruction::iconst(0),
        0x04 => Instruction::iconst(1),
        0x05 => Instruction::iconst(2),
        0x06 => Instruction::iconst(3),
        0x07 => Instruction::iconst(4),
        0x08 => Instruction::iconst(5),
        0x09 => Instruction::lconst(0),
        0x0A => Instruction::lconst(1),
        0x0B => Instruction::fconst(0.0),
        0x0C => Instruction::fconst(1.0),
        0x0D => Instruction::fconst(2.0),
        0x0E => Instruction::dconst(0.0),
        0x0F => Instruction::dconst(1.0),
        0x10 => Instruction::bipush(reader.read_u8()? as i8),
        0x11 => Instruction::sipush(reader.read_u16()? as i16),
        0x12 => Instruction::ldc(reader.read_u8()?),
        0x13 => Instruction::ldc_w(reader.read_u16()?),
        0x14 => Instruction::ldc2_w(reader.read_u16()?),

        // Loads
        0x15 => Instruction::iload(reader.read_u8()?),
        0x16 => Instruction::lload(reader.read_u8()?),
        0x17 => Instruction::fload(reader.read_u8()?),
        0x18 => Instruction::dload(reader.read_u8()?),
        0x19 => Instruction::aload(reader.read_u8()?),
        0x1A => Instruction::iload(0),
        0x1B => Instruction::iload(1),
        0x1C => Instruction::iload(2),
        0x1D => Instruction::iload(3),
        0x1E => Instruction::lload(0),
        0x1F => Instruction::lload(1),
        0x20 => Instruction::lload(2),
        0x21 => Instruction::lload(3),
        0x22 => Instruction::fload(0),
        0x23 => Instruction::fload(1),
        0x24 => Instruction::fload(2),
        0x25 => Instruction::fload(3),
        0x26 => Instruction::dload(0),
        0x27 => Instruction::dload(1),
        0x28 => Instruction::dload(2),
        0x29 => Instruction::dload(3),
        0x2A => Instruction::aload(0),
        0x2B => Instruction::aload(1),
        0x2C => Instruction::aload(2),
        0x2D => Instruction::aload(3),
        0x2E => Instruction::iaload,
        0x2F => Instruction::laload,
        0x30 => Instruction::faload,
        0x31 => Instruction::daload,
        0x32 => Instruction::aaload,
        0x33 => Instruction::baload,
        0x34 => Instruction::caload,
        0x35 => Instruction::saload,

        // Store
        0x36 => Instruction::istore(reader.read_u8()?),
        0x37 => Instruction::lstore(reader.read_u8()?),
        0x38 => Instruction::fstore(reader.read_u8()?),
        0x39 => Instruction::dstore(reader.read_u8()?),
        0x3A => Instruction::astore(reader.read_u8()?),
        0x3B => Instruction::istore(0),
        0x3C => Instruction::istore(1),
        0x3D => Instruction::istore(2),
        0x3E => Instruction::istore(3),
        0x3F => Instruction::lstore(0),
        0x40 => Instruction::lstore(1),
        0x41 => Instruction::lstore(2),
        0x42 => Instruction::lstore(3),
        0x43 => Instruction::fstore(0),
        0x44 => Instruction::fstore(1),
        0x45 => Instruction::fstore(2),
        0x46 => Instruction::fstore(3),
        0x47 => Instruction::dstore(0),
        0x48 => Instruction::dstore(1),
        0x49 => Instruction::dstore(2),
        0x4A => Instruction::dstore(3),
        0x4B => Instruction::astore(0),
        0x4C => Instruction::astore(1),
        0x4D => Instruction::astore(2),
        0x4E => Instruction::astore(3),
        0x4F => Instruction::iastore,
        0x50 => Instruction::lastore,
        0x51 => Instruction::fastore,
        0x52 => Instruction::dastore,
        0x53 => Instruction::aastore,
        0x54 => Instruction::bastore,
        0x55 => Instruction::castore,
        0x56 => Instruction::sastore,

        // Stack
        0x57 => Instruction::pop,
        0x58 => Instruction::pop2,
        0x59 => Instruction::dup,
        0x5A => Instruction::dup_x1,
        0x5B => Instruction::dup_x2,
        0x5C => Instruction::dup2,
        0x5D => Instruction::dup2_x1,
        0x5E => Instruction::dup2_x2,
        0x5F => Instruction::swap,

        // Math
        0x60 => Instruction::iadd,
        0x61 => Instruction::ladd,
        0x62 => Instruction::fadd,
        0x63 => Instruction::dadd,
        0x64 => Instruction::isub,
        0x65 => Instruction::lsub,
        0x66 => Instruction::fsub,
        0x67 => Instruction::dsub,
        0x68 => Instruction::imul,
        0x69 => Instruction::lmul,
        0x6A => Instruction::fmul,
        0x6B => Instruction::dmul,
        0x6C => Instruction::idiv,
        0x6D => Instruction::ldiv,
        0x6E => Instruction::fdiv,
        0x6F => Instruction::ddiv,
        0x70 => Instruction::irem,
        0x71 => Instruction::lrem,
        0x72 => Instruction::frem,
        0x73 => Instruction::drem,
        0x74 => Instruction::ineg,
        0x75 => Instruction::lneg,
        0x76 => Instruction::fneg,
        0x77 => Instruction::dneg,
        0x78 => Instruction::ishl,
        0x79 => Instruction::lshl,
        0x7A => Instruction::ishr,
        0x7B => Instruction::lshr,
        0x7C => Instruction::iushr,
        0x7D => Instruction::lushr,
        0x7E => Instruction::iand,
        0x7F => Instruction::land,
        0x80 => Instruction::ior,
        0x81 => Instruction::lor,
        0x82 => Instruction::ixor,
        0x83 => Instruction::lxor,
        0x84 => Instruction::iinc(reader.read_u8()?, reader.read_u8()? as i8),

        // Conversions
        0x85 => Instruction::i2l,
        0x86 => Instruction::i2f,
        0x87 => Instruction::i2d,
        0x88 => Instruction::l2i,
        0x89 => Instruction::l2f,
        0x8A => Instruction::l2d,
        0x8B => Instruction::f2i,
        0x8C => Instruction::f2l,
        0x8D => Instruction::f2d,
        0x8E => Instruction::d2i,
        0x8F => Instruction::d2l,
        0x90 => Instruction::d2f,
        0x91 => Instruction::i2b,
        0x92 => Instruction::i2c,
        0x93 => Instruction::i2s,

        // Comparisons
        0x94 => Instruction::lcmp,
        0x95 => Instruction::fcmp(false),
        0x96 => Instruction::fcmp(true),
        0x97 => Instruction::dcmp(false),
        0x98 => Instruction::dcmp(true),
        0x99 => Instruction::if_eq(reader.read_u16()? as i16),
        0x9A => Instruction::if_ne(reader.read_u16()? as i16),
        0x9B => Instruction::if_lt(reader.read_u16()? as i16),
        0x9C => Instruction::if_ge(reader.read_u16()? as i16),
        0x9D => Instruction::if_gt(reader.read_u16()? as i16),
        0x9E => Instruction::if_le(reader.read_u16()? as i16),

        0x9F => Instruction::if_icmp_eq(reader.read_u16()? as i16),
        0xA0 => Instruction::if_icmp_ne(reader.read_u16()? as i16),
        0xA1 => Instruction::if_icmp_lt(reader.read_u16()? as i16),
        0xA2 => Instruction::if_icmp_ge(reader.read_u16()? as i16),
        0xA3 => Instruction::if_icmp_gt(reader.read_u16()? as i16),
        0xA4 => Instruction::if_icmp_le(reader.read_u16()? as i16),
        0xA5 => Instruction::if_acmp_eq(reader.read_u16()? as i16),
        0xA6 => Instruction::if_acmp_ne(reader.read_u16()? as i16),

        // Control
        0xA7 => Instruction::goto(reader.read_u16()? as i16),
        0xA8 => Instruction::jsr(reader.read_u16()? as i16),
        0xA9 => Instruction::ret(reader.read_u8()?),
        0xAA => {
            reader.skip(4 - (current_offset + 1) % 4)?;

            let default = reader.read_u32()? as i32;
            let low = reader.read_u32()? as i32;
            let high = reader.read_u32()? as i32;

            let offset_count = high - low + 1;
            let slice = reader.read_slice(offset_count as usize * 4)?;

            Instruction::tableswitch(TableSwitch::new(default, low, high, slice))
        }
        0xAB => {
            reader.skip(4 - (current_offset + 1) % 4)?;

            let default = reader.read_u32()? as i32;
            let pair_count = reader.read_u32()?;
            let slice = reader.read_slice(pair_count as usize * 8)?;

            Instruction::lookupswitch(LookupSwitch::new(default, slice))
        }
        0xAC => Instruction::ireturn,
        0xAD => Instruction::lreturn,
        0xAE => Instruction::freturn,
        0xAF => Instruction::dreturn,
        0xB0 => Instruction::areturn,
        0xB1 => Instruction::ret_void,

        // References
        0xB2 => Instruction::getstatic(reader.read_u16()?),
        0xB3 => Instruction::putstatic(reader.read_u16()?),
        0xB4 => Instruction::getfield(reader.read_u16()?),
        0xB5 => Instruction::putfield(reader.read_u16()?),
        0xB6 => Instruction::invokevirtual(reader.read_u16()?),
        0xB7 => Instruction::invokespecial(reader.read_u16()?),
        0xB8 => Instruction::invokestatic(reader.read_u16()?),
        0xB9 => {
            let index = reader.read_u16()?;
            let count = reader.read_u8()?;
            assert_eq!(&[0x00], reader.read_slice(1)?);

            Instruction::invokeinterface(index, count)
        }
        0xBA => {
            let index = reader.read_u16()?;
            assert_eq!(&[0x00, 0x00], reader.read_slice(2)?);

            Instruction::invokedynamic(index)
        }
        0xBB => Instruction::new(reader.read_u16()?),
        0xBC => Instruction::newarray(reader.read_u8()?),
        0xBD => Instruction::anewarray(reader.read_u16()?),
        0xBE => Instruction::arraylength,
        0xBF => Instruction::athrow,
        0xC0 => Instruction::checkcast(reader.read_u16()?),
        0xC1 => Instruction::instanceof(reader.read_u16()?),
        0xC2 => Instruction::monitorenter,
        0xC3 => Instruction::monitorexit,

        // Extended
        0xC4 => {
            let opcode = reader.read_u8()?;
            let index = reader.read_u16()?;
            match opcode {
                // Loads
                0x15 => Instruction::wide_iload(index),
                0x16 => Instruction::wide_lload(index),
                0x17 => Instruction::wide_fload(index),
                0x18 => Instruction::wide_dload(index),
                0x19 => Instruction::wide_aload(index),
                // Stores
                0x36 => Instruction::wide_istore(index),
                0x37 => Instruction::wide_lstore(index),
                0x38 => Instruction::wide_fstore(index),
                0x39 => Instruction::wide_dstore(index),
                0x3A => Instruction::wide_astore(index),

                // Other
                0xA9 => Instruction::wide_ret(index),
                0x84 => Instruction::wide_iinc(index, reader.read_u16()? as i16),
                opcode => panic!("invalid opcode: {opcode:04X}"),
            }
        }
        0xC5 => Instruction::multianewarray(reader.read_u16()?, reader.read_u8()?),
        0xC6 => Instruction::ifnull(reader.read_u16()? as i16),
        0xC7 => Instruction::ifnonnull(reader.read_u16()? as i16),
        0xC8 => Instruction::goto_w(reader.read_u32()? as i32),
        0xC9 => Instruction::jsr_w(reader.read_u32()? as i32),

        opcode => panic!("invalid opcode: {opcode:04X}"),
    };
    Ok(instruction)
}
