#![allow(unused_imports)]
#![allow(dead_code)]

// pub mod benchmarks;
pub mod complex;
#[cfg(feature = "db")]
pub mod database;
pub mod process;
#[cfg(feature = "rpc")]
pub mod rpc;
pub mod strings;
pub mod unsigned_integers;
pub mod vtable;

use docbuf_core::traits::DocBuf;
use serde::{Deserialize, Serialize};

// Re-export testing dependencies
pub mod test_deps {
    pub use bincode;
    pub use serde_json;
}

trait TestHarness<'de>:
    DocBuf + SetTestValues + Default + Deserialize<'de> + Serialize + std::fmt::Debug + PartialEq
{
    /// Assert the serialization size of the document buffer is less than or equal to the size of the
    /// bincode and JSON serialization.
    fn assert_serialization_size<'a>(
        self,
        buffer: &'a mut Vec<u8>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        self.to_docbuf(buffer)?;

        let bincode_bytes = bincode::serialize(&self)?;
        let json_bytes = serde_json::to_vec(&self)?;

        assert!(
            buffer.len() <= bincode_bytes.len(),
            "docbuf byte length: {}\nbincode byte length: {}",
            buffer.len(),
            bincode_bytes.len()
        );
        assert!(
            buffer.len() <= json_bytes.len(),
            "docbuf byte length: {}\njson byte length: {}",
            buffer.len(),
            json_bytes.len()
        );

        Ok(self)
    }

    /// Assert the serialize and deserialization round trip of the document buffer.
    fn assert_serialization_round_trip<'a>(
        self,
        buffer: &'a mut Vec<u8>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        // Serialize
        self.to_docbuf(buffer)?;

        let docbuf = buffer.clone();

        // deserialize
        let doc = Self::from_docbuf(buffer)?;

        assert_eq!(buffer.len(), 0);

        // Serialize again
        doc.to_docbuf(buffer)?;

        assert_ne!(buffer.len(), 0);

        // Check the buffer length is the same.
        // It is not guaranteed that all the bytes are the same, but the length should be the same.
        // The bytes may be different because of the way the document buffer is serialized, namely the hashmap
        // keys are serialized in a different order.
        assert_eq!(docbuf.len(), buffer.len());

        Ok(self)
    }
}

trait SetTestValues {
    fn set_string_value(&mut self, _value: String) {
        unimplemented!("set_string");
    }

    fn set_u8_value(&mut self, _value: u8) {
        unimplemented!("set_u8");
    }

    fn set_u16_value(&mut self, _value: u16) {
        unimplemented!("set_u16");
    }

    fn set_u32_value(&mut self, _value: u32) {
        unimplemented!("set_u32");
    }

    fn set_u64_value(&mut self, _value: u64) {
        unimplemented!("set_u64");
    }

    fn set_usize_value(&mut self, _value: usize) {
        unimplemented!("set_usize");
    }

    fn set_i8_value(&mut self, _value: i8) {
        unimplemented!("set_i8");
    }

    fn set_i16_value(&mut self, _value: i16) {
        unimplemented!("set_i16");
    }

    fn set_i32_value(&mut self, _value: i32) {
        unimplemented!("set_i32");
    }

    fn set_i64_value(&mut self, _value: i64) {
        unimplemented!("set_i64");
    }

    fn set_isize_value(&mut self, _value: isize) {
        unimplemented!("set_isize");
    }

    fn set_f32_value(&mut self, _value: f32) {
        unimplemented!("set_f32");
    }

    fn set_f64_value(&mut self, _value: f64) {
        unimplemented!("set_f64");
    }

    fn set_bool_value(&mut self, _value: bool) {
        unimplemented!("set_bool");
    }

    fn set_bytes_value(&mut self, _value: Vec<u8>) {
        unimplemented!("set_bytes");
    }

    fn set_byte_slice_value(&mut self, _value: &[u8]) {
        unimplemented!("set_byte_slice");
    }

    fn set_option_value(&mut self, _value: Option<String>) {
        unimplemented!("set_option");
    }

    fn set_vec_value(&mut self, _value: Vec<String>) {
        unimplemented!("set_vec");
    }
}
