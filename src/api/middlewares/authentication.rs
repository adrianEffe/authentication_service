use crate::{
    api::model::api_error::ApiError,
    application::AppState,
    domain::{
        auth_service::AuthService,
        model::auth::{AuthRequest, AuthorizationError},
    },
};

use axum::{
    body::Body,
    extract::State,
    http::{header, Request},
    middleware::Next,
    response::IntoResponse,
};
use axum_extra::extract::CookieJar;
use std::sync::Arc;

/// Middleware function for authentication in the Axum web framework.
///
/// This asynchronous function is used as middleware to handle authentication for incoming requests. It extracts the
/// access token from cookies or request headers, validates it using the authentication service, and then proceeds
/// with the request if authentication is successful. If the token is missing or invalid, it returns an error.
///
/// The process includes:
/// 1. **Extracting Access Token:** Retrieves the access token from cookies or headers using the `extract_access_token`
///    helper function.
/// 2. **Authentication:** Uses the extracted token to create an `AuthRequest` and calls the `auth` method on the
///    provided `AuthService` to authenticate the request.
/// 3. **Request Extension:** Inserts the result of the authentication middleware into the request's extensions to
///    make it available for downstream handlers.
/// 4. **Forwarding Request:** Passes the request to the next middleware or handler in the pipeline if authentication
///    succeeds.
///
/// # Arguments
///
/// * `cookie_jar` - A `CookieJar` object used to retrieve cookies from the request.
/// * `State(state)` - The application state, which includes an `AuthService` instance, provided as `State` by Axum.
/// * `req` - The incoming HTTP request being processed.
/// * `next` - The next middleware or handler to execute if authentication is successful.
///
/// # Returns
///
/// A `Result` containing the response from the next middleware or handler if authentication succeeds, or an `ApiError`
/// if authentication fails.
///
/// # Errors
///
/// This function returns an `ApiError` if the access token cannot be extracted or if the authentication fails.
pub async fn auth<AS: AuthService>(
    cookie_jar: CookieJar,
    State(state): State<Arc<AppState<AS>>>,
    mut req: Request<Body>,
    next: Next,
) -> Result<impl IntoResponse, ApiError> {
    let access_token = extract_access_token(cookie_jar, &req).map_err(ApiError::from)?;
    let domain_request = AuthRequest::new(access_token);

    let auth_middleware = state.auth_service.auth(&domain_request).await?;

    req.extensions_mut().insert(auth_middleware);
    Ok(next.run(req).await)
}

fn extract_access_token(
    cookie_jar: CookieJar,
    req: &Request<Body>,
) -> Result<String, AuthorizationError> {
    let access_token = cookie_jar
        .get("access_token")
        .map(|cookie| cookie.value().to_string())
        .or_else(|| {
            req.headers()
                .get(header::AUTHORIZATION)
                .and_then(|auth_header| auth_header.to_str().ok())
                .and_then(|auth_value| {
                    auth_value
                        .strip_prefix("Bearer ")
                        .map(|token| token.to_owned())
                })
        });

    access_token.ok_or_else(|| AuthorizationError::InvalidCredentials {
        reason: "You are not logged in".to_string(),
    })
}
