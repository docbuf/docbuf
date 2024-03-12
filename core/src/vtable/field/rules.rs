use super::*;

use crate::validate::regex::Regex;

/// Optional rules for a field
#[derive(Debug, Clone)]
pub struct VTableFieldRules {
    pub ignore: bool,
    pub max_value: Option<NumericValue>,
    pub min_value: Option<NumericValue>,
    pub max_length: Option<usize>,
    pub min_length: Option<usize>,
    // An absolute length
    pub length: Option<usize>,
    #[cfg(feature = "regex")]
    pub regex: Option<Regex>,
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
        self.regex = Regex::from_str(value).ok();
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
    pub fn regex(&self) -> Option<&::regex::Regex> {
        self.regex.as_ref()
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
        if let Some(regex_rules) = self.regex() {
            if !regex_rules.is_match(data) {
                let msg = format!("data does not match regex: {regex_rules}");
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
}
