use super::*;

// Implement DocBufDecodeField for VTableField

impl<'a> DocBufDecodeField<String> for VTableField<'a> {
    fn decode(&self, buffer: &mut Vec<u8>) -> Result<String, Error> {
        match self.field_type {
            VTableFieldType::String | VTableFieldType::HashMap { .. } => {
                let length =
                    u32::from_le_bytes([buffer[0], buffer[1], buffer[2], buffer[3]]) as usize;

                // Consume the data from the buffer
                buffer.drain(0..DEFAULT_FIELD_LENGTH_LE_BYTES);

                let data = String::from_utf8(buffer.drain(0..length).collect())?;

                Ok(data)
            }
            _ => Err(Error::DocBufDecodeFieldType(self.field_type.to_string())),
        }
    }
}

impl<'a> DocBufDecodeField<bool> for VTableField<'a> {
    fn decode(&self, buffer: &mut Vec<u8>) -> Result<bool, Error> {
        match self.field_type {
            VTableFieldType::Bool => {
                let data = buffer[0] == 1;

                // Consume the data from the buffer
                buffer.drain(0..1);

                Ok(data)
            }
            _ => Err(Error::DocBufDecodeFieldType(self.field_type.to_string())),
        }
    }
}

impl<'a> DocBufDecodeField<u8> for VTableField<'a> {
    fn decode(&self, buffer: &mut Vec<u8>) -> Result<u8, Error> {
        match self.field_type {
            VTableFieldType::U8 => {
                let data = buffer[0];

                // Consume the data from the buffer
                buffer.drain(0..1);

                Ok(data)
            }
            _ => Err(Error::DocBufDecodeFieldType(self.field_type.to_string())),
        }
    }
}

impl<'a> DocBufDecodeField<u16> for VTableField<'a> {
    fn decode(&self, buffer: &mut Vec<u8>) -> Result<u16, Error> {
        match self.field_type {
            VTableFieldType::U16 => {
                let data = u16::from_le_bytes([buffer[0], buffer[1]]);

                // Consume the data from the buffer
                buffer.drain(0..2);

                Ok(data)
            }
            _ => Err(Error::DocBufDecodeFieldType(self.field_type.to_string())),
        }
    }
}

impl<'a> DocBufDecodeField<u32> for VTableField<'a> {
    fn decode(&self, buffer: &mut Vec<u8>) -> Result<u32, Error> {
        match self.field_type {
            VTableFieldType::U32 => {
                let data = u32::from_le_bytes([buffer[0], buffer[1], buffer[2], buffer[3]]);

                // Consume the data from the buffer
                buffer.drain(0..4);

                Ok(data)
            }
            _ => Err(Error::DocBufDecodeFieldType(self.field_type.to_string())),
        }
    }
}

impl<'a> DocBufDecodeField<u64> for VTableField<'a> {
    fn decode(&self, buffer: &mut Vec<u8>) -> Result<u64, Error> {
        match self.field_type {
            VTableFieldType::U64 | VTableFieldType::USIZE => {
                let data = u64::from_le_bytes([
                    buffer[0], buffer[1], buffer[2], buffer[3], buffer[4], buffer[5], buffer[6],
                    buffer[7],
                ]);

                // Consume the data from the buffer
                buffer.drain(0..8);

                Ok(data)
            }
            _ => Err(Error::DocBufDecodeFieldType(self.field_type.to_string())),
        }
    }
}

impl<'a> DocBufDecodeField<u128> for VTableField<'a> {
    fn decode(&self, buffer: &mut Vec<u8>) -> Result<u128, Error> {
        match self.field_type {
            VTableFieldType::U128 => {
                let data = u128::from_le_bytes([
                    buffer[0], buffer[1], buffer[2], buffer[3], buffer[4], buffer[5], buffer[6],
                    buffer[7], buffer[8], buffer[9], buffer[10], buffer[11], buffer[12],
                    buffer[13], buffer[14], buffer[15],
                ]);

                // Consume the data from the buffer
                buffer.drain(0..16);

                Ok(data)
            }
            _ => Err(Error::DocBufDecodeFieldType(self.field_type.to_string())),
        }
    }
}

impl<'a> DocBufDecodeField<usize> for VTableField<'a> {
    fn decode(&self, buffer: &mut Vec<u8>) -> Result<usize, Error> {
        match self.field_type {
            VTableFieldType::USIZE => {
                let data = usize::from_le_bytes([
                    buffer[0], buffer[1], buffer[2], buffer[3], buffer[4], buffer[5], buffer[6],
                    buffer[7],
                ]);

                // Consume the data from the buffer
                buffer.drain(0..8);

                Ok(data)
            }
            _ => Err(Error::DocBufDecodeFieldType(self.field_type.to_string())),
        }
    }
}

impl<'a> DocBufDecodeField<i8> for VTableField<'a> {
    fn decode(&self, buffer: &mut Vec<u8>) -> Result<i8, Error> {
        match self.field_type {
            VTableFieldType::I8 => {
                let data = i8::from_le_bytes([buffer[0]]);

                // Consume the data from the buffer
                buffer.drain(0..1);

                Ok(data)
            }
            _ => Err(Error::DocBufDecodeFieldType(self.field_type.to_string())),
        }
    }
}

