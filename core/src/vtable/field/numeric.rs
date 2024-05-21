use super::*;

/// Value enum for the field type
#[derive(Debug, Clone, Deserialize, Serialize)]
#[repr(u8)]
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

impl NumericValue {
    fn as_u8_type(&self) -> u8 {
        match self {
            NumericValue::U8(_) => NumericValue::U8 as u8,
            NumericValue::U16(_) => NumericValue::U16 as u8,
            NumericValue::U32(_) => NumericValue::U32 as u8,
            NumericValue::U64(_) => NumericValue::U64 as u8,
            NumericValue::U128(_) => NumericValue::U128 as u8,
            NumericValue::USIZE(_) => NumericValue::USIZE as u8,
            NumericValue::F32(_) => NumericValue::F32 as u8,
            NumericValue::F64(_) => NumericValue::F64 as u8,
            NumericValue::I8(_) => NumericValue::I8 as u8,
            NumericValue::I16(_) => NumericValue::I16 as u8,
            NumericValue::I32(_) => NumericValue::I32 as u8,
            NumericValue::I64(_) => NumericValue::I64 as u8,
            NumericValue::I128(_) => NumericValue::I128 as u8,
            NumericValue::ISIZE(_) => NumericValue::ISIZE as u8,
        }
    }

    pub fn from_u8_type(value: u8) -> Result<Self, Error> {
        let value = match value {
            0 => NumericValue::U8(0),
            1 => NumericValue::U16(0),
            2 => NumericValue::U32(0),
            3 => NumericValue::U64(0),
            4 => NumericValue::U128(0),
            5 => NumericValue::USIZE(0),
            6 => NumericValue::F32(0.0),
            7 => NumericValue::F64(0.0),
            8 => NumericValue::I8(0),
            9 => NumericValue::I16(0),
            10 => NumericValue::I32(0),
            11 => NumericValue::I64(0),
            12 => NumericValue::I128(0),
            13 => NumericValue::ISIZE(0),
            _ => return Err(Error::InvalidNumericValueType(value)),
        };

        Ok(value)
    }

    pub fn write_to_buffer(&self, buffer: &mut Vec<u8>) -> Result<(), Error> {
        // Write the type of the buffer
        buffer.push(self.as_u8_type());

        match self {
            NumericValue::U8(value) => {
                buffer.push(*value);
            }
            NumericValue::U16(value) => buffer.extend_from_slice(&value.to_le_bytes()),
            NumericValue::U32(value) => buffer.extend_from_slice(&value.to_le_bytes()),
            NumericValue::U64(value) => buffer.extend_from_slice(&value.to_le_bytes()),
            NumericValue::U128(value) => buffer.extend_from_slice(&value.to_le_bytes()),
            NumericValue::USIZE(value) => buffer.extend_from_slice(&value.to_le_bytes()),
            NumericValue::F32(value) => buffer.extend_from_slice(&value.to_le_bytes()),
            NumericValue::F64(value) => buffer.extend_from_slice(&value.to_le_bytes()),
            NumericValue::I8(value) => buffer.extend_from_slice(&value.to_le_bytes()),
            NumericValue::I16(value) => buffer.extend_from_slice(&value.to_le_bytes()),
            NumericValue::I32(value) => buffer.extend_from_slice(&value.to_le_bytes()),
            NumericValue::I64(value) => buffer.extend_from_slice(&value.to_le_bytes()),
            NumericValue::I128(value) => buffer.extend_from_slice(&value.to_le_bytes()),
            NumericValue::ISIZE(value) => buffer.extend_from_slice(&value.to_le_bytes()),
        };

        Ok(())
    }

