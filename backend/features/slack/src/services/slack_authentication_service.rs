use rocket::http::Status;
use store::redis_store::{RedisResult, RedisStore};
use utils::logger;
use utils::time::Time;
use utils::uuid::Uuid;

use crate::client::{Client, SlackAccessTokenResponse, SlackRequestError};

type SessionId = String;

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
    let client = Client::new_defaults();
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
    let client = Client::new_defaults();
    logger::debug(format!(
        "[SLACK_AUTHENTICATION_SERVICE] - refreshing token with: {} and session id: {}",
        refresh_token, &session_id
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
