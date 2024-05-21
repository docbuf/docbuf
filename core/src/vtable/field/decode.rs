use super::*;

// Implement DocBufDecodeField for VTableField

impl DocBufDecodeField<String> for VTableField {
    fn decode(&self, buffer: &mut Vec<u8>) -> Result<String, Error> {
        match self.r#type {
            // VTableFieldType::Uuid => {},
            VTableFieldType::String | VTableFieldType::HashMap { .. } => {
                let length =
                    u32::from_le_bytes([buffer[0], buffer[1], buffer[2], buffer[3]]) as usize;

                // Consume the data from the buffer
                buffer.drain(0..DEFAULT_FIELD_LENGTH_LE_BYTES);

                let data = String::from_utf8(buffer.drain(0..length).collect())?;

                Ok(data)
            }
            _ => {
                dbg!("Failed to Decode Type: String");
                Err(Error::DocBufDecodeFieldType(self.to_owned()))
            }
        }
    }
}

impl DocBufDecodeField<bool> for VTableField {
    fn decode(&self, buffer: &mut Vec<u8>) -> Result<bool, Error> {
        match self.r#type {
            VTableFieldType::Bool => {
                let data = buffer[0] == 1;

                // Consume the data from the buffer
                buffer.drain(0..1);

                Ok(data)
            }
            _ => {
                dbg!("Failed to Decode Type: bool");
                Err(Error::DocBufDecodeFieldType(self.to_owned()))
            }
        }
    }
}

impl DocBufDecodeField<u8> for VTableField {
    fn decode(&self, buffer: &mut Vec<u8>) -> Result<u8, Error> {
        match self.r#type {
            VTableFieldType::U8 | VTableFieldType::Uuid | VTableFieldType::Bytes => {
                let data = buffer[0];

                // Consume the data from the buffer
                buffer.drain(0..1);

                Ok(data)
            }
            _ => {
                dbg!("Failed to Decode Type: u8");
                Err(Error::DocBufDecodeFieldType(self.to_owned()))
            }
        }
    }
}

impl DocBufDecodeField<u16> for VTableField {
    fn decode(&self, buffer: &mut Vec<u8>) -> Result<u16, Error> {
        match self.r#type {
            VTableFieldType::U16 => {
                let data = u16::from_le_bytes([buffer[0], buffer[1]]);

                // Consume the data from the buffer
                buffer.drain(0..2);

                Ok(data)
            }
            _ => {
                dbg!("Failed to Decode Type: u16");
                Err(Error::DocBufDecodeFieldType(self.to_owned()))
            }
        }
    }
}

impl DocBufDecodeField<u32> for VTableField {
    fn decode(&self, buffer: &mut Vec<u8>) -> Result<u32, Error> {
        match self.r#type {
            VTableFieldType::U32 => {
                let data = u32::from_le_bytes([buffer[0], buffer[1], buffer[2], buffer[3]]);

                // Consume the data from the buffer
                buffer.drain(0..4);

                Ok(data)
            }
            _ => {
                dbg!("Failed to Decode Type: u32");
                Err(Error::DocBufDecodeFieldType(self.to_owned()))
            }
        }
    }
}

impl DocBufDecodeField<u64> for VTableField {
    fn decode(&self, buffer: &mut Vec<u8>) -> Result<u64, Error> {
        match self.r#type {
            VTableFieldType::U64 | VTableFieldType::USIZE => {
                let data = u64::from_le_bytes([
                    buffer[0], buffer[1], buffer[2], buffer[3], buffer[4], buffer[5], buffer[6],
                    buffer[7],
                ]);

                // Consume the data from the buffer
                buffer.drain(0..8);

                Ok(data)
            }
            _ => {
                dbg!("Failed to Decode Type: u64");
                Err(Error::DocBufDecodeFieldType(self.to_owned()))
            }
        }
    }
}

impl DocBufDecodeField<u128> for VTableField {
    fn decode(&self, buffer: &mut Vec<u8>) -> Result<u128, Error> {
        match self.r#type {
            VTableFieldType::Uuid => {
                unimplemented!("VTableFieldType::Uuid")
            }

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
            _ => {
                dbg!("Failed to Decode Type: u128");
                Err(Error::DocBufDecodeFieldType(self.to_owned()))
            }
        }
    }
}

impl DocBufDecodeField<usize> for VTableField {
    fn decode(&self, buffer: &mut Vec<u8>) -> Result<usize, Error> {
        match self.r#type {
            VTableFieldType::USIZE => {
                let data = usize::from_le_bytes([
                    buffer[0], buffer[1], buffer[2], buffer[3], buffer[4], buffer[5], buffer[6],
                    buffer[7],
                ]);

                // Consume the data from the buffer
                buffer.drain(0..8);

                Ok(data)
            }
            _ => {
                dbg!("Failed to Decode Type: usize");
                Err(Error::DocBufDecodeFieldType(self.to_owned()))
            }
        }
    }
}

