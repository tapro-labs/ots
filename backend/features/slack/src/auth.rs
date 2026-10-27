use rocket::http::Status;
use rocket::request::{FromRequest, Outcome};
use rocket::{async_trait, Request};
use store::redis_store::RedisStore;

use crate::services::slack_authentication_service::{refresh_token, AuthenticationStatus};

#[derive(Debug)]
pub enum AccessTokenError {
    Missing,
    Invalid,
}

pub struct SlackAccessToken {
    pub access_token: String,
}

/// Returns the session id of a `Bearer <id>` header value, or `None` if the
/// header uses another scheme or has no id.
fn bearer_token(authorization: &str) -> Option<&str> {
    authorization
        .strip_prefix("Bearer ")
        .map(str::trim)
        .filter(|id| !id.is_empty())
}

#[async_trait]
impl<'r> FromRequest<'r> for SlackAccessToken {
    type Error = AccessTokenError;

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let authorization = request.headers().get_one("Authorization");

        match authorization {
            Some(authorization) => {
                let Some(id) = bearer_token(authorization) else {
                    return Outcome::Error((Status::Unauthorized, AccessTokenError::Missing));
                };

                let mut redis_client = RedisStore::connect_default();
                let redis_key = format!("session:{}", id);

                if let Ok(access_token) = redis_client.get::<String, &str>(&redis_key) {
                    return Outcome::Success(Self { access_token });
                }

                let redis_refresh_key = format!("session-refresh:{}", id);

                // try to refresh token if we still have refresh token
                if let Ok(r_token) = redis_client.get::<String, &str>(&redis_refresh_key)
                    && let AuthenticationStatus::Ok(response, _) =
                        refresh_token(&r_token, id.to_owned()).await
                {
                    return Outcome::Success(Self {
                        access_token: response.access_token,
                    });
                }

                Outcome::Error((Status::Unauthorized, AccessTokenError::Invalid))
            }
            _ => Outcome::Error((Status::Unauthorized, AccessTokenError::Missing)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rocket::get;
    use rocket::http::Header;
    use rocket::local::blocking::Client;

    #[get("/protected")]
    fn protected(_token: SlackAccessToken) -> &'static str {
        "ok"
    }

    fn get_with_authorization(authorization: Option<&str>) -> Status {
        let rocket = rocket::build().mount("/", rocket::routes![protected]);
        let client = Client::tracked(rocket).expect("valid rocket instance");
        let mut request = client.get("/protected");

        if let Some(value) = authorization {
            request = request.header(Header::new("Authorization", value.to_owned()));
        }

        request.dispatch().status()
    }

    // --- bearer_token ---

    #[test]
    fn test_bearer_token_extracts_session_id() {
        assert_eq!(bearer_token("Bearer abc-123"), Some("abc-123"));
    }

    #[test]
    fn test_bearer_token_rejects_other_schemes() {
        assert_eq!(bearer_token("Basic dXNlcjpwYXNz"), None);
        assert_eq!(bearer_token("Token Bearer abc"), None);
        assert_eq!(bearer_token("bearer abc"), None);
    }

    #[test]
    fn test_bearer_token_rejects_missing_session_id() {
        assert_eq!(bearer_token(""), None);
        assert_eq!(bearer_token("Bearer"), None);
        assert_eq!(bearer_token("Bearer "), None);
        assert_eq!(bearer_token("Bearer    "), None);
    }

    // --- SlackAccessToken request guard ---
    // Only the guard paths that do not reach Redis are covered here.

    #[test]
    fn test_guard_rejects_request_without_authorization_header() {
        assert_eq!(get_with_authorization(None), Status::Unauthorized);
    }

    #[test]
    fn test_guard_rejects_non_bearer_authorization() {
        assert_eq!(
            get_with_authorization(Some("Basic dXNlcjpwYXNz")),
            Status::Unauthorized
        );
    }

    #[test]
    fn test_guard_rejects_bearer_without_token() {
        assert_eq!(get_with_authorization(Some("Bearer")), Status::Unauthorized);
        assert_eq!(
            get_with_authorization(Some("Bearer ")),
            Status::Unauthorized
        );
    }

    #[test]
    fn test_guard_rejects_header_that_only_contains_bearer() {
        assert_eq!(
            get_with_authorization(Some("NotBearer")),
            Status::Unauthorized
        );
    }
}
