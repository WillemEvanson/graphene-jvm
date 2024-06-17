#[derive(Debug, Clone, Copy)]
pub enum Value {
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    RetAddr(u32),
}

impl Value {
    pub(super) fn size(&self) -> usize {
        match self {
            Self::Int(_) => 1,
            Self::Long(_) => 2,
            Self::Float(_) => 1,
            Self::Double(_) => 2,
            Self::RetAddr(_) => 1,
        }
    }
}