impl<'a> DocBufDecodeField<i16> for VTableField<'a> {
    fn decode(&self, buffer: &mut Vec<u8>) -> Result<i16, Error> {
        match self.field_type {
            VTableFieldType::I16 => {
                let data = i16::from_le_bytes([buffer[0], buffer[1]]);

                // Consume the data from the buffer
                buffer.drain(0..2);

                Ok(data)
            }
            _ => Err(Error::DocBufDecodeFieldType(self.field_type.to_string())),
        }
    }
}

impl<'a> DocBufDecodeField<i32> for VTableField<'a> {
    fn decode(&self, buffer: &mut Vec<u8>) -> Result<i32, Error> {
        match self.field_type {
            VTableFieldType::I32 => {
                let data = i32::from_le_bytes([buffer[0], buffer[1], buffer[2], buffer[3]]);

                // Consume the data from the buffer
                buffer.drain(0..4);

                Ok(data)
            }
            _ => Err(Error::DocBufDecodeFieldType(self.field_type.to_string())),
        }
    }
}

impl<'a> DocBufDecodeField<i64> for VTableField<'a> {
    fn decode(&self, buffer: &mut Vec<u8>) -> Result<i64, Error> {
        match self.field_type {
            VTableFieldType::I64 | VTableFieldType::ISIZE => {
                let data = i64::from_le_bytes([
                    buffer[0], buffer[1], buffer[2], buffer[3], buffer[4], buffer[5], buffer[6],
                    buffer[7],
                ]);

                // Consume the data from the buffer
                buffer.drain(0..8);

                Ok(data)
            }
            _ => Err(Error::DocBufDecodeFieldType(self.field_type.to_string())),
        }
    }
}

impl<'a> DocBufDecodeField<i128> for VTableField<'a> {
    fn decode(&self, buffer: &mut Vec<u8>) -> Result<i128, Error> {
        match self.field_type {
            VTableFieldType::I128 => {
                let data = i128::from_le_bytes([
                    buffer[0], buffer[1], buffer[2], buffer[3], buffer[4], buffer[5], buffer[6],
                    buffer[7], buffer[8], buffer[9], buffer[10], buffer[11], buffer[12],
                    buffer[13], buffer[14], buffer[15],
                ]);

                // Consume the data from the buffer
                buffer.drain(0..16);

                Ok(data)
            }
            _ => Err(Error::DocBufDecodeFieldType(self.field_type.to_string())),
        }
    }
}

impl<'a> DocBufDecodeField<isize> for VTableField<'a> {
    fn decode(&self, buffer: &mut Vec<u8>) -> Result<isize, Error> {
        match self.field_type {
            VTableFieldType::ISIZE => {
                let data = isize::from_le_bytes([
                    buffer[0], buffer[1], buffer[2], buffer[3], buffer[4], buffer[5], buffer[6],
                    buffer[7],
                ]);

                // Consume the data from the buffer
                buffer.drain(0..8);

                Ok(data)
            }
            _ => Err(Error::DocBufDecodeFieldType(self.field_type.to_string())),
        }
    }
}

impl<'a> DocBufDecodeField<f32> for VTableField<'a> {
    fn decode(&self, buffer: &mut Vec<u8>) -> Result<f32, Error> {
        match self.field_type {
            VTableFieldType::F32 => {
                let data = f32::from_le_bytes([buffer[0], buffer[1], buffer[2], buffer[3]]);

                // Consume the data from the buffer
                buffer.drain(0..4);

                Ok(data)
            }
            _ => Err(Error::DocBufDecodeFieldType(self.field_type.to_string())),
        }
    }
}

impl<'a> DocBufDecodeField<f64> for VTableField<'a> {
    fn decode(&self, buffer: &mut Vec<u8>) -> Result<f64, Error> {
        match self.field_type {
            VTableFieldType::F64 => {
                let data = f64::from_le_bytes([
                    buffer[0], buffer[1], buffer[2], buffer[3], buffer[4], buffer[5], buffer[6],
                    buffer[7],
                ]);

                // Consume the data from the buffer
                buffer.drain(0..8);

                Ok(data)
            }
            _ => Err(Error::DocBufDecodeFieldType(self.field_type.to_string())),
        }
    }
}

impl<'a> DocBufDecodeField<Vec<u8>> for VTableField<'a> {
    fn decode(&self, buffer: &mut Vec<u8>) -> Result<Vec<u8>, Error> {
        match self.field_type {
            VTableFieldType::Bytes => {
                let length =
                    u32::from_le_bytes([buffer[0], buffer[1], buffer[2], buffer[3]]) as usize;

                // Consume the data from the buffer
                buffer.drain(0..DEFAULT_FIELD_LENGTH_LE_BYTES);

                Ok(buffer.drain(0..length).collect())
            }
            _ => Err(Error::DocBufDecodeFieldType(self.field_type.to_string())),
        }
    }
}