impl DocBufDecodeField<i8> for VTableField {
    fn decode(&self, buffer: &mut Vec<u8>) -> Result<i8, Error> {
        match self.r#type {
            VTableFieldType::I8 => {
                let data = i8::from_le_bytes([buffer[0]]);

                // Consume the data from the buffer
                buffer.drain(0..1);

                Ok(data)
            }
            _ => {
                dbg!("Failed to Decode Type: i8");
                Err(Error::DocBufDecodeFieldType(self.to_owned()))
            }
        }
    }
}

impl DocBufDecodeField<i16> for VTableField {
    fn decode(&self, buffer: &mut Vec<u8>) -> Result<i16, Error> {
        match self.r#type {
            VTableFieldType::I16 => {
                let data = i16::from_le_bytes([buffer[0], buffer[1]]);

                // Consume the data from the buffer
                buffer.drain(0..2);

                Ok(data)
            }
            _ => {
                dbg!("Failed to Decode Type: i16");
                Err(Error::DocBufDecodeFieldType(self.to_owned()))
            }
        }
    }
}

impl DocBufDecodeField<i32> for VTableField {
    fn decode(&self, buffer: &mut Vec<u8>) -> Result<i32, Error> {
        match self.r#type {
            VTableFieldType::I32 => {
                let data = i32::from_le_bytes([buffer[0], buffer[1], buffer[2], buffer[3]]);

                // Consume the data from the buffer
                buffer.drain(0..4);

                Ok(data)
            }
            _ => {
                dbg!("Failed to Decode Type: i32");
                Err(Error::DocBufDecodeFieldType(self.to_owned()))
            }
        }
    }
}

impl DocBufDecodeField<i64> for VTableField {
    fn decode(&self, buffer: &mut Vec<u8>) -> Result<i64, Error> {
        match self.r#type {
            VTableFieldType::I64 | VTableFieldType::ISIZE => {
                let data = i64::from_le_bytes([
                    buffer[0], buffer[1], buffer[2], buffer[3], buffer[4], buffer[5], buffer[6],
                    buffer[7],
                ]);

                // Consume the data from the buffer
                buffer.drain(0..8);

                Ok(data)
            }
            _ => {
                dbg!("Failed to Decode Type: i64");
                Err(Error::DocBufDecodeFieldType(self.to_owned()))
            }
        }
    }
}

impl DocBufDecodeField<i128> for VTableField {
    fn decode(&self, buffer: &mut Vec<u8>) -> Result<i128, Error> {
        match self.r#type {
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
            _ => {
                dbg!("Failed to Decode Type: i128");
                Err(Error::DocBufDecodeFieldType(self.to_owned()))
            }
        }
    }
}

impl DocBufDecodeField<isize> for VTableField {
    fn decode(&self, buffer: &mut Vec<u8>) -> Result<isize, Error> {
        match self.r#type {
            VTableFieldType::ISIZE => {
                let data = isize::from_le_bytes([
                    buffer[0], buffer[1], buffer[2], buffer[3], buffer[4], buffer[5], buffer[6],
                    buffer[7],
                ]);

                // Consume the data from the buffer
                buffer.drain(0..8);

                Ok(data)
            }
            _ => {
                dbg!("Failed to Decode Type: isize");
                Err(Error::DocBufDecodeFieldType(self.to_owned()))
            }
        }
    }
}

impl DocBufDecodeField<f32> for VTableField {
    fn decode(&self, buffer: &mut Vec<u8>) -> Result<f32, Error> {
        match self.r#type {
            VTableFieldType::F32 => {
                let data = f32::from_le_bytes([buffer[0], buffer[1], buffer[2], buffer[3]]);

                // Consume the data from the buffer
                buffer.drain(0..4);

                Ok(data)
            }
            _ => {
                dbg!("Failed to Decode Type: f32");
                Err(Error::DocBufDecodeFieldType(self.to_owned()))
            }
        }
    }
}

impl DocBufDecodeField<f64> for VTableField {
    fn decode(&self, buffer: &mut Vec<u8>) -> Result<f64, Error> {
        match self.r#type {
            VTableFieldType::F64 => {
                let data = f64::from_le_bytes([
                    buffer[0], buffer[1], buffer[2], buffer[3], buffer[4], buffer[5], buffer[6],
                    buffer[7],
                ]);

                // Consume the data from the buffer
                buffer.drain(0..8);

                Ok(data)
            }
            _ => {
                dbg!("Failed to Decode Type: f64");
                Err(Error::DocBufDecodeFieldType(self.to_owned()))
            }
        }
    }
}

impl DocBufDecodeField<Vec<u8>> for VTableField {
    fn decode(&self, buffer: &mut Vec<u8>) -> Result<Vec<u8>, Error> {
        match self.r#type {
            VTableFieldType::Uuid => Ok(buffer.drain(0..16).collect()),
            VTableFieldType::Bytes => {
                let length =
                    u32::from_le_bytes([buffer[0], buffer[1], buffer[2], buffer[3]]) as usize;

                // Consume the data from the buffer
                buffer.drain(0..DEFAULT_FIELD_LENGTH_LE_BYTES);

                Ok(buffer.drain(0..length).collect())
            }
            _ => {
                dbg!("Failed to Decode Type: Vec<u8>");
                Err(Error::DocBufDecodeFieldType(self.to_owned()))
            }
        }
    }
}
