use std::num::NonZeroU16;

use crate::java_str;
use crate::reader::{Reader, ReaderError};
use crate::string::{EncodingError, JavaStr};

use super::{
    Class, Code, ConstantIdx, ConstantPool, Entry, Field, Instruction, LookupSwitch, Method,
    MethodFlags, ReferenceKind, TableSwitch,
};

type Result<T> = std::result::Result<T, ParseError>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseError {
    InvalidCesu8String,
    UnexpectedEndOfFile,

    InvalidMagicNumber,
    InvalidConstantTag,
    InvalidConstantIdx,
}

impl From<EncodingError> for ParseError {
    fn from(_: EncodingError) -> Self {
        Self::InvalidCesu8String
    }
}

impl From<ReaderError> for ParseError {
    fn from(_: ReaderError) -> Self {
        Self::UnexpectedEndOfFile
    }
}

impl TryFrom<u8> for ReferenceKind {
    type Error = ParseError;

    fn try_from(value: u8) -> std::result::Result<Self, Self::Error> {
        let kind = match value {
            1 => Self::GetField,
            2 => Self::GetStatic,
            3 => Self::PutField,
            4 => Self::PutStatic,
            5 => Self::InvokeVirtual,
            6 => Self::InvokeStatic,
            7 => Self::InvokeSpecial,
            8 => Self::NewInvokeSpecial,
            9 => Self::InvokeInterface,
            _ => return Err(ParseError::InvalidConstantIdx),
        };
        Ok(kind)
    }
}

pub fn parse(slice: &[u8]) -> Result<Class> {
    let mut reader = Reader::new(slice);

    // Magic Number
    if reader.read_u32()? != 0xCAFE_BABE {
        return Err(ParseError::InvalidMagicNumber);
    }

    // File Version
    let _minor = reader.read_u16()?;
    let _major = reader.read_u16()?;

    // Constant Pool
    let constants = parse_constant_pool(&mut reader)?;

    // Access Flags
    let _access_flags = reader.read_u16()?;

    // Class Name
    let this_class = ConstantIdx::try_from(reader.read_u16()?)?;

    // Super Class Name
    let super_class = NonZeroU16::new(reader.read_u16()?).map(ConstantIdx);

    // Interfaces
    let interface_count = reader.read_u16()? as usize;
    let mut interfaces = Vec::with_capacity(interface_count);
    for _ in 0..interface_count {
        interfaces.push(reader.read_u16()?);
    }

    // Field
    let field_count = reader.read_u16()? as usize;
    let mut fields = Vec::with_capacity(field_count);
    for _ in 0..field_count {
        fields.push(parse_field(&mut reader)?);
    }

    // Method
    let method_count = reader.read_u16()? as usize;
    let mut methods = Vec::with_capacity(field_count);
    for _ in 0..method_count {
        methods.push(parse_method(&mut reader, &constants)?);
    }

    let attribute_count = reader.read_u16()?;
    for _ in 0..attribute_count {
        let _ = parse_attribute(&mut reader)?;
    }

    Ok(Class {
        constants,
        this_class,
        super_class,
        fields,
        methods,
    })
}