    pub fn read_from_buffer(buffer: &mut Vec<u8>) -> Result<Self, Error> {
        let value_type = Self::from_u8_type(buffer.remove(0))?;

        let value = match value_type {
            NumericValue::U8(_) => {
                let value = buffer.remove(0);
                NumericValue::U8(value)
            }
            NumericValue::U16(_) => {
                let value = NumericValue::U16(u16::from_le_bytes([buffer[0], buffer[1]]));
                buffer.drain(0..2);
                value
            }
            NumericValue::U32(_) => {
                let value = NumericValue::U32(u32::from_le_bytes([
                    buffer[0], buffer[1], buffer[2], buffer[3],
                ]));
                buffer.drain(0..4);
                value
            }
            NumericValue::U64(_) => {
                let value = NumericValue::U64(u64::from_le_bytes([
                    buffer[0], buffer[1], buffer[2], buffer[3], buffer[4], buffer[5], buffer[6],
                    buffer[7],
                ]));
                buffer.drain(0..8);
                value
            }
            NumericValue::U128(_) => {
                let value = NumericValue::U128(u128::from_le_bytes([
                    buffer[0], buffer[1], buffer[2], buffer[3], buffer[4], buffer[5], buffer[6],
                    buffer[7], buffer[8], buffer[9], buffer[10], buffer[11], buffer[12],
                    buffer[13], buffer[14], buffer[15],
                ]));
                buffer.drain(0..16);
                value
            }
            NumericValue::USIZE(_) => {
                let value = NumericValue::USIZE(usize::from_le_bytes([
                    buffer[0], buffer[1], buffer[2], buffer[3], buffer[4], buffer[5], buffer[6],
                    buffer[7],
                ]));
                buffer.drain(0..8);
                value
            }
            NumericValue::F32(_) => {
                let value = NumericValue::F32(f32::from_le_bytes([
                    buffer[0], buffer[1], buffer[2], buffer[3],
                ]));
                buffer.drain(0..4);
                value
            }
            NumericValue::F64(_) => {
                let value = NumericValue::F64(f64::from_le_bytes([
                    buffer[0], buffer[1], buffer[2], buffer[3], buffer[4], buffer[5], buffer[6],
                    buffer[7],
                ]));
                buffer.drain(0..8);
                value
            }
            NumericValue::I8(_) => {
                let value = NumericValue::I8(buffer.remove(0) as i8);
                value
            }
            NumericValue::I16(_) => {
                let value = NumericValue::I16(i16::from_le_bytes([buffer[0], buffer[1]]));
                buffer.drain(0..2);
                value
            }

            NumericValue::I32(_) => {
                let value = NumericValue::I32(i32::from_le_bytes([
                    buffer[0], buffer[1], buffer[2], buffer[3],
                ]));
                buffer.drain(0..4);
                value
            }
            NumericValue::I64(_) => {
                let value = NumericValue::I64(i64::from_le_bytes([
                    buffer[0], buffer[1], buffer[2], buffer[3], buffer[4], buffer[5], buffer[6],
                    buffer[7],
                ]));
                buffer.drain(0..8);
                value
            }
            NumericValue::I128(_) => {
                let value = NumericValue::I128(i128::from_le_bytes([
                    buffer[0], buffer[1], buffer[2], buffer[3], buffer[4], buffer[5], buffer[6],
                    buffer[7], buffer[8], buffer[9], buffer[10], buffer[11], buffer[12],
                    buffer[13], buffer[14], buffer[15],
                ]));
                buffer.drain(0..16);
                value
            }
            NumericValue::ISIZE(_) => {
                let value = NumericValue::ISIZE(isize::from_le_bytes([
                    buffer[0], buffer[1], buffer[2], buffer[3], buffer[4], buffer[5], buffer[6],
                    buffer[7],
                ]));
                buffer.drain(0..8);
                value
            }
        };

        Ok(value)
    }
}

impl From<u8> for NumericValue {
    fn from(value: u8) -> Self {
        NumericValue::U8(value)
    }
}

impl From<u16> for NumericValue {
    fn from(value: u16) -> Self {
        NumericValue::U16(value)
    }
}

impl From<u32> for NumericValue {
    fn from(value: u32) -> Self {
        NumericValue::U32(value)
    }
}

impl From<u64> for NumericValue {
    fn from(value: u64) -> Self {
        NumericValue::U64(value)
    }
}

impl From<u128> for NumericValue {
    fn from(value: u128) -> Self {
        NumericValue::U128(value)
    }
}

impl From<usize> for NumericValue {
    fn from(value: usize) -> Self {
        NumericValue::USIZE(value)
    }
}

impl From<f32> for NumericValue {
    fn from(value: f32) -> Self {
        NumericValue::F32(value)
    }
}

impl From<f64> for NumericValue {
    fn from(value: f64) -> Self {
        NumericValue::F64(value)
    }
}

impl From<i8> for NumericValue {
    fn from(value: i8) -> Self {
        NumericValue::I8(value)
    }
}

impl From<i16> for NumericValue {
    fn from(value: i16) -> Self {
        NumericValue::I16(value)
    }
}

impl From<i32> for NumericValue {
    fn from(value: i32) -> Self {
        NumericValue::I32(value)
    }
}

impl From<i64> for NumericValue {
    fn from(value: i64) -> Self {
        NumericValue::I64(value)
    }
}

impl From<i128> for NumericValue {
    fn from(value: i128) -> Self {
        NumericValue::I128(value)
    }
}

impl From<isize> for NumericValue {
    fn from(value: isize) -> Self {
        NumericValue::ISIZE(value)
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
