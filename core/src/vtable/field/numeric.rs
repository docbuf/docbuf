use super::*;

/// Value enum for the field type
#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum NumericValue {
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    U128(u128),
    USIZE(usize),
    F32(f32),
    F64(f64),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    I128(i128),
    ISIZE(isize),
}

// impl NumericValue {
//     pub fn encode(&self, output: &mut Vec<u8>) -> Result<(), Error> {
// match self {
//     NumericValue::U8(value) => {
//         output.push(*value);
//     }
//     NumericValue::U16(value) => output.extend_from_slice(&value.to_le_bytes()),
//     NumericValue::U32(value) => output.extend_from_slice(&value.to_le_bytes()),
//     NumericValue::U64(value) => output.extend_from_slice(&value.to_le_bytes()),
//     NumericValue::U128(value) => output.extend_from_slice(&value.to_le_bytes()),
//     NumericValue::USIZE(value) => output.extend_from_slice(&value.to_le_bytes()),
//     NumericValue::F32(value) => output.extend_from_slice(&value.to_le_bytes()),
//     NumericValue::F64(value) => output.extend_from_slice(&value.to_le_bytes()),
//     NumericValue::I8(value) => output.extend_from_slice(&value.to_le_bytes()),
//     NumericValue::I16(value) => output.extend_from_slice(&value.to_le_bytes()),
//     NumericValue::I32(value) => output.extend_from_slice(&value.to_le_bytes()),
//     NumericValue::I64(value) => output.extend_from_slice(&value.to_le_bytes()),
//     NumericValue::I128(value) => output.extend_from_slice(&value.to_le_bytes()),
//     NumericValue::ISIZE(value) => output.extend_from_slice(&value.to_le_bytes()),
// };

//         Ok(())
//     }
// }

impl Into<NumericValue> for u8 {
    fn into(self) -> NumericValue {
        NumericValue::U8(self)
    }
}

impl Into<NumericValue> for u16 {
    fn into(self) -> NumericValue {
        NumericValue::U16(self)
    }
}

impl Into<NumericValue> for u32 {
    fn into(self) -> NumericValue {
        NumericValue::U32(self)
    }
}

impl Into<NumericValue> for u64 {
    fn into(self) -> NumericValue {
        NumericValue::U64(self)
    }
}

impl Into<NumericValue> for u128 {
    fn into(self) -> NumericValue {
        NumericValue::U128(self)
    }
}

impl Into<NumericValue> for usize {
    fn into(self) -> NumericValue {
        NumericValue::USIZE(self)
    }
}

impl Into<NumericValue> for f32 {
    fn into(self) -> NumericValue {
        NumericValue::F32(self)
    }
}

impl Into<NumericValue> for f64 {
    fn into(self) -> NumericValue {
        NumericValue::F64(self)
    }
}

impl Into<NumericValue> for i8 {
    fn into(self) -> NumericValue {
        NumericValue::I8(self)
    }
}

impl Into<NumericValue> for i16 {
    fn into(self) -> NumericValue {
        NumericValue::I16(self)
    }
}

impl Into<NumericValue> for i32 {
    fn into(self) -> NumericValue {
        NumericValue::I32(self)
    }
}

impl Into<NumericValue> for i64 {
    fn into(self) -> NumericValue {
        NumericValue::I64(self)
    }
}

impl Into<NumericValue> for i128 {
    fn into(self) -> NumericValue {
        NumericValue::I128(self)
    }
}

impl Into<NumericValue> for isize {
    fn into(self) -> NumericValue {
        NumericValue::ISIZE(self)
    }
}

/// Implement Eq for NumericValue
impl Eq for NumericValue {}

/// Implement PartialEq for NumericValue
impl PartialEq for NumericValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (NumericValue::U8(a), NumericValue::U8(b)) => a.eq(b),
            (NumericValue::U16(a), NumericValue::U16(b)) => a.eq(b),
            (NumericValue::U32(a), NumericValue::U32(b)) => a.eq(b),
            (NumericValue::U64(a), NumericValue::U64(b)) => a.eq(b),
            (NumericValue::U128(a), NumericValue::U128(b)) => a.eq(b),
            (NumericValue::USIZE(a), NumericValue::USIZE(b)) => a.eq(b),
            (NumericValue::F32(a), NumericValue::F32(b)) => a.eq(b),
            (NumericValue::F64(a), NumericValue::F64(b)) => a.eq(b),
            (NumericValue::I8(a), NumericValue::I8(b)) => a.eq(b),
            (NumericValue::I16(a), NumericValue::I16(b)) => a.eq(b),
            (NumericValue::I32(a), NumericValue::I32(b)) => a.eq(b),
            (NumericValue::I64(a), NumericValue::I64(b)) => a.eq(b),
            (NumericValue::I128(a), NumericValue::I128(b)) => a.eq(b),
            (NumericValue::ISIZE(a), NumericValue::ISIZE(b)) => a.eq(b),
            _ => false,
        }
    }
}

/// Implement Partial Ord for NumericValue
impl PartialOrd for NumericValue {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            (NumericValue::U8(a), NumericValue::U8(b)) => a.partial_cmp(b),
            (NumericValue::U16(a), NumericValue::U16(b)) => a.partial_cmp(b),
            (NumericValue::U32(a), NumericValue::U32(b)) => a.partial_cmp(b),
            (NumericValue::U64(a), NumericValue::U64(b)) => a.partial_cmp(b),
            (NumericValue::U128(a), NumericValue::U128(b)) => a.partial_cmp(b),
            (NumericValue::USIZE(a), NumericValue::USIZE(b)) => a.partial_cmp(b),
            (NumericValue::F32(a), NumericValue::F32(b)) => a.partial_cmp(b),
            (NumericValue::F64(a), NumericValue::F64(b)) => a.partial_cmp(b),
            (NumericValue::I8(a), NumericValue::I8(b)) => a.partial_cmp(b),
            (NumericValue::I16(a), NumericValue::I16(b)) => a.partial_cmp(b),
            (NumericValue::I32(a), NumericValue::I32(b)) => a.partial_cmp(b),
            (NumericValue::I64(a), NumericValue::I64(b)) => a.partial_cmp(b),
            (NumericValue::I128(a), NumericValue::I128(b)) => a.partial_cmp(b),
            (NumericValue::ISIZE(a), NumericValue::ISIZE(b)) => a.partial_cmp(b),
            _ => None,
        }
    }
}
