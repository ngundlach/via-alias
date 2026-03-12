use axum::{
    Extension,
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::IntoResponse,
};
use jsonwebtoken::{DecodingKey, TokenData, Validation, decode};

use crate::{AppContext, JwtConfig, model::UserClaimsDTO, service::DbServiceError};

pub(crate) async fn auth_middleware(
    State(app_context): State<AppContext>,
    headers: HeaderMap,
    mut request: Request,
    next: Next,
) -> impl IntoResponse {
    let Ok(token_data) =
        validate_bearer_token::<UserClaimsDTO>(&headers, &app_context.app_config.jwt_config)
    else {
        return StatusCode::UNAUTHORIZED.into_response();
    };

    request.extensions_mut().insert(token_data.claims);

    next.run(request).await
}
pub(crate) async fn is_admin_middleware(
    Extension(user_claims): Extension<UserClaimsDTO>,
    request: Request,
    next: Next,
) -> impl IntoResponse {
    if !user_claims.is_admin {
        return StatusCode::FORBIDDEN.into_response();
    }
    next.run(request).await
}

fn extract_bearer_token(headers: &HeaderMap) -> Result<String, DbServiceError> {
    let token = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or(DbServiceError::AuthError(
            "Could not extract bearer token".to_owned(),
        ))?;
    if token.is_empty() {
        return Err(DbServiceError::AuthError("invalid Token".to_owned()));
    }
    Ok(token.to_owned())
}

fn validate_bearer_token<C: serde::de::DeserializeOwned>(
    headers: &HeaderMap,
    jwt_config: &JwtConfig,
) -> Result<TokenData<C>, jsonwebtoken::errors::Error> {
    let token =
        extract_bearer_token(headers).map_err(|_| jsonwebtoken::errors::ErrorKind::InvalidToken)?;

    decode(
        token,
        &DecodingKey::from_secret(jwt_config.secret.as_bytes()),
        &Validation::new(jwt_config.alg),
    )
}

#[cfg(test)]
mod tests {

    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    use axum::http::{HeaderMap, HeaderValue};
    use jsonwebtoken::{EncodingKey, Header, jws::encode};

    use crate::{
        JwtConfig,
        middleware::auth_middleware::{extract_bearer_token, validate_bearer_token},
        model::UserClaimsDTO,
    };

    enum JwtState {
        Valid,
        Expired,
    }

    fn test_jwt_config() -> JwtConfig {
        JwtConfig {
            secret: "super_secure_test_secret".to_owned(),
            alg: jsonwebtoken::Algorithm::HS512,
            ttl: 900,
        }
    }

    fn create_test_user_claims(exp_timestamp: u64) -> UserClaimsDTO {
        UserClaimsDTO {
            user_id: "1234".to_owned(),
            is_admin: false,
            exp: exp_timestamp,
            jti: "5678".to_owned(),
        }
    }

    fn create_jwt(jwt_state: JwtState) -> (String, UserClaimsDTO) {
        let expiration_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is before Unix epoch");
        let expiration_time = match jwt_state {
            JwtState::Valid => expiration_time.checked_add(Duration::from_mins(15)),
            JwtState::Expired => expiration_time.checked_sub(Duration::from_mins(15)),
        };
        let expiration_time = expiration_time.expect("timestamp overflow").as_secs();

        let user_claims = create_test_user_claims(expiration_time);
        let jwt_config = test_jwt_config();
        let jwt_header = Header::new(jwt_config.alg);
        let jwt_token = encode(
            &jwt_header,
            Some(&user_claims),
            &EncodingKey::from_secret(jwt_config.secret.as_bytes()),
        )
        .unwrap();

        (
            format!(
                "{}.{}.{}",
                jwt_token.protected, jwt_token.payload, jwt_token.signature,
            ),
            user_claims,
        )
    }

    fn create_auth_header_with_bearer_token(
        jwt_state: JwtState,
    ) -> (HeaderMap, String, UserClaimsDTO) {
        let mut header = HeaderMap::new();
        let (jwt, user_claims) = create_jwt(jwt_state);
        let bearer = format!("Bearer {}", jwt);
        let header_val = HeaderValue::from_str(bearer.as_str()).unwrap();
        header.append("authorization", header_val);
        (header, jwt, user_claims)
    }

    #[test]
    fn test_extract_bearer_token_success() {
        let (header, expected, _) = create_auth_header_with_bearer_token(JwtState::Valid);

        let token_result = extract_bearer_token(&header);
        assert!(token_result.is_ok());
        assert_eq!(token_result.unwrap(), expected);
    }

    #[test]
    fn test_extract_bearer_token_with_missing_auth_header_fails() {
        let header = HeaderMap::new();
        let token_result = extract_bearer_token(&header);
        assert!(token_result.is_err())
    }

    #[test]
    fn test_extract_bearer_token_with_missing_empty_bearer_token_fails() {
        let mut header = HeaderMap::new();
        header.append("authorization", HeaderValue::from_static("Bearer "));
        let token_result = extract_bearer_token(&header);
        assert!(token_result.is_err())
    }

    #[test]
    fn test_validate_token_with_valid_jwt_success() {
        let (header, _, expected_claims) = create_auth_header_with_bearer_token(JwtState::Valid);
        let claims = validate_bearer_token::<UserClaimsDTO>(&header, &test_jwt_config());
        dbg!(claims.as_ref().err());
        assert!(claims.is_ok());
        assert_eq!(claims.unwrap().claims, expected_claims)
    }

    #[test]
    fn test_validate_token_with_expired_jwt_fails() {
        let (header, _, _) = create_auth_header_with_bearer_token(JwtState::Expired);
        let claims = validate_bearer_token::<UserClaimsDTO>(&header, &test_jwt_config());
        assert!(claims.is_err());
    }
}
