use super::*;

use crate::validate::regex::Regex;

use serde_derive::{Deserialize, Serialize};

/// Optional rules for a field
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct VTableFieldRules {
    pub ignore: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_value: Option<NumericValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_value: Option<NumericValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_length: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_length: Option<usize>,
    /// An absolute length
    #[serde(skip_serializing_if = "Option::is_none")]
    pub length: Option<usize>,
    // #[cfg(feature = "regex")]
    /// A regex pattern to match
    #[serde(skip_serializing_if = "Option::is_none")]
    pub regex: Option<String>,
    pub sign: bool,
}

impl VTableFieldRules {
    #[inline]
    pub fn new() -> Self {
        Self {
            ignore: false,
            max_value: None,
            min_value: None,
            max_length: None,
            min_length: None,
            length: None,
            regex: None,
            sign: false,
        }
    }

    // Check whether the field rules are empty
    // Usefule for optimization in validation checking
    #[inline]
    pub fn is_none(&self) -> bool {
        !self.ignore
            && self.max_value.is_none()
            && self.min_value.is_none()
            && self.max_length.is_none()
            && self.min_length.is_none()
            && self.length.is_none()
            && self.regex.is_none()
            && !self.sign
    }

    #[inline]
    pub fn set_max_length(mut self, max_length: usize) -> Self {
        self.max_length = Some(max_length);
        self
    }

    #[inline]
    pub fn set_min_length(mut self, min_length: usize) -> Self {
        self.min_length = Some(min_length);
        self
    }

    #[inline]
    pub fn set_length(mut self, length: usize) -> Self {
        self.length = Some(length);
        self
    }

    #[inline]
    pub fn set_ignore(mut self, ignore: bool) -> Self {
        self.ignore = ignore;
        self
    }

    #[cfg(feature = "regex")]
    #[inline]
    pub fn set_regex(mut self, value: &str) -> Self {
        self.regex = Some(value.to_string());
        self
    }

    // Return the max length of the string
    #[inline]
    pub fn max_length(&self) -> Option<usize> {
        self.max_length
    }

    // Return the min length of the string
    #[inline]
    pub fn min_length(&self) -> Option<usize> {
        self.min_length
    }

    // Return the length of the string
    #[inline]
    pub fn length(&self) -> Option<usize> {
        self.length
    }

    #[cfg(feature = "regex")]
    #[inline]
    pub fn regex(&self) -> Option<::regex::Regex> {
        match &self.regex {
            None => None,
            Some(pattern) => Regex::from_str(&pattern).ok(),
        }
    }

    // Return if the field should be ignored
    #[inline]
    pub fn ignore(&self) -> bool {
        self.ignore
    }

    // Return if the field should be signed
    #[inline]
    pub fn sign(&self) -> bool {
        self.sign
    }

    #[inline]
    pub fn check_bool(&self, _data: &bool) -> Result<(), Error> {
        unimplemented!("Boolean value checking is not yet implemented")
    }

    #[cfg(feature = "regex")]
    #[inline]
    pub fn check_regex(&self, data: &str) -> Result<(), Error> {
        if let Some(regex_pattern) = self.regex() {
            if !regex_pattern.is_match(data) {
                let msg = format!("data does not match regex: {regex_pattern}");
                return Err(Error::FieldRulesRegex(msg));
            }
        }

        Ok(())
    }

    #[inline]
    pub fn check_length(&self, length: usize) -> Result<(), Error> {
        if length > MAX_FIELD_SIZE {
            let msg = format!("data size exceeds 1 gigabyte");
            return Err(Error::FieldRulesLength(msg));
        }

        if let Some(length_rule) = self.length {
            if length != length_rule {
                let msg = format!("data size does not match required length: {length}");
                return Err(Error::FieldRulesLength(msg));
            }
        } else {
            // If exact length is not set, check the min and max length values.
            if let Some(max_length) = self.max_length {
                if length > max_length {
                    let msg = format!("data size exceeds field max length: {max_length}");
                    return Err(Error::FieldRulesLength(msg));
                }
            }

            if let Some(min_length) = self.min_length {
                if length < min_length {
                    let msg = format!("data size is less than min length: {min_length}");
                    return Err(Error::FieldRulesLength(msg));
                }
            };
        }

        Ok(())
    }

