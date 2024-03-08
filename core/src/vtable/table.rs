use std::collections::{HashMap, HashSet};

use super::*;

/// VTable Items are the only structs or enums
#[derive(Debug, Clone)]
pub enum VTableItem<'a> {
    Struct(VTableStruct<'a>),
}

#[derive(Debug, Clone)]
pub struct VTableItems<'a>(pub Vec<VTableItem<'a>>);

impl<'a> VTableItems<'a> {
    pub fn new() -> Self {
        Self(Vec::with_capacity(256))
    }

    pub fn add_struct(&mut self, vtable_struct: VTableStruct<'a>) {
        self.0.push(VTableItem::Struct(vtable_struct));
    }

    pub fn iter(&self) -> std::slice::Iter<'_, VTableItem<'a>> {
        self.0.iter()
    }

    // Inner values
    pub fn inner(&self) -> &Vec<VTableItem<'a>> {
        &self.0
    }

    // Inner values mutable
    pub fn inner_mut(&mut self) -> &mut Vec<VTableItem<'a>> {
        &mut self.0
    }

    // Returns the length of items
    pub fn len(&self) -> usize {
        self.0.len()
    }
}

pub type VTableItemIndex = u8;

#[derive(Debug, Clone)]
pub struct VTable<'a> {
    pub items: VTableItems<'a>,
    pub num_items: VTableItemIndex,
}

impl std::fmt::Display for VTable<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl<'a> VTable<'a> {
    pub fn new() -> Self {
        Self {
            items: VTableItems::new(),
            num_items: 0,
        }
    }

    pub fn add_struct(&mut self, vtable_struct: VTableStruct<'a>) {
        let mut vtable_struct = vtable_struct;
        vtable_struct.set_item_index(self.num_items);
        self.items.add_struct(vtable_struct);
        self.num_items += 1;
    }

    pub fn merge_vtable(&mut self, vtable: &'static VTable) {
        for vtable_item in vtable.items.iter() {
            match vtable_item {
                VTableItem::Struct(vtable_struct) => {
                    self.add_struct(vtable_struct.to_owned());
                }
            }
        }
    }

    pub fn item_by_index(&self, index: VTableItemIndex) -> Result<&VTableItem, Error> {
        if index as usize >= self.items.len() {
            return Err(Error::ItemNotFound);
        }

        for vtable_item in self.items.iter() {
            match vtable_item {
                VTableItem::Struct(vtable_struct) => {
                    if vtable_struct.item_index == index {
                        return Ok(vtable_item);
                    }
                }
            }
        }

        Err(Error::ItemNotFound)
    }

    // Return the struct name from the struct index
    pub fn struct_by_index(&self, index: VTableItemIndex) -> Result<&VTableStruct, Error> {
        if index as usize >= self.items.len() {
            return Err(Error::StructNotFound);
        }

        for vtable_item in self.items.iter() {
            match vtable_item {
                VTableItem::Struct(vtable_struct) => {
                    if vtable_struct.item_index == index {
                        return Ok(vtable_struct);
                    }
                }
            }
        }

        Err(Error::StructNotFound)
    }

    // Return the struct index from the struct name
    pub fn struct_by_name(&self, name: &str) -> Result<&VTableStruct, Error> {
        for vtable_item in self.items.iter() {
            match vtable_item {
                VTableItem::Struct(vtable_struct) => {
                    if vtable_struct.struct_name == name {
                        return Ok(vtable_struct);
                    }
                }
            }
        }

        Err(Error::StructNotFound)
    }

    // Return the field from the vtable by struct index and field index
    pub fn get_struct_field_by_index(
        &self,
        item_index: VTableItemIndex,
        field_index: VTableFieldIndex,
    ) -> Result<&VTableField, Error> {
        let vtable_struct = self.struct_by_index(item_index)?;
        vtable_struct.field_by_index(&field_index)
    }

    // Return the field from the current item index and field index
    pub fn get_item_field_by_index(
        &self,
        item_index: VTableItemIndex,
        field_index: VTableFieldIndex,
    ) -> Result<&VTableField, Error> {
        // println!("Get Item Field By Index");

        self.item_by_index(item_index)
            .and_then(|vtable_item| match vtable_item {
                VTableItem::Struct(vtable_struct) => vtable_struct
                    .field_by_index(&field_index)
                    .map_err(|_| Error::FieldNotFound),
            })
    }

    // pub fn parse_raw_values(&self, input: &[u8]) -> Result<DocBufRawValues, Error> {
    //     let mut current_item_index = self.num_items - 1;
    //     let mut current_field_index = 0;
    //     let mut has_descended = false;
    //     let mut has_visited = HashSet::<(VTableItemIndex, VTableFieldIndex)>::new();

    //     let mut data = DocBufRawValues::new();
    //     let mut input = input.to_vec();

    //     while !input.is_empty() {
    //         // Skip if the field has already been visited.
    //         if has_visited
    //             .get(&(current_item_index, current_field_index))
    //             .is_some()
    //         {
    //             current_field_index += 1;
    //             // println!("Skipping visited field");
    //             continue;
    //         }

