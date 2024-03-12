use super::*;

// Implement DocBufValidate for VTableField

impl<'a> DocBufValidateField<String> for VTableFieldRules {
    #[inline]
    fn validate(&self, data: &String) -> Result<(), Error> {
        // Skip validation if no field rules are present
        if self.is_none() {
            return Ok(());
        }

        // Check for string length rules
        self.check_length(data.len())?;

        // Check for regex rules
        #[cfg(feature = "regex")]
        self.check_regex(data)?;

        Ok(())
    }
}

impl<'a> DocBufValidateField<&str> for VTableFieldRules {
    #[inline]
    fn validate(&self, data: &&str) -> Result<(), Error> {
        // Skip validation if no field rules are present
        if self.is_none() {
            return Ok(());
        }

        // Check for string length rules
        self.check_length(data.len())?;

        // Check for regex rules
        #[cfg(feature = "regex")]
        self.check_regex(data)?;

        Ok(())
    }
}

impl<'a> DocBufValidateField<Vec<u8>> for VTableFieldRules {
    #[inline]
    fn validate(&self, data: &Vec<u8>) -> Result<(), Error> {
        // Skip validation if no field rules are present
        if self.is_none() {
            return Ok(());
        }

        // Check for byte length rules
        self.check_length(data.len())?;

        Ok(())
    }
}

impl<'a> DocBufValidateField<&[u8]> for VTableFieldRules {
    #[inline]
    fn validate(&self, data: &&[u8]) -> Result<(), Error> {
        // Skip validation if no field rules are present
        if self.is_none() {
            return Ok(());
        }

        // Check for byte length rules
        self.check_length(data.len())?;

        Ok(())
    }
}

impl<'a> DocBufValidateField<NumericValue> for VTableFieldRules {
    fn validate(&self, value: &NumericValue) -> Result<(), crate::vtable::Error> {
        // Skip validation if no field rules are present
        if self.is_none() {
            return Ok(());
        }

        // Check for numeric value rules
        self.check_numeric(value)?;

        Ok(())
    }
}

impl<'a> DocBufValidateField<bool> for VTableFieldRules {
    fn validate(&self, value: &bool) -> Result<(), crate::vtable::Error> {
        // Skip validation if no field rules are present
        if self.is_none() {
            return Ok(());
        }

        // Check for boolean value rules
        self.check_bool(value)?;

        Ok(())
    }
}
