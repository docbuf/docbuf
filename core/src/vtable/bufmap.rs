use crate::traits::DocBufMap;

use super::*;

impl DocBufMap<String> for &'static VTable<'static> {
    fn docbuf_map(&self, docbuf: &[u8], offset: &VTableFieldOffset) -> Result<String, Error> {
        let field = self.get_item_field_by_index(offset.0, offset.1)?;

        // println!("Field: {:?}", field);

        match field.field_type {
            VTableFieldType::String => {
                let data = String::from_utf8(docbuf[offset.2.clone()].to_vec())?;

                Ok(data)
            }
            _ => Err(Error::DocBufMapInvalidFieldType(
                field.field_type.to_string(),
            )),
        }
    }
}

impl DocBufMap<Vec<u8>> for &'static VTable<'static> {
    fn docbuf_map(&self, docbuf: &[u8], offset: &VTableFieldOffset) -> Result<Vec<u8>, Error> {
        let field = self.get_item_field_by_index(offset.0, offset.1)?;

        // println!("Field: {:?}", field);

        match field.field_type {
            VTableFieldType::Bytes => {
                let data = docbuf[offset.2.clone()].to_vec();

                Ok(data)
            }
            _ => Err(Error::DocBufMapInvalidFieldType(
                field.field_type.to_string(),
            )),
        }
    }
}

impl DocBufMap<u8> for &'static VTable<'static> {
    fn docbuf_map(&self, docbuf: &[u8], offset: &VTableFieldOffset) -> Result<u8, Error> {
        let field = self.get_item_field_by_index(offset.0, offset.1)?;

        match field.field_type {
            VTableFieldType::U8 => {
                let data = docbuf[offset.2.clone()][0];

                Ok(data)
            }
            _ => Err(Error::DocBufMapInvalidFieldType(
                field.field_type.to_string(),
            )),
        }
    }
}

impl DocBufMap<u16> for &'static VTable<'static> {
    fn docbuf_map(&self, docbuf: &[u8], offset: &VTableFieldOffset) -> Result<u16, Error> {
        let field = self.get_item_field_by_index(offset.0, offset.1)?;

        match field.field_type {
            VTableFieldType::U16 => {
                let bytes = &docbuf[offset.2.clone()];

                let data = u16::from_le_bytes([bytes[0], bytes[1]]);

                Ok(data)
            }
            _ => Err(Error::DocBufMapInvalidFieldType(
                field.field_type.to_string(),
            )),
        }
    }
}

impl DocBufMap<u32> for &'static VTable<'static> {
    fn docbuf_map(&self, docbuf: &[u8], offset: &VTableFieldOffset) -> Result<u32, Error> {
        let field = self.get_item_field_by_index(offset.0, offset.1)?;

        match field.field_type {
            VTableFieldType::U32 => {
                let bytes = &docbuf[offset.2.clone()];

                let data = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);

                Ok(data)
            }
            _ => Err(Error::DocBufMapInvalidFieldType(
                field.field_type.to_string(),
            )),
        }
    }
}

impl DocBufMap<u64> for &'static VTable<'static> {
    fn docbuf_map(&self, docbuf: &[u8], offset: &VTableFieldOffset) -> Result<u64, Error> {
        let field = self.get_item_field_by_index(offset.0, offset.1)?;

        match field.field_type {
            VTableFieldType::U64 => {
                let bytes = &docbuf[offset.2.clone()];

                let data = u64::from_le_bytes([
                    bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
                ]);

                Ok(data)
            }
            _ => Err(Error::DocBufMapInvalidFieldType(
                field.field_type.to_string(),
            )),
        }
    }
}

impl DocBufMap<usize> for &'static VTable<'static> {
    fn docbuf_map(&self, docbuf: &[u8], offset: &VTableFieldOffset) -> Result<usize, Error> {
        let field = self.get_item_field_by_index(offset.0, offset.1)?;

        match field.field_type {
            VTableFieldType::USIZE => {
                let bytes = &docbuf[offset.2.clone()];

                let data = usize::from_le_bytes([
                    bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
                ]);

                Ok(data)
            }
            _ => Err(Error::DocBufMapInvalidFieldType(
                field.field_type.to_string(),
            )),
        }
    }
}

impl DocBufMap<i8> for &'static VTable<'static> {
    fn docbuf_map(&self, docbuf: &[u8], offset: &VTableFieldOffset) -> Result<i8, Error> {
        let field = self.get_item_field_by_index(offset.0, offset.1)?;

        match field.field_type {
            VTableFieldType::I8 => {
                let data = docbuf[offset.2.clone()][0] as i8;

                Ok(data)
            }
            _ => Err(Error::DocBufMapInvalidFieldType(
                field.field_type.to_string(),
            )),
        }
    }
}

