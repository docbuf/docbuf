pub mod complex;
pub mod strings;

trait SetTestValues {
    fn set_string_value(&mut self, value: String) {
        unimplemented!("set_string");
    }

    fn set_u8_value(&mut self, value: u8) {
        unimplemented!("set_u8");
    }

    fn set_u16_value(&mut self, value: u16) {
        unimplemented!("set_u16");
    }

    fn set_u32_value(&mut self, value: u32) {
        unimplemented!("set_u32");
    }

    fn set_u64_value(&mut self, value: u64) {
        unimplemented!("set_u64");
    }

    fn set_usize_value(&mut self, value: usize) {
        unimplemented!("set_usize");
    }

    fn set_i8_value(&mut self, value: i8) {
        unimplemented!("set_i8");
    }

    fn set_i16_value(&mut self, value: i16) {
        unimplemented!("set_i16");
    }

    fn set_i32_value(&mut self, value: i32) {
        unimplemented!("set_i32");
    }

    fn set_i64_value(&mut self, value: i64) {
        unimplemented!("set_i64");
    }

    fn set_isize_value(&mut self, value: isize) {
        unimplemented!("set_isize");
    }

    fn set_f32_value(&mut self, value: f32) {
        unimplemented!("set_f32");
    }

    fn set_f64_value(&mut self, value: f64) {
        unimplemented!("set_f64");
    }

    fn set_bool_value(&mut self, value: bool) {
        unimplemented!("set_bool");
    }

    fn set_bytes_value(&mut self, value: Vec<u8>) {
        unimplemented!("set_bytes");
    }

    fn set_byte_slice_value(&mut self, value: &[u8]) {
        unimplemented!("set_byte_slice");
    }

    fn set_option_value(&mut self, value: Option<String>) {
        unimplemented!("set_option");
    }

    fn set_vec_value(&mut self, value: Vec<String>) {
        unimplemented!("set_vec");
    }
}
