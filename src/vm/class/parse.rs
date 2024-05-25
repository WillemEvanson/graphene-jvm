use std::num::NonZeroU16;

use cesu8_str::java::JavaStr;
use cesu8_str::EncodingError;

use crate::reader::{Reader, ReaderError};

use super::{Class, Code, ConstantIdx, ConstantPool, Entry, Field, Method, ReferenceKind};

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

pub fn parse_class(slice: &[u8]) -> Result<Class> {
    let mut reader = Reader::new(slice);

    // Magic Number
    if reader.read_u32()? != 0xCAFE_BABE {
        return Err(ParseError::InvalidMagicNumber);
    }

    // File Version
    let _minor = reader.read_u16()?;
    let _major = reader.read_u16()?;

    // Constant Pool
    let entry_count = reader.read_u16()? as usize - 1;
    let mut constants = ConstantPool::new(entry_count);
    while constants.len() < entry_count {
        let entry = match reader.read_u8()? {
            1 => {
                let length = reader.read_u16()?;
                let slice = reader.read_slice(length as usize)?;
                let str = JavaStr::from_java_cesu8(slice)?.to_owned();
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
                let name_idx = parse_constant_idx(&mut reader)?;
                Entry::Class(name_idx)
            }
            8 => {
                let string_idx = parse_constant_idx(&mut reader)?;
                Entry::String(string_idx)
            }
            9 => {
                let class_idx = parse_constant_idx(&mut reader)?;
                let name_type_idx = parse_constant_idx(&mut reader)?;
                Entry::FieldRef(class_idx, name_type_idx)
            }
            10 => {
                let class_idx = parse_constant_idx(&mut reader)?;
                let name_type_idx = parse_constant_idx(&mut reader)?;
                Entry::MethodRef(class_idx, name_type_idx)
            }
            11 => {
                let class_idx = parse_constant_idx(&mut reader)?;
                let name_type_idx = parse_constant_idx(&mut reader)?;
                Entry::InterfaceMethodRef(class_idx, name_type_idx)
            }
            12 => {
                let name_idx = parse_constant_idx(&mut reader)?;
                let descriptor_idx = parse_constant_idx(&mut reader)?;
                Entry::NameType(name_idx, descriptor_idx)
            }
            15 => {
                let ref_kind = ReferenceKind::try_from(reader.read_u8()?)?;
                let ref_idx = parse_constant_idx(&mut reader)?;
                Entry::MethodHandle(ref_kind, ref_idx)
            }
            16 => {
                let descriptor_idx = parse_constant_idx(&mut reader)?;
                Entry::MethodType(descriptor_idx)
            }
            18 => {
                let bootstrap_method_attr_idx = reader.read_u16()?;
                let name_type_idx = parse_constant_idx(&mut reader)?;
                Entry::InvokeDynamic(bootstrap_method_attr_idx, name_type_idx)
            }
            _ => return Err(ParseError::InvalidConstantTag),
        };
        constants.add(entry);
    }

    // Access Flags
    let _access_flags = reader.read_u16()?;

    // Class Name
    let this_class = parse_constant_idx(&mut reader)?;

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
        let _access_flags = reader.read_u16()?;
        let name = parse_constant_idx(&mut reader)?;
        let descriptor = parse_constant_idx(&mut reader)?;

        let attribute_count = reader.read_u16()?;
        for _ in 0..attribute_count {
            let _ = parse_attribute(&mut reader)?;
        }

        fields.push(Field { name, descriptor });
    }

    // Method
    let method_count = reader.read_u16()? as usize;
    let mut methods = Vec::with_capacity(field_count);
    for _ in 0..method_count {
        let _access_flags = reader.read_u16()?;
        let name = parse_constant_idx(&mut reader)?;
        let descriptor = parse_constant_idx(&mut reader)?;

        let mut code = None;
        let attribute_count = reader.read_u16()?;
        for _ in 0..attribute_count {
            let (name, slice) = parse_attribute(&mut reader)?;

            if constants.get(name).into_utf8() == &*cesu8_str::java::from_utf8("Code") {
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

        methods.push(Method {
            name,
            descriptor,

            code,
        });
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

fn parse_attribute<'a>(reader: &mut Reader<'a>) -> Result<(ConstantIdx, &'a [u8])> {
    let name = parse_constant_idx(reader)?;
    let length = reader.read_u32()?;
    let slice = reader.read_slice(length as usize)?;
    Ok((name, slice))
}

fn parse_constant_idx(reader: &mut Reader) -> Result<ConstantIdx> {
    if let Some(idx) = NonZeroU16::new(reader.read_u16()?) {
        Ok(ConstantIdx(idx))
    } else {
        Err(ParseError::InvalidConstantIdx)
    }
}