impl DocBufMap<i16> for &'static VTable<'static> {
    fn docbuf_map(&self, docbuf: &[u8], offset: &VTableFieldOffset) -> Result<i16, Error> {
        let field = self.get_item_field_by_index(offset.0, offset.1)?;

        match field.field_type {
            VTableFieldType::I16 => {
                let bytes = &docbuf[offset.2.clone()];

                let data = i16::from_le_bytes([bytes[0], bytes[1]]);

                Ok(data)
            }
            _ => Err(Error::DocBufMapInvalidFieldType(
                field.field_type.to_string(),
            )),
        }
    }
}

impl DocBufMap<i32> for &'static VTable<'static> {
    fn docbuf_map(&self, docbuf: &[u8], offset: &VTableFieldOffset) -> Result<i32, Error> {
        let field = self.get_item_field_by_index(offset.0, offset.1)?;

        match field.field_type {
            VTableFieldType::I32 => {
                let bytes = &docbuf[offset.2.clone()];

                let data = i32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);

                Ok(data)
            }
            _ => Err(Error::DocBufMapInvalidFieldType(
                field.field_type.to_string(),
            )),
        }
    }
}

impl DocBufMap<i64> for &'static VTable<'static> {
    fn docbuf_map(&self, docbuf: &[u8], offset: &VTableFieldOffset) -> Result<i64, Error> {
        let field = self.get_item_field_by_index(offset.0, offset.1)?;

        match field.field_type {
            VTableFieldType::I64 => {
                let bytes = &docbuf[offset.2.clone()];

                let data = i64::from_le_bytes([
                    bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
                ]);

                Ok(data)
            }
            _ => Err(Error::DocBufMapInvalidFieldType(
                field.field_type.to_string(),
            )),
        }
    }
}

impl DocBufMap<isize> for &'static VTable<'static> {
    fn docbuf_map(&self, docbuf: &[u8], offset: &VTableFieldOffset) -> Result<isize, Error> {
        let field = self.get_item_field_by_index(offset.0, offset.1)?;

        match field.field_type {
            VTableFieldType::ISIZE => {
                let bytes = &docbuf[offset.2.clone()];

                let data = isize::from_le_bytes([
                    bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
                ]);

                Ok(data)
            }
            _ => Err(Error::DocBufMapInvalidFieldType(
                field.field_type.to_string(),
            )),
        }
    }
}

impl DocBufMap<f32> for &'static VTable<'static> {
    fn docbuf_map(&self, docbuf: &[u8], offset: &VTableFieldOffset) -> Result<f32, Error> {
        let field = self.get_item_field_by_index(offset.0, offset.1)?;

        match field.field_type {
            VTableFieldType::F32 => {
                let bytes = &docbuf[offset.2.clone()];

                let data = f32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);

                Ok(data)
            }
            _ => Err(Error::DocBufMapInvalidFieldType(
                field.field_type.to_string(),
            )),
        }
    }
}

impl DocBufMap<f64> for &'static VTable<'static> {
    fn docbuf_map(&self, docbuf: &[u8], offset: &VTableFieldOffset) -> Result<f64, Error> {
        let field = self.get_item_field_by_index(offset.0, offset.1)?;

        match field.field_type {
            VTableFieldType::F64 => {
                let bytes = &docbuf[offset.2.clone()];

                let data = f64::from_le_bytes([
                    bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
                ]);

                Ok(data)
            }
            _ => Err(Error::DocBufMapInvalidFieldType(
                field.field_type.to_string(),
            )),
        }
    }
}

impl DocBufMap<bool> for &'static VTable<'static> {
    fn docbuf_map(&self, docbuf: &[u8], offset: &VTableFieldOffset) -> Result<bool, Error> {
        let field = self.get_item_field_by_index(offset.0, offset.1)?;

        match field.field_type {
            VTableFieldType::Bool => {
                let data = docbuf[offset.2.clone()][0] != 0;

                Ok(data)
            }
            _ => Err(Error::DocBufMapInvalidFieldType(
                field.field_type.to_string(),
            )),
        }
    }
}

// impl DocBufMap<&'static str> for &'static VTable<'static> {
//     fn docbuf_map(&self, docbuf: &[u8], offset: &VTableFieldOffset) -> Result<&'static str, Error> {
//         let field = self.get_item_field_by_index(offset.0, offset.1)?;

//         match field.field_type {
//             VTableFieldType::Str => {
//                 let data = &docbuf[offset.2.clone()];

//                 let data = unsafe { core::str::from_utf8_unchecked(data) };

//                 Ok(data)
//             }
//             _ => Err(Error::DocBufMapInvalidFieldType(
//                 field.field_type.to_string(),
//             )),
//         }
//     }
// }

// impl DocBufMap<&'static [u8]> for &'static VTable<'static> {
//     fn docbuf_map(
//         &self,
//         docbuf: &[u8],
//         offset: &VTableFieldOffset,
//     ) -> Result<&'static [u8], Error> {
//         let field = self.get_item_field_by_index(offset.0, offset.1)?;

//         match field.field_type {
//             VTableFieldType::Bytes => {
//                 let data = &docbuf[offset.2.clone()];

//                 Ok(data)
//             }
//             _ => Err(Error::DocBufMapInvalidFieldType(
//                 field.field_type.to_string(),
//             )),
//         }
//     }
// }
