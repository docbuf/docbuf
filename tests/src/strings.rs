use docbuf_core::traits::DocBuf;
use docbuf_db::traits::*;
use docbuf_macros::*;
use serde::{Deserialize, Serialize};

use crate::{SetTestValues, TestHarness};

// Run test cases for string values
fn run_test_cases<'de, D: TestHarness<'de>>(
    test_cases: Vec<(bool, impl Into<String>)>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut doc = D::default();

    // Create buffer to store serialized data
    let mut buffer = Vec::with_capacity(1024);

    for (expected, value) in test_cases {
        doc.set_string_value(value.into());

        match expected {
            true => {
                doc = doc.assert_serialization_size(&mut buffer)?;
            }
            false => {
                assert!(doc.to_docbuf(&mut buffer).is_err());
            }
        }
    }

    Ok(())
}

#[test]
fn test_regex_lowercase() -> Result<(), Box<dyn std::error::Error>> {
    #[docbuf]
    #[derive(Debug, Clone, Serialize, Deserialize, Default)]
    pub struct Document {
        #[docbuf {
            // Check only for lowercase letters with three characters
            regex = r"^[a-z]{3}$";
        }]
        pub string_value: String,
    }

    // Implement Eq and Partial Eq for the Document struct
    impl PartialEq for Document {
        fn eq(&self, other: &Self) -> bool {
            self.string_value == other.string_value
        }
    }

    impl Eq for Document {}

    impl SetTestValues for Document {
        fn set_string_value(&mut self, value: String) {
            self.string_value = value;
        }
    }

    impl<'de> TestHarness<'de> for Document {}

    // Test cases
    let test_cases = vec![
        (true, "ell"),
        (false, "helloword"),
        (false, "abcdefghijklmnopqrstuvwxyz"),
        (false, "ABCDEF"),
        (false, "Hello"),
        (false, "HelloWorld"),
        (false, "123456"),
        (false, "Hello, World!"),
        (false, ""),
        (false, "123"),
        (true, "abc"),
        (true, "def"),
        (true, "ghi"),
    ];

    run_test_cases::<Document>(test_cases)?;

    Ok(())
}

#[test]
fn test_regex_uuid() -> Result<(), Box<dyn std::error::Error>> {
    #[docbuf]
    #[derive(Debug, Clone, Serialize, Deserialize, Default)]
    pub struct Document {
        #[docbuf {
            // Check regex for uuid v4
            regex = r"^[a-f0-9]{8}-[a-f0-9]{4}-[a-f0-9]{4}-[a-f0-9]{4}-[a-f0-9]{12}$";
        }]
        pub string_value: String,
    }

    impl Eq for Document {}

    impl PartialEq for Document {
        fn eq(&self, other: &Self) -> bool {
            self.string_value == other.string_value
        }
    }

    impl SetTestValues for Document {
        fn set_string_value(&mut self, value: String) {
            self.string_value = value;
        }
    }

    impl<'de> TestHarness<'de> for Document {}

    // Test cases
    let test_cases = vec![
        (true, "0f9a72d4-cc66-11ee-885c-6b81f58bbf63"),
        (false, "0f9a72d4-cc66-11ee-885c-6b81f"),
        (
            false,
            "0f9a72d4-cc66-11ee-885c-6b81f58bbf63-0f9a72d4-cc66-11ee-885c-6b81f58bbf63",
        ),
        (true, "26140d5e-cc66-11ee-8027-4b82f4324732"),
        (false, "hello"),
        (false, "6140d5e-cc66-11ee-8027-4b82f4324732"),
    ];

    run_test_cases::<Document>(test_cases)?;

    Ok(())
}

#[test]
fn test_min_max_length() -> Result<(), Box<dyn std::error::Error>> {
    #[docbuf]
    #[derive(Debug, Clone, Serialize, Deserialize, Default)]
    pub struct Document {
        #[docbuf {
            min_length = 4;
            max_length = 100;
        }]
        pub string_value: String,
    }

    impl Eq for Document {}

    impl PartialEq for Document {
        fn eq(&self, other: &Self) -> bool {
            self.string_value == other.string_value
        }
    }

    impl SetTestValues for Document {
        fn set_string_value(&mut self, value: String) {
            self.string_value = value;
        }
    }

    impl<'de> TestHarness<'de> for Document {}

    // Test cases
    let test_cases = vec![
        (true, "Hello"),
        (true, "Hello, World!"),
        (false, "el"),
        (false, ""),
        (false, "123"),
    ];

    run_test_cases::<Document>(test_cases)?;

    Ok(())
}

#[test]
fn test_length() -> Result<(), Box<dyn std::error::Error>> {
    #[docbuf]
    #[derive(Debug, Clone, Serialize, Deserialize, Default)]
    pub struct Document {
        #[docbuf {
            length = 5;
        }]
        pub string_value: String,
    }

    impl Eq for Document {}

    impl PartialEq for Document {
        fn eq(&self, other: &Self) -> bool {
            self.string_value == other.string_value
        }
    }

    impl SetTestValues for Document {
        fn set_string_value(&mut self, value: String) {
            self.string_value = value;
        }
    }

    impl<'de> TestHarness<'de> for Document {}

    // Test cases
    let test_cases = vec![
        (true, "Hello"),
        (true, "abcde"),
        (true, "ABCDE"),
        (false, "abcdef"),
        (false, "Hello, World!"),
        (false, "el"),
        (false, ""),
        (false, "123"),
        (true, "12345"),
    ];

    run_test_cases::<Document>(test_cases)?;

    Ok(())
}
