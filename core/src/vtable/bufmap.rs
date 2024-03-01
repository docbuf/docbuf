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
            _ => Err(Error::InvalidMemoryMapFieldType(
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
            _ => Err(Error::InvalidMemoryMapFieldType(
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
            _ => Err(Error::InvalidMemoryMapFieldType(
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
            _ => Err(Error::InvalidMemoryMapFieldType(
                field.field_type.to_string(),
            )),
        }
    }
}
