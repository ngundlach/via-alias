use std::time::{SystemTime, UNIX_EPOCH};

use argon2::{Argon2, PasswordHash, PasswordVerifier};

use crate::{
    model::{User, UserCredentialsDTO, UserRegistrationToken},
    service::DbServiceError,
};

pub struct PayloadValidator<'a> {
    value: &'a str,
    errors: Vec<String>,
}

impl<'a> PayloadValidator<'a> {
    const ERR_EMPTY: &'static str = "can not be empty";
    const ERR_ALPHANUMERIC: &'static str =
        "allowed characters are alphanumeric, hyphens and underscores";
    const ERR_MAX_LENGTH: &'static str = "max length is ";
    const ERR_MIN_LENGTH: &'static str = "min length is ";
    const ERR_URL_SCHEMA: &'static str =
        "has to start with 'http://' or 'https://' and can not contain any whitespaces";
    const ERR_REQUIRED_CHAR: &'static str = "must contain the following character: ";
    const ERR_AT_LEAST_ONE_NUMERIC: &'static str = "must contain at least one numeric characters";
    const ERR_AT_LEAST_ONE_ALPHABETIC: &'static str =
        "must contain at least one alphabetic characters";
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

    pub fn min_length(mut self, length: usize) -> Self {
        if self.value.len() < length {
            let mut err = String::from(Self::ERR_MIN_LENGTH);
            err.push_str(&length.to_string());
            self.errors.push(err);
        }
        self
    }
    pub fn max_length(mut self, length: usize) -> Self {
        if self.value.len() > length {
            let mut err = String::from(Self::ERR_MAX_LENGTH);
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
    pub fn one_numeric(mut self) -> Self {
        if self.value.chars().any(|c| c.is_ascii_digit()) {
            return self;
        }

        self.errors.push(Self::ERR_AT_LEAST_ONE_NUMERIC.to_owned());
        self
    }
    pub fn one_alphabetic(mut self) -> Self {
        if self.value.chars().any(|c| c.is_ascii_alphabetic()) {
            return self;
        }

        self.errors
            .push(Self::ERR_AT_LEAST_ONE_ALPHABETIC.to_owned());
        self
    }
    #[allow(unused)]
    pub fn required_character(mut self, required: char) -> Self {
        if self.value.chars().any(|c| c == required) {
            return self;
        }

        let mut err = String::from(Self::ERR_REQUIRED_CHAR);
        err.push(required);

        self.errors.push(err);
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

pub(crate) fn check_user_credentials(
    user: &UserCredentialsDTO,
    user_data: &User,
) -> Result<(), DbServiceError> {
    let argon2 = Argon2::default();
    let parsed_hash = PasswordHash::new(&user_data.pwhash)
        .map_err(|e| DbServiceError::DatabaseError(e.to_string()))?;
    argon2
        .verify_password(user.pw.as_bytes(), &parsed_hash)
        .map_err(|e| DbServiceError::AuthError(e.to_string()))?;
    Ok(())
}

pub(crate) fn validate_registration_token(
    token: &UserRegistrationToken,
) -> Result<(), DbServiceError> {
    let current_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock is before Unix epoch")
        .as_secs();
    if token.exp_at < current_time {
        return Err(DbServiceError::TokenInvalid);
    }
    Ok(())
}

#[cfg(test)]
mod test {

    use crate::service::PayloadValidator;

    #[test]
    fn not_empty_fails() {
        let result = PayloadValidator::new("").not_empty().validate();
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.len(), 1);
        assert_eq!(err[0], PayloadValidator::ERR_EMPTY);
    }
    #[test]
    fn not_empty_succeeds() {
        let result = PayloadValidator::new("sometext").not_empty().validate();
        assert!(result.is_ok());
    }
    #[test]
    fn min_length_fails() {
        let len = 5;
        let result = PayloadValidator::new("som").min_length(len).validate();
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.len(), 1);
        let expected_err_msg = format!("{}{len}", PayloadValidator::ERR_MIN_LENGTH);
        assert_eq!(err[0], expected_err_msg);
    }
    #[test]
    fn min_length_succeeds() {
        let result = PayloadValidator::new("sometext").min_length(2).validate();
        assert!(result.is_ok());
    }
    #[test]
    fn max_length_fails() {
        let len = 5;
        let result = PayloadValidator::new("sometext").max_length(len).validate();
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.len(), 1);
        let expected_err_msg = format!("{}{len}", PayloadValidator::ERR_MAX_LENGTH);
        assert_eq!(err[0], expected_err_msg);
    }
    #[test]
    fn max_length_succeeds() {
        let result = PayloadValidator::new("sometext").max_length(50).validate();
        assert!(result.is_ok());
    }
    #[test]
    fn valid_characters_fails() {
        let result = PayloadValidator::new("sometext_$")
            .valid_characters()
            .validate();
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.len(), 1);
        assert_eq!(err[0], PayloadValidator::ERR_ALPHANUMERIC);
    }
    #[test]
    fn valid_characters_succeeds() {
        let result = PayloadValidator::new("-abcdefghijklmnopqrstuv-wxyz0123456789-")
            .valid_characters()
            .validate();
        assert!(result.is_ok());
    }
    #[test]
    fn one_alphabetic_succeeds() {
        let result = PayloadValidator::new("ab1c").one_alphabetic().validate();
        assert!(result.is_ok());
    }
    #[test]
    fn one_alphabetic_fails() {
        let result = PayloadValidator::new("123").one_alphabetic().validate();
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.len(), 1);
        assert!(err.contains(&format!(
            "{}",
            PayloadValidator::ERR_AT_LEAST_ONE_ALPHABETIC
        )));
    }
    #[test]
    fn one_numeric_succeeds() {
        let result = PayloadValidator::new("ab1c").one_numeric().validate();
        assert!(result.is_ok());
    }
    #[test]
    fn one_numeric_fails() {
        let result = PayloadValidator::new("abc").one_numeric().validate();
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.len(), 1);
        assert!(err.contains(&format!("{}", PayloadValidator::ERR_AT_LEAST_ONE_NUMERIC)));
    }
    #[test]
    fn required_character_fails() {
        let result = PayloadValidator::new("abc")
            .required_character('c')
            .required_character('f')
            .required_character('a')
            .required_character('z')
            .validate();
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.len(), 2);
        dbg!(&err);
        assert!(err.contains(&format!("{}f", PayloadValidator::ERR_REQUIRED_CHAR)));
        assert!(err.contains(&format!("{}z", PayloadValidator::ERR_REQUIRED_CHAR)));
    }
    #[test]
    fn required_characters_succeeds() {
        let result = PayloadValidator::new("abc")
            .required_character('a')
            .validate();
        assert!(result.is_ok());
    }
    #[test]
    fn has_url_schema_succeeds_with_http() {
        let result = PayloadValidator::new("http://somedomain.de")
            .has_url_schema()
            .validate();
        assert!(result.is_ok());
    }
    #[test]
    fn has_url_schema_succeeds_with_https() {
        let result = PayloadValidator::new("https://somedomain.de")
            .has_url_schema()
            .validate();
        assert!(result.is_ok());
    }
    #[test]
    fn has_url_schema_fails_with_whitespace() {
        let result = PayloadValidator::new("https://some   domain.de")
            .has_url_schema()
            .validate();
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.len(), 1);
        assert_eq!(err[0], PayloadValidator::ERR_URL_SCHEMA)
    }
    #[test]
    fn has_url_schema_fails() {
        let result = PayloadValidator::new("hts://somedomain.de")
            .has_url_schema()
            .validate();
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.len(), 1);
        assert_eq!(err[0], PayloadValidator::ERR_URL_SCHEMA)
    }
    #[test]
    fn validation_combination_fails() {
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
            format!("{}{len}", PayloadValidator::ERR_MAX_LENGTH),
        ];
        expected_errors
            .iter()
            .for_each(|e| assert!(err.contains(e)));
    }
    #[test]
    fn validation_combination_succeeds() {
        let len = 50;
        let result = PayloadValidator::new("sometext")
            .not_empty()
            .max_length(len)
            .valid_characters()
            .validate();
        assert!(result.is_ok());
    }
}