    //         // println!("Input: {:?}", input);

    //         // println!("Current Item Index: {}", current_item_index);
    //         // println!("Current Field Index: {}", current_field_index);
    //         // println!("Number of Items: {}", self.num_items);
    //         // println!("Has Descended: {}", has_descended);

    //         let item = self.item_by_index(current_item_index)?;

    //         // println!("Item: {:?}", item);

    //         match item {
    //             VTableItem::Struct(s) => {
    //                 match s.field_by_index(&current_field_index) {
    //                     Ok(field) => {
    //                         // println!("Decoding Field: {:?}", field);

    //                         // Memoize the current item and field index
    //                         has_visited.insert((current_item_index, current_field_index));

    //                         // If the field type is a struct, decrement the current item index
    //                         // to descend into the struct. reset the field index to 0.
    //                         match field.field_type {
    //                             VTableFieldType::Struct(_) => {
    //                                 if !has_descended {
    //                                     current_item_index -= 1;
    //                                     current_field_index = 0;
    //                                     continue;
    //                                 }
    //                             }
    //                             _ => {
    //                                 let field_data = field.decode(&mut input)?;

    //                                 if !field_data.is_empty() {
    //                                     data.insert(
    //                                         current_item_index,
    //                                         current_field_index,
    //                                         field_data,
    //                                     );
    //                                 }
    //                             }
    //                         };

    //                         if current_field_index < s.num_fields - 1 {
    //                             // println!("Incrementing Field Index");
    //                             current_field_index += 1;
    //                         } else if current_item_index > 0 && !has_descended {
    //                             // println!("Decrementing Item Index");
    //                             current_item_index -= 1;
    //                             current_field_index = 0;
    //                         } else if current_item_index == 0
    //                             && current_field_index == s.num_fields - 1
    //                             && !has_descended
    //                         {
    //                             // println!("Has Descended");
    //                             has_descended = true;
    //                             current_item_index += 1;
    //                             current_field_index = 0;
    //                         } else if current_item_index < self.num_items - 1 && has_descended {
    //                             // println!("Incrementing Item Index");
    //                             current_item_index += 1;
    //                             current_field_index = 0;
    //                         } else {
    //                             return Err(Error::FailedToParseData);
    //                         }
    //                     }
    //                     _ => {
    //                         unimplemented!("error handle parse data");

    //                         // if current_item_index == self.num_items - 1
    //                         //     && current_field_index == s.num_fields - 1
    //                         //     && has_descended
    //                         // {
    //                         //     println!("Data: {:?}", data);
    //                         //     println!("Input: {:?}", input);

    //                         //     // If we've reached the end of the items
    //                         //     // and the data is not empty, we must
    //                         //     // return an error.
    //                         //     return Err(Error::FailedToParseData);
    //                         // }

    //                         // if !has_descended
    //                         //     && current_item_index == 0
    //                         //     && current_field_index == s.num_fields
    //                         // {
    //                         //     // reached the end of the items
    //                         //     has_descended = true;
    //                         //     // begin to "resurface" to the top of the vtable
    //                         //     current_item_index += 1;
    //                         //     // Reset the field index to 0
    //                         //     current_field_index = 0;
    //                         // }

    //                         // if has_descended {
    //                         //     current_item_index += 1;
    //                         // } else {
    //                         //     current_item_index -= 1;
    //                         // }

    //                         // current_field_index = 0;
    //                     }
    //                 }
    //             }
    //         };
    //     }

    //     // println!("Data: {:?}", data);

    //     // unimplemented!("parse raw values");

    //     Ok(data)
    // }
}

#[derive(Debug, Clone)]
pub struct DocBufRawValues(HashMap<VTableItemIndex, HashMap<VTableFieldIndex, Vec<u8>>>);

impl DocBufRawValues {
    pub fn new() -> Self {
        DocBufRawValues(HashMap::new())
    }

    pub fn insert(
        &mut self,
        item_index: VTableItemIndex,
        field_index: VTableFieldIndex,
        value: Vec<u8>,
    ) {
        self.0
            .entry(item_index)
            .or_insert_with(HashMap::new)
            .insert(field_index, value);
    }

    pub fn get(
        &self,
        item_index: VTableItemIndex,
        field_index: VTableFieldIndex,
    ) -> Option<&Vec<u8>> {
        self.0.get(&item_index)?.get(&field_index)
    }

    // Check if the raw values is empty
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    // Remove the value from the raw values
    pub fn remove(
        &mut self,
        item_index: VTableItemIndex,
        field_index: VTableFieldIndex,
    ) -> Option<Vec<u8>> {
        let structs = self.0.get_mut(&item_index)?;
        let value = structs.remove(&field_index);

        // Remove the struct if it's empty
        if structs.is_empty() {
            self.0.remove(&item_index);
        }

        value
    }
}
