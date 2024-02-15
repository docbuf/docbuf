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
        // Sort the vtable structs by struct name
        let mut vtable_structs = vtable.structs.values().collect::<Vec<&VTableStruct>>();

        // Sorting is required to ensure the structs are added in a consistent order
        vtable_structs.sort_by(|a, b| {
            a.struct_name_as_bytes
                .cmp(&b.struct_name_as_bytes)
                .then(a.struct_index.cmp(&b.struct_index))
        });

        for vtable_struct in vtable_structs {
            self.add_struct(vtable_struct.clone());
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        for vtable_struct in self.structs.values() {
            let packed_bytes = vtable_struct.to_bytes();
            bytes.extend_from_slice(&packed_bytes);
            // Add a separator value type of `0xFF`
            // bytes.push(STRUCT_SEPARATOR);
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

    // Return the field from the vtable by struct index and field index
    pub fn get_struct_field_by_index(
        &self,
        struct_index: StructIndex,
        field_index: FieldIndex,
    ) -> Result<&VTableField, Error> {
        let vtable_struct = self.struct_by_index(&struct_index)?;
        vtable_struct.field_by_index(&field_index)
    }

    pub fn parse_raw_values(&self, input: &[u8]) -> Result<DocBufRawValues, Error> {
        let mut current_struct_index = 0;
        let mut current_field_index = 0;

        let mut data = DocBufRawValues::new();
        let mut input = input.to_vec();

        while !input.is_empty() {
            match self.get_struct_field_by_index(current_struct_index, current_field_index) {
                Ok(field) => {
                    let field_data = field.decode(&mut input)?;

                    if !field_data.is_empty() {
                        data.insert_value(current_struct_index, current_field_index, field_data);
                    }

                    current_field_index += 1;
                }
                _ => {
                    current_struct_index += 1;
                    current_field_index = 0;
                }
            }
        }

        Ok(data)
    }
}

#[derive(Debug, Clone)]
pub struct DocBufRawValues(HashMap<StructIndex, HashMap<FieldIndex, Vec<u8>>>);

impl DocBufRawValues {
    pub fn new() -> Self {
        DocBufRawValues(HashMap::new())
    }

    pub fn insert_value(
        &mut self,
        struct_index: StructIndex,
        field_index: FieldIndex,
        value: Vec<u8>,
    ) {
        self.0
            .entry(struct_index)
            .or_insert_with(HashMap::new)
            .insert(field_index, value);
    }

    pub fn get(&self, struct_index: StructIndex, field_index: FieldIndex) -> Option<&Vec<u8>> {
        self.0.get(&struct_index)?.get(&field_index)
    }

    // Check if the raw values is empty
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    // Remove the value from the raw values
    pub fn remove(
        &mut self,
        struct_index: StructIndex,
        field_index: FieldIndex,
    ) -> Option<Vec<u8>> {
        let structs = self.0.get_mut(&struct_index)?;
        let value = structs.remove(&field_index);

        // Remove the struct if it's empty
        if structs.is_empty() {
            self.0.remove(&struct_index);
        }

        value
    }
}
