use super::*;

use crate::traits::{DocBufMap, DocBufValidateField};

impl DocBufMap<String> for &'static VTable {
    #[inline]
    fn docbuf_map(&self, buffer: &[u8], offset: &VTableFieldOffset) -> Result<String, Error> {
        let field = self.get_field_by_offset_index(offset.0)?;

        match field.r#type {
            VTableFieldType::String => {
                let data = String::from_utf8(buffer[offset.range()].to_vec())?;

                Ok(data)
            }
            _ => Err(Error::DocBufMapInvalidFieldType(field.r#type.to_string())),
        }
    }

    #[inline]
    fn docbuf_map_replace(
        &self,
        new_value: &String,
        buffer: &mut Vec<u8>,
        offset: &VTableFieldOffset,
        offsets: &mut VTableFieldOffsets,
    ) -> Result<(), Error> {
        let field = self.get_field_by_offset_index(offset.0)?;

        match field.r#type {
            VTableFieldType::String => {
                let data = new_value.as_bytes();

                buffer.splice(offset.range(), data.iter().cloned());

                offsets.resize(
                    offset.range().start,
                    VTableFieldOffsetDiff::new(offset.len(), data.len()),
                );

                Ok(())
            }
            _ => Err(Error::DocBufMapInvalidFieldType(field.r#type.to_string())),
        }
    }
}

impl DocBufMap<Vec<u8>> for &'static VTable {
    #[inline]
    fn docbuf_map(&self, buffer: &[u8], offset: &VTableFieldOffset) -> Result<Vec<u8>, Error> {
        let field = self.get_field_by_offset_index(offset.0)?;
        match field.r#type {
            VTableFieldType::Bytes => {
                let data = buffer[offset.range()].to_vec();

                Ok(data)
            }
            _ => Err(Error::DocBufMapInvalidFieldType(field.r#type.to_string())),
        }
    }

    #[inline]
    fn docbuf_map_replace(
        &self,
        new_value: &Vec<u8>,
        buffer: &mut Vec<u8>,
        offset: &VTableFieldOffset,
        offsets: &mut VTableFieldOffsets,
    ) -> Result<(), Error> {
        let field = self.get_field_by_offset_index(offset.0)?;

        field.rules.validate(new_value)?;

        match field.r#type {
            VTableFieldType::Bytes => {
                buffer.splice(offset.range(), new_value.iter().cloned());

                offsets.resize(
                    offset.range().start,
                    VTableFieldOffsetDiff::new(offset.len(), new_value.len()),
                );

                Ok(())
            }
            _ => Err(Error::DocBufMapInvalidFieldType(field.r#type.to_string())),
        }
    }
}

impl DocBufMap<u8> for &'static VTable {
    #[inline]
    fn docbuf_map(&self, buffer: &[u8], offset: &VTableFieldOffset) -> Result<u8, Error> {
        let field = self.get_field_by_offset_index(offset.0)?;

        match field.r#type {
            VTableFieldType::U8 => {
                let data = buffer[offset.range()][0];

                Ok(data)
            }
            _ => Err(Error::DocBufMapInvalidFieldType(field.r#type.to_string())),
        }
    }

    #[inline]
    fn docbuf_map_replace(
        &self,
        _new_value: &u8,
        _buffer: &mut Vec<u8>,
        _offset: &VTableFieldOffset,
        _offsets: &mut VTableFieldOffsets,
    ) -> Result<(), Error> {
        unimplemented!("docbuf_map_replace")
    }
}

impl DocBufMap<u16> for &'static VTable {
    #[inline]
    fn docbuf_map(&self, buffer: &[u8], offset: &VTableFieldOffset) -> Result<u16, Error> {
        let field = self.get_field_by_offset_index(offset.0)?;

        match field.r#type {
            VTableFieldType::U16 => {
                let bytes = &buffer[offset.range()];

                let data = u16::from_le_bytes([bytes[0], bytes[1]]);

                Ok(data)
            }
            _ => Err(Error::DocBufMapInvalidFieldType(field.r#type.to_string())),
        }
    }

    #[inline]
    fn docbuf_map_replace(
        &self,
        _new_value: &u16,
        _buffer: &mut Vec<u8>,
        _offset: &VTableFieldOffset,
        _offsets: &mut VTableFieldOffsets,
    ) -> Result<(), Error> {
        unimplemented!("docbuf_map_replace")
    }
}

impl DocBufMap<u32> for &'static VTable {
    #[inline]
    fn docbuf_map(&self, buffer: &[u8], offset: &VTableFieldOffset) -> Result<u32, Error> {
        let field = self.get_field_by_offset_index(offset.0)?;

        match field.r#type {
            VTableFieldType::U32 => {
                let bytes = &buffer[offset.range()];

                let data = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);

                Ok(data)
            }
            _ => Err(Error::DocBufMapInvalidFieldType(field.r#type.to_string())),
        }
    }

    #[inline]
    fn docbuf_map_replace(
        &self,
        _new_value: &u32,
        _buffer: &mut Vec<u8>,
        _offset: &VTableFieldOffset,
        _offsets: &mut VTableFieldOffsets,
    ) -> Result<(), Error> {
        unimplemented!("docbuf_map_replace")
    }
}

impl DocBufMap<u64> for &'static VTable {
    #[inline]
    fn docbuf_map(&self, buffer: &[u8], offset: &VTableFieldOffset) -> Result<u64, Error> {
        let field = self.get_field_by_offset_index(offset.0)?;

        match field.r#type {
            VTableFieldType::U64 => {
                let bytes = &buffer[offset.range()];

                let data = u64::from_le_bytes([
                    bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
                ]);

                Ok(data)
            }
            _ => Err(Error::DocBufMapInvalidFieldType(field.r#type.to_string())),
        }
    }

    #[inline]
    fn docbuf_map_replace(
        &self,
        _new_value: &u64,
        _buffer: &mut Vec<u8>,
        _offset: &VTableFieldOffset,
        _offsets: &mut VTableFieldOffsets,
    ) -> Result<(), Error> {
        unimplemented!("docbuf_map_replace")
    }
}

impl DocBufMap<usize> for &'static VTable {
    #[inline]
    fn docbuf_map(&self, buffer: &[u8], offset: &VTableFieldOffset) -> Result<usize, Error> {
        let field = self.get_field_by_offset_index(offset.0)?;

        match field.r#type {
            VTableFieldType::USIZE => {
                let bytes = &buffer[offset.range()];

                let data = usize::from_le_bytes([
                    bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
                ]);

                Ok(data)
            }
            _ => Err(Error::DocBufMapInvalidFieldType(field.r#type.to_string())),
        }
    }

    #[inline]
    fn docbuf_map_replace(
        &self,
        _new_value: &usize,
        _buffer: &mut Vec<u8>,
        _offset: &VTableFieldOffset,
        _offsets: &mut VTableFieldOffsets,
    ) -> Result<(), Error> {
        unimplemented!("docbuf_map_replace")
    }
}

impl DocBufMap<i8> for &'static VTable {
    #[inline]
    fn docbuf_map(&self, buffer: &[u8], offset: &VTableFieldOffset) -> Result<i8, Error> {
        let field = self.get_field_by_offset_index(offset.0)?;

        match field.r#type {
            VTableFieldType::I8 => {
                let data = buffer[offset.range()][0] as i8;

                Ok(data)
            }
            _ => Err(Error::DocBufMapInvalidFieldType(field.r#type.to_string())),
        }
    }

    #[inline]
    fn docbuf_map_replace(
        &self,
        _new_value: &i8,
        _buffer: &mut Vec<u8>,
        _offset: &VTableFieldOffset,
        _offsets: &mut VTableFieldOffsets,
    ) -> Result<(), Error> {
        unimplemented!("docbuf_map_replace")
    }
}

impl DocBufMap<i16> for &'static VTable {
    #[inline]
    fn docbuf_map(&self, buffer: &[u8], offset: &VTableFieldOffset) -> Result<i16, Error> {
        let field = self.get_field_by_offset_index(offset.0)?;

        match field.r#type {
            VTableFieldType::I16 => {
                let bytes = &buffer[offset.range()];

                let data = i16::from_le_bytes([bytes[0], bytes[1]]);

                Ok(data)
            }
            _ => Err(Error::DocBufMapInvalidFieldType(field.r#type.to_string())),
        }
    }

    #[inline]
    fn docbuf_map_replace(
        &self,
        _new_value: &i16,
        _buffer: &mut Vec<u8>,
        _offset: &VTableFieldOffset,
        _offsets: &mut VTableFieldOffsets,
    ) -> Result<(), Error> {
        unimplemented!("docbuf_map_replace")
    }
}

impl DocBufMap<i32> for &'static VTable {
    #[inline]
    fn docbuf_map(&self, buffer: &[u8], offset: &VTableFieldOffset) -> Result<i32, Error> {
        let field = self.get_field_by_offset_index(offset.0)?;

        match field.r#type {
            VTableFieldType::I32 => {
                let bytes = &buffer[offset.range()];

                let data = i32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);

                Ok(data)
            }
            _ => Err(Error::DocBufMapInvalidFieldType(field.r#type.to_string())),
        }
    }

    #[inline]
    fn docbuf_map_replace(
        &self,
        _new_value: &i32,
        _buffer: &mut Vec<u8>,
        _offset: &VTableFieldOffset,
        _offsets: &mut VTableFieldOffsets,
    ) -> Result<(), Error> {
        unimplemented!("docbuf_map_replace")
    }
}

