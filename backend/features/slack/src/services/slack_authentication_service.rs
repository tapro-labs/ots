use rocket::http::Status;
use store::redis_store::{RedisResult, RedisStore};
use utils::logger;
use utils::time::Time;
use utils::uuid::Uuid;

use crate::client::{Client, SlackAccessTokenResponse, SlackRequestError};

type SessionId = String;

#[derive(Debug)]
pub enum AuthenticationStatus {
    BadRequest,
    #[allow(dead_code)]
    Unauthorized,
    Ok(SlackAccessTokenResponse, SessionId),
}

impl From<SlackRequestError> for AuthenticationStatus {
    fn from(request_error: SlackRequestError) -> Self {
        match request_error {
            SlackRequestError::Unauthorized => Self::Unauthorized,
            _ => Self::BadRequest,
        }
    }
}

#[allow(clippy::from_over_into)]
impl Into<Status> for AuthenticationStatus {
    fn into(self) -> Status {
        match self {
            Self::Ok(_, _) => Status::Ok,
            Self::BadRequest => Status::BadRequest,
            Self::Unauthorized => Status::Unauthorized,
        }
    }
}

pub async fn authenticate(code: &str) -> AuthenticationStatus {
    authenticate_with(&Client::new_defaults(), code).await
}

async fn authenticate_with(client: &Client, code: &str) -> AuthenticationStatus {
    logger::debug(format!(
        "[SLACK_AUTHENTICATION_SERVICE] - authenticating via code: {}",
        code
    ));

    match client.fetch_auth_token(code).await {
        Ok(response) => {
            let session_id = Uuid::new_v4().to_string();

            if update_sessions(&response, &session_id).is_ok() {
                AuthenticationStatus::Ok(response, session_id)
            } else {
                AuthenticationStatus::BadRequest
            }
        }
        Err(request_error) => AuthenticationStatus::from(request_error),
    }
}

pub async fn refresh_token(refresh_token: &str, session_id: SessionId) -> AuthenticationStatus {
    refresh_token_with(&Client::new_defaults(), refresh_token, session_id).await
}

async fn refresh_token_with(
    client: &Client,
    refresh_token: &str,
    session_id: SessionId,
) -> AuthenticationStatus {
    logger::debug(format!(
        "[SLACK_AUTHENTICATION_SERVICE] - refreshing token with: {} and session id: {}",
        refresh_token, session_id
    ));

    match client.refresh_token(refresh_token).await {
        Ok(response) => {
            if update_sessions(&response, &session_id).is_ok() {
                AuthenticationStatus::Ok(response, session_id)
            } else {
                AuthenticationStatus::BadRequest
            }
        }
        Err(request_error) => AuthenticationStatus::from(request_error),
    }
}

fn update_sessions(response: &SlackAccessTokenResponse, session_id: &SessionId) -> RedisResult<()> {
    let session_key = format!("session:{}", session_id);
    let session_refresh_key = format!("session-refresh:{}", session_id);
    let mut redis_store = RedisStore::connect_default();

    redis_store.store_for_a_period(
        &session_key,
        &response.access_token,
        Time::from_seconds(response.expires_in as usize),
    )?;

    match redis_store.store_for_a_period(
        &session_refresh_key,
        &response.refresh_token,
        Time::from_days(30),
    ) {
        Ok(_) => Ok(()),
        Err(e) => {
            // reset session as the refresh one didn't succeed
            redis_store.forget(&session_key).unwrap();

            Err(e)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{client_for, unreachable_url, StubServer};

    // The success paths store the session in Redis, so only the paths that
    // fail before Redis is reached are covered here.

    #[test]
    fn test_from_slack_request_error() {
        assert!(matches!(
            AuthenticationStatus::from(SlackRequestError::Unauthorized),
            AuthenticationStatus::Unauthorized
        ));
        assert!(matches!(
            AuthenticationStatus::from(SlackRequestError::BadRequest),
            AuthenticationStatus::BadRequest
        ));
    }

    #[test]
    fn test_authentication_status_into_status() {
        let response = token_response();
        let ok: Status = AuthenticationStatus::Ok(response, "session".to_owned()).into();
        let bad_request: Status = AuthenticationStatus::BadRequest.into();
        let unauthorized: Status = AuthenticationStatus::Unauthorized.into();

        assert_eq!(ok, Status::Ok);
        assert_eq!(bad_request, Status::BadRequest);
        assert_eq!(unauthorized, Status::Unauthorized);
    }

    fn token_response() -> crate::client::SlackAccessTokenResponse {
        rocket::serde::json::serde_json::from_str(
            r#"{"access_token":"a","refresh_token":"r","expires_in":1}"#,
        )
        .unwrap()
    }

    #[rocket::async_test]
    async fn test_authenticate_with_invalid_code_is_bad_request() {
        let server = StubServer::json(r#"{"ok":false,"error":"invalid_code"}"#);

        let status = authenticate_with(&server.client(), "bad-code").await;

        assert!(matches!(status, AuthenticationStatus::BadRequest));
        assert_eq!(server.single_request().form()["code"], "bad-code");
    }

    #[rocket::async_test]
    async fn test_authenticate_with_denied_access_is_unauthorized() {
        let server = StubServer::json(r#"{"ok":false,"error":"access_denied"}"#);

        let status = authenticate_with(&server.client(), "code").await;

        assert!(matches!(status, AuthenticationStatus::Unauthorized));
    }

    #[rocket::async_test]
    async fn test_authenticate_when_slack_is_unreachable_is_bad_request() {
        let status = authenticate_with(&client_for(&unreachable_url()), "code").await;

        assert!(matches!(status, AuthenticationStatus::BadRequest));
    }

    #[rocket::async_test]
    async fn test_refresh_token_with_revoked_token_is_unauthorized() {
        let server = StubServer::json(r#"{"ok":false,"error":"token_revoked"}"#);

        let status = refresh_token_with(&server.client(), "xoxe-1-old", "session".to_owned()).await;

        assert!(matches!(status, AuthenticationStatus::Unauthorized));
        assert_eq!(
            server.single_request().form()["refresh_token"],
            "xoxe-1-old"
        );
    }

    #[rocket::async_test]
    async fn test_refresh_token_with_unexpected_response_is_bad_request() {
        let server = StubServer::json(r#"{"ok":true}"#);

        let status = refresh_token_with(&server.client(), "xoxe-1-old", "session".to_owned()).await;

        assert!(matches!(status, AuthenticationStatus::BadRequest));
    }
}
