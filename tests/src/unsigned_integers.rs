use docbuf_core::{traits::DocBuf, uuid::Uuid};
use docbuf_db::traits::*;
use docbuf_macros::*;
use serde::{Deserialize, Serialize};

use crate::{SetTestValues, TestHarness};

#[derive(Debug)]
#[allow(dead_code)]
enum UnsignedInteger {
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    Usize(usize),
}

impl UnsignedInteger {
    pub fn set_doc_value<'de, D: TestHarness<'de>>(&self, doc: &mut D) {
        match self {
            UnsignedInteger::U8(value) => doc.set_u8_value(*value),
            UnsignedInteger::U16(value) => doc.set_u16_value(*value),
            UnsignedInteger::U32(value) => doc.set_u32_value(*value),
            UnsignedInteger::U64(value) => doc.set_u64_value(*value),
            UnsignedInteger::Usize(value) => doc.set_usize_value(*value),
        }
    }
}

// Run test cases for unsigned integer values
// This function is used to run test cases for unsigned integer values
// It takes a generic type D that implements the DocBuf and SetTestValues traits
// It also takes a vector of test cases, where each test case is a tuple of a boolean and an unsigned integer
// The boolean value indicates whether the test case is expected to pass or fail
// The function returns a Result with an empty tuple or an error
fn run_test_cases<'de, D: TestHarness<'de>>(
    test_cases: Vec<(bool, UnsignedInteger)>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut doc = D::default();

    let mut buffer = Vec::with_capacity(1024);

    for (expected, value) in test_cases {
        value.set_doc_value(&mut doc);

        match expected {
            true => {
                // Test serialization size
                doc = doc.assert_serialization_size(&mut buffer)?;
            }
            false => {
                assert!(
                    doc.to_docbuf(&mut buffer).is_err(),
                    "Expected `{:?}` to be `{expected}`",
                    value
                );
            }
        }
    }

    Ok(())
}

#[docbuf]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct U8Value {
    #[docbuf {
        // Check only for values between 200 and 255
        min_value = 200,
        max_value = 255,
    }]
    pub u8_value: u8,
}

impl PartialEq for U8Value {
    fn eq(&self, other: &Self) -> bool {
        self.u8_value == other.u8_value
    }
}

impl Eq for U8Value {}

impl std::fmt::Display for U8Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "U8Value {{ u8_value: {} }}", self.u8_value)
    }
}

impl SetTestValues for U8Value {
    fn set_u8_value(&mut self, value: u8) {
        self.u8_value = value;
    }
}

impl<'de> TestHarness<'de> for U8Value {}

#[test]
fn test_u8() -> Result<(), Box<dyn std::error::Error>> {
    // Test cases
    let test_cases = vec![
        (false, UnsignedInteger::U8(0)),
        (true, UnsignedInteger::U8(255)),
        (false, UnsignedInteger::U8(199)),
        (false, UnsignedInteger::U8(80)),
        (true, UnsignedInteger::U8(200)),
        (true, UnsignedInteger::U8(230)),
    ];

    run_test_cases::<U8Value>(test_cases)
}

// #[derive(Debug, Clone, DocBuf, Serialize, Deserialize, Default)]
#[docbuf]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct U16Value {
    #[docbuf {
        // Check only for values between 200 and 255
        min_value = 256,
    }]
    pub u16_value: u16,
}

impl PartialEq for U16Value {
    fn eq(&self, other: &Self) -> bool {
        self.u16_value == other.u16_value
    }
}

impl Eq for U16Value {}

impl std::fmt::Display for U16Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "U16Value {{ u16_value: {} }}", self.u16_value)
    }
}

impl SetTestValues for U16Value {
    fn set_u16_value(&mut self, value: u16) {
        self.u16_value = value;
    }
}

impl<'de> TestHarness<'de> for U16Value {}

#[test]
fn test_u16() -> Result<(), Box<dyn std::error::Error>> {
    // Test cases
    let test_cases = vec![
        (false, UnsignedInteger::U16(0)),
        (false, UnsignedInteger::U16(255)),
        (false, UnsignedInteger::U16(199)),
        (false, UnsignedInteger::U16(80)),
        (false, UnsignedInteger::U16(200)),
        (false, UnsignedInteger::U16(230)),
        (true, UnsignedInteger::U16(300)),
    ];

    run_test_cases::<U16Value>(test_cases)
}