impl DocBufMap<i64> for &'static VTable {
    #[inline]
    fn docbuf_map(&self, buffer: &[u8], offset: &VTableFieldOffset) -> Result<i64, Error> {
        let field = self.get_field_by_offset_index(offset.0)?;

        match field.r#type {
            VTableFieldType::I64 => {
                let bytes = &buffer[offset.range()];

                let data = i64::from_le_bytes([
                    bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
                ]);

                Ok(data)
            }
            _ => Err(Error::DocBufMapInvalidFieldType(field.r#type.to_string())),
        }
    }

    #[inline]
    fn docbuf_map_replace(
        &self,
        _new_value: &i64,
        _buffer: &mut Vec<u8>,
        _offset: &VTableFieldOffset,
        _offsets: &mut VTableFieldOffsets,
    ) -> Result<(), Error> {
        unimplemented!("docbuf_map_replace")
    }
}

impl DocBufMap<isize> for &'static VTable {
    #[inline]
    fn docbuf_map(&self, buffer: &[u8], offset: &VTableFieldOffset) -> Result<isize, Error> {
        let field = self.get_field_by_offset_index(offset.0)?;

        match field.r#type {
            VTableFieldType::ISIZE => {
                let bytes = &buffer[offset.range()];

                let data = isize::from_le_bytes([
                    bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
                ]);

                Ok(data)
            }
            _ => Err(Error::DocBufMapInvalidFieldType(field.r#type.to_string())),
        }
    }

    #[inline]
    fn docbuf_map_replace(
        &self,
        _new_value: &isize,
        _buffer: &mut Vec<u8>,
        _offset: &VTableFieldOffset,
        _offsets: &mut VTableFieldOffsets,
    ) -> Result<(), Error> {
        unimplemented!("docbuf_map_replace")
    }
}

