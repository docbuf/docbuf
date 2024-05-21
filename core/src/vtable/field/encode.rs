use super::*;

// Implement DocBufEncodeField for VTableField

impl DocBufEncodeField<String> for VTableField {
    fn encode(&self, data: &String, buffer: &mut Vec<u8>) -> Result<VTableFieldOffset, Error> {
        // Ensure the field data corresponds to the field rules
        #[cfg(feature = "validate")]
        self.rules.validate(data)?;

        match &self.r#type {
            VTableFieldType::String | VTableFieldType::HashMap { .. } => {
                // prepend length to the field data
                let data_length = (data.len() as u32).to_le_bytes();

                // Encode the data length
                buffer.extend_from_slice(&data_length);

                let offset_start = buffer.len();

                // Encode the field data
                buffer.extend_from_slice(data.as_bytes());

                let offset_end = buffer.len();

                // Return the offset of the field data, disregarding the data length
                Ok(self.as_offset(offset_start..offset_end))
            }
            _ => {
                println!("Error encoding field type: String");
                Err(Error::DocBufEncodeFieldType(self.r#type.to_string()))
            }
        }
    }
}

impl DocBufEncodeField<&str> for VTableField {
    fn encode(&self, data: &&str, buffer: &mut Vec<u8>) -> Result<VTableFieldOffset, Error> {
        // Ensure the field data corresponds to the field rules
        #[cfg(feature = "validate")]
        self.rules.validate(data)?;

        match &self.r#type {
            VTableFieldType::Uuid => {
                let offset_start = buffer.len();

                // Encode the field data
                buffer.extend_from_slice(data.as_bytes());

                let offset_end = buffer.len();

                // Return the offset of the field data, disregarding the data length
                Ok(self.as_offset(offset_start..offset_end))
            }
            VTableFieldType::String | VTableFieldType::HashMap { .. } => {
                // prepend length to the field data
                let data_length = (data.len() as u32).to_le_bytes();

                // Encode the data length
                buffer.extend_from_slice(&data_length);

                let offset_start = buffer.len();

                // Encode the field data
                buffer.extend_from_slice(data.as_bytes());

                let offset_end = buffer.len();

                // Return the offset of the field data, disregarding the data length
                Ok(self.as_offset(offset_start..offset_end))
            }
            _ => {
                println!("Error encoding field type: &str");
                Err(Error::DocBufEncodeFieldType(self.r#type.to_string()))
            }
        }
    }
}

impl DocBufEncodeField<&[u8]> for VTableField {
    fn encode(&self, data: &&[u8], buffer: &mut Vec<u8>) -> Result<VTableFieldOffset, Error> {
        // Ensure the field data corresponds to the field rules
        #[cfg(feature = "validate")]
        self.rules.validate(data)?;

        match &self.r#type {
            VTableFieldType::Uuid => {
                let offset_start = buffer.len();

                // Encode the field data
                buffer.extend_from_slice(data);

                let offset_end = buffer.len();

                // Return the offset of the field data, disregarding the data length
                Ok(self.as_offset(offset_start..offset_end))
            }
            VTableFieldType::Bytes => {
                // prepend length to the field data
                let data_length = (data.len() as u32).to_le_bytes();

                // Encode the data length
                buffer.extend_from_slice(&data_length);

                let offset_start = buffer.len();

                // Encode the field data
                buffer.extend_from_slice(data);

                let offset_end = buffer.len();

                // Return the offset of the field data, disregarding the data length
                Ok(self.as_offset(offset_start..offset_end))
            }
            _ => {
                println!("Error encoding field type: &[u8]");
                Err(Error::DocBufEncodeFieldType(self.r#type.to_string()))
            }
        }
    }
}

impl DocBufEncodeField<bool> for VTableField {
    fn encode(&self, data: &bool, buffer: &mut Vec<u8>) -> Result<VTableFieldOffset, Error> {
        // Ensure the field data corresponds to the field rules
        #[cfg(feature = "validate")]
        self.rules.validate(data)?;

        match &self.r#type {
            VTableFieldType::Bool => {
                let offset_start = buffer.len();

                // Encode the field data
                buffer.push(if *data { 1 } else { 0 });

                let offset_end = buffer.len();

                // Return the offset of the field data
                Ok(self.as_offset(offset_start..offset_end))
            }
            _ => {
                println!("Error encoding field type: bool");
                Err(Error::DocBufEncodeFieldType(self.r#type.to_string()))
            }
        }
    }
}

impl DocBufEncodeField<NumericValue> for VTableField {
    fn encode(
        &self,
        data: &NumericValue,
        buffer: &mut Vec<u8>,
    ) -> Result<VTableFieldOffset, Error> {
        #[cfg(feature = "validate")]
        self.rules.validate(data)?;

        let offset_start = buffer.len();

        match data {
            NumericValue::U8(value) => buffer.push(*value),
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

        let offset_end = buffer.len();

        // Return the offset of the field data
        Ok(self.as_offset(offset_start..offset_end))
    }
}
