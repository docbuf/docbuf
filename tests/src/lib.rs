// pub mod benchmarks;
pub mod complex;
pub mod strings;
pub mod unsigned_integers;

use docbuf_core::traits::DocBuf;
use serde::{Deserialize, Serialize};

// Re-export testing dependencies
pub mod test_deps {
    pub use bincode;
    pub use serde_json;
}

trait TestHarness<'de>: DocBuf + SetTestValues + Default + Deserialize<'de> + Serialize {
    /// Assert the serialization size of the document buffer is less than or equal to the size of the
    /// bincode and JSON serialization.
    fn assert_serialization_size(&self) -> Result<(), Box<dyn std::error::Error>> {
        let docbuf_bytes = self.to_docbuf()?;
        let bincode_bytes = bincode::serialize(self)?;
        let json_bytes = serde_json::to_vec(self)?;

        assert!(
            docbuf_bytes.len() <= bincode_bytes.len(),
            "docbuf byte length: {}\nbincode byte length: {}",
            docbuf_bytes.len(),
            bincode_bytes.len()
        );
        assert!(
            docbuf_bytes.len() <= json_bytes.len(),
            "docbuf byte length: {}\njson byte length: {}",
            docbuf_bytes.len(),
            json_bytes.len()
        );

        Ok(())
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
