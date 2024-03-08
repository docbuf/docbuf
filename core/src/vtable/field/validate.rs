use super::*;

// Implement DocBufValidate for VTableField

impl<'a> DocBufValidateField<String> for VTableField<'a> {
    #[inline]
    fn validate(&self, data: &String) -> Result<(), Error> {
        // Skip validation if no field rules are present
        if self.field_rules.is_none() {
            return Ok(());
        }

        match self.field_type {
            VTableFieldType::String | VTableFieldType::Str => {
                // Check for string length rules
                self.field_rules.check_data_length_field_rules(data.len())?;

                // Check for regex rules
                #[cfg(feature = "regex")]
                self.field_rules.check_regex_field_rules(data)?;
            }
            _ => {
                return Err(Error::InvalidValidationType(format!(
                    "Invalid validation type for string field: {:#?}",
                    self.field_type
                )))
            }
        }

        Ok(())
    }
}

impl<'a> DocBufValidateField<&str> for VTableField<'a> {
    #[inline]
    fn validate(&self, data: &&str) -> Result<(), Error> {
        // Skip validation if no field rules are present
        if self.field_rules.is_none() {
            return Ok(());
        }

        match self.field_type {
            VTableFieldType::String | VTableFieldType::Str => {
                // Check for string length rules
                self.field_rules.check_data_length_field_rules(data.len())?;

                // Check for regex rules
                #[cfg(feature = "regex")]
                self.field_rules.check_regex_field_rules(data)?;
            }
            _ => {
                return Err(Error::InvalidValidationType(format!(
                    "Invalid validation type for str field: {:#?}",
                    self.field_type
                )))
            }
        }

        Ok(())
    }
}

impl<'a> DocBufValidateField<&[u8]> for VTableField<'a> {
    #[inline]
    fn validate(&self, data: &&[u8]) -> Result<(), Error> {
        // Skip validation if no field rules are present
        if self.field_rules.is_none() {
            return Ok(());
        }

        match self.field_type {
            VTableFieldType::Bytes => {
                // Check for byte length rules
                self.field_rules.check_data_length_field_rules(data.len())?;
            }
            _ => {
                return Err(Error::InvalidValidationType(format!(
                    "Invalid validation type for bytes field: {:#?}",
                    self.field_type
                )))
            }
        }

        Ok(())
    }
}

impl<'a> DocBufValidateField<NumericValue> for VTableField<'a> {
    fn validate(&self, value: &NumericValue) -> Result<(), crate::vtable::Error> {
        // Skip validation if no field rules are present
        if self.field_rules.is_none() {
            return Ok(());
        }

        // Check for numeric value rules
        self.field_rules.check_numeric_value(value)?;

        Ok(())
    }
}
