pub struct PayloadValidator<'a> {
    value: &'a str,
    errors: Vec<String>,
}

impl<'a> PayloadValidator<'a> {
    const ERR_EMPTY: &'static str = "can not be empty";
    const ERR_ALPHANUMERIC: &'static str = "allowed characters are alphanumeric and hyphens";
    const ERR_MAX_LENGTH: &'static str = "max length is";
    const ERR_URL_SCHEMA: &'static str =
        "has to start with 'http://' or 'https://' and does not contain any whitespaces";

    pub fn new(value: &'a str) -> Self {
        PayloadValidator {
            value,
            errors: vec![],
        }
    }

    pub fn not_empty(mut self) -> Self {
        if self.value.is_empty() {
            self.errors.push(Self::ERR_EMPTY.to_owned());
        }
        self
    }
    pub fn max_length(mut self, length: usize) -> Self {
        if self.value.len() > length {
            let mut err = String::from(Self::ERR_MAX_LENGTH);
            err.push(' ');
            err.push_str(&length.to_string());
            self.errors.push(err);
        }
        self
    }
    pub fn valid_characters(mut self) -> Self {
        if !self
            .value
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
        {
            self.errors.push(Self::ERR_ALPHANUMERIC.to_owned());
        }
        self
    }
    pub fn has_url_schema(mut self) -> Self {
        if !self.value.starts_with("http://") && !self.value.starts_with("https://") {
            self.errors.push(Self::ERR_URL_SCHEMA.to_owned());
            return self;
        }
        for c in self.value.chars() {
            if c.is_whitespace() {
                self.errors.push(Self::ERR_URL_SCHEMA.to_owned());
                break;
            }
        }
        self
    }
    pub fn validate(self) -> Result<(), Vec<String>> {
        if !self.errors.is_empty() {
            return Err(self.errors);
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::data::PayloadValidator;

    #[tokio::test]
    async fn not_empty_fails() {
        let result = PayloadValidator::new("").not_empty().validate();
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.len(), 1);
        assert_eq!(err[0], PayloadValidator::ERR_EMPTY);
    }
    #[tokio::test]
    async fn not_empty_succeeds() {
        let result = PayloadValidator::new("sometext").not_empty().validate();
        assert!(result.is_ok());
    }
    #[tokio::test]
    async fn max_length_fails() {
        let len = 5;
        let result = PayloadValidator::new("sometext").max_length(len).validate();
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.len(), 1);
        let expected_err_msg = format!("{} {len}", PayloadValidator::ERR_MAX_LENGTH);
        assert_eq!(err[0], expected_err_msg);
    }
    #[tokio::test]
    async fn max_length_succeeds() {
        let result = PayloadValidator::new("sometext").max_length(50).validate();
        assert!(result.is_ok());
    }
    #[tokio::test]
    async fn valid_characters_fails() {
        let result = PayloadValidator::new("sometext_$")
            .valid_characters()
            .validate();
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.len(), 1);
        assert_eq!(err[0], PayloadValidator::ERR_ALPHANUMERIC);
    }
    #[tokio::test]
    async fn valid_characters_succeeds() {
        let result = PayloadValidator::new("-abcdefghijklmnopqrstuv-wxyz0123456789-")
            .valid_characters()
            .validate();
        assert!(result.is_ok());
    }
    #[tokio::test]
    async fn has_url_schema_succeeds_with_http() {
        let result = PayloadValidator::new("http://somedomain.de")
            .has_url_schema()
            .validate();
        assert!(result.is_ok());
    }
    #[tokio::test]
    async fn has_url_schema_succeeds_with_https() {
        let result = PayloadValidator::new("https://somedomain.de")
            .has_url_schema()
            .validate();
        assert!(result.is_ok());
    }
    #[tokio::test]
    async fn has_url_schema_fails_with_whitespace() {
        let result = PayloadValidator::new("https://some   domain.de")
            .has_url_schema()
            .validate();
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.len(), 1);
        assert_eq!(err[0], PayloadValidator::ERR_URL_SCHEMA)
    }
    #[tokio::test]
    async fn has_url_schema_fails() {
        let result = PayloadValidator::new("hts://somedomain.de")
            .has_url_schema()
            .validate();
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.len(), 1);
        assert_eq!(err[0], PayloadValidator::ERR_URL_SCHEMA)
    }
    #[tokio::test]
    async fn validation_combination_fails() {
        let len = 3;
        let result = PayloadValidator::new("sometext_$")
            .not_empty()
            .max_length(len)
            .valid_characters()
            .validate();
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.len(), 2);
        let expected_errors = [
            PayloadValidator::ERR_ALPHANUMERIC.to_owned(),
            format!("{} {len}", PayloadValidator::ERR_MAX_LENGTH),
        ];
        expected_errors
            .iter()
            .for_each(|e| assert!(err.contains(e)));
    }
    #[tokio::test]
    async fn validation_combination_succeeds() {
        let len = 50;
        let result = PayloadValidator::new("sometext")
            .not_empty()
            .max_length(len)
            .valid_characters()
            .validate();
        assert!(result.is_ok());
    }
}
