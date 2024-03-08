use super::*;

// Implement DocBufEncodeField for VTableField

impl<'a> DocBufEncodeField<String> for VTableField<'a> {
    fn encode(&self, data: &String, buffer: &mut Vec<u8>) -> Result<VTableFieldOffset, Error> {
        // Ensure the field data corresponds to the field rules
        #[cfg(feature = "validate")]
        self.validate(data)?;

        match &self.field_type {
            VTableFieldType::String | VTableFieldType::HashMap { .. } => {
                // prepend length to the field data
                let data_length = (data.len() as u32).to_le_bytes();

                // Encode the data length
                buffer.extend_from_slice(&data_length);

                let offset_start = buffer.len();
                let offset_length = data_length.len();

                // Encode the field data
                buffer.extend_from_slice(data.as_bytes());

                let offset_end = buffer.len();

                // Return the offset of the field data, disregarding the data length
                Ok((
                    self.item_index,
                    self.field_index,
                    offset_start + offset_length..offset_end,
                ))
            }
            _ => Err(Error::DocBufEncodeFieldType(self.field_type.to_string())),
        }
    }
}

impl<'a> DocBufEncodeField<&str> for VTableField<'a> {
    fn encode(&self, data: &&str, buffer: &mut Vec<u8>) -> Result<VTableFieldOffset, Error> {
        // Ensure the field data corresponds to the field rules
        #[cfg(feature = "validate")]
        self.validate(data)?;

        match &self.field_type {
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
                Ok((self.item_index, self.field_index, offset_start..offset_end))
            }
            _ => Err(Error::DocBufEncodeFieldType(self.field_type.to_string())),
        }
    }
}

impl<'a> DocBufEncodeField<&[u8]> for VTableField<'a> {
    fn encode(&self, data: &&[u8], buffer: &mut Vec<u8>) -> Result<VTableFieldOffset, Error> {
        // Ensure the field data corresponds to the field rules
        #[cfg(feature = "validate")]
        self.validate(data)?;

        match &self.field_type {
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
                Ok((self.item_index, self.field_index, offset_start..offset_end))
            }
            _ => Err(Error::DocBufEncodeFieldType(self.field_type.to_string())),
        }
    }
}

impl<'a> DocBufEncodeField<bool> for VTableField<'a> {
    fn encode(&self, data: &bool, buffer: &mut Vec<u8>) -> Result<VTableFieldOffset, Error> {
        // Ensure the field data corresponds to the field rules
        // #[cfg(feature = "validate")]
        // self.validate(data)?;

        match &self.field_type {
            VTableFieldType::Bool => {
                let offset_start = buffer.len();

                // Encode the field data
                buffer.push(if *data { 1 } else { 0 });

                let offset_end = buffer.len();

                // Return the offset of the field data
                Ok((self.item_index, self.field_index, offset_start..offset_end))
            }
            _ => Err(Error::DocBufEncodeFieldType(self.field_type.to_string())),
        }
    }
}

impl<'a> DocBufEncodeField<NumericValue> for VTableField<'a> {
    fn encode(
        &self,
        data: &NumericValue,
        buffer: &mut Vec<u8>,
    ) -> Result<VTableFieldOffset, Error> {
        #[cfg(feature = "validate")]
        self.validate(data)?;

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
        Ok((self.item_index, self.field_index, offset_start..offset_end))
    }
}
