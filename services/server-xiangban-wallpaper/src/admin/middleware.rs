use super::auth::SESSION_COOKIE;
use crate::{api::state::AppState, error::AppError};
use axum::{
    extract::{Request, State},
    http::{header, HeaderMap, HeaderName, Method},
    middleware::Next,
    response::{IntoResponse, Response},
};

/// Guards every admin route except login: a valid session cookie is required, and requests that
/// change data must come from the console's own origin.
pub async fn require_admin(State(state): State<AppState>, mut request: Request, next: Next) -> Response {
    if let Err(unavailable) = state.admin.store() {
        return unavailable.into_response();
    }

    let changes_data = !matches!(*request.method(), Method::GET | Method::HEAD | Method::OPTIONS);
    if changes_data && !same_origin(request.headers()) {
        return AppError::Forbidden("请求来源不被允许".to_string()).into_response();
    }

    let session = session_token(request.headers()).and_then(|token| state.admin.session(&token));
    match session {
        Some(session) => {
            request.extensions_mut().insert(session);
            next.run(request).await
        }
        None => AppError::Unauthorized("请先登录".to_string()).into_response(),
    }
}

pub fn session_token(headers: &HeaderMap) -> Option<String> {
    headers
        .get_all(header::COOKIE)
        .iter()
        .filter_map(|value| value.to_str().ok())
        .flat_map(|cookies| cookies.split(';'))
        .filter_map(|pair| pair.trim().split_once('='))
        .find(|(name, _)| *name == SESSION_COOKIE)
        .map(|(_, value)| value.to_string())
}

/// Browsers send `Origin` on cross-site and on state-changing requests. When it is present it must
/// match the host the console is served from; tools like curl send none and pass. This backs up the
/// `SameSite=Strict` session cookie.
fn same_origin(headers: &HeaderMap) -> bool {
    let Some(origin) = headers.get(header::ORIGIN) else {
        return true;
    };
    let Some(authority) = origin
        .to_str()
        .ok()
        .and_then(|o| o.split_once("://"))
        .map(|(_, authority)| authority)
    else {
        return false;
    };

    let host_matches = |name: HeaderName| {
        headers
            .get(name)
            .and_then(|v| v.to_str().ok())
            .is_some_and(|host| host.eq_ignore_ascii_case(authority))
    };
    host_matches(header::HOST) || host_matches(HeaderName::from_static("x-forwarded-host"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderValue;

    fn headers(pairs: &[(&'static str, &'static str)]) -> HeaderMap {
        let mut map = HeaderMap::new();
        for (name, value) in pairs {
            map.append(HeaderName::from_static(name), HeaderValue::from_static(value));
        }
        map
    }

    #[test]
    fn finds_the_session_cookie_among_others() {
        let h = headers(&[("cookie", "theme=dark; admin_session=abc123; other=1")]);
        assert_eq!(session_token(&h).as_deref(), Some("abc123"));
        assert_eq!(session_token(&headers(&[("cookie", "theme=dark")])), None);
        assert_eq!(session_token(&HeaderMap::new()), None);
    }

    #[test]
    fn origin_must_match_host_when_present() {
        assert!(same_origin(&HeaderMap::new()), "no Origin header: not a browser cross-site request");
        assert!(same_origin(&headers(&[("origin", "http://localhost:8081"), ("host", "localhost:8081")])));
        assert!(same_origin(&headers(&[
            ("origin", "https://wall.example.com"),
            ("host", "127.0.0.1:8081"),
            ("x-forwarded-host", "wall.example.com"),
        ])));
        assert!(!same_origin(&headers(&[("origin", "https://evil.example"), ("host", "localhost:8081")])));
        assert!(!same_origin(&headers(&[("origin", "null"), ("host", "localhost:8081")])));
    }
}
