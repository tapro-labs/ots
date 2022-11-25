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

#[async_trait]
impl<'r> FromRequest<'r> for SlackAccessToken {
    type Error = AccessTokenError;

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let authorization = request.headers().get_one("Authorization");

        match authorization {
            Some(authorization) => {
                if !authorization.contains("Bearer") {
                    return Outcome::Failure((Status::Unauthorized, AccessTokenError::Missing));
                }

                let id: &str = authorization
                    .split("Bearer ")
                    .collect::<Vec<&str>>()
                    .get(1)
                    .unwrap();
                let mut redis_client = RedisStore::connect_default();
                let redis_key = format!("session:{}", &id);

                if let Ok(access_token) = redis_client.get::<String, &str>(&redis_key) {
                    return Outcome::Success(Self { access_token });
                }

                let redis_refresh_key = format!("session-refresh:{}", &id);

                // try to refresh token if we still have refresh token
                if let Ok(r_token) = redis_client.get::<String, &str>(&redis_refresh_key) {
                    if let AuthenticationStatus::Ok(response, _) =
                        refresh_token(&r_token, id.to_owned()).await
                    {
                        return Outcome::Success(Self {
                            access_token: response.access_token,
                        });
                    }
                }

                Outcome::Failure((Status::Unauthorized, AccessTokenError::Invalid))
            }
            _ => Outcome::Failure((Status::Unauthorized, AccessTokenError::Missing)),
        }
    }
}