    #[inline]
    pub fn check_numeric(&self, value: &NumericValue) -> Result<(), Error> {
        if let Some(max_value) = &self.max_value {
            if value > max_value {
                let msg = format!("data value exceeds field max value: {:?}", max_value);
                return Err(Error::FieldRulesValue(msg));
            }
        }

        if let Some(min_value) = &self.min_value {
            if value < min_value {
                let msg = format!("data value is less than min value: {:?}", min_value);
                return Err(Error::FieldRulesValue(msg));
            }
        };

        Ok(())
    }

    #[inline]
    pub fn write_to_buffer(&self, buffer: &mut Vec<u8>) -> Result<(), Error> {
        buffer.push(self.ignore as u8);
        buffer.push(self.sign as u8);

        if let Some(max_value) = &self.max_value {
            buffer.push(1);
            max_value.write_to_buffer(buffer)?;
        } else {
            buffer.push(0);
        }

        if let Some(min_value) = &self.min_value {
            buffer.push(1);
            min_value.write_to_buffer(buffer)?;
        } else {
            buffer.push(0);
        }

        if let Some(max_length) = self.max_length {
            buffer.push(1);
            buffer.extend_from_slice(&max_length.to_le_bytes());
        } else {
            buffer.push(0);
        }

        if let Some(min_length) = self.min_length {
            buffer.push(1);
            buffer.extend_from_slice(&min_length.to_le_bytes());
        } else {
            buffer.push(0);
        }

        if let Some(length) = self.length {
            buffer.push(1);
            buffer.extend_from_slice(&length.to_le_bytes());
        } else {
            buffer.push(0);
        }

        if let Some(regex) = &self.regex {
            buffer.push(1);
            let regex_bytes = regex.as_bytes();
            // Max Regex length is 255
            let regex_len = regex_bytes.len() as u16;
            buffer.extend_from_slice(&regex_len.to_le_bytes());
            buffer.extend_from_slice(regex_bytes);
        } else {
            buffer.push(0);
        }

        Ok(())
    }

    #[inline]
    pub fn read_from_buffer(buffer: &mut Vec<u8>) -> Result<Self, Error> {
        let ignore = buffer.remove(0) == 1;
        let sign = buffer.remove(0) == 1;

        let read_value = |buffer: &mut Vec<u8>| -> usize {
            let value = usize::from_le_bytes([
                buffer[0], buffer[1], buffer[2], buffer[3], buffer[4], buffer[5], buffer[6],
                buffer[7],
            ]);

            buffer.drain(0..8);

            value
        };

        let max_value = if buffer.remove(0) == 1 {
            Some(NumericValue::read_from_buffer(buffer)?)
        } else {
            None
        };

        let min_value = if buffer.remove(0) == 1 {
            Some(NumericValue::read_from_buffer(buffer)?)
        } else {
            None
        };

        let max_length = if buffer.remove(0) == 1 {
            Some(read_value(buffer))
        } else {
            None
        };

        let min_length = if buffer.remove(0) == 1 {
            Some(read_value(buffer))
        } else {
            None
        };

        let length = if buffer.remove(0) == 1 {
            Some(read_value(buffer))
        } else {
            None
        };

        let regex = if buffer.remove(0) == 1 {
            let len = u16::from_be_bytes([buffer[0], buffer[1]]) as usize;
            buffer.drain(0..2);
            let regex = String::from_utf8(buffer.drain(0..len).collect())?;
            Some(regex)
        } else {
            None
        };

        Ok(Self {
            ignore,
            sign,
            max_value,
            min_value,
            max_length,
            min_length,
            length,
            regex,
        })
    }
}

impl PartialEq for VTableFieldRules {
    fn eq(&self, other: &Self) -> bool {
        self.ignore == other.ignore
            && self.sign == other.sign
            && self.max_value == other.max_value
            && self.min_value == other.min_value
            && self.max_length == other.max_length
            && self.min_length == other.min_length
            && self.length == other.length
            && self.regex == other.regex
    }
}

impl Eq for VTableFieldRules {}
