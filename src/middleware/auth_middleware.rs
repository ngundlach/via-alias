use axum::{
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::IntoResponse,
};
use jsonwebtoken::{DecodingKey, TokenData, Validation, decode};

use crate::{AppContext, model::UserClaimsDTO};

pub(crate) async fn auth_middleware(
    State(app_context): State<AppContext>,
    headers: HeaderMap,
    mut request: Request,
    next: Next,
) -> impl IntoResponse {
    let token_data = match extract_bearer_token::<UserClaimsDTO>(
        &headers,
        app_context.app_config.jwt_secret.as_bytes(),
    ) {
        Ok(x) => x,
        Err(_) => {
            return StatusCode::UNAUTHORIZED.into_response();
        }
    };
    request.extensions_mut().insert(token_data.claims);

    next.run(request).await
}

fn extract_bearer_token<C: serde::de::DeserializeOwned>(
    headers: &HeaderMap,
    secret: &[u8],
) -> Result<TokenData<C>, jsonwebtoken::errors::Error> {
    let token = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or(jsonwebtoken::errors::Error::from(
            jsonwebtoken::errors::ErrorKind::InvalidToken,
        ))?;

    decode::<C>(
        token,
        &DecodingKey::from_secret(secret),
        &Validation::new(jsonwebtoken::Algorithm::HS512),
    )
}