fn parse_constant_pool(reader: &mut Reader) -> Result<ConstantPool> {
    let entry_count = reader.read_u16()? as usize - 1;
    let mut constants = ConstantPool::new(entry_count);
    while constants.len() < entry_count {
        let entry = match reader.read_u8()? {
            1 => {
                let length = reader.read_u16()?;
                let slice = reader.read_slice(length as usize)?;
                let str = JavaStr::from_java(slice)?.to_owned();
                Entry::Utf8(str)
            }
            3 => {
                let integer = reader.read_u32()?;
                Entry::Integer(integer as i32)
            }
            4 => {
                let float = f32::from_bits(reader.read_u32()?);
                Entry::Float(float)
            }
            5 => {
                let high = reader.read_u32()?;
                let low = reader.read_u32()?;
                let long = ((high as u64) << 32) + low as u64;
                Entry::Long(long as i64)
            }
            6 => {
                let high = reader.read_u32()?;
                let low = reader.read_u32()?;
                let double = f64::from_bits(((high as u64) << 32) + low as u64);
                Entry::Double(double)
            }
            7 => {
                let name_idx = ConstantIdx::try_from(reader.read_u16()?)?;
                Entry::Class(name_idx)
            }
            8 => {
                let string_idx = ConstantIdx::try_from(reader.read_u16()?)?;
                Entry::String(string_idx)
            }
            9 => {
                let class_idx = ConstantIdx::try_from(reader.read_u16()?)?;
                let name_type_idx = ConstantIdx::try_from(reader.read_u16()?)?;
                Entry::FieldRef(class_idx, name_type_idx)
            }
            10 => {
                let class_idx = ConstantIdx::try_from(reader.read_u16()?)?;
                let name_type_idx = ConstantIdx::try_from(reader.read_u16()?)?;
                Entry::MethodRef(class_idx, name_type_idx)
            }
            11 => {
                let class_idx = ConstantIdx::try_from(reader.read_u16()?)?;
                let name_type_idx = ConstantIdx::try_from(reader.read_u16()?)?;
                Entry::InterfaceMethodRef(class_idx, name_type_idx)
            }
            12 => {
                let name_idx = ConstantIdx::try_from(reader.read_u16()?)?;
                let descriptor_idx = ConstantIdx::try_from(reader.read_u16()?)?;
                Entry::NameType(name_idx, descriptor_idx)
            }
            15 => {
                let ref_kind = ReferenceKind::try_from(reader.read_u8()?)?;
                let ref_idx = ConstantIdx::try_from(reader.read_u16()?)?;
                Entry::MethodHandle(ref_kind, ref_idx)
            }
            16 => {
                let descriptor_idx = ConstantIdx::try_from(reader.read_u16()?)?;
                Entry::MethodType(descriptor_idx)
            }
            18 => {
                let bootstrap_method_attr_idx = reader.read_u16()?;
                let name_type_idx = ConstantIdx::try_from(reader.read_u16()?)?;
                Entry::InvokeDynamic(bootstrap_method_attr_idx, name_type_idx)
            }
            _ => return Err(ParseError::InvalidConstantTag),
        };
        constants.add(entry);
    }

    Ok(constants)
}

fn parse_field(reader: &mut Reader) -> Result<Field> {
    let _access_flags = reader.read_u16()?;
    let name = ConstantIdx::try_from(reader.read_u16()?)?;
    let descriptor = ConstantIdx::try_from(reader.read_u16()?)?;

    let attribute_count = reader.read_u16()?;
    for _ in 0..attribute_count {
        let _ = parse_attribute(reader)?;
    }

    Ok(Field { name, descriptor })
}

fn parse_method(reader: &mut Reader, constants: &ConstantPool) -> Result<Method> {
    let flags = MethodFlags::from_bits(reader.read_u16()?);
    let name = ConstantIdx::try_from(reader.read_u16()?)?;
    let descriptor = ConstantIdx::try_from(reader.read_u16()?)?;

    let mut code = None;
    let attribute_count = reader.read_u16()?;
    for _ in 0..attribute_count {
        let (name, slice) = parse_attribute(reader)?;

        if constants.get(name).into_utf8() == java_str!("Code") {
            let mut reader = Reader::new(slice);

            let max_stack = reader.read_u16()?;
            let max_locals = reader.read_u16()?;
            let code_len = reader.read_u32()?;
            let code_bytes = reader.read_slice(code_len as usize)?;

            let exception_table_len = reader.read_u16()?;
            for _ in 0..exception_table_len {
                let _start_pc = reader.read_u16()?;
                let _end_pc = reader.read_u16()?;
                let _handler_pc = reader.read_u16()?;
                let _catch_type = reader.read_u16()?;
            }

            let attribute_count = reader.read_u16()?;
            for _ in 0..attribute_count {
                let _ = parse_attribute(&mut reader)?;
            }

            code = Some(Code {
                max_stack,
                max_locals,
                bytecode: code_bytes.to_owned(),
            });
        }
    }

    Ok(Method {
        name,
        descriptor,
        flags,

        code,
    })
}

fn parse_attribute<'a>(reader: &mut Reader<'a>) -> Result<(ConstantIdx, &'a [u8])> {
    let name = ConstantIdx::try_from(reader.read_u16()?)?;
    let length = reader.read_u32()?;
    let slice = reader.read_slice(length as usize)?;
    Ok((name, slice))
}

pub(super) fn parse_instruction<'a>(
    reader: &mut Reader<'a>,
    current_offset: usize,
) -> Result<Instruction<'a>> {
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

impl ConstantIdx {
    pub(super) fn try_from(value: u16) -> Result<Self> {
        if let Some(idx) = NonZeroU16::new(value) {
            Ok(ConstantIdx(idx))
        } else {
            Err(ParseError::InvalidConstantIdx)
        }
    }
}