impl DocBufMap<f32> for &'static VTable {
    #[inline]
    fn docbuf_map(&self, buffer: &[u8], offset: &VTableFieldOffset) -> Result<f32, Error> {
        let field = self.get_field_by_offset_index(offset.0)?;

        match field.r#type {
            VTableFieldType::F32 => {
                let bytes = &buffer[offset.range()];

                let data = f32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);

                Ok(data)
            }
            _ => Err(Error::DocBufMapInvalidFieldType(field.r#type.to_string())),
        }
    }

    #[inline]
    fn docbuf_map_replace(
        &self,
        _new_value: &f32,
        _buffer: &mut Vec<u8>,
        _offset: &VTableFieldOffset,
        _offsets: &mut VTableFieldOffsets,
    ) -> Result<(), Error> {
        unimplemented!("docbuf_map_replace")
    }
}

impl DocBufMap<f64> for &'static VTable {
    #[inline]
    fn docbuf_map(&self, buffer: &[u8], offset: &VTableFieldOffset) -> Result<f64, Error> {
        let field = self.get_field_by_offset_index(offset.0)?;

        match field.r#type {
            VTableFieldType::F64 => {
                let bytes = &buffer[offset.range()];

                let data = f64::from_le_bytes([
                    bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
                ]);

                Ok(data)
            }
            _ => Err(Error::DocBufMapInvalidFieldType(field.r#type.to_string())),
        }
    }

    #[inline]
    fn docbuf_map_replace(
        &self,
        _new_value: &f64,
        _buffer: &mut Vec<u8>,
        _offset: &VTableFieldOffset,
        _offsets: &mut VTableFieldOffsets,
    ) -> Result<(), Error> {
        unimplemented!("docbuf_map_replace")
    }
}

impl DocBufMap<bool> for &'static VTable {
    #[inline]
    fn docbuf_map(&self, buffer: &[u8], offset: &VTableFieldOffset) -> Result<bool, Error> {
        let field = self.get_field_by_offset_index(offset.0)?;

        match field.r#type {
            VTableFieldType::Bool => {
                let data = buffer[offset.range()][0] != 0;

                Ok(data)
            }
            _ => Err(Error::DocBufMapInvalidFieldType(field.r#type.to_string())),
        }
    }

    #[inline]
    fn docbuf_map_replace(
        &self,
        _new_value: &bool,
        _buffer: &mut Vec<u8>,
        _offset: &VTableFieldOffset,
        _offsets: &mut VTableFieldOffsets,
    ) -> Result<(), Error> {
        unimplemented!("docbuf_map_replace")
    }
}

// impl DocBufMap<&str> for &'static VTable {
//     fn docbuf_map(&self, buffer: &[u8], offset: &VTableFieldOffset) -> Result<&&str, Error> {
//         let field = self.get_field_by_offset_index(offset.0)?;

//         match field.r#type {
//             VTableFieldType::Str => {
//                 let data = &buffer[offset.range()];

//                 let data = unsafe { core::str::from_utf8_unchecked(data) };

//                 Ok(&data)
//             }
//             _ => Err(Error::DocBufMapInvalidFieldType(
//                 field.r#type.to_string(),
//             )),
//         }
//     }
// }

// impl DocBufMap<&[u8]> for &'static VTable {
//     fn docbuf_map(&self, buffer: &[u8], offset: &VTableFieldOffset) -> Result<&&[u8], Error> {
//         let field = self.get_field_by_offset_index(offset.0)?;

//         match field.r#type {
//             VTableFieldType::Bytes => {
//                 let data = &buffer[offset.range()];

//                 Ok(&data)
//             }
//             _ => Err(Error::DocBufMapInvalidFieldType(
//                 field.r#type.to_string(),
//             )),
//         }
//     }
// }
