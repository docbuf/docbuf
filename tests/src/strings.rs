#[cfg(test)]
mod tests {
    use docbuf_core::{error::Error, traits::DocBuf};
    use docbuf_macros::*;
    use serde::{Deserialize, Serialize};

    use crate::SetTestValues;

    // Run test cases for string values
    fn run_test_cases<D: DocBuf + SetTestValues + Default>(
        test_cases: Vec<(bool, &str)>,
    ) -> Result<(), Error> {
        let mut doc = D::default();

        for (expected, value) in test_cases {
            doc.set_string_value(String::from(value));

            match expected {
                true => {
                    let bytes = doc.to_docbuf()?;
                    D::from_docbuf(&bytes)?;
                }
                false => {
                    assert!(doc.to_docbuf().is_err());
                }
            }
        }

        Ok(())
    }

    #[test]
    fn test_regex_lowercase() -> Result<(), Error> {
        #[derive(Debug, Clone, DocBuf, Serialize, Deserialize, Default)]
        #[docbuf {}]
        pub struct Document {
            #[docbuf {
                // Check only for lowercase letters with at least one character
                regex = r"^[a-z]+$";
            }]
            pub string_value: String,
        }

        impl SetTestValues for Document {
            fn set_string_value(&mut self, value: String) {
                self.string_value = value;
            }
        }

        // Test cases
        let test_cases = vec![
            (true, "el"),
            (true, "helloword"),
            (true, "abcdefghijklmnopqrstuvwxyz"),
            (false, "ABCDEF"),
            (false, "Hello"),
            (false, "HelloWorld"),
            (false, "123456"),
            (false, "Hello, World!"),
            (false, ""),
            (false, "123"),
        ];

        run_test_cases::<Document>(test_cases)?;

        Ok(())
    }

    #[test]
    fn test_regex_uuid() -> Result<(), Error> {
        #[derive(Debug, Clone, DocBuf, Serialize, Deserialize, Default)]
        #[docbuf {}]
        pub struct Document {
            #[docbuf {
                // Check regex for uuid v4
                regex = r"^[a-f0-9]{8}-[a-f0-9]{4}-[a-f0-9]{4}-[a-f0-9]{4}-[a-f0-9]{12}$";

            }]
            pub string_value: String,
        }

        impl SetTestValues for Document {
            fn set_string_value(&mut self, value: String) {
                self.string_value = value;
            }
        }

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
    fn test_min_max_length() -> Result<(), Error> {
        #[derive(Debug, Clone, DocBuf, Serialize, Deserialize, Default)]
        #[docbuf {}]
        pub struct Document {
            #[docbuf {
                min_length = 4;
                max_length = 100;
            }]
            pub string_value: String,
        }

        impl SetTestValues for Document {
            fn set_string_value(&mut self, value: String) {
                self.string_value = value;
            }
        }

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
    fn test_length() -> Result<(), Error> {
        #[derive(Debug, Clone, DocBuf, Serialize, Deserialize, Default)]
        #[docbuf {}]
        pub struct Document {
            #[docbuf {
                length = 5;
            }]
            pub string_value: String,
        }

        impl SetTestValues for Document {
            fn set_string_value(&mut self, value: String) {
                self.string_value = value;
            }
        }

        // Test cases
        let test_cases = vec![
            (true, "Hello"),
            (true, "abcde"),
            (false, "abcdef"),
            (false, "Hello, World!"),
            (false, "el"),
            (false, ""),
            (false, "123"),
        ];

        run_test_cases::<Document>(test_cases)?;

        Ok(())
    }
}
