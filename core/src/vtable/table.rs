use std::collections::HashMap;

use super::*;

#[derive(Debug, Clone)]
pub struct VTable {
    pub structs: HashMap<StructNameAsBytes, VTableStruct>,
    pub num_structs: StructIndex,
}

impl VTable {
    pub fn new() -> Self {
        Self {
            structs: HashMap::new(),
            num_structs: 0,
        }
    }

    pub fn add_struct(&mut self, vtable_struct: VTableStruct) {
        let mut vtable_struct = vtable_struct;
        vtable_struct.set_struct_index(self.num_structs);
        self.structs.insert(
            vtable_struct.struct_name_as_bytes.clone(),
            vtable_struct.clone(),
        );
        self.num_structs += 1;
    }

    pub fn merge_vtable(&mut self, vtable: VTable) {
        for vtable_struct in vtable.structs.values() {
            self.add_struct(vtable_struct.clone());
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        for vtable_struct in self.structs.values() {
            let packed_bytes = vtable_struct.to_bytes();
            println!("VTable Struct: {:?}", packed_bytes);
            bytes.extend_from_slice(&packed_bytes);
            // Add a separator value type of `0xFF`
            bytes.push(STRUCT_SEPARATOR);
        }

        bytes
    }

    // Return the struct name from the struct index
    pub fn struct_by_index(&self, index: &StructIndex) -> Result<&VTableStruct, Error> {
        for vtable_struct in self.structs.values() {
            if vtable_struct.struct_index == *index {
                return Ok(vtable_struct);
            }
        }

        Err(Error::StructNotFound)
    }

    // Return the struct index from the struct name
    pub fn struct_by_name(&self, name: impl AsRef<[u8]>) -> Result<&VTableStruct, Error> {
        for vtable_struct in self.structs.values() {
            if vtable_struct.struct_name_as_bytes == *name.as_ref() {
                return Ok(vtable_struct);
            }
        }

        Err(Error::StructNotFound)
    }
}
